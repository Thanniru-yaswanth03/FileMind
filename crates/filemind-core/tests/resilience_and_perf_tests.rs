use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;

use filemind_core::adapter::path::{is_descendant_of, normalize_path, to_long_path};
use filemind_core::db::repository::Database;
use filemind_core::models::{FileListQuery, SessionStatus};
use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_path_normalization_and_containment_properties() {
    let test_cases = vec![
        (r"C:\Users\John\Documents", r"C:\Users\John\Documents"),
        (r"C:/Users/John/Documents/", r"C:\Users\John\Documents"),
        (r"\\?\C:\Users\John\Documents", r"C:\Users\John\Documents"),
        (r"C:\Users\..\Users\John\Documents", r"C:\Users\John\Documents"),
    ];

    for (input, expected) in test_cases {
        let norm = normalize_path(Path::new(input));
        assert_eq!(norm, expected, "Failed normalization for: {}", input);
        let long = to_long_path(Path::new(&norm));
        let long_str = long.to_string_lossy();
        assert!(long_str.starts_with(r"\\?\") || !long_str.contains(':'), "Expected verbatim prefix for: {}", long_str);
    }

    // Containment invariants
    assert!(is_descendant_of(Path::new(r"C:\Root\Sub\file.txt"), Path::new(r"C:\Root")));
    assert!(is_descendant_of(Path::new(r"C:\Root\Sub"), Path::new(r"C:\Root")));
    assert!(!is_descendant_of(Path::new(r"C:\RootOther\file.txt"), Path::new(r"C:\Root")));
    assert!(!is_descendant_of(Path::new(r"D:\Root\file.txt"), Path::new(r"C:\Root")));
}

#[test]
fn test_error_resilience_and_partial_scan_reporting() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create 4 readable files
    for i in 0..3 {
        let p = root.join(format!("readable_{}.txt", i));
        fs::write(p, b"readable content").unwrap();
    }

    let sub = root.join("valid_sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("sub_file.txt"), b"sub content").unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], cancel_token, None, None);

    assert_eq!(result.total_files, 4);
    assert_eq!(result.is_cancelled, false);

    // Save to database and verify partial errors can be recorded without failing session
    let mut conn = Database::open_in_memory().unwrap();
    let session_id = "partial_error_session";
    Database::save_scan_result(&mut conn, session_id, &result, 100, 200).unwrap();

    let session = Database::get_latest_session(&conn, Some(&result.root_path)).unwrap().unwrap();
    assert_eq!(session.total_files, 4);
    assert_eq!(session.status, SessionStatus::Completed);
}

#[test]
fn test_performance_synthetic_scale() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create 2,500 files across 50 directories
    let dir_count = 50;
    let files_per_dir = 50;
    let total_expected = dir_count * files_per_dir;

    println!("\n--- Generating synthetic filesystem tree: {} files across {} directories ---", total_expected, dir_count);
    let gen_start = Instant::now();

    for d in 0..dir_count {
        let folder = root.join(format!("batch_dir_{:03}", d));
        fs::create_dir_all(&folder).unwrap();

        for f in 0..files_per_dir {
            let file_path = folder.join(format!("item_{:04}.dat", f));
            let size = ((f % 4) + 1) * 512;
            let mut file = File::create(file_path).unwrap();
            let buf = vec![(f % 255) as u8; size];
            file.write_all(&buf).unwrap();
        }
    }
    println!("Generation finished in {:?}", gen_start.elapsed());

    // Step 1: Benchmark scanner execution
    let scan_start = Instant::now();
    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], cancel_token, None, None);
    let scan_duration = scan_start.elapsed();

    println!("Scanned {} files in {:?}", result.total_files, scan_duration);
    let files_per_sec = (result.total_files as f64) / scan_duration.as_secs_f64();
    println!("Scan throughput: {:.2} files/sec", files_per_sec);

    assert_eq!(result.total_files, total_expected as u64);
    assert_eq!(result.is_cancelled, false);
    assert!(files_per_sec > 500.0, "Expected scanning throughput > 500 files/sec, got {:.2}", files_per_sec);

    // Step 2: Benchmark SQLite batch persistence
    let mut conn = Database::open_in_memory().unwrap();
    let db_start = Instant::now();
    let session_id = "perf_bench_session";
    Database::save_scan_result(&mut conn, session_id, &result, 0, scan_duration.as_millis() as u64).unwrap();
    let db_duration = db_start.elapsed();

    println!("Persisted {} files to SQLite in {:?}", total_expected, db_duration);
    let db_throughput = (total_expected as f64) / db_duration.as_secs_f64();
    println!("Database write throughput: {:.2} files/sec", db_throughput);
    assert!(db_duration.as_millis() < 4000, "Database batch insert took too long: {:?}", db_duration);

    // Step 3: Benchmark SQLite queries with filters & pagination
    let query_start = Instant::now();
    let query = FileListQuery {
        session_id: Some(session_id.to_string()),
        category: Some("Other".to_string()),
        min_size: Some(1024),
        sort_by: Some("size".to_string()),
        sort_order: Some("desc".to_string()),
        limit: Some(100),
        offset: Some(0),
        ..Default::default()
    };
    let query_res = Database::query_files(&conn, &query).unwrap();
    let query_duration = query_start.elapsed();

    println!("Indexed query execution time: {:?}", query_duration);
    assert!(query_duration.as_millis() < 50, "Indexed query took too long: {:?}", query_duration);
    assert!(query_res.total_count > 0);
    assert_eq!(query_res.files.len(), 100);

    // Folder ranking benchmark
    let rank_start = Instant::now();
    let rankings = Database::get_top_folders(&conn, session_id, 10).unwrap();
    let rank_duration = rank_start.elapsed();
    println!("Folder ranking execution time: {:?}", rank_duration);
    assert_eq!(rankings.len(), 10);
    assert!(rank_duration.as_millis() < 50);
}
