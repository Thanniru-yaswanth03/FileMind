use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

use filemind_core::server::{start_server, AppState};

#[test]
fn test_http_server_security_and_authorization_guards() {
    let temp_dir = tempdir().expect("Failed to create tempdir");
    let db_path = temp_dir.path().join("test_security.db");
    let test_port = 41799;

    let test_token = "test_capability_token_abcdef1234567890".to_string();
    let state = Arc::new(AppState::with_token(db_path, test_token.clone()));

    // Spawn server on test port
    thread::spawn(move || {
        let _ = start_server(test_port, state);
    });

    // Wait for server to bind
    thread::sleep(Duration::from_millis(600));

    // 1. Test Health endpoint (Public GET)
    let client = ureq_or_curl_health(test_port);
    assert!(client, "Health check should succeed on loopback");

    // 2. Test Path Traversal Protection on Static Asset endpoint
    // Attempting /../Cargo.toml or /..\Cargo.toml must return 403 Forbidden
    let traversal_blocked = test_traversal_blocked(test_port);
    assert!(traversal_blocked, "Path traversal attempt on static assets must be blocked with 403");

    // 3. Test Untrusted Origin Rejection (e.g. localhost.evil.com)
    let untrusted_blocked = test_untrusted_origin_blocked(test_port);
    assert!(untrusted_blocked, "Untrusted origin http://localhost.evil.com must be blocked with 403");

    // 4. Test Mutating Endpoint without Session Token (must return 401 Unauthorized)
    let unauth_blocked = test_missing_token_blocked(test_port);
    assert!(unauth_blocked, "Mutating request without X-FileMind-Session-Token must be rejected with 401");

    // 5. Test Mutating Endpoint with Valid Session Token (must succeed)
    let auth_accepted = test_valid_token_accepted(test_port, &test_token);
    assert!(auth_accepted, "Mutating request with valid X-FileMind-Session-Token must succeed");
}

fn ureq_or_curl_health(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/api/health", port);
    let output = std::process::Command::new("curl.exe")
        .args(["-s", "-o", "nul", "-w", "%{http_code}", &url])
        .output();

    if let Ok(out) = output {
        let code = String::from_utf8_lossy(&out.stdout);
        code.trim() == "200"
    } else {
        false
    }
}

fn test_traversal_blocked(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/../Cargo.toml", port);
    let output = std::process::Command::new("curl.exe")
        .args(["--path-as-is", "-s", "-o", "nul", "-w", "%{http_code}", &url])
        .output();

    if let Ok(out) = output {
        let code = String::from_utf8_lossy(&out.stdout);
        code.trim() == "403"
    } else {
        false
    }
}

fn test_untrusted_origin_blocked(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/api/roots", port);
    let output = std::process::Command::new("curl.exe")
        .args([
            "-s",
            "-H", "Origin: http://localhost.attacker.com",
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ])
        .output();

    if let Ok(out) = output {
        let code = String::from_utf8_lossy(&out.stdout);
        code.trim() == "403"
    } else {
        false
    }
}

fn test_missing_token_blocked(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/api/scan/cancel", port);
    let output = std::process::Command::new("curl.exe")
        .args([
            "-s",
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ])
        .output();

    if let Ok(out) = output {
        let code = String::from_utf8_lossy(&out.stdout);
        code.trim() == "401"
    } else {
        false
    }
}

fn test_valid_token_accepted(port: u16, token: &str) -> bool {
    let url = format!("http://127.0.0.1:{}/api/scan/cancel", port);
    let output = std::process::Command::new("curl.exe")
        .args([
            "-s",
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "-H", &format!("X-FileMind-Session-Token: {}", token),
            "-o", "nul",
            "-w", "%{http_code}",
            &url,
        ])
        .output();

    if let Ok(out) = output {
        let code = String::from_utf8_lossy(&out.stdout);
        // scan/cancel returns 200 { status: "no_active_scan" }
        code.trim() == "200"
    } else {
        false
    }
}
