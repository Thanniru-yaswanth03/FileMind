use std::env;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::SystemTime;

use filemind_core::db::repository::Database;
use filemind_core::db::retention::RetentionManager;
use filemind_core::scanner::walker::execute_scan;
use filemind_core::server::{start_server, AppState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let db_path = Database::default_app_data_path();

    if args.len() > 1 && args[1] == "scan" {
        let scan_target = if args.len() > 2 {
            PathBuf::from(&args[2])
        } else {
            PathBuf::from(r"C:\")
        };

        println!(
            "[FileMind CLI] Starting real filesystem scan on: {:?}",
            scan_target
        );
        let cancel_token = Arc::new(AtomicBool::new(false));
        let progress_cb = Arc::new(|prog: filemind_core::models::ScanProgressState| {
            print!(
                "\r[Scan Progress] {:.1}% | Files: {} | Bytes: {} | Dirs: {} | Errors: {}",
                prog.percent,
                prog.files_count,
                prog.bytes_count,
                prog.directories_count,
                prog.errors_count
            );
        });

        let start_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let result = execute_scan(
            &scan_target,
            vec![],
            cancel_token,
            Some(progress_cb),
            Some("cli_scan".to_string()),
        );

        let end_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        println!("\n[Scan Completed in {} ms]", end_ms - start_ms);
        println!("- Total Files: {}", result.total_files);
        println!("- Total Directories: {}", result.total_directories);
        println!("- Logical Size: {} bytes", result.total_logical_bytes);
        println!(
            "- Allocated Physical Size: {} bytes",
            result.total_physical_bytes
        );
        println!("- Cluster Size: {} bytes", result.cluster_size_bytes);
        println!("- Errors Encountered: {}", result.errors.len());

        // Save to SQLite
        let db = Database::new(db_path.clone());
        let mut conn = db.get_connection()?;
        let session_id = format!("cli_session_{}", start_ms);
        Database::save_scan_result(&mut conn, &session_id, &result, start_ms, end_ms)?;
        println!("[Database] Saved scan results to {:?}", db_path);

        return Ok(());
    }

    if args.len() > 1 && args[1] == "prune" {
        let days = if args.len() > 2 {
            args[2].parse::<u32>().unwrap_or(30)
        } else {
            30
        };
        let db = Database::new(db_path.clone());
        let mut conn = db.get_connection()?;
        let pruned = RetentionManager::prune_older_than_days(&mut conn, days)?;
        println!(
            "[Retention] Pruned {} sessions older than {} days",
            pruned, days
        );
        return Ok(());
    }

    // Default: Run HTTP API server for desktop frontend
    let port: u16 = env::var("FILEMIND_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(41732);

    println!("[FileMind Core Engine v0.2.0]");
    println!("Database path: {:?}", db_path);

    let state = Arc::new(AppState::new(db_path));

    // In desktop standalone mode, automatically launch browser UI unless --no-open is specified
    let should_open = !args.iter().any(|a| a == "--no-open") && env::var("CI").is_err();
    if should_open && args.len() <= 1 {
        let open_url = format!("http://localhost:{}", port);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = std::process::Command::new("explorer.exe")
                .arg(&open_url)
                .spawn();
        });
    }

    start_server(port, state)?;

    Ok(())
}
