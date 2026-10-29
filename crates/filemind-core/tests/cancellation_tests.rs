use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_immediate_cancellation() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create 50 files
    for i in 0..50 {
        let p = root.join(format!("file_{}.txt", i));
        let mut f = File::create(p).unwrap();
        f.write_all(b"sample data").unwrap();
    }

    let cancel_token = Arc::new(AtomicBool::new(true)); // Already cancelled before start
    let result = execute_scan(root, vec![], cancel_token, None, None);

    assert!(result.is_cancelled);
    assert_eq!(result.total_files, 0);
}

#[test]
fn test_cancellation_during_scan() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create deeply nested structure with 1,000 files so scan takes measurable time
    for i in 0..20 {
        let sub = root.join(format!("folder_{}", i));
        fs::create_dir_all(&sub).unwrap();
        for j in 0..50 {
            let p = sub.join(format!("data_{}.bin", j));
            fs::write(p, &vec![0u8; 1024]).unwrap();
        }
    }

    let cancel_token = Arc::new(AtomicBool::new(false));
    let cancel_clone = Arc::clone(&cancel_token);

    // Spawn thread to trigger cancellation after 1ms
    let bg_thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(2));
        cancel_clone.store(true, Ordering::Relaxed);
    });

    let result = execute_scan(root, vec![], cancel_token, None, None);
    let _ = bg_thread.join();

    assert!(result.is_cancelled);
    assert!(result.total_files < 1000); // Stopped before completing all 1000 files
}
