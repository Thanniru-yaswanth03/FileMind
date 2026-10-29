// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::thread;
use filemind_core::db::repository::Database;
use filemind_core::server::{start_server, AppState};

fn main() {
    let port = 41732;
    let db_path = Database::default_app_data_path();
    let state = Arc::new(AppState::new(db_path));

    // Spawn native filemind-core engine in background thread for local WebView requests
    thread::spawn(move || {
        let _ = start_server(port, state);
    });

    tauri::Builder::default()
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running FileMind desktop application");
}

