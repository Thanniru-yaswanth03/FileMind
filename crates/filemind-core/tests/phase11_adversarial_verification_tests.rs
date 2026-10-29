use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

use filemind_core::cleanup::quarantine::{verified_quarantine_move, QuarantineManager};
use filemind_core::db::repository::Database;
use filemind_core::server::{start_server, AppState};

/// Helper to find a free loopback port
fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

/// Helper to execute curl commands against local test server
fn curl_status(args: &[&str]) -> String {
    let output = std::process::Command::new("curl.exe")
        .args(args)
        .output()
        .expect("Failed to execute curl.exe");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// ============================================================================
// 1. HTTP SERVER ADVERSARIAL TESTS
// ============================================================================

#[test]
fn test_http_server_occupied_port_and_clean_restart() {
    let port = get_free_port();
    let temp_dir = tempdir().expect("Failed to create tempdir");
    let db_path = temp_dir.path().join("port_test.db");

    // Occupy the port with a raw TCP listener
    let occupied_listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect("Failed to bind occupying listener");

    // Attempting to start the FileMind server on the occupied port must fail
    let state = Arc::new(AppState::new(db_path.clone()));
    let res = start_server(port, state);
    assert!(res.is_err(), "Server must fail to bind if port is already occupied");
    let err_msg = res.err().unwrap().to_string();
    assert!(
        err_msg.contains("Failed to bind server") || err_msg.contains("address already in use"),
        "Error message must indicate bind failure, got: {}",
        err_msg
    );

    // Release the port (clean shutdown of occupying listener)
    drop(occupied_listener);
    thread::sleep(Duration::from_millis(100));

    // Clean restart: now the server must be able to bind successfully
    let state2 = Arc::new(AppState::new(db_path));
    let server_handle = thread::spawn(move || {
        let _ = start_server(port, state2);
    });

    thread::sleep(Duration::from_millis(500));
    let url = format!("http://127.0.0.1:{}/api/health", port);
    let code = curl_status(&["-s", "-o", "nul", "-w", "%{http_code}", &url]);
    assert_eq!(code, "200", "Clean restart on released port must succeed");
    drop(server_handle);
}

#[test]
fn test_http_server_encoded_and_complex_traversal_attacks() {
    let port = get_free_port();
    let temp_dir = tempdir().expect("Failed to create tempdir");
    let db_path = temp_dir.path().join("traversal_test.db");
    let state = Arc::new(AppState::new(db_path));

    thread::spawn(move || {
        let _ = start_server(port, state);
    });
    thread::sleep(Duration::from_millis(600));

    let traversal_vectors = [
        "../Cargo.toml",
        "..%2fCargo.toml",
        "%2e%2e%2fCargo.toml",
        "%2e%2e/Cargo.toml",
        "..%5cCargo.toml",
        "%252e%252e%252fCargo.toml",
        "..\\..\\Cargo.toml",
        "C:\\Windows\\System32\\drivers\\etc\\hosts",
        "C:%5cWindows%5cwin.ini",
        "dist/index.html::$DATA",
        "index.html:secret_stream",
    ];

    for vector in traversal_vectors {
        let url = format!("http://127.0.0.1:{}/{}", port, vector);
        let code = curl_status(&["--path-as-is", "-s", "-o", "nul", "-w", "%{http_code}", &url]);
        assert!(
            code == "403" || code == "404",
            "Traversal vector '{}' must be rejected with 403 or 404, got HTTP {}",
            vector,
            code
        );
    }
}

#[test]
fn test_http_server_hostile_origins() {
    let port = get_free_port();
    let temp_dir = tempdir().expect("Failed to create tempdir");
    let db_path = temp_dir.path().join("origin_test.db");
    let state = Arc::new(AppState::new(db_path));

    thread::spawn(move || {
        let _ = start_server(port, state);
    });
    thread::sleep(Duration::from_millis(600));

    let hostile_origins = [
        "http://localhost.attacker.com",
        "http://127.0.0.1.evil.com",
        "http://evil.com",
        "http://localhost:abc",
        "http://127.0.0.1:fake",
        "null",
        "file://",
        "ftp://localhost",
        "https://attacker.org:41732",
    ];

    let url = format!("http://127.0.0.1:{}/api/roots", port);
    for origin in hostile_origins {
        let code = curl_status(&[
            "-s",
            "-H", &format!("Origin: {}", origin),
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ]);
        assert_eq!(
            code, "403",
            "Hostile origin '{}' must be blocked with 403 Forbidden, got HTTP {}",
            origin, code
        );
    }

    // Valid origins must succeed
    let valid_origins = [
        "http://localhost",
        "http://127.0.0.1",
        "http://localhost:3000",
        "http://127.0.0.1:41732",
        "tauri://localhost",
        "http://tauri.localhost",
    ];

    for origin in valid_origins {
        let code = curl_status(&[
            "-s",
            "-H", &format!("Origin: {}", origin),
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ]);
        assert_eq!(
            code, "200",
            "Valid origin '{}' must be permitted with 200 OK, got HTTP {}",
            origin, code
        );
    }
}

#[test]
fn test_mutating_endpoints_strictly_require_session_capability() {
    let port = get_free_port();
    let temp_dir = tempdir().expect("Failed to create tempdir");
    let db_path = temp_dir.path().join("auth_test.db");
    let valid_token = "secret_valid_session_token_12345".to_string();
    let state = Arc::new(AppState::with_token(db_path, valid_token.clone()));

    thread::spawn(move || {
        let _ = start_server(port, state);
    });
    thread::sleep(Duration::from_millis(600));

    let mutating_routes = [
        ("POST", "/api/scan/cancel"),
        ("POST", "/api/actions/open-folder"),
        ("POST", "/api/content/clear"),
        ("POST", "/api/safety/execute"),
        ("POST", "/api/cleanup/plan"),
        ("POST", "/api/cleanup/dry-run"),
        ("POST", "/api/cleanup/execute"),
        ("POST", "/api/quarantine/restore"),
        ("POST", "/api/quarantine/purge"),
        ("POST", "/api/copilot/query"),
        ("POST", "/api/developer/clean"),
    ];

    for (method, route) in mutating_routes {
        let url = format!("http://127.0.0.1:{}{}", port, route);

        // 1. Missing token -> 401 Unauthorized
        let code_missing = curl_status(&[
            "-s",
            "-X", method,
            "-H", "Content-Type: application/json",
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ]);
        assert_eq!(
            code_missing, "401",
            "Route '{}' without token must return 401, got {}",
            route, code_missing
        );

        // 2. Invalid / forged token -> 401 Unauthorized
        let code_invalid = curl_status(&[
            "-s",
            "-X", method,
            "-H", "Content-Type: application/json",
            "-H", "X-FileMind-Session-Token: forged_token_99999",
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ]);
        assert_eq!(
            code_invalid, "401",
            "Route '{}' with forged token must return 401, got {}",
            route, code_invalid
        );

        // 3. Valid token -> Not 401 (either 200, 400 for empty body, etc., but authorized)
        let code_valid = curl_status(&[
            "-s",
            "-X", method,
            "-H", "Content-Type: application/json",
            "-H", &format!("X-FileMind-Session-Token: {}", valid_token),
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ]);
        assert_ne!(
            code_valid, "401",
            "Route '{}' with valid token must not return 401, got {}",
            route, code_valid
        );
    }
}

// ============================================================================
// 2. QUARANTINE AND RESTORATION ADVERSARIAL TESTS
// ============================================================================

#[test]
fn test_quarantine_rollback_on_concurrent_source_modification() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let src = temp_root.path().join("source_doc.txt");
    let dest = temp_root.path().join("vault_doc.txt");

    fs::write(&src, "Original content version 1").unwrap();

    // Compute expected hash of v1
    let hash_v1 = blake3::hash(b"Original content version 1").to_hex().to_string();

    // Now tamper with the source file prior to move execution
    fs::write(&src, "Tampered content version 2!").unwrap();

    // Execute quarantine move expecting v1
    let res = verified_quarantine_move(&src, &dest, false, Some(&hash_v1));
    assert!(res.is_err(), "Must abort if source hash does not match expected");

    // Source must remain preserved and not deleted
    assert!(src.exists(), "Source file must never be deleted if verification fails");
    assert!(!dest.exists(), "Destination must not exist if move failed");
    assert_eq!(fs::read_to_string(&src).unwrap(), "Tampered content version 2!");
}

#[test]
fn test_restore_conflict_prevents_silent_overwrite() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let vault_dir = temp_root.path().join("vault");
    let original_dir = temp_root.path().join("original");
    fs::create_dir_all(&vault_dir).unwrap();
    fs::create_dir_all(&original_dir).unwrap();

    let target_file = original_dir.join("report.pdf");
    fs::write(&target_file, "Original Report Content").unwrap();

    let db_path = temp_root.path().join("restore_test.db");
    let db = Database::new(db_path);
    let conn = db.get_connection().unwrap();

    let quarantine = QuarantineManager::new(Some(vault_dir));
    let q_res = quarantine.quarantine_target(&target_file, "conflict_test", &conn);
    assert!(q_res.is_ok(), "Quarantine should succeed");
    let record = q_res.unwrap();

    // Recreate a new file at original_dir/report.pdf to simulate user or app replacing it
    fs::write(&target_file, "NEW UNRELATED FILE CREATED AT ORIGINAL LOCATION").unwrap();

    // Attempt to restore: MUST FAIL with conflict error rather than silently clobbering new file
    let restore_res = quarantine.restore_target(&record.quarantine_id, None, &conn);
    assert!(!restore_res.success, "Restore must fail if target location already contains a file");
    assert!(
        restore_res.error.as_deref().unwrap_or("").contains("already exists"),
        "Error must report that destination already exists"
    );

    // Existing file at original location must be untouched
    assert_eq!(
        fs::read_to_string(&target_file).unwrap(),
        "NEW UNRELATED FILE CREATED AT ORIGINAL LOCATION"
    );
    // Quarantined file in vault must remain safely preserved
    assert!(PathBuf::from(&record.quarantine_path).exists());
}

// ============================================================================
// 3. DATABASE RECOVERY AND ISOLATION TESTS
// ============================================================================

#[test]
fn test_corrupt_database_cleans_wal_and_shm_sidecars() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let db_path = temp_root.path().join("wal_corrupt.db");
    let wal_path = temp_root.path().join("wal_corrupt.db-wal");
    let shm_path = temp_root.path().join("wal_corrupt.db-shm");

    // Create garbage DB and sidecars
    fs::write(&db_path, b"GARBAGE_DB_DATA_12345").unwrap();
    fs::write(&wal_path, b"STALE_WAL_TRANSACTIONS").unwrap();
    fs::write(&shm_path, b"STALE_SHM_INDEX").unwrap();

    let db = Database::new(db_path.clone());
    let conn = db.get_connection();
    assert!(conn.is_ok(), "Database recovery must succeed despite corrupt DB and WAL");

    let conn = conn.unwrap();
    let is_ok = Database::verify_integrity(&conn).unwrap();
    assert!(is_ok, "New recovered database must pass integrity pragma");

    // 1. Verify original corrupted DB was safely quarantined to corrupt_<timestamp>.db
    let entries = fs::read_dir(temp_root.path()).unwrap();
    let mut found_corrupt_db = false;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains("corrupt_") && name.ends_with(".db") {
            found_corrupt_db = true;
            let content = fs::read(entry.path()).unwrap();
            assert_eq!(content, b"GARBAGE_DB_DATA_12345", "Preserved database artifact must preserve original bytes");
            break;
        }
    }
    assert!(found_corrupt_db, "Corrupt database must be quarantined and preserved as forensic artifact");

    // 2. Any active WAL file created by SQLite for fresh DB must be fresh and not contain stale corrupt payload
    if wal_path.exists() {
        let fresh_wal_content = fs::read(&wal_path).unwrap();
        assert_ne!(fresh_wal_content, b"STALE_WAL_TRANSACTIONS", "Active WAL must not contain stale corrupt transactions");
    }

    // 3. Verify recovered clean database has 0 scans and 0 files (no phantom records)
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM scan_sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0, "Fresh recovered database must report 0 scan sessions");

    let file_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
        .unwrap();
    assert_eq!(file_count, 0, "Fresh recovered database must report 0 files");
}
