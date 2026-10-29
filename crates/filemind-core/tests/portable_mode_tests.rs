use std::fs;
use tempfile::tempdir;

use filemind_core::cleanup::quarantine::QuarantineManager;
use filemind_core::db::repository::Database;

#[test]
fn test_spaces_and_unicode_in_portable_path() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    // Directory containing spaces, accents, and CJK characters
    let unicode_folder = temp_root.path().join("FileMind 空间测试 🚀 v0.2.0");
    fs::create_dir_all(&unicode_folder).unwrap();

    let db_path = unicode_folder.join("data").join("filemind.db");
    let db = Database::new(db_path.clone());

    // 1. Connection and migrations must succeed in Unicode space path
    let conn = db.get_connection();
    assert!(conn.is_ok(), "Database connection must succeed in path with spaces & Unicode: {:?}", conn.err());
    let conn = conn.unwrap();

    // 2. Structural integrity pragma must pass
    let is_ok = Database::verify_integrity(&conn).unwrap();
    assert!(is_ok, "Integrity check must return ok");

    // 3. Quarantine vault in Unicode path
    let vault_dir = unicode_folder.join("data").join("Quarantine");
    let quarantine = QuarantineManager::new(Some(vault_dir.clone()));
    assert!(vault_dir.exists(), "Quarantine directory must be created in Unicode path");

    // Test staging a file in the unicode vault
    let dummy_file = unicode_folder.join("sample test file.txt");
    fs::write(&dummy_file, "Portable Test Content").unwrap();
    let res = quarantine.quarantine_target(&dummy_file, "portable_test", &conn);
    assert!(res.is_ok(), "Quarantine target must succeed in Unicode path: {:?}", res.err());
}

#[test]
fn test_corrupt_database_automatic_self_healing() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let db_path = temp_root.path().join("filemind_corrupt_test.db");

    // Write invalid/garbage header to simulate unexpected power outage or sector corruption
    fs::write(&db_path, b"CORRUPTED_SQLITE_HEADER_GARBAGE_BYTES_1234567890!@#$%^&*()").unwrap();

    let db = Database::new(db_path.clone());

    // Connection must self-heal: backup corrupted DB and initialize fresh clean DB
    let conn = db.get_connection();
    assert!(conn.is_ok(), "Database must self-heal on corruption and return fresh connection: {:?}", conn.err());

    let conn = conn.unwrap();
    let is_ok = Database::verify_integrity(&conn).unwrap();
    assert!(is_ok, "Recreated database must pass SQLite integrity checks");

    // Verify that the corrupt file was quarantined to a backup file
    let entries = fs::read_dir(temp_root.path()).unwrap();
    let mut found_corrupt_backup = false;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains("corrupt_") && name.ends_with(".db") {
            found_corrupt_backup = true;
            break;
        }
    }
    assert!(found_corrupt_backup, "Corrupted database file must be quarantined for post-mortem forensics");
}

