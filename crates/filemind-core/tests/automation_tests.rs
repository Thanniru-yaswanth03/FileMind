use std::fs;
use tempfile::tempdir;

use filemind_core::automation::{AutomationEngine, AutomationEventWatcher};
use filemind_core::cleanup::quarantine::QuarantineManager;
use filemind_core::db::repository::Database;
use filemind_core::developer::DeveloperWorkspaceScanner;
use filemind_core::models::*;

#[test]
fn test_automation_rule_persistence_and_seeding() {
    let conn = Database::open_in_memory().unwrap();

    // 1. Verify default rules were seeded
    let rules = Database::list_automation_rules(&conn).unwrap();
    assert!(rules.len() >= 3, "Expected at least 3 default seeded automation rules, found {}", rules.len());

    let low_space = rules.iter().find(|r| r.rule_id == "rule_low_disk_space").unwrap();
    assert_eq!(low_space.name, "Low Disk Space Alert");
    assert!(low_space.is_enabled);

    // 2. Toggle rule status
    Database::toggle_automation_rule(&conn, "rule_low_disk_space", false).unwrap();
    let updated = Database::get_automation_rule(&conn, "rule_low_disk_space").unwrap().unwrap();
    assert!(!updated.is_enabled);

    // 3. Insert custom rule
    let custom_rule = AutomationRuleDto {
        rule_id: "rule_custom_test".to_string(),
        name: "Test Downloads Archive".to_string(),
        description: "Organizes archives".to_string(),
        is_enabled: true,
        trigger: AutomationTrigger::Manual,
        conditions: vec![AutomationCondition::FileTypeMatches {
            file_type: "Archives".to_string(),
        }],
        actions: vec![AutomationAction::NotifyUser {
            title: "Archives detected".to_string(),
            message: "Organized archives".to_string(),
            severity: "info".to_string(),
        }],
        scope_paths: vec!["C:\\Downloads".to_string()],
        exclusion_paths: vec![],
        schedule_expression: "Manual".to_string(),
        requires_explicit_approval: false,
        dry_run_mode: true,
        retry_policy: RetryPolicy::default(),
        execution_limits: ExecutionLimits::default(),
        last_run_at: None,
        last_status: None,
        created_at: 1000,
        updated_at: 1000,
    };

    Database::insert_automation_rule(&conn, &custom_rule).unwrap();
    let fetched = Database::get_automation_rule(&conn, "rule_custom_test").unwrap().unwrap();
    assert_eq!(fetched.name, "Test Downloads Archive");

    // 4. Delete rule
    Database::delete_automation_rule(&conn, "rule_custom_test").unwrap();
    let deleted = Database::get_automation_rule(&conn, "rule_custom_test").unwrap();
    assert!(deleted.is_none());
}

#[test]
fn test_execution_limits_cooldown() {
    let conn = Database::open_in_memory().unwrap();
    let quarantine = QuarantineManager::new(None);

    let mut rule = Database::get_automation_rule(&conn, "rule_low_disk_space").unwrap().unwrap();
    rule.execution_limits.cooldown_seconds = 600; // 10 minute cooldown
    rule.last_run_at = Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64);

    let history = AutomationEngine::execute_rule(&conn, &rule, &quarantine, None, Some("test"));
    assert_eq!(history.status, "blocked");
    assert!(history.result_summary.contains("cooldown period active"));
    assert_eq!(history.files_affected_count, 0);
}

#[test]
fn test_safety_engine_blocks_protected_paths_in_automation() {
    let conn = Database::open_in_memory().unwrap();
    let quarantine = QuarantineManager::new(None);

    // Rule maliciously or mistakenly targeting Windows OS directory
    let protected_rule = AutomationRuleDto {
        rule_id: "rule_unsafe_attempt".to_string(),
        name: "Unsafe System Purge Attempt".to_string(),
        description: "Attempts to target C:\\Windows".to_string(),
        is_enabled: true,
        trigger: AutomationTrigger::Manual,
        conditions: vec![],
        actions: vec![AutomationAction::StageToQuarantine { dry_run: false }],
        scope_paths: vec![r"C:\Windows\System32".to_string()],
        exclusion_paths: vec![],
        schedule_expression: "Manual".to_string(),
        requires_explicit_approval: false,
        dry_run_mode: false,
        retry_policy: RetryPolicy::default(),
        execution_limits: ExecutionLimits {
            max_files_per_run: 10,
            max_bytes_per_run: 1000000,
            cooldown_seconds: 0,
        },
        last_run_at: None,
        last_status: None,
        created_at: 1000,
        updated_at: 1000,
    };

    Database::insert_automation_rule(&conn, &protected_rule).unwrap();

    let history = AutomationEngine::execute_rule(&conn, &protected_rule, &quarantine, None, Some("test"));
    assert_eq!(history.status, "failed");
    assert!(history.result_summary.contains("AUTOMATION STRICTLY BLOCKED"));
    assert_eq!(history.files_affected_count, 0);
    assert_eq!(history.bytes_affected, 0);

    // Verify history log was persisted to SQLite
    let history_logs = Database::list_automation_history(&conn, Some("rule_unsafe_attempt"), 10).unwrap();
    assert_eq!(history_logs.len(), 1);
    assert_eq!(history_logs[0].status, "failed");
}

#[test]
fn test_developer_workspace_scanner_manifest_detection() {
    let temp = tempdir().unwrap();
    let base = temp.path();

    // 1. Create a synthetic Node.js project with package.json, package-lock.json, and node_modules
    let node_dir = base.join("my-web-app");
    fs::create_dir_all(node_dir.join("node_modules").join("lodash")).unwrap();
    fs::write(node_dir.join("package.json"), r#"{"name": "my-web-app"}"#).unwrap();
    fs::write(node_dir.join("package-lock.json"), r#"{"lockfileVersion": 3}"#).unwrap();
    fs::write(node_dir.join("node_modules").join("lodash").join("index.js"), "module.exports = {};").unwrap();

    // 2. Create a synthetic Rust project with Cargo.toml, Cargo.lock, and target
    let rust_dir = base.join("my-rust-crate");
    fs::create_dir_all(rust_dir.join("target").join("debug")).unwrap();
    fs::write(rust_dir.join("Cargo.toml"), "[package]\nname = \"my-rust-crate\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(rust_dir.join("Cargo.lock"), "# Lockfile\n").unwrap();
    fs::write(rust_dir.join("target").join("debug").join("app.exe"), "binary data").unwrap();

    // 3. Create a synthetic Python project with pyproject.toml and .venv
    let py_dir = base.join("my-python-tool");
    fs::create_dir_all(py_dir.join(".venv").join("Lib")).unwrap();
    fs::write(py_dir.join("pyproject.toml"), "[tool.poetry]\nname = \"tool\"\n").unwrap();
    fs::write(py_dir.join(".venv").join("pyvenv.cfg"), "home = C:\\Python\n").unwrap();

    // Run the developer scanner
    let projects = DeveloperWorkspaceScanner::scan_workspace(base, 5);
    assert_eq!(projects.len(), 3, "Expected 3 detected projects, got {}", projects.len());

    let node_proj = projects.iter().find(|p| p.ecosystem == "Node.js").unwrap();
    assert_eq!(node_proj.manifest_file, "package.json");
    assert_eq!(node_proj.lockfile_file, Some("package-lock.json".to_string()));
    assert!(!node_proj.artifacts.is_empty());
    assert_eq!(node_proj.artifacts[0].name, "node_modules");

    let rust_proj = projects.iter().find(|p| p.ecosystem == "Rust").unwrap();
    assert_eq!(rust_proj.manifest_file, "Cargo.toml");
    assert_eq!(rust_proj.lockfile_file, Some("Cargo.lock".to_string()));
    assert!(!rust_proj.artifacts.is_empty());
    assert_eq!(rust_proj.artifacts[0].name, "target");

    let py_proj = projects.iter().find(|p| p.ecosystem == "Python").unwrap();
    assert_eq!(py_proj.manifest_file, "pyproject.toml");
    assert!(!py_proj.artifacts.is_empty());
    assert_eq!(py_proj.artifacts[0].name, ".venv");
}

#[test]
fn test_event_coalescing_and_debouncing() {
    let temp = tempdir().unwrap();
    let root = temp.path().to_path_buf();

    // Create watcher with 200ms debounce
    let watcher = AutomationEventWatcher::new(vec![root.clone()], 200, 100).unwrap();

    // Touch files inside watched root
    let test_file = root.join("test_coalesce.txt");
    fs::write(&test_file, "initial").unwrap();
    // Rapid multiple writes
    for i in 0..5 {
        fs::write(&test_file, format!("update {}", i)).unwrap();
    }

    // Wait 400ms for debounce window to settle
    std::thread::sleep(std::time::Duration::from_millis(400));

    let (settled, had_overflow) = watcher.drain_settled_events();
    assert!(!had_overflow);
    assert!(!settled.is_empty());

    // Verify all rapid writes coalesced into a single path entry
    let matches: Vec<_> = settled.iter().filter(|e| e.path == test_file).collect();
    assert_eq!(matches.len(), 1, "Expected exactly 1 coalesced event for test_file");

    watcher.stop();
}

#[test]
fn test_repeated_trigger_does_not_cause_duplicate_destructive_actions() {
    let conn = Database::open_in_memory().unwrap();
    let quarantine = QuarantineManager::new(None);

    let temp = tempdir().unwrap();
    let staging_file = temp.path().join("safe_artifact.tmp");
    fs::write(&staging_file, "temporary cache data").unwrap();

    let rule = AutomationRuleDto {
        rule_id: "rule_dry_run_test".to_string(),
        name: "Dry Run Safe Rule".to_string(),
        description: "Dry run test".to_string(),
        is_enabled: true,
        trigger: AutomationTrigger::Manual,
        conditions: vec![],
        actions: vec![AutomationAction::StageToQuarantine { dry_run: true }],
        scope_paths: vec![staging_file.to_string_lossy().to_string()],
        exclusion_paths: vec![],
        schedule_expression: "Manual".to_string(),
        requires_explicit_approval: true,
        dry_run_mode: true,
        retry_policy: RetryPolicy::default(),
        execution_limits: ExecutionLimits {
            max_files_per_run: 10,
            max_bytes_per_run: 1000000,
            cooldown_seconds: 0,
        },
        last_run_at: None,
        last_status: None,
        created_at: 1000,
        updated_at: 1000,
    };

    // Execute twice
    let run1 = AutomationEngine::execute_rule(&conn, &rule, &quarantine, None, Some("run1"));
    let run2 = AutomationEngine::execute_rule(&conn, &rule, &quarantine, None, Some("run2"));

    assert_eq!(run1.status, "dry_run_completed");
    assert_eq!(run2.status, "dry_run_completed");
    // Verify file remains completely untouched
    assert!(staging_file.exists());
    assert_eq!(fs::read_to_string(&staging_file).unwrap(), "temporary cache data");
}
