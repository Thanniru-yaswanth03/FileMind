use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::accounting::StorageAccountant;
use crate::adapter::volume::{enumerate_all_volumes, query_volume_info};
use crate::cleanup::QuarantineManager;
use crate::db::repository::Database;
use crate::incremental::reconcile::ReconciliationScanner;
use crate::models::*;
use crate::scanner::walker::execute_scan;
use crate::server::actions::open_containing_folder;

pub struct ActiveScanState {
    pub session_id: String,
    pub root_path: String,
    pub cancel_token: Arc<AtomicBool>,
}

pub struct AppState {
    pub db: Database,
    pub quarantine: QuarantineManager,
    pub active_scan: Mutex<Option<ActiveScanState>>,
    pub latest_progress: Mutex<ScanProgressState>,
    pub scheduler: Arc<crate::automation::AutomationScheduler>,
    pub session_token: String,
}

fn generate_session_capability_token() -> String {
    let mut hasher = blake3::Hasher::new();
    let pid = std::process::id();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let rand_box = Box::new(0u64);
    let ptr_val = rand_box.as_ref() as *const u64 as usize;
    hasher.update(&pid.to_le_bytes());
    hasher.update(&now.to_le_bytes());
    hasher.update(&ptr_val.to_le_bytes());
    hasher.update(b"filemind-capability-token-salt-v0.2.0");
    hasher.finalize().to_hex().to_string()
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        let db = Database::new(db_path.clone());
        let quarantine = QuarantineManager::new(None);
        let scheduler = Arc::new(crate::automation::AutomationScheduler::new(
            db_path,
            quarantine.clone(),
        ));
        scheduler.start();
        let session_token = generate_session_capability_token();

        Self {
            db,
            quarantine,
            active_scan: Mutex::new(None),
            latest_progress: Mutex::new(ScanProgressState {
                percent: 0.0,
                current_path: String::new(),
                files_count: 0,
                bytes_count: 0,
                directories_count: 0,
                errors_count: 0,
                elapsed_ms: 0,
                is_scanning: false,
                session_id: None,
            }),
            scheduler,
            session_token,
        }
    }

    pub fn with_token(db_path: PathBuf, token: String) -> Self {
        let mut state = Self::new(db_path);
        state.session_token = token;
        state
    }
}

pub fn start_server(port: u16, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port);
    let server =
        Server::http(&addr).map_err(|e| format!("Failed to bind server on {}: {}", addr, e))?;
    println!(
        "[FileMind Core] Native Storage Engine listening on http://{}",
        addr
    );

    for request in server.incoming_requests() {
        let state_clone = Arc::clone(&state);
        thread::spawn(move || {
            handle_request(request, state_clone);
        });
    }

    Ok(())
}

fn is_trusted_origin(origin: &str) -> bool {
    let lower = origin.trim().to_ascii_lowercase();
    if lower == "http://localhost"
        || lower == "http://127.0.0.1"
        || lower == "tauri://localhost"
        || lower == "http://tauri.localhost"
        || lower == "vscode-webview://"
    {
        return true;
    }
    for prefix in &["http://localhost:", "http://127.0.0.1:"] {
        if let Some(port_str) = lower.strip_prefix(prefix) {
            // Must be purely numeric port, e.g. 3000, 41732
            if !port_str.is_empty() && port_str.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    false
}

fn get_allowed_origin(request: &Request) -> Option<String> {
    for header in request.headers() {
        if header
            .field
            .as_str()
            .as_str()
            .eq_ignore_ascii_case("origin")
        {
            let origin_str = header.value.as_str();
            if is_trusted_origin(origin_str) {
                return Some(origin_str.to_string());
            } else {
                return None;
            }
        }
    }
    // No Origin header: direct native CLI, desktop webview IPC, or curl on loopback
    Some("http://127.0.0.1".to_string())
}

fn verify_session_capability(request: &Request, expected_token: &str) -> bool {
    for header in request.headers() {
        if header
            .field
            .as_str()
            .as_str()
            .eq_ignore_ascii_case("x-filemind-session-token")
        {
            let val = header.value.as_str().trim();
            return val == expected_token;
        }
    }
    false
}

fn handle_request(mut request: Request, state: Arc<AppState>) {
    let allowed_origin = match get_allowed_origin(&request) {
        Some(orig) => orig,
        None => {
            let resp = Response::from_string(r#"{"error":"Cross-Origin Request Blocked: Access from untrusted origin is prohibited for desktop security"}"#)
                .with_status_code(StatusCode(403))
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            let _ = request.respond(resp);
            return;
        }
    };

    let url_str = request.url().to_string();
    let (path, query_params) = parse_url(&url_str);

    // Handle CORS preflight
    if request.method() == &Method::Options {
        let resp = Response::empty(StatusCode(200))
            .with_header(
                Header::from_bytes(
                    b"Access-Control-Allow-Origin".as_slice(),
                    allowed_origin.as_bytes(),
                )
                .unwrap(),
            )
            .with_header(
                Header::from_bytes(
                    &b"Access-Control-Allow-Methods"[..],
                    &b"GET, POST, DELETE, OPTIONS"[..],
                )
                .unwrap(),
            )
            .with_header(
                Header::from_bytes(
                    &b"Access-Control-Allow-Headers"[..],
                    &b"Content-Type, X-FileMind-Session-Token"[..],
                )
                .unwrap(),
            );
        let _ = request.respond(resp);
        return;
    }

    let method = request.method().clone();

    // Static Web Asset Serving for Standalone & Portable Distribution
    if method == Method::Get && !path.starts_with("/api") {
        let raw_rel_path = path.trim_start_matches('/');
        let decoded_rel_path = recursive_url_decode(raw_rel_path);

        // Path Traversal Defense: Reject paths containing '..' or Windows drive separators or alternate data streams immediately
        if raw_rel_path.contains("..")
            || raw_rel_path.contains('\\')
            || raw_rel_path.contains(':')
            || decoded_rel_path.contains("..")
            || decoded_rel_path.contains('\\')
            || decoded_rel_path.contains(':')
            || decoded_rel_path.starts_with('/')
            || decoded_rel_path.starts_with('\\')
        {
            let resp = Response::from_string(r#"{"error":"Forbidden: Invalid asset path"}"#)
                .with_status_code(StatusCode(403))
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                )
                .with_header(
                    Header::from_bytes(
                        b"Access-Control-Allow-Origin".as_slice(),
                        allowed_origin.as_bytes(),
                    )
                    .unwrap(),
                );
            let _ = request.respond(resp);
            return;
        }

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let dist_dirs = [
            exe_dir.join("dist"),
            std::path::PathBuf::from("dist"),
            exe_dir.join("../dist"),
            std::path::PathBuf::from("../../dist"),
            exe_dir.join("../../dist"),
        ];

        let base_dist = dist_dirs.into_iter().find(|d| d.exists());

        if let Some(dist_path) = base_dist {
            let canonical_dist = dist_path
                .canonicalize()
                .unwrap_or_else(|_| dist_path.clone());
            let candidate = if decoded_rel_path.is_empty() {
                dist_path.join("index.html")
            } else {
                dist_path.join(&decoded_rel_path)
            };

            let target_file = if candidate.exists() && candidate.is_file() {
                if let Ok(canonical_target) = candidate.canonicalize() {
                    if !canonical_target.starts_with(&canonical_dist) {
                        let resp = Response::from_string(
                            r#"{"error":"Forbidden: Path traversal blocked"}"#,
                        )
                        .with_status_code(StatusCode(403))
                        .with_header(
                            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                                .unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(
                                b"Access-Control-Allow-Origin".as_slice(),
                                allowed_origin.as_bytes(),
                            )
                            .unwrap(),
                        );
                        let _ = request.respond(resp);
                        return;
                    }
                }
                candidate
            } else if !decoded_rel_path.contains('.') && !decoded_rel_path.starts_with("assets/") {
                dist_path.join("index.html") // SPA fallback for extension-less client-side routes
            } else {
                let resp = Response::from_string(r#"{"error":"Asset Not Found"}"#)
                    .with_status_code(StatusCode(404))
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    )
                    .with_header(
                        Header::from_bytes(
                            b"Access-Control-Allow-Origin".as_slice(),
                            allowed_origin.as_bytes(),
                        )
                        .unwrap(),
                    );
                let _ = request.respond(resp);
                return;
            };

            if target_file.exists() {
                if let Ok(content) = std::fs::read(&target_file) {
                    let ext = target_file
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let mime = match ext {
                        "html" => "text/html; charset=utf-8",
                        "js" | "mjs" => "application/javascript; charset=utf-8",
                        "css" => "text/css; charset=utf-8",
                        "json" => "application/json",
                        "png" => "image/png",
                        "svg" => "image/svg+xml",
                        "ico" => "image/x-icon",
                        "woff2" => "font/woff2",
                        _ => "application/octet-stream",
                    };

                    // For index.html, inject session capability token into meta tag
                    let data = if ext == "html" {
                        let html_str = String::from_utf8_lossy(&content);
                        let meta_tag = format!(
                            r#"<meta name="filemind-session-token" content="{}"></head>"#,
                            state.session_token
                        );
                        let modified = html_str.replace("</head>", &meta_tag);
                        modified.into_bytes()
                    } else {
                        content
                    };

                    let resp = Response::from_data(data)
                        .with_status_code(StatusCode(200))
                        .with_header(
                            Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(
                                b"Access-Control-Allow-Origin".as_slice(),
                                allowed_origin.as_bytes(),
                            )
                            .unwrap(),
                        );
                    let _ = request.respond(resp);
                    return;
                }
            }
        }
    }

    // Local Capability Security Check: Mutating / Sensitive operations require X-FileMind-Session-Token
    let requires_auth = (method == Method::Post || method == Method::Delete)
        && path != "/api/health"
        && path != "/api/auth/handshake";

    if requires_auth && !verify_session_capability(&request, &state.session_token) {
        let resp = Response::from_string(r#"{"error":"Unauthorized: Missing or invalid X-FileMind-Session-Token capability header"}"#)
            .with_status_code(StatusCode(401))
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
            .with_header(Header::from_bytes(b"Access-Control-Allow-Origin".as_slice(), allowed_origin.as_bytes()).unwrap());
        let _ = request.respond(resp);
        return;
    }

    let (status_code, body_json) = match (method.clone(), path.as_str()) {
        (Method::Get, "/api/health") => (200, json!({ "status": "ok", "version": "0.2.0" })),

        (Method::Get, "/api/auth/handshake") => {
            (200, json!({ "session_token": state.session_token }))
        }

        (Method::Get, "/api/roots") => {
            let volumes = enumerate_all_volumes();
            let roots_dto: Vec<VolumeRootDto> = volumes
                .into_iter()
                .map(|v| VolumeRootDto {
                    id: format!("root-{}", v.drive_letter.to_lowercase()),
                    drive_letter: v.drive_letter.clone(),
                    label: v.volume_label,
                    filesystem: v.filesystem_type,
                    cluster_size_bytes: v.cluster_size_bytes,
                    total_bytes: v.total_bytes,
                    free_bytes: v.free_bytes,
                    used_bytes: if v.total_bytes >= v.free_bytes {
                        v.total_bytes - v.free_bytes
                    } else {
                        0
                    },
                    is_system_drive: v.is_system_drive,
                    last_scanned_at: "2026-10-28T12:00:00Z".to_string(),
                    scan_freshness: "fresh".to_string(),
                })
                .collect();
            (200, json!(roots_dto))
        }

        (Method::Post, "/api/scan/start") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let req: Result<StartScanRequest, _> = serde_json::from_str(&body);

            match req {
                Ok(start_req) => {
                    let target_path = PathBuf::from(&start_req.path);
                    if !target_path.exists() {
                        (
                            400,
                            json!({ "error": format!("Scan root does not exist: '{}'", start_req.path) }),
                        )
                    } else {
                        let cancel_token = Arc::new(AtomicBool::new(false));
                        let session_id = format!(
                            "scan_{}",
                            SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or(0)
                        );

                        // Set active scan
                        {
                            let mut active =
                                state.active_scan.lock().unwrap_or_else(|p| p.into_inner());
                            *active = Some(ActiveScanState {
                                session_id: session_id.clone(),
                                root_path: start_req.path.clone(),
                                cancel_token: Arc::clone(&cancel_token),
                            });
                        }

                        // Reset progress
                        {
                            let mut prog = state
                                .latest_progress
                                .lock()
                                .unwrap_or_else(|p| p.into_inner());
                            *prog = ScanProgressState {
                                percent: 0.0,
                                current_path: start_req.path.clone(),
                                files_count: 0,
                                bytes_count: 0,
                                directories_count: 0,
                                errors_count: 0,
                                elapsed_ms: 0,
                                is_scanning: true,
                                session_id: Some(session_id.clone()),
                            };
                        }

                        let state_bg = Arc::clone(&state);
                        let exclusions = start_req.excluded_patterns.unwrap_or_default();
                        let sid = session_id.clone();

                        thread::spawn(move || {
                            let started_at = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);

                            let state_progress = Arc::clone(&state_bg);
                            let progress_cb = Arc::new(move |snap: ScanProgressState| {
                                if let Ok(mut p) = state_progress.latest_progress.try_lock() {
                                    *p = snap;
                                }
                            });

                            let scan_result = execute_scan(
                                &target_path,
                                exclusions,
                                cancel_token,
                                Some(progress_cb),
                                Some(sid.clone()),
                            );

                            let completed_at = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);

                            // Save to SQLite
                            if let Ok(mut conn) = state_bg.db.get_connection() {
                                let _ = Database::save_scan_result(
                                    &mut conn,
                                    &sid,
                                    &scan_result,
                                    started_at,
                                    completed_at,
                                );
                            }

                            // Finalize active scan and progress
                            {
                                let mut active = state_bg
                                    .active_scan
                                    .lock()
                                    .unwrap_or_else(|p| p.into_inner());
                                *active = None;
                            }
                            {
                                let mut prog = state_bg
                                    .latest_progress
                                    .lock()
                                    .unwrap_or_else(|p| p.into_inner());
                                prog.is_scanning = false;
                                prog.percent = 100.0;
                            }
                        });

                        (200, json!({ "status": "started", "sessionId": session_id }))
                    }
                }
                Err(e) => (
                    400,
                    json!({ "error": format!("Invalid JSON request: {}", e) }),
                ),
            }
        }

        (Method::Post, "/api/scan/cancel") => {
            let mut active = state.active_scan.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(scan) = active.take() {
                scan.cancel_token.store(true, Ordering::Relaxed);
                let mut prog = state
                    .latest_progress
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());
                prog.is_scanning = false;
                (
                    200,
                    json!({ "status": "cancelled", "sessionId": scan.session_id }),
                )
            } else {
                (200, json!({ "status": "not_running" }))
            }
        }

        (Method::Get, "/api/scan/progress") => {
            let prog = state
                .latest_progress
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .clone();
            (200, json!(prog))
        }

        (Method::Get, "/api/scan/summary") => {
            let root = query_params.get("root").map(|s| s.as_str());
            match state.db.get_connection() {
                Ok(conn) => match Database::get_latest_session(&conn, root) {
                    Ok(Some(session)) => {
                        let errors = Database::get_session_errors(&conn, &session.session_id, 20)
                            .unwrap_or_default();
                        let vol_info = query_volume_info(Path::new(&session.root_path))
                            .unwrap_or_else(|_| crate::adapter::volume::VolumeInfo {
                                root_path: session.root_path.clone(),
                                drive_letter: "C".to_string(),
                                volume_label: "Local Disk".to_string(),
                                filesystem_type: "NTFS".to_string(),
                                cluster_size_bytes: 4096,
                                total_bytes: 0,
                                free_bytes: 0,
                                available_bytes: 0,
                                is_system_drive: false,
                                volume_serial: 0,
                            });

                        let dummy_result = crate::scanner::walker::ScanResult {
                            root_path: session.root_path.clone(),
                            is_cancelled: session.status == crate::models::SessionStatus::Cancelled,
                            files: vec![],
                            directories: vec![],
                            errors: vec![],
                            total_files: session.total_files,
                            total_directories: session.total_directories,
                            total_logical_bytes: session.logical_size_bytes,
                            total_physical_bytes: session.physical_size_bytes,
                            cluster_size_bytes: vol_info.cluster_size_bytes,
                            volume_total_bytes: vol_info.total_bytes,
                            volume_free_bytes: vol_info.free_bytes,
                        };

                        let accounting = StorageAccountant::compute(&dummy_result);

                        let summary = ScanSummaryDto {
                            session,
                            storage_accounting: accounting,
                            recent_errors: errors,
                        };
                        (200, json!(summary))
                    }
                    Ok(None) => (404, json!({ "error": "No scan sessions found" })),
                    Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                },
                Err(e) => (
                    500,
                    json!({ "error": format!("Failed to connect to database: {}", e) }),
                ),
            }
        }

        (Method::Get, "/api/scan/sessions") => match state.db.get_connection() {
            Ok(conn) => match Database::list_sessions(&conn) {
                Ok(sessions) => (200, json!(sessions)),
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            },
            Err(e) => (
                500,
                json!({ "error": format!("Failed to connect to database: {}", e) }),
            ),
        },

        (Method::Get, "/api/files") => {
            let mut query = FileListQuery::default();
            query.session_id = query_params.get("sessionId").cloned();
            query.path_prefix = query_params.get("prefix").cloned();
            query.category = query_params.get("category").cloned();
            query.extension = query_params.get("extension").cloned();
            query.search = query_params.get("search").cloned();
            query.sort_by = query_params.get("sortBy").cloned();
            query.sort_order = query_params.get("sortOrder").cloned();
            query.limit = query_params.get("limit").and_then(|s| s.parse().ok());
            query.offset = query_params.get("offset").and_then(|s| s.parse().ok());

            match state.db.get_connection() {
                Ok(conn) => match Database::query_files(&conn, &query) {
                    Ok(resp) => (200, json!(resp)),
                    Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                },
                Err(e) => (
                    500,
                    json!({ "error": format!("Failed to connect to database: {}", e) }),
                ),
            }
        }

        (Method::Get, "/api/folders/ranking") => {
            let limit: u32 = query_params
                .get("limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(20);
            match state.db.get_connection() {
                Ok(conn) => {
                    let session_id = match query_params.get("sessionId") {
                        Some(sid) => sid.clone(),
                        None => Database::get_latest_session(&conn, None)
                            .ok()
                            .flatten()
                            .map(|s| s.session_id)
                            .unwrap_or_default(),
                    };

                    match Database::get_top_folders(&conn, &session_id, limit) {
                        Ok(rankings) => (200, json!(rankings)),
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    }
                }
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        (Method::Get, "/api/categories") => match state.db.get_connection() {
            Ok(conn) => {
                let session_id = match query_params.get("sessionId") {
                    Some(sid) => sid.clone(),
                    None => Database::get_latest_session(&conn, None)
                        .ok()
                        .flatten()
                        .map(|s| s.session_id)
                        .unwrap_or_default(),
                };

                match Database::get_category_breakdown(&conn, &session_id) {
                    Ok(breakdown) => (200, json!(breakdown)),
                    Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                }
            }
            Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
        },

        (Method::Get, "/api/large-files") => {
            let limit: u32 = query_params
                .get("limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            let mut query = FileListQuery::default();
            query.session_id = query_params.get("sessionId").cloned();
            query.sort_by = Some("size".to_string());
            query.sort_order = Some("desc".to_string());
            query.limit = Some(limit);

            match state.db.get_connection() {
                Ok(conn) => match Database::query_files(&conn, &query) {
                    Ok(resp) => (200, json!(resp.files)),
                    Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                },
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        (Method::Get, "/api/errors") => {
            let limit: u32 = query_params
                .get("limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            match state.db.get_connection() {
                Ok(conn) => {
                    let session_id = match query_params.get("sessionId") {
                        Some(sid) => sid.clone(),
                        None => Database::get_latest_session(&conn, None)
                            .ok()
                            .flatten()
                            .map(|s| s.session_id)
                            .unwrap_or_default(),
                    };

                    match Database::get_session_errors(&conn, &session_id, limit) {
                        Ok(errors) => (200, json!(errors)),
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    }
                }
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        (Method::Post, "/api/actions/open-folder") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(target) = val.get("path").and_then(|v| v.as_str()) {
                    match open_containing_folder(target) {
                        Ok(()) => (200, json!({ "status": "opened", "path": target })),
                        Err(e) => (400, json!({ "error": e })),
                    }
                } else {
                    (
                        400,
                        json!({ "error": "Missing 'path' parameter in request" }),
                    )
                }
            } else {
                (400, json!({ "error": "Invalid JSON body" }))
            }
        }

        (Method::Post, "/api/reconcile") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let (Some(sid), Some(root)) = (
                    val.get("sessionId").and_then(|v| v.as_str()),
                    val.get("rootPath").and_then(|v| v.as_str()),
                ) {
                    match state.db.get_connection() {
                        Ok(mut conn) => {
                            match ReconciliationScanner::reconcile(&mut conn, sid, Path::new(root))
                            {
                                Ok(report) => (200, json!(report)),
                                Err(e) => (
                                    500,
                                    json!({ "error": format!("Reconciliation failed: {}", e) }),
                                ),
                            }
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    }
                } else {
                    (
                        400,
                        json!({ "error": "Missing sessionId or rootPath in request" }),
                    )
                }
            } else {
                (400, json!({ "error": "Invalid JSON body" }))
            }
        }

        // -------------------------------------------------------------------
        // Phase 3: Duplicate Detection Routes
        // -------------------------------------------------------------------
        (Method::Get, "/api/duplicates") => {
            match state.db.get_connection() {
                Ok(conn) => {
                    let mut req = DuplicateScanRequest::default();
                    req.session_id = query_params.get("sessionId").cloned();
                    req.min_size_bytes = query_params
                        .get("minSizeBytes")
                        .and_then(|s| s.parse().ok());
                    req.max_groups = query_params.get("maxGroups").and_then(|s| s.parse().ok());
                    req.verify_byte_by_byte = query_params
                        .get("verifyBytes")
                        .map(|s| s == "true" || s == "1");

                    // If no sessionId specified, pick latest session
                    if req.session_id.is_none() {
                        req.session_id = Database::get_latest_session(&conn, None)
                            .ok()
                            .flatten()
                            .map(|s| s.session_id);
                    }

                    match crate::duplicate::pipeline::DuplicatePipeline::execute(&conn, &req) {
                        Ok(res) => (200, json!(res)),
                        Err(e) => (
                            500,
                            json!({ "error": format!("Duplicate detection failed: {}", e) }),
                        ),
                    }
                }
                Err(e) => (
                    500,
                    json!({ "error": format!("Database connection error: {}", e) }),
                ),
            }
        }

        (Method::Post, "/api/duplicates/verify-group") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(req) = serde_json::from_str::<VerifyGroupRequest>(&body) {
                if req.file_paths.len() < 2 {
                    (
                        400,
                        json!({ "error": "At least 2 file paths required for comparison" }),
                    )
                } else {
                    let base_path = PathBuf::from(&req.file_paths[0]);
                    let mut all_match = true;

                    for other in req.file_paths.iter().skip(1) {
                        match crate::duplicate::hashing::verify_byte_by_byte(
                            &base_path,
                            Path::new(other),
                        ) {
                            Ok(true) => {}
                            _ => {
                                all_match = false;
                                break;
                            }
                        }
                    }

                    let resp = VerifyGroupResponse {
                        group_id: req.group_id,
                        confidence: if all_match {
                            DuplicateConfidence::ByteForByteVerified
                        } else {
                            DuplicateConfidence::CryptographicHashMatch
                        },
                        verified_byte_for_byte: all_match,
                        message: if all_match {
                            "100% byte-for-byte verified identical stream".to_string()
                        } else {
                            "Byte-for-byte stream mismatch detected".to_string()
                        },
                    };
                    (200, json!(resp))
                }
            } else {
                (400, json!({ "error": "Invalid VerifyGroupRequest JSON" }))
            }
        }

        // -------------------------------------------------------------------
        // Phase 3: Metadata Search & Full-Text Content Search
        // -------------------------------------------------------------------
        (Method::Get, "/api/search") => {
            let mut query = FileListQuery::default();
            query.session_id = query_params.get("sessionId").cloned();
            query.search = query_params
                .get("q")
                .or_else(|| query_params.get("search"))
                .cloned();
            query.category = query_params.get("category").cloned();
            query.extension = query_params.get("extension").cloned();
            query.path_prefix = query_params.get("prefix").cloned();
            query.min_size = query_params.get("minSize").and_then(|s| s.parse().ok());
            query.max_size = query_params.get("maxSize").and_then(|s| s.parse().ok());
            query.sort_by = query_params.get("sortBy").cloned();
            query.sort_order = query_params.get("sortOrder").cloned();
            query.limit = query_params.get("limit").and_then(|s| s.parse().ok());
            query.offset = query_params.get("offset").and_then(|s| s.parse().ok());

            match state.db.get_connection() {
                Ok(conn) => {
                    match crate::search::metadata::MetadataSearchEngine::search(&conn, &query) {
                        Ok(resp) => (200, json!(resp)),
                        Err(e) => (
                            500,
                            json!({ "error": format!("Search query error: {}", e) }),
                        ),
                    }
                }
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        (Method::Get, "/api/search/content") => {
            let q = query_params.get("q").cloned().unwrap_or_default();
            let limit = query_params.get("limit").and_then(|s| s.parse().ok());
            let offset = query_params.get("offset").and_then(|s| s.parse().ok());

            let query = ContentSearchQuery {
                query: q,
                extension: query_params.get("extension").cloned(),
                path_prefix: query_params.get("prefix").cloned(),
                limit,
                offset,
            };

            match state.db.get_connection() {
                Ok(conn) => match crate::search::fts::FtsEngine::search_content(&conn, &query) {
                    Ok(resp) => (200, json!(resp)),
                    Err(e) => (
                        500,
                        json!({ "error": format!("Content search error: {}", e) }),
                    ),
                },
                Err(e) => (
                    500,
                    json!({ "error": format!("Database connection error: {}", e) }),
                ),
            }
        }

        (Method::Post, "/api/content/index") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let val =
                serde_json::from_str::<serde_json::Value>(&body).unwrap_or(serde_json::Value::Null);

            let session_id = val.get("sessionId").and_then(|v| v.as_str());
            let max_files = val.get("maxFiles").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

            match state.db.get_connection() {
                Ok(mut conn) => match crate::search::fts::FtsEngine::index_documents(
                    &mut conn, session_id, max_files,
                ) {
                    Ok(progress) => (200, json!(progress)),
                    Err(e) => (
                        500,
                        json!({ "error": format!("Document indexing failed: {}", e) }),
                    ),
                },
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        (Method::Get, "/api/content/status") => match state.db.get_connection() {
            Ok(conn) => match crate::search::fts::FtsEngine::get_status(&conn) {
                Ok(status) => (200, json!(status)),
                Err(e) => (
                    500,
                    json!({ "error": format!("Content status query error: {}", e) }),
                ),
            },
            Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
        },

        (Method::Post, "/api/content/clear") => match state.db.get_connection() {
            Ok(mut conn) => match crate::search::fts::FtsEngine::clear(&mut conn) {
                Ok(()) => (200, json!({ "status": "cleared" })),
                Err(e) => (
                    500,
                    json!({ "error": format!("Failed to clear content index: {}", e) }),
                ),
            },
            Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
        },

        (Method::Post, "/api/safety/evaluate") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<SafetyEvaluationRequest>(&body_str) {
                    Ok(req) => {
                        let op = req.operation.as_deref().unwrap_or("evaluate");
                        let assessments =
                            crate::safety::SafetyGuardian::evaluate_batch(&req.paths, op);

                        // Persist evaluation audits in SQLite
                        if let Ok(conn) = state.db.get_connection() {
                            for a in &assessments {
                                let _ = crate::safety::AuditLogger::record_audit(
                                    &conn,
                                    &SafetyAuditRecordDto {
                                        audit_id: 0,
                                        target_path: a.target_path.clone(),
                                        canonical_path: a.canonical_path.clone(),
                                        operation: op.to_string(),
                                        classification: a.classification,
                                        is_permitted: a.is_permitted,
                                        rule_matches: a.rule_matches.clone(),
                                        evidence: a.evidence.clone(),
                                        block_reasons: a
                                            .block_reason
                                            .clone()
                                            .map(|r| vec![r])
                                            .unwrap_or_default(),
                                        toctou_verified: false,
                                        executed: false,
                                        execution_result: None,
                                        timestamp_epoch_ms: a.evaluated_at_epoch_ms,
                                    },
                                );
                            }
                        }

                        let total = assessments.len();
                        let permitted = assessments.iter().filter(|a| a.is_permitted).count();
                        let review = assessments
                            .iter()
                            .filter(|a| a.classification == SafetyClassification::ReviewRequired)
                            .count();
                        let blocked = total - permitted;

                        (
                            200,
                            json!(SafetyEvaluationResponse {
                                assessments,
                                total_evaluated: total,
                                permitted_count: permitted,
                                review_required_count: review,
                                blocked_count: blocked,
                            }),
                        )
                    }
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Get, "/api/safety/audit") => {
            let limit = query_params
                .get("limit")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(50);

            match state.db.get_connection() {
                Ok(conn) => match crate::safety::AuditLogger::fetch_recent(&conn, limit) {
                    Ok(records) => (200, json!(records)),
                    Err(e) => (
                        500,
                        json!({ "error": format!("Failed to fetch safety audit log: {}", e) }),
                    ),
                },
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        (Method::Post, "/api/safety/execute") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<ExecuteCleanupRequest>(&body_str) {
                    Ok(req) => {
                        let conn_res = state.db.get_connection();
                        let mut results = Vec::new();
                        let mut succeeded = 0;
                        let mut failed = 0;

                        for p in &req.paths {
                            let path_obj = Path::new(p);
                            let res = match &conn_res {
                                Ok(conn) => {
                                    crate::safety::SafetyGuardian::execute_guarded_operation(
                                        path_obj,
                                        &req.operation,
                                        None,
                                        Some(conn),
                                    )
                                }
                                Err(_) => crate::safety::SafetyGuardian::execute_guarded_operation(
                                    path_obj,
                                    &req.operation,
                                    None,
                                    None,
                                ),
                            };

                            if res.success {
                                succeeded += 1;
                            } else {
                                failed += 1;
                            }
                            results.push(res);
                        }

                        (
                            200,
                            json!(ExecuteCleanupResponse {
                                results,
                                total_succeeded: succeeded,
                                total_failed: failed,
                                total_bytes_reclaimed: 0,
                            }),
                        )
                    }
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Get, "/api/safety/rules") => (
            200,
            json!([
                {
                    "ruleId": "WIN-SYS-ROOT",
                    "name": "Windows OS & System32 Protected Tree",
                    "riskTier": "danger",
                    "pattern": "C:\\Windows\\* | System32 | SysWOW64 | WinSxS",
                    "description": "Hardcoded protection blocking deletion of Windows root, kernel libraries, and system manifests",
                    "isEnabled": true
                },
                {
                    "ruleId": "WIN-PROGRAM-FILES",
                    "name": "Program Files Application Binaries",
                    "riskTier": "danger",
                    "pattern": "C:\\Program Files\\* | C:\\Program Files (x86)\\*",
                    "description": "Installed application binaries and components must be removed via official uninstallers",
                    "isEnabled": true
                },
                {
                    "ruleId": "WSL-VHDX-IMAGE",
                    "name": "Linux Subsystem & WSL2 Virtual Hard Disks",
                    "riskTier": "danger",
                    "pattern": "*.vhdx | ext4.vhdx | \\\\wsl$\\*",
                    "description": "Blocks deletion of virtual hard disks and mounted filesystems belonging to WSL Linux distributions",
                    "isEnabled": true
                },
                {
                    "ruleId": "DEV-SECRET-CREDENTIAL",
                    "name": "Secrets, Private Keys & Environment Variables",
                    "riskTier": "danger",
                    "pattern": ".env* | id_rsa* | *.pem | *.key | credentials.json",
                    "description": "Strictly prevents deletion of API keys, private certificates, and environment configuration",
                    "isEnabled": true
                },
                {
                    "ruleId": "DEV-REGENERABLE-TARGET",
                    "name": "Developer Regenerable Build Directories",
                    "riskTier": "safe",
                    "pattern": "target/ | node_modules/ | build/ | dist/",
                    "description": "Allows cleanup only when confirmed by parent package manifest (Cargo.toml/package.json) and lockfile",
                    "isEnabled": true
                },
                {
                    "ruleId": "WIN32-TOCTOU-GUARD",
                    "name": "Time-Of-Check to Time-Of-Use Revalidation",
                    "riskTier": "caution",
                    "pattern": "BY_HANDLE_FILE_INFORMATION | Non-traversing open",
                    "description": "Revalidates NTFS file ID, volume serial, and size immediately prior to execution to stop symlink races",
                    "isEnabled": true
                }
            ]),
        ),

        (Method::Post, "/api/cleanup/plan") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<CreatePlanRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            match crate::cleanup::CleanupPlanner::create_plan(&req, &conn) {
                                Ok(plan) => (200, json!(plan)),
                                Err(e) => (
                                    500,
                                    json!({ "error": format!("Plan generation failed: {}", e) }),
                                ),
                            }
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Post, "/api/cleanup/dry-run") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<DryRunRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            match crate::cleanup::DryRunEngine::execute_dry_run(&req.plan_id, &conn)
                            {
                                Ok(resp) => (200, json!(resp)),
                                Err(e) => {
                                    (500, json!({ "error": format!("Dry run failed: {}", e) }))
                                }
                            }
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Post, "/api/cleanup/execute") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<ExecutePlanRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => match crate::cleanup::CleanupExecutor::execute_plan(
                            &req.plan_id,
                            req.selected_item_ids.as_deref(),
                            &state.quarantine,
                            &conn,
                        ) {
                            Ok(resp) => (200, json!(resp)),
                            Err(e) => (
                                500,
                                json!({ "error": format!("Plan execution failed: {}", e) }),
                            ),
                        },
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Get, "/api/quarantine/records") => match state.db.get_connection() {
            Ok(conn) => match state.quarantine.list_all(&conn) {
                Ok(records) => (200, json!(records)),
                Err(e) => (
                    500,
                    json!({ "error": format!("Failed to list quarantine vault: {}", e) }),
                ),
            },
            Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
        },

        (Method::Post, "/api/quarantine/restore") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<RestoreRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            let mut results = Vec::new();
                            let mut restored = 0;
                            let mut failed = 0;

                            let alt = req.alternate_destination.as_deref().map(Path::new);
                            for q_id in &req.quarantine_ids {
                                let res = state.quarantine.restore_target(q_id, alt, &conn);
                                if res.success {
                                    restored += 1;
                                } else {
                                    failed += 1;
                                }
                                results.push(res);
                            }

                            (
                                200,
                                json!(RestoreResponse {
                                    results,
                                    total_restored: restored,
                                    total_failed: failed,
                                }),
                            )
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Post, "/api/quarantine/purge") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<PurgeQuarantineRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            let mut count = 0;
                            let mut freed_bytes = 0u64;

                            for q_id in &req.quarantine_ids {
                                if let Ok(bytes) = state.quarantine.purge_target(q_id, &conn) {
                                    count += 1;
                                    freed_bytes += bytes;
                                }
                            }

                            (
                                200,
                                json!(PurgeResponse {
                                    purged_count: count,
                                    bytes_permanently_freed: freed_bytes,
                                }),
                            )
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Post, "/api/copilot/query") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<CopilotQueryRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            let resp = crate::copilot::CopilotDispatcher::handle_query(&req, &conn);
                            (200, json!(resp))
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Get, "/api/copilot/models") => {
            let endpoint = query_params
                .get("endpoint")
                .map(|s| s.as_str())
                .unwrap_or("http://127.0.0.1:11434");
            let resp = crate::copilot::OllamaClient::check_status(endpoint);
            (200, json!(resp))
        }

        (Method::Post, "/api/copilot/tool") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<ExecuteToolRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            let resp = crate::copilot::CopilotToolRunner::execute(
                                req.tool,
                                &req.arguments,
                                &conn,
                            );
                            (200, json!(resp))
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        // ==========================================
        // Phase 7: Automation & Developer Routes
        // ==========================================
        (Method::Get, "/api/automation/rules") => match state.db.get_connection() {
            Ok(conn) => match Database::list_automation_rules(&conn) {
                Ok(rules) => (200, json!(rules)),
                Err(e) => (
                    500,
                    json!({ "error": format!("Failed to list automation rules: {}", e) }),
                ),
            },
            Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
        },

        (Method::Post, "/api/automation/rules") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<CreateAutomationRuleRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            let now = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);
                            let rule_id = format!("rule_{}", now);

                            let rule = AutomationRuleDto {
                                rule_id: rule_id.clone(),
                                name: req.name,
                                description: req.description,
                                is_enabled: true,
                                trigger: req.trigger,
                                conditions: req.conditions,
                                actions: req.actions,
                                scope_paths: req.scope_paths,
                                exclusion_paths: req.exclusion_paths.unwrap_or_default(),
                                schedule_expression: req
                                    .schedule_expression
                                    .unwrap_or_else(|| "Manual / On Demand".to_string()),
                                requires_explicit_approval: req
                                    .requires_explicit_approval
                                    .unwrap_or(true),
                                dry_run_mode: req.dry_run_mode.unwrap_or(true),
                                retry_policy: req.retry_policy.unwrap_or_default(),
                                execution_limits: req.execution_limits.unwrap_or_default(),
                                last_run_at: None,
                                last_status: None,
                                created_at: now,
                                updated_at: now,
                            };

                            match Database::insert_automation_rule(&conn, &rule) {
                                Ok(_) => (201, json!(rule)),
                                Err(e) => (
                                    500,
                                    json!({ "error": format!("Failed to insert rule: {}", e) }),
                                ),
                            }
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        (Method::Get, "/api/automation/history") => match state.db.get_connection() {
            Ok(conn) => {
                let rule_id = query_params.get("ruleId").map(|s| s.as_str());
                let limit = query_params
                    .get("limit")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(50);

                match Database::list_automation_history(&conn, rule_id, limit) {
                    Ok(history) => (200, json!(history)),
                    Err(e) => (
                        500,
                        json!({ "error": format!("Failed to retrieve history: {}", e) }),
                    ),
                }
            }
            Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
        },

        (Method::Get, "/api/automation/scheduler/status") => {
            let status = state.scheduler.get_status();
            (200, json!(status))
        }

        (Method::Get, "/api/developer/projects") => {
            let scan_root = if let Some(p) = query_params.get("path") {
                PathBuf::from(p)
            } else {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            };

            let projects =
                crate::developer::DeveloperWorkspaceScanner::scan_workspace(&scan_root, 5);
            (200, json!(projects))
        }

        (Method::Post, "/api/developer/clean") => {
            let mut body_str = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body_str) {
                (
                    400,
                    json!({ "error": format!("Failed to read request body: {}", e) }),
                )
            } else {
                match serde_json::from_str::<CleanProjectArtifactsRequest>(&body_str) {
                    Ok(req) => match state.db.get_connection() {
                        Ok(conn) => {
                            let scan_root =
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                            let all_projects =
                                crate::developer::DeveloperWorkspaceScanner::scan_workspace(
                                    &scan_root, 5,
                                );

                            let selected: Vec<_> = all_projects
                                .into_iter()
                                .filter(|p| req.project_ids.contains(&p.id))
                                .collect();

                            let mut total_staged_bytes: u64 = 0;
                            let mut staged_count: usize = 0;

                            for proj in selected {
                                for art in proj.artifacts {
                                    if art.risk_tier == RiskTier::Safe {
                                        total_staged_bytes += art.size_bytes;
                                        staged_count += 1;
                                    }
                                }
                            }

                            // Generate a reviewable cleanup plan for developer artifacts
                            let plan_req = CreatePlanRequest {
                                goal: "clean_developer_caches".to_string(),
                                target_bytes: Some(total_staged_bytes),
                                folder_path: Some(scan_root.to_string_lossy().to_string()),
                                session_id: None,
                                selected_paths: None,
                            };

                            match crate::cleanup::CleanupPlanner::create_plan(&plan_req, &conn) {
                                Ok(plan) => {
                                    let resp = CleanProjectArtifactsResponse {
                                        plan_id: plan.plan_id,
                                        staged_artifacts_count: staged_count,
                                        total_bytes: total_staged_bytes,
                                        requires_explicit_approval: true,
                                    };
                                    (200, json!(resp))
                                }
                                Err(e) => (
                                    500,
                                    json!({ "error": format!("Failed to assemble plan: {}", e) }),
                                ),
                            }
                        }
                        Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                    },
                    Err(e) => (
                        400,
                        json!({ "error": format!("Invalid JSON request: {}", e) }),
                    ),
                }
            }
        }

        // Dynamic Parameterized Automation Routes
        _ if path.starts_with("/api/automation/rules/")
            && path.ends_with("/toggle")
            && method == Method::Post =>
        {
            let prefix = "/api/automation/rules/";
            let suffix = "/toggle";
            let rule_id = &path[prefix.len()..path.len() - suffix.len()];

            let mut body_str = String::new();
            let _ = request.as_reader().read_to_string(&mut body_str);
            let val: serde_json::Value =
                serde_json::from_str(&body_str).unwrap_or(json!({ "enabled": true }));
            let enabled = val.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            match state.db.get_connection() {
                Ok(conn) => match Database::toggle_automation_rule(&conn, rule_id, enabled) {
                    Ok(_) => (
                        200,
                        json!({ "success": true, "ruleId": rule_id, "enabled": enabled }),
                    ),
                    Err(e) => (500, json!({ "error": format!("Toggle failed: {}", e) })),
                },
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        _ if path.starts_with("/api/automation/rules/")
            && path.ends_with("/run")
            && method == Method::Post =>
        {
            let prefix = "/api/automation/rules/";
            let suffix = "/run";
            let rule_id = &path[prefix.len()..path.len() - suffix.len()];

            let mut body_str = String::new();
            let _ = request.as_reader().read_to_string(&mut body_str);
            let req: TriggerRuleRequest =
                serde_json::from_str(&body_str).unwrap_or(TriggerRuleRequest {
                    dry_run_override: None,
                });

            match state.db.get_connection() {
                Ok(conn) => match Database::get_automation_rule(&conn, rule_id) {
                    Ok(Some(rule)) => {
                        let hist = crate::automation::AutomationEngine::execute_rule(
                            &conn,
                            &rule,
                            &state.quarantine,
                            req.dry_run_override,
                            Some("manual_api"),
                        );
                        (200, json!(hist))
                    }
                    Ok(None) => (
                        404,
                        json!({ "error": format!("Rule not found: {}", rule_id) }),
                    ),
                    Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
                },
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        _ if path.starts_with("/api/automation/rules/") && method == Method::Delete => {
            let prefix = "/api/automation/rules/";
            let rule_id = &path[prefix.len()..];

            match state.db.get_connection() {
                Ok(conn) => match Database::delete_automation_rule(&conn, rule_id) {
                    Ok(_) => (200, json!({ "success": true, "deletedRuleId": rule_id })),
                    Err(e) => (500, json!({ "error": format!("Delete failed: {}", e) })),
                },
                Err(e) => (500, json!({ "error": format!("Database error: {}", e) })),
            }
        }

        _ => (404, json!({ "error": "Endpoint not found" })),
    };

    let body_bytes = serde_json::to_vec(&body_json).unwrap_or_default();
    let resp = Response::from_data(body_bytes)
        .with_status_code(StatusCode(status_code))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(
            Header::from_bytes(
                b"Access-Control-Allow-Origin".as_slice(),
                allowed_origin.as_bytes(),
            )
            .unwrap(),
        );

    let _ = request.respond(resp);
}

fn parse_url(url: &str) -> (String, HashMap<String, String>) {
    let mut parts = url.splitn(2, '?');
    let path = parts.next().unwrap_or("/").to_string();
    let mut params = HashMap::new();

    if let Some(query) = parts.next() {
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                let decoded_val = urlencoding_decode(v);
                params.insert(k.to_string(), decoded_val);
            }
        }
    }

    (path, params)
}

fn urlencoding_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(c1), Some(c2)) = (h1, h2) {
                let hex_str = format!("{}{}", c1, c2);
                if let Ok(byte_val) = u8::from_str_radix(&hex_str, 16) {
                    if byte_val != 0 {
                        bytes.push(byte_val);
                    }
                    continue;
                }
            }
        } else if ch == '+' {
            bytes.push(b' ');
        } else {
            let mut buf = [0u8; 4];
            let encoded = ch.encode_utf8(&mut buf);
            bytes.extend_from_slice(encoded.as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn recursive_url_decode(s: &str) -> String {
    let mut current = s.to_string();
    for _ in 0..3 {
        let decoded = urlencoding_decode(&current);
        if decoded == current {
            break;
        }
        current = decoded;
    }
    current
}
