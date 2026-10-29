use std::fs;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tempfile::tempdir;

use filemind_core::db::repository::Database;
use filemind_core::duplicate::hashing::compute_file_blake3_streaming;
use filemind_core::models::*;
use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_deep_directory_nesting_and_aggregation() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let base = temp_root.path().join("deep_tree");

    // Construct a 35-level deep directory hierarchy
    let mut current = base.clone();
    for i in 1..=35 {
        current = current.join(format!("level_{:02}", i));
    }
    fs::create_dir_all(&current).unwrap();

    // Place files along various depths
    let mut path_step = base.clone();
    let mut total_created_files = 0;
    for i in 1..=35 {
        path_step = path_step.join(format!("level_{:02}", i));
        let file_path = path_step.join(format!("payload_depth_{}.dat", i));
        fs::write(&file_path, format!("Depth payload {} data", i)).unwrap();
        total_created_files += 1;
    }

    let cancel_token = Arc::new(AtomicBool::new(false));
    let t0 = Instant::now();

    let scan_result = execute_scan(
        &base,
        vec![],
        cancel_token,
        None,
        Some("deep_bench".to_string()),
    );
    let elapsed = t0.elapsed();

    println!(
        "\n[Deep Nesting Benchmark] Depth: 35 | Files: {} | Directories: {} | Scan Elapsed: {:?}",
        scan_result.total_files, scan_result.total_directories, elapsed
    );

    assert_eq!(scan_result.total_files, total_created_files as u64);
    assert!(scan_result.total_directories >= 35);
    assert!(
        elapsed.as_millis() < 500,
        "Deep scan must complete under 500ms"
    );
}

#[test]
fn test_large_file_streaming_hashing_throughput_and_bounds() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let test_file = temp_root.path().join("large_100mb.bin");

    // Write a 50MB file in 64KB blocks to measure real disk streaming
    let target_bytes: usize = 50 * 1024 * 1024; // 50 MB
    {
        let mut file = fs::File::create(&test_file).unwrap();
        let chunk = [0x5Au8; 64 * 1024]; // 64 KB pattern
        let chunks = target_bytes / chunk.len();
        for _ in 0..chunks {
            file.write_all(&chunk).unwrap();
        }
        file.flush().unwrap();
    }

    let t0 = Instant::now();
    let (hash, total_bytes_read, initial_mtime) =
        compute_file_blake3_streaming(&test_file).unwrap();
    let duration = t0.elapsed();

    let mb = (target_bytes as f64) / (1024.0 * 1024.0);
    let sec = duration.as_secs_f64().max(0.0001);
    let throughput_mb_s = mb / sec;

    println!(
        "\n[Large File Streaming Benchmark] Size: {:.2} MB | Elapsed: {:?} | Throughput: {:.2} MB/s | Hash: {}",
        mb, duration, throughput_mb_s, hash
    );

    assert_eq!(
        total_bytes_read, target_bytes as u64,
        "Large file hashing must read 100% of stream"
    );
    assert!(initial_mtime > 0);
    assert!(!hash.is_empty());
    assert!(
        throughput_mb_s > 30.0,
        "Streaming hashing must achieve at least 30 MB/s"
    );
}

#[test]
fn test_cancellation_latency_under_load() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let base = temp_root.path().join("cancel_bench");
    fs::create_dir_all(&base).unwrap();

    // Create 1,000 files across 10 subdirectories
    for d in 0..10 {
        let dir = base.join(format!("dir_{}", d));
        fs::create_dir_all(&dir).unwrap();
        for f in 0..100 {
            fs::write(
                dir.join(format!("item_{}.txt", f)),
                "filler data for cancellation test",
            )
            .unwrap();
        }
    }

    let cancel_token = Arc::new(AtomicBool::new(false));
    let cancel_clone = Arc::clone(&cancel_token);

    // Cancel after 5ms of scanning
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(5));
        cancel_clone.store(true, Ordering::SeqCst);
    });

    let t0 = Instant::now();
    let result = execute_scan(
        &base,
        vec![],
        cancel_token,
        None,
        Some("cancel_bench".to_string()),
    );
    let cancel_latency = t0.elapsed();

    println!(
        "\n[Cancellation Under Load] Elapsed: {:?} | Processed: {} files | Was Cancelled: {}",
        cancel_latency, result.total_files, result.is_cancelled
    );

    assert!(result.is_cancelled, "Scan result must reflect cancellation");
    assert!(
        cancel_latency.as_millis() < 250,
        "Cancellation latency must respond within 250ms of cancellation token trip"
    );
}

#[test]
fn test_synthetic_scale_25000_records_sqlite_query_latency() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let db_path = temp_root.path().join("scale_test.db");
    let db = Database::new(db_path.clone());
    let mut conn = db.get_connection().unwrap();

    let session_id = "scale_session_25000";

    // Begin bulk insertion of 25,000 records
    let t0 = Instant::now();
    let tx = conn.transaction().unwrap();

    tx.execute(
        r#"
        INSERT INTO scan_sessions (
            session_id, root_path, started_at, completed_at, status,
            total_files, total_directories, total_logical_bytes,
            total_physical_bytes, error_count
        ) VALUES (?1, ?2, ?3, ?4, 'completed', 25000, 100, 500000000, 520000000, 0)
        "#,
        rusqlite::params![session_id, "C:\\ScaleTest", 1000, 2000],
    )
    .unwrap();

    {
        let mut file_stmt = tx
            .prepare(
                r#"
            INSERT INTO files (
                session_id, directory_path, filename, extension, file_type,
                logical_size_bytes, physical_size_bytes, attributes, ctime, mtime, atime,
                ntfs_file_id, risk_tier, safety_rationale, is_locked
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 1000, 1000, 1000,
                NULL, 'safe', 'Generated benchmark file', 0
            )
            "#,
            )
            .unwrap();

        for i in 1..=25_000 {
            let ext = match i % 5 {
                0 => "rs",
                1 => "ts",
                2 => "png",
                3 => "zip",
                _ => "log",
            };
            let cat = match ext {
                "rs" | "ts" => "Code",
                "png" => "Media",
                "zip" => "Archives",
                _ => "Caches",
            };
            let fname = format!("source_document_{:06}.{}", i, ext);
            let dir_p = format!("C:\\ScaleTest\\sub_{:02}", i % 50);
            let sz = (i as u64) * 1024;

            file_stmt
                .execute(rusqlite::params![
                    session_id,
                    dir_p,
                    fname,
                    ext,
                    cat,
                    sz,
                    sz + 4096
                ])
                .unwrap();
        }
    }

    tx.commit().unwrap();
    let insert_duration = t0.elapsed();
    println!(
        "\n[Scale Test] 25,000 records inserted in {:?}",
        insert_duration
    );

    // 1. Query: Paginated files sorted by size DESC
    let q_start = Instant::now();
    let query = FileListQuery {
        session_id: Some(session_id.to_string()),
        path_prefix: None,
        category: Some("Code".to_string()),
        extension: None,
        search: None,
        min_size: None,
        max_size: None,
        sort_by: Some("size".to_string()),
        sort_order: Some("desc".to_string()),
        limit: Some(50),
        offset: Some(0),
    };

    let res = Database::query_files(&conn, &query).unwrap();
    let query_latency = q_start.elapsed();

    println!(
        "[Scale Test] 25,000 records pagination query (limit 50, category 'Code'): {:?} | Total Matches: {}",
        query_latency, res.total_count
    );

    assert_eq!(res.files.len(), 50);
    assert_eq!(res.total_count, 10_000); // 2/5 of 25,000
    assert!(
        query_latency.as_millis() < 50,
        "Indexed pagination on 25,000 records must execute in under 50ms"
    );

    // 2. Measure SQLite DB file size
    let db_file_meta = fs::metadata(&db_path).unwrap();
    let db_mb = (db_file_meta.len() as f64) / (1024.0 * 1024.0);
    println!(
        "[Scale Test] SQLite Database on-disk size for 25,000 rows: {:.2} MB",
        db_mb
    );
    assert!(
        db_mb < 20.0,
        "Database size for 25,000 records must remain under 20MB"
    );
}
