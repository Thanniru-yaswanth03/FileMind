use filemind_core::db::repository::Database;
use filemind_core::duplicate::hashing::compute_file_blake3_streaming;
use filemind_core::models::{ContentSearchQuery, FileListQuery};
use filemind_core::scanner::walker::execute_scan;
use filemind_core::search::fts::FtsEngine;
use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn bench_blake3_streaming_throughput() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("large_bench_file.bin");

    // Write a 32 MB file
    let size_bytes: usize = 32 * 1024 * 1024;
    {
        let mut f = File::create(&file_path).unwrap();
        let chunk = vec![0x5Au8; 64 * 1024];
        let chunks = size_bytes / chunk.len();
        for _ in 0..chunks {
            f.write_all(&chunk).unwrap();
        }
    }

    let start = Instant::now();
    let (hash, logical_bytes, elapsed_ms) =
        compute_file_blake3_streaming(&file_path).expect("Hashing failed");
    let duration = start.elapsed();

    assert_eq!(logical_bytes, size_bytes as u64);
    assert!(!hash.is_empty());

    let mb_hashed = (size_bytes as f64) / (1024.0 * 1024.0);
    let throughput_mb_s = mb_hashed / duration.as_secs_f64();

    println!("\n=== BENCHMARK: BLAKE3 Streaming Hashing ===");
    println!("File Size: {:.2} MB", mb_hashed);
    println!("Wall Time: {:?}", duration);
    println!("Reported Elapsed: {} ms", elapsed_ms);
    println!("BLAKE3 Throughput: {:.2} MB/s", throughput_mb_s);

    assert!(
        throughput_mb_s > 100.0,
        "BLAKE3 hashing throughput too low: {:.2} MB/s",
        throughput_mb_s
    );
}

#[test]
fn bench_scan_throughput_and_cancellation_latency() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Generate 3,000 files across 60 directories
    let dir_count = 60;
    let files_per_dir = 50;
    let total_expected = dir_count * files_per_dir;

    for d in 0..dir_count {
        let folder = root.join(format!("bench_dir_{:03}", d));
        fs::create_dir_all(&folder).unwrap();
        for f in 0..files_per_dir {
            let p = folder.join(format!("file_{:04}.txt", f));
            fs::write(p, b"Benchmark payload for storage scanner performance test").unwrap();
        }
    }

    // 1. Full Scan Throughput
    let scan_start = Instant::now();
    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], Arc::clone(&cancel_token), None, None);
    let scan_duration = scan_start.elapsed();

    let files_per_sec = (result.total_files as f64) / scan_duration.as_secs_f64();

    println!("\n=== BENCHMARK: Storage Scanner Throughput ===");
    println!("Total Files Scanned: {}", result.total_files);
    println!("Total Dirs Scanned: {}", result.total_directories);
    println!("Scan Time: {:?}", scan_duration);
    println!("Scan Throughput: {:.2} files/sec", files_per_sec);

    assert_eq!(result.total_files, total_expected as u64);
    assert!(
        files_per_sec > 600.0,
        "Scan throughput too low: {:.2} files/sec",
        files_per_sec
    );

    // 2. Cancellation Latency
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = Arc::clone(&cancel_flag);

    let cancel_thread = std::thread::spawn(move || {
        // Sleep 5ms, then flip cancel flag
        std::thread::sleep(std::time::Duration::from_millis(5));
        let cancel_trigger = Instant::now();
        cancel_flag_clone.store(true, Ordering::SeqCst);
        cancel_trigger
    });

    let _cancel_scan_start = Instant::now();
    let cancelled_result = execute_scan(root, vec![], Arc::clone(&cancel_flag), None, None);
    let cancel_trigger_time = cancel_thread.join().unwrap();
    let cancel_latency = Instant::now().duration_since(cancel_trigger_time);

    println!("\n=== BENCHMARK: Scanner Cancellation Latency ===");
    println!("Cancellation Latency: {:?}", cancel_latency);
    println!(
        "Items scanned before halt: {}",
        cancelled_result.total_files
    );
    println!("Is Cancelled Flag: {}", cancelled_result.is_cancelled);

    assert!(cancelled_result.is_cancelled);
    assert!(
        cancel_latency.as_millis() < 50,
        "Cancellation took too long: {:?}",
        cancel_latency
    );
}

#[test]
fn bench_fts5_indexing_and_search_latency() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let docs = [
        (
            "report_q1.txt",
            "Quarterly earnings financial statement for review and storage audit",
        ),
        (
            "architecture.md",
            "System design notes regarding safety guardian rules and lockfiles",
        ),
        (
            "server_log.txt",
            "Connection accepted from 127.0.0.1 on port 41732 with OK status",
        ),
        (
            "readme.txt",
            "High performance native desktop utility for Windows filesystem intelligence",
        ),
    ];

    for i in 0..200 {
        let (name, content) = docs[i % docs.len()];
        let path = root.join(format!("doc_{:03}_{}", i, name));
        fs::write(path, content).unwrap();
    }

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "bench_fts_session", &scan_res, 0, 100).unwrap();

    let index_start = Instant::now();
    let prog = FtsEngine::index_documents(&mut conn, Some("bench_fts_session"), 500).unwrap();
    let index_duration = index_start.elapsed();

    println!("\n=== BENCHMARK: FTS5 Full-Text Indexing & Search ===");
    println!(
        "Indexed {} documents in: {:?}",
        prog.indexed_documents, index_duration
    );
    println!(
        "Indexing Throughput: {:.2} docs/sec",
        (prog.indexed_documents as f64) / index_duration.as_secs_f64()
    );

    // Measure Search Latency
    let query = ContentSearchQuery {
        query: "financial".to_string(),
        ..Default::default()
    };

    let search_start = Instant::now();
    let results = FtsEngine::search_content(&conn, &query).unwrap();
    let search_duration = search_start.elapsed();

    println!("FTS5 Search Latency for 'financial': {:?}", search_duration);
    println!("Matches Returned: {}", results.total_matches);

    assert!(results.total_matches > 0);
    assert!(
        search_duration.as_millis() < 50,
        "FTS5 search exceeded 50ms: {:?}",
        search_duration
    );
}

#[test]
fn bench_sqlite_metadata_pagination_latency() {
    let mut conn = Database::open_in_memory().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create 1,000 files
    for i in 0..1000 {
        let p = root.join(format!("item_{:04}.log", i));
        fs::write(p, b"log data").unwrap();
    }

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);
    Database::save_scan_result(&mut conn, "bench_query_session", &scan_res, 0, 100).unwrap();

    let query = FileListQuery {
        session_id: Some("bench_query_session".to_string()),
        category: Some("Caches".to_string()),
        min_size: Some(0),
        sort_by: Some("size".to_string()),
        sort_order: Some("desc".to_string()),
        limit: Some(100),
        offset: Some(200),
        ..Default::default()
    };

    let query_start = Instant::now();
    let res = Database::query_files(&conn, &query).unwrap();
    let query_duration = query_start.elapsed();

    println!("\n=== BENCHMARK: SQLite Complex Query & Pagination ===");
    println!("Query Execution Time: {:?}", query_duration);
    println!("Total Matches in DB: {}", res.total_count);
    println!("Page Item Count: {}", res.files.len());

    assert_eq!(res.files.len(), 100);
    assert!(
        query_duration.as_millis() < 30,
        "Query pagination exceeded 30ms: {:?}",
        query_duration
    );
}
