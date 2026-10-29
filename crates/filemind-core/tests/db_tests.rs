use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

use filemind_core::db::migrations::Migrator;
use filemind_core::db::repository::Database;
use filemind_core::db::retention::RetentionManager;
use filemind_core::models::FileListQuery;
use filemind_core::scanner::walker::execute_scan;

#[test]
fn test_database_initialization_and_migration() {
    let mut conn = Connection::open_in_memory().unwrap();
    assert_eq!(Migrator::get_version(&conn).unwrap(), 0);

    Migrator::migrate(&mut conn).unwrap();
    assert_eq!(Migrator::get_version(&conn).unwrap(), 5);

    // Verify tables exist
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('files', 'directories', 'scan_sessions', 'safety_audit_log', 'cleanup_plans', 'quarantine_records', 'automation_rules', 'automation_execution_history')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 8);
}

#[test]
fn test_save_and_query_scan_results() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    std::fs::write(root.join("code.rs"), b"fn main() {}").unwrap();
    std::fs::write(root.join("readme.md"), b"# Test Project").unwrap();

    let cancel_token = Arc::new(AtomicBool::new(false));
    let scan_result = execute_scan(root, vec![], cancel_token, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    let session_id = "test_session_1";
    Database::save_scan_result(&mut conn, session_id, &scan_result, 1000, 2000).unwrap();

    // Query latest session
    let latest = Database::get_latest_session(&conn, None).unwrap().unwrap();
    assert_eq!(latest.session_id, session_id);
    assert_eq!(latest.total_files, 2);

    // Query files
    let query = FileListQuery {
        session_id: Some(session_id.to_string()),
        ..Default::default()
    };
    let file_resp = Database::query_files(&conn, &query).unwrap();
    assert_eq!(file_resp.total_count, 2);
    assert_eq!(file_resp.files.len(), 2);

    // Test category breakdown
    let breakdown = Database::get_category_breakdown(&conn, session_id).unwrap();
    assert!(!breakdown.is_empty());

    // Test folder ranking
    let rankings = Database::get_top_folders(&conn, session_id, 10).unwrap();
    assert!(!rankings.is_empty());
}

#[test]
fn test_retention_pruning() {
    let mut conn = Database::open_in_memory().unwrap();

    // Insert dummy old sessions
    conn.execute(
        "INSERT INTO scan_sessions (session_id, root_path, started_at, status, total_files)
         VALUES ('old_1', 'C:\\test', 1000, 'completed', 10)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scan_sessions (session_id, root_path, started_at, status, total_files)
         VALUES ('new_1', 'C:\\test', 9999999999999, 'completed', 20)",
        [],
    )
    .unwrap();

    let pruned = RetentionManager::prune_older_than_days(&mut conn, 30).unwrap();
    assert_eq!(pruned, 1);

    let remaining: i64 = conn
        .query_row("SELECT COUNT(*) FROM scan_sessions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(remaining, 1);
}
