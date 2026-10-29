use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

use filemind_core::db::repository::Database;
use filemind_core::duplicate::hashing::{compute_file_blake3_streaming, verify_byte_by_byte};
use filemind_core::duplicate::heuristics::LocationEvaluator;
use filemind_core::duplicate::pipeline::DuplicatePipeline;
use filemind_core::models::{DuplicateConfidence, DuplicateScanRequest};
use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_identical_files_with_different_names() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create 3 identical files with distinct names across subfolders
    let sub = root.join("nested_folder");
    fs::create_dir_all(&sub).unwrap();

    let content = b"FILEMIND_CRYPTOGRAPHIC_IDENTICAL_PAYLOAD_TEST_DATA_2026";
    fs::write(root.join("original.dat"), content).unwrap();
    fs::write(root.join("copy_renamed.bin"), content).unwrap();
    fs::write(sub.join("backup_different_name.dat"), content).unwrap();

    // Scan directory
    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "dup_session_1", &scan_res, 10, 20).unwrap();

    // Run duplicate detection
    let req = DuplicateScanRequest {
        session_id: Some("dup_session_1".to_string()),
        min_size_bytes: Some(1),
        max_groups: Some(10),
        verify_byte_by_byte: Some(true),
    };

    let dup_res = DuplicatePipeline::execute(&conn, &req).unwrap();

    assert_eq!(dup_res.groups.len(), 1);
    let grp = &dup_res.groups[0];
    assert_eq!(grp.file_count, 3);
    assert_eq!(grp.confidence, DuplicateConfidence::ByteForByteVerified);
    assert_eq!(grp.size_bytes, content.len() as u64);
    assert_eq!(grp.wasted_bytes, (content.len() * 2) as u64);

    // Verify each file has its exact path
    assert!(grp.files.iter().any(|f| f.filename == "original.dat"));
    assert!(grp.files.iter().any(|f| f.filename == "copy_renamed.bin"));
    assert!(grp
        .files
        .iter()
        .any(|f| f.filename == "backup_different_name.dat"));
}

#[test]
fn test_same_filename_different_contents() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let dir_a = root.join("project_alpha");
    let dir_b = root.join("project_beta");
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();

    // Same filename "config.json" but different content and size
    fs::write(
        dir_a.join("config.json"),
        b"{\"project\": \"alpha\", \"version\": 1}",
    )
    .unwrap();
    fs::write(
        dir_b.join("config.json"),
        b"{\"project\": \"beta\", \"features\": [1, 2, 3]}",
    )
    .unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "same_name_diff_content", &scan_res, 10, 20).unwrap();

    let req = DuplicateScanRequest {
        session_id: Some("same_name_diff_content".to_string()),
        min_size_bytes: Some(1),
        ..Default::default()
    };

    let dup_res = DuplicatePipeline::execute(&conn, &req).unwrap();
    // Must NOT be grouped as duplicates because contents differ!
    assert_eq!(dup_res.groups.len(), 0);
}

#[test]
fn test_zero_byte_files_not_grouped_as_wasted_space() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create multiple empty 0-byte files (like .gitkeep or lockfiles)
    fs::write(root.join("empty1.txt"), b"").unwrap();
    fs::write(root.join("empty2.txt"), b"").unwrap();
    fs::write(root.join("empty3.txt"), b"").unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "zero_byte_session", &scan_res, 10, 20).unwrap();

    let req = DuplicateScanRequest {
        session_id: Some("zero_byte_session".to_string()),
        min_size_bytes: Some(1), // Exclude 0-byte files
        ..Default::default()
    };

    let dup_res = DuplicatePipeline::execute(&conn, &req).unwrap();
    assert_eq!(dup_res.groups.len(), 0);
    assert_eq!(dup_res.total_wasted_bytes, 0);
}

#[test]
fn test_streaming_large_file_hashing() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a 2MB file to verify streaming read with 64KB bounded buffer
    let file_path = root.join("large_file.bin");
    let mut f = File::create(&file_path).unwrap();
    let chunk = vec![0x5Au8; 65536]; // 64KB chunk
    for _ in 0..32 {
        f.write_all(&chunk).unwrap();
    }
    f.flush().unwrap();

    let (hash, bytes_read, mtime) = compute_file_blake3_streaming(&file_path).unwrap();
    assert_eq!(bytes_read, 2 * 1024 * 1024);
    assert!(!hash.is_empty());
    assert!(mtime > 0);
}

#[test]
fn test_byte_for_byte_verification_disambiguation() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let p_a = root.join("file_a.txt");
    let p_b = root.join("file_b.txt");
    let p_c = root.join("file_c.txt");

    fs::write(&p_a, b"EXACT_MATCHING_CONTENT_123456").unwrap();
    fs::write(&p_b, b"EXACT_MATCHING_CONTENT_123456").unwrap();
    fs::write(&p_c, b"EXACT_MATCHING_CONTENT_DIFFER").unwrap(); // Same length, 1 char diff

    assert_eq!(verify_byte_by_byte(&p_a, &p_b).unwrap(), true);
    assert_eq!(verify_byte_by_byte(&p_a, &p_c).unwrap(), false);
}

#[test]
fn test_unicode_and_nested_duplicate_paths() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let unicode_dir_a = root.join("📁_日本語_A");
    let unicode_dir_b = root.join("📁_日本語_B");
    fs::create_dir_all(&unicode_dir_a).unwrap();
    fs::create_dir_all(&unicode_dir_b).unwrap();

    let content = "Café au lait — 特急列車 2026".as_bytes();
    fs::write(unicode_dir_a.join("ドキュメント.txt"), content).unwrap();
    fs::write(unicode_dir_b.join("ドキュメント_コピー.txt"), content).unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "unicode_dup_session", &scan_res, 10, 20).unwrap();

    let req = DuplicateScanRequest {
        session_id: Some("unicode_dup_session".to_string()),
        min_size_bytes: Some(1),
        ..Default::default()
    };

    let dup_res = DuplicatePipeline::execute(&conn, &req).unwrap();
    assert_eq!(dup_res.groups.len(), 1);
    assert_eq!(dup_res.groups[0].files.len(), 2);
    assert!(dup_res.groups[0].files[0].path.contains("日本語"));
}

#[test]
fn test_location_heuristics_recommends_safest_copy_not_blindly_newest() {
    let path_sys = r"C:\Windows\System32\driver_config.dll";
    let path_temp = r"C:\Users\John\Downloads\driver_config.dll";
    let path_docs = r"C:\Users\John\Documents\driver_config.dll";

    // System copy must ALWAYS be protected and recommended to keep over Downloads
    let (best_idx, rationale) = LocationEvaluator::rank_copies(&[path_temp, path_sys]);
    assert_eq!(best_idx, 1); // path_sys is at index 1
    assert!(rationale.contains("protected system"));

    // User documents should be preferred over Downloads temp folder
    let (best_idx2, rationale2) = LocationEvaluator::rank_copies(&[path_temp, path_docs]);
    assert_eq!(best_idx2, 1); // path_docs is at index 1
    assert!(rationale2.contains("primary workspace") || rationale2.contains("clean path"));
}

#[test]
fn test_duplicates_strictly_non_destructive() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let file_1 = root.join("keep_me.txt");
    let file_2 = root.join("duplicate_me.txt");
    let data = b"CRITICAL_DO_NOT_DELETE_DATA";

    fs::write(&file_1, data).unwrap();
    fs::write(&file_2, data).unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "non_destructive_session", &scan_res, 10, 20).unwrap();

    let req = DuplicateScanRequest {
        session_id: Some("non_destructive_session".to_string()),
        min_size_bytes: Some(1),
        ..Default::default()
    };

    let _dup_res = DuplicatePipeline::execute(&conn, &req).unwrap();

    // Verify BOTH files remain completely untouched on disk
    assert!(file_1.exists());
    assert!(file_2.exists());
    assert_eq!(fs::read(&file_1).unwrap(), data.as_slice());
    assert_eq!(fs::read(&file_2).unwrap(), data.as_slice());
}
