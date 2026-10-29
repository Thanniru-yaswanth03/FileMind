use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::tempdir;

use filemind_core::db::repository::Database;
use filemind_core::models::{ContentSearchQuery, FileListQuery};
use filemind_core::scanner::walker::execute_scan;
use filemind_core::search::extractor::{DocumentExtractor, ExtractionStatus};
use filemind_core::search::fts::FtsEngine;
use filemind_core::search::metadata::MetadataSearchEngine;
use filemind_core::search::security::SecretFilter;

#[test]
fn test_fast_metadata_search_by_name_and_extension() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("report_2026.pdf"), b"%PDF-1.4 sample report").unwrap();
    fs::write(root.join("architecture.md"), b"# Storage Engine").unwrap();
    fs::write(root.join("notes.txt"), b"Meeting notes").unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "search_session_1", &scan_res, 10, 20).unwrap();

    // Query 1: Search by name substring
    let q1 = FileListQuery {
        session_id: Some("search_session_1".to_string()),
        search: Some("report".to_string()),
        ..Default::default()
    };
    let res1 = MetadataSearchEngine::search(&conn, &q1).unwrap();
    assert_eq!(res1.total_count, 1);
    assert_eq!(res1.files[0].name, "report_2026.pdf");

    // Query 2: Search by extension
    let q2 = FileListQuery {
        session_id: Some("search_session_1".to_string()),
        extension: Some("md".to_string()),
        ..Default::default()
    };
    let res2 = MetadataSearchEngine::search(&conn, &q2).unwrap();
    assert_eq!(res2.total_count, 1);
    assert_eq!(res2.files[0].name, "architecture.md");
}

#[test]
fn test_fts5_document_content_indexing_and_search() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let doc_a = root.join("architecture.md");
    let doc_b = root.join("notes.txt");

    fs::write(&doc_a, b"# Storage Engine\nHigh performance deterministic storage intelligence").unwrap();
    fs::write(&doc_b, b"Team sync: discuss memory allocation and cache pruning").unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "fts_session_1", &scan_res, 10, 20).unwrap();

    // Run FTS5 Document Content Indexer
    let progress = FtsEngine::index_documents(&mut conn, Some("fts_session_1"), 100).unwrap();
    assert_eq!(progress.indexed_documents, 2);
    assert_eq!(progress.skipped_sensitive, 0);

    // Full-Text Search for "deterministic"
    let search_q = ContentSearchQuery {
        query: "deterministic".to_string(),
        ..Default::default()
    };
    let search_res = FtsEngine::search_content(&conn, &search_q).unwrap();

    assert_eq!(search_res.total_matches, 1);
    assert_eq!(search_res.results[0].filename, "architecture.md");
    assert!(search_res.results[0].snippet.contains("<mark>deterministic</mark>"));
    assert_eq!(search_res.results[0].is_stale, false);
}

#[test]
fn test_sensitive_credentials_and_keys_excluded_from_indexing() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let env_file = root.join(".env");
    let key_file = root.join("server.key");
    let creds_file = root.join("credentials.json");
    let normal_file = root.join("readme.txt");

    fs::write(&env_file, b"DATABASE_PASSWORD=super_secret_password_123").unwrap();
    fs::write(&key_file, b"-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkq...").unwrap();
    fs::write(&creds_file, b"{\"api_key\": \"AIzaSyD-secret-token-here\"}").unwrap();
    fs::write(&normal_file, b"Public documentation text").unwrap();

    // Verify secret filter detects all of them
    assert!(SecretFilter::is_sensitive(&env_file));
    assert!(SecretFilter::is_sensitive(&key_file));
    assert!(SecretFilter::is_sensitive(&creds_file));
    assert!(!SecretFilter::is_sensitive(&normal_file));

    // Verify extractor rejects them
    let ext_env = DocumentExtractor::extract(&env_file);
    assert_eq!(ext_env.status, ExtractionStatus::SensitiveExcluded);
    assert_eq!(ext_env.text, "");

    let ext_normal = DocumentExtractor::extract(&normal_file);
    assert_eq!(ext_normal.status, ExtractionStatus::Complete);
    assert!(ext_normal.text.contains("Public documentation text"));

    // Index via FtsEngine
    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "sec_session", &scan_res, 10, 20).unwrap();

    let progress = FtsEngine::index_documents(&mut conn, Some("sec_session"), 100).unwrap();
    assert_eq!(progress.skipped_sensitive, 3); // .env, server.key, credentials.json
    assert_eq!(progress.indexed_documents, 1); // readme.txt

    // Verify searching for the secret yields 0 results!
    let q = ContentSearchQuery {
        query: "super_secret_password".to_string(),
        ..Default::default()
    };
    let res = FtsEngine::search_content(&conn, &q).unwrap();
    assert_eq!(res.total_matches, 0);
}

#[test]
fn test_stale_document_index_detection() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let doc = root.join("spec.txt");
    fs::write(&doc, b"Initial specification text content").unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "stale_session", &scan_res, 10, 20).unwrap();
    FtsEngine::index_documents(&mut conn, Some("stale_session"), 100).unwrap();

    // Now mutate the file on disk after indexing
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(&doc, b"Updated content with different length and modified timestamp").unwrap();

    // Query for "specification"
    let q = ContentSearchQuery {
        query: "specification".to_string(),
        ..Default::default()
    };
    let res = FtsEngine::search_content(&conn, &q).unwrap();

    assert_eq!(res.total_matches, 1);
    // Must be flagged as is_stale because file changed on disk!
    assert_eq!(res.results[0].is_stale, true);
}

#[test]
fn test_clear_content_index_preserves_file_metadata() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("doc1.txt"), b"Document one text").unwrap();
    fs::write(root.join("doc2.txt"), b"Document two text").unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let scan_res = execute_scan(root, vec![], cancel, None, None);

    let mut conn = Database::open_in_memory().unwrap();
    Database::save_scan_result(&mut conn, "clear_session", &scan_res, 10, 20).unwrap();
    FtsEngine::index_documents(&mut conn, Some("clear_session"), 100).unwrap();

    // Status shows 2 indexed docs
    let status_before = FtsEngine::get_status(&conn).unwrap();
    assert_eq!(status_before.total_indexed_documents, 2);

    // Clear content index
    FtsEngine::clear(&mut conn).unwrap();

    let status_after = FtsEngine::get_status(&conn).unwrap();
    assert_eq!(status_after.total_indexed_documents, 0);

    // Verify metadata files table is completely intact!
    let files_res = MetadataSearchEngine::search(
        &conn,
        &FileListQuery {
            session_id: Some("clear_session".to_string()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(files_res.total_count, 2);
}

