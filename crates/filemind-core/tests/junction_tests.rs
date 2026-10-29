use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_symlink_or_junction_loop_prevention() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let real_dir = root.join("real_folder");
    fs::create_dir_all(&real_dir).unwrap();

    let mut f1 = File::create(real_dir.join("sample.txt")).unwrap();
    f1.write_all(b"sample file").unwrap();

    // Try creating a directory symlink on platforms supporting it (Windows developer mode or admin)
    // If symlinks cannot be created without privilege, test our loop detection logic directly with filter
    let link_path = root.join("link_to_real");
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_dir(&real_dir, &link_path);

    let cancel_token = Arc::new(AtomicBool::new(false));
    let result = execute_scan(root, vec![], cancel_token, None, None);

    // Scanner must terminate successfully without infinite recursion
    assert!(!result.is_cancelled);
    assert!(result.total_files >= 1);
}

#[test]
fn test_loop_filter_cycle_detection() {
    use filemind_core::scanner::filter::{ReparsePointPolicy, ScanFilter};

    let mut filter = ScanFilter::new(
        std::path::PathBuf::from(r"C:\test"),
        ReparsePointPolicy::NeverTraverse,
        vec![],
    );

    // First visit to volume 1234, file ID 5678 -> allowed
    assert!(filter.mark_visited_or_loop(1234, 5678));

    // Second visit to the same volume and file ID -> cycle detected!
    assert!(!filter.mark_visited_or_loop(1234, 5678));

    // Different directory -> allowed
    assert!(filter.mark_visited_or_loop(1234, 9999));
}
