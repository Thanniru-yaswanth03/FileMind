use filemind_core::cleanup::{CleanupExecutor, CleanupPlanner, DryRunEngine, QuarantineManager};
use filemind_core::db::Database;
use filemind_core::models::{CreatePlanRequest, PlanStatus};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_plan_generation_with_target_budget_and_explanation() {
    let conn = Database::open_in_memory().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create a mock Cargo project with target directory
    File::create(root.join("Cargo.toml")).unwrap().write_all(b"[package]\nname = \"demo\"\n").unwrap();
    File::create(root.join("Cargo.lock")).unwrap().write_all(b"# lock\n").unwrap();
    let target_dir = root.join("target");
    fs::create_dir_all(&target_dir).unwrap();
    File::create(target_dir.join("demo.exe")).unwrap().write_all(&vec![0u8; 100_000]).unwrap(); // 100 KB

    let req = CreatePlanRequest {
        goal: "clean_folder".to_string(),
        target_bytes: Some(10 * 1024 * 1024 * 1024), // 10 GB requested
        folder_path: Some(root.to_string_lossy().to_string()),
        selected_paths: None,
        session_id: None,
    };

    let plan = CleanupPlanner::create_plan(&req, &conn).unwrap();
    assert_eq!(plan.goal_type, "clean_folder");
    assert!(!plan.items.is_empty());

    // 10 GB requested vs ~100 KB available -> must report unattainable safely
    assert!(!plan.is_target_attainable_safely);
    assert!(plan.unmet_target_explanation.is_some());
    let explanation = plan.unmet_target_explanation.unwrap();
    assert!(explanation.contains("cannot be reached safely"));
}

#[test]
fn test_dry_run_does_not_modify_filesystem() {
    let conn = Database::open_in_memory().unwrap();
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("dry_run_sample.txt");
    File::create(&file_path).unwrap().write_all(b"sample content for dry run").unwrap();

    let initial_meta = fs::metadata(&file_path).unwrap();

    let req = CreatePlanRequest {
        goal: "user_custom".to_string(),
        target_bytes: None,
        folder_path: None,
        selected_paths: Some(vec![file_path.to_string_lossy().to_string()]),
        session_id: None,
    };

    let plan = CleanupPlanner::create_plan(&req, &conn).unwrap();
    assert_eq!(plan.items.len(), 1);

    // Execute dry run
    let dry_run = DryRunEngine::execute_dry_run(&plan.plan_id, &conn).unwrap();
    assert!(dry_run.can_execute);
    assert_eq!(dry_run.validated_items_count, 1);
    assert!(dry_run.conflicts_detected.is_empty());

    // Invariant verification: file must be completely untouched!
    assert!(file_path.exists());
    let current_meta = fs::metadata(&file_path).unwrap();
    assert_eq!(initial_meta.len(), current_meta.len());
    assert_eq!(initial_meta.modified().unwrap(), current_meta.modified().unwrap());
}

#[test]
fn test_quarantine_moves_target_and_preserves_metadata() {
    let conn = Database::open_in_memory().unwrap();
    let tmp_src = tempdir().unwrap();
    let tmp_vault = tempdir().unwrap();

    let manager = QuarantineManager::new(Some(tmp_vault.path().to_path_buf()));
    let file_path = tmp_src.path().join("candidate_log.log");
    File::create(&file_path).unwrap().write_all(b"temporary server log line 1\nline 2").unwrap();

    // Quarantine the target
    let record = manager
        .quarantine_target(&file_path, "Cleaned via automated plan", &conn)
        .unwrap();

    // Original must no longer be at source
    assert!(!file_path.exists());

    // Quarantined file must exist in vault
    let vault_file = Path::new(&record.quarantine_path);
    assert!(vault_file.exists());
    assert_eq!(record.status, "quarantined");
    assert_eq!(record.filename, "candidate_log.log");
    assert!(record.size_bytes > 0);

    // Verify record query from DB
    let queried = manager.get_record(&record.quarantine_id, &conn).unwrap().unwrap();
    assert_eq!(queried.quarantine_id, record.quarantine_id);
    assert_eq!(queried.original_path, record.original_path);
}

#[test]
fn test_quarantine_cannot_bypass_safety_protection() {
    let conn = Database::open_in_memory().unwrap();
    let tmp_vault = tempdir().unwrap();
    let manager = QuarantineManager::new(Some(tmp_vault.path().to_path_buf()));

    // Attempt to quarantine protected OS file
    let protected = Path::new(r"C:\Windows\System32\ntoskrnl.exe");
    let res = manager.quarantine_target(protected, "Attempting illegal move", &conn);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains("Quarantine blocked by security policy"));
}

#[test]
fn test_restore_preserves_data_and_rejects_overwrite_conflict() {
    let conn = Database::open_in_memory().unwrap();
    let tmp_src = tempdir().unwrap();
    let tmp_vault = tempdir().unwrap();

    let manager = QuarantineManager::new(Some(tmp_vault.path().to_path_buf()));
    let file_path = tmp_src.path().join("restorable_data.txt");
    let original_bytes = b"important user text content";
    File::create(&file_path).unwrap().write_all(original_bytes).unwrap();

    // 1. Quarantine
    let record = manager
        .quarantine_target(&file_path, "Testing restore flow", &conn)
        .unwrap();
    assert!(!file_path.exists());

    // 2. Restore cleanly
    let restore_res = manager.restore_target(&record.quarantine_id, None, &conn);
    assert!(restore_res.success);
    assert!(file_path.exists());
    assert_eq!(fs::read(&file_path).unwrap(), original_bytes);

    // 3. Now test conflict detection: Attempt to restore again while file exists
    // Move a dummy file back to vault to simulate second restore attempt
    let dummy_vault_file = Path::new(&record.quarantine_path);
    File::create(&dummy_vault_file).unwrap().write_all(b"new data").unwrap();

    let conflict_res = manager.restore_target(&record.quarantine_id, None, &conn);
    assert!(!conflict_res.success);
    assert!(conflict_res.error.is_some());
    let err = conflict_res.error.unwrap();
    assert!(err.contains("already exists"));
    assert!(err.contains("Silently overwriting is prohibited"));
}

#[test]
fn test_purge_quarantine_permanently_removes_vault_file() {
    let conn = Database::open_in_memory().unwrap();
    let tmp_src = tempdir().unwrap();
    let tmp_vault = tempdir().unwrap();

    let manager = QuarantineManager::new(Some(tmp_vault.path().to_path_buf()));
    let file_path = tmp_src.path().join("purge_me.tmp");
    File::create(&file_path).unwrap().write_all(b"temporary").unwrap();

    let record = manager.quarantine_target(&file_path, "To be purged", &conn).unwrap();
    let vault_file = Path::new(&record.quarantine_path);
    assert!(vault_file.exists());

    // Purge
    let freed_bytes = manager.purge_target(&record.quarantine_id, &conn).unwrap();
    assert!(freed_bytes > 0);
    assert!(!vault_file.exists());

    let updated = manager.get_record(&record.quarantine_id, &conn).unwrap().unwrap();
    assert_eq!(updated.status, "purged");
}

#[test]
fn test_cleanup_executor_transactional_execution() {
    let conn = Database::open_in_memory().unwrap();
    let tmp_src = tempdir().unwrap();
    let tmp_vault = tempdir().unwrap();
    let manager = QuarantineManager::new(Some(tmp_vault.path().to_path_buf()));

    // Create Cargo fixture
    let root = tmp_src.path();
    File::create(root.join("Cargo.toml")).unwrap().write_all(b"[package]\nname = \"demo\"\n").unwrap();
    File::create(root.join("Cargo.lock")).unwrap().write_all(b"# lock\n").unwrap();
    let target_dir = root.join("target");
    fs::create_dir_all(&target_dir).unwrap();
    File::create(target_dir.join("build_output.dat")).unwrap().write_all(&vec![1u8; 5000]).unwrap();

    let req = CreatePlanRequest {
        goal: "clean_developer_caches".to_string(),
        target_bytes: None,
        folder_path: None,
        selected_paths: Some(vec![target_dir.to_string_lossy().to_string()]),
        session_id: None,
    };

    let plan = CleanupPlanner::create_plan(&req, &conn).unwrap();
    assert_eq!(plan.items.len(), 1);

    // Dry run
    let dry_run = DryRunEngine::execute_dry_run(&plan.plan_id, &conn).unwrap();
    assert!(dry_run.can_execute);

    // Execute plan
    let exec_res = CleanupExecutor::execute_plan(&plan.plan_id, None, &manager, &conn).unwrap();
    eprintln!("EXEC_RES: {:?}", exec_res);
    assert_eq!(exec_res.status, PlanStatus::Completed);
    assert_eq!(exec_res.succeeded_count, 1);
    assert_eq!(exec_res.failed_count, 0);
    assert_eq!(exec_res.quarantined_count, 1);

    // Target dir must be moved to quarantine
    assert!(!target_dir.exists());
}
