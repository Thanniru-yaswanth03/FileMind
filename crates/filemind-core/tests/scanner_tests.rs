use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_known_file_sizes_and_nested_folders() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create nested structure:
    // root/
    //   file1.txt (100 bytes)
    //   subdir1/
    //     file2.bin (250 bytes)
    //     empty_dir/
    //     nested/
    //       file3.log (500 bytes)
    let sub1 = root.join("subdir1");
    let empty_dir = sub1.join("empty_dir");
    let nested = sub1.join("nested");

    fs::create_dir_all(&empty_dir).unwrap();
    fs::create_dir_all(&nested).unwrap();

    let mut f1 = File::create(root.join("file1.txt")).unwrap();
    f1.write_all(&vec![b'A'; 100]).unwrap();

    let mut f2 = File::create(sub1.join("file2.bin")).unwrap();
    f2.write_all(&vec![b'B'; 250]).unwrap();

    let mut f3 = File::create(nested.join("file3.log")).unwrap();
    f3.write_all(&vec![b'C'; 500]).unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], cancel_token, None, None);

    assert_eq!(result.total_files, 3);
    assert_eq!(result.total_logical_bytes, 850);
    assert_eq!(result.is_cancelled, false);
    assert_eq!(result.errors.len(), 0);

    // Verify empty directory was tracked
    let empty_record = result
        .directories
        .iter()
        .find(|d| d.path.ends_with("empty_dir"));
    assert!(empty_record.is_some());
    assert_eq!(empty_record.unwrap().file_count, 0);
    assert_eq!(empty_record.unwrap().total_size_bytes, 0);

    // Verify root directory aggregated all nested files
    let root_record = result.directories.iter().find(|d| d.depth == 0).unwrap();
    assert_eq!(root_record.file_count, 3);
    assert_eq!(root_record.total_size_bytes, 850);
}

#[test]
fn test_duplicate_filenames_in_different_directories() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let dir_a = root.join("folder_a");
    let dir_b = root.join("folder_b");
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();

    // Same filename "document.pdf" in two different folders
    let mut fa = File::create(dir_a.join("document.pdf")).unwrap();
    fa.write_all(&vec![b'X'; 300]).unwrap();

    let mut fb = File::create(dir_b.join("document.pdf")).unwrap();
    fb.write_all(&vec![b'Y'; 600]).unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], cancel_token, None, None);

    assert_eq!(result.total_files, 2);
    assert_eq!(result.total_logical_bytes, 900);

    let doc_a = result
        .files
        .iter()
        .find(|f| f.parent_path.ends_with("folder_a"))
        .unwrap();
    let doc_b = result
        .files
        .iter()
        .find(|f| f.parent_path.ends_with("folder_b"))
        .unwrap();

    assert_eq!(doc_a.name, "document.pdf");
    assert_eq!(doc_a.size_bytes, 300);
    assert_eq!(doc_b.name, "document.pdf");
    assert_eq!(doc_b.size_bytes, 600);
}

#[test]
fn test_unicode_filenames_and_paths() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Unicode directory and file names: Japanese, emojis, accents, Arabic
    let unicode_dir = root.join("日本語フォルダ_📁");
    fs::create_dir_all(&unicode_dir).unwrap();

    let f1_path = unicode_dir.join("レポート_2026.pdf");
    let mut f1 = File::create(&f1_path).unwrap();
    f1.write_all(b"Unicode PDF content").unwrap();

    let f2_path = unicode_dir.join("café_au_lait_☕.txt");
    let mut f2 = File::create(&f2_path).unwrap();
    f2.write_all("Café coffee".as_bytes()).unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], cancel_token, None, None);

    assert_eq!(result.total_files, 2);
    assert!(result.files.iter().any(|f| f.name.contains("レポート")));
    assert!(result.files.iter().any(|f| f.name.contains("café_au_lait")));
}

#[test]
fn test_scanner_is_strictly_read_only() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let test_file = root.join("immutable.txt");
    let original_bytes = b"CRITICAL_USER_DATA_DO_NOT_MUTATE";
    fs::write(&test_file, original_bytes).unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let _result = execute_scan(root, vec![], cancel_token, None, None);

    // Verify file content, size, and existence remain completely unchanged
    let after_bytes = fs::read(&test_file).unwrap();
    assert_eq!(original_bytes.as_slice(), after_bytes.as_slice());
}
