use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

use filemind_core::db::repository::Database;
use filemind_core::incremental::reconcile::ReconciliationScanner;
use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_incremental_reconciliation_add_modify_delete() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Initial files
    let f1 = root.join("file1.txt");
    let f2 = root.join("file2.txt");
    fs::write(&f1, b"Initial file 1").unwrap();
    fs::write(&f2, b"Initial file 2").unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let initial_result = execute_scan(root, vec![], cancel_token, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    let session_id = "reconcile_session";
    Database::save_scan_result(&mut conn, session_id, &initial_result, 1000, 2000).unwrap();

    // Step 1: Add a new file
    let f3 = root.join("file3_new.txt");
    fs::write(&f3, b"New file content").unwrap();

    // Step 2: Modify f1
    fs::write(&f1, b"Modified file 1 with longer content").unwrap();

    // Step 3: Delete f2
    fs::remove_file(&f2).unwrap();

    // Run reconciliation scan
    let report = ReconciliationScanner::reconcile(&mut conn, session_id, root).unwrap();

    assert_eq!(report.added_files, 1);
    assert_eq!(report.modified_files, 1);
    assert_eq!(report.deleted_files, 1);
    assert_eq!(report.new_total_files, 2); // f1 (modified) + f3 (new)

    // Verify database reflects the updated state
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM files WHERE session_id = ?1",
            rusqlite::params![session_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
}
