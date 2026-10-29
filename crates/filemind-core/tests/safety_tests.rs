use filemind_core::safety::{
    capture_identity_snapshot, revalidate_handle, AuditLogger, SafetyGuardian,
};
use filemind_core::models::{
    AllowedOperation, SafetyAuditRecordDto, SafetyClassification,
};
use filemind_core::db::Database;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_windows_os_protected_paths_blocked() {
    let protected_paths = [
        r"C:\Windows\System32\ntoskrnl.exe",
        r"C:\Windows\WinSxS\manifests.dll",
        r"C:\Program Files\Common Files\system.dll",
        r"C:\Program Files (x86)\Microsoft\Edge\msedge.exe",
        r"C:\System Volume Information",
        r"C:\$Recycle.Bin",
        r"C:\pagefile.sys",
        r"C:\swapfile.sys",
        r"C:\bootmgr",
        r"C:\Windows\System32\drivers\pci.sys",
        r"C:\ProgramData\Microsoft\Crypto\Keys",
        r"C:\Users\Admin\AppData\Local\Microsoft\Windows\UsrClass.dat",
    ];

    for path_str in protected_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "Path '{}' should be ProtectedAndBlocked",
            path_str
        );
        assert_eq!(assessment.allowed_operation, AllowedOperation::Blocked);
        assert!(!assessment.is_permitted);
        assert!(assessment.block_reason.is_some());
    }
}

#[test]
fn test_path_traversal_attempts_strictly_blocked() {
    let traversal_paths = [
        r"C:\Users\Public\..\Windows\System32",
        r"C:\temp\..\..\Windows\explorer.exe",
        r"relative\..\path",
    ];

    for path_str in traversal_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "Traversal path '{}' must be ProtectedAndBlocked",
            path_str
        );
        assert!(!assessment.is_permitted);
        assert!(assessment.rule_matches.iter().any(|r| r.contains("TRAVERSAL")));
    }
}

#[test]
fn test_wsl_distributions_and_virtual_disks_blocked() {
    let wsl_paths = [
        r"C:\Users\User\AppData\Local\Packages\CanonicalGroupLimited.Ubuntu_79rhkp1fndgsc\LocalState\ext4.vhdx",
        r"\\wsl$\Ubuntu-22.04\etc\shadow",
        r"\\wsl.localhost\Debian\home\user",
        r"C:\Users\User\AppData\Local\Docker\wsl\data\ext4.vhdx",
    ];

    for path_str in wsl_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "WSL path '{}' must be ProtectedAndBlocked",
            path_str
        );
        assert_eq!(assessment.allowed_operation, AllowedOperation::Blocked);
        assert!(!assessment.is_permitted);
        assert!(assessment.rule_matches.iter().any(|r| r.starts_with("WSL-")));
    }
}

#[test]
fn test_developer_secrets_and_private_keys_blocked() {
    let secret_names = [
        ".env",
        ".env.local",
        ".env.production",
        "id_rsa",
        "id_ed25519",
        "private.pem",
        "server.key",
        "credentials.json",
        "secrets.kdbx",
    ];

    for secret in secret_names {
        let fake_path = format!(r"C:\MyProjects\cool-app\{}", secret);
        let assessment = SafetyGuardian::evaluate_target(Path::new(&fake_path), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "Secret '{}' must be ProtectedAndBlocked",
            secret
        );
        assert!(!assessment.is_permitted);
    }
}

#[test]
fn test_developer_source_files_require_manual_review() {
    let source_paths = [
        r"C:\MyProjects\app\src\main.rs",
        r"C:\MyProjects\app\src\components\Button.tsx",
        r"C:\MyProjects\app\backend\server.py",
        r"C:\MyProjects\app\main.go",
    ];

    for path_str in source_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ReviewRequired,
            "Source file '{}' must be ReviewRequired",
            path_str
        );
        assert_eq!(
            assessment.allowed_operation,
            AllowedOperation::RequiresExplicitUserConfirmation
        );
        assert!(!assessment.is_permitted, "Source files must not be deleted automatically");
    }
}

#[test]
fn test_regenerable_build_directory_with_manifest_and_lockfile_permitted() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create Cargo.toml and Cargo.lock
    File::create(root.join("Cargo.toml")).unwrap().write_all(b"[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();
    File::create(root.join("Cargo.lock")).unwrap().write_all(b"# lockfile\n").unwrap();

    // Create target directory with some build artifacts
    let target_dir = root.join("target");
    fs::create_dir_all(&target_dir).unwrap();
    File::create(target_dir.join("demo.exe")).unwrap().write_all(b"binary data").unwrap();

    let assessment = SafetyGuardian::evaluate_target(&target_dir, "recycle");
    assert_eq!(
        assessment.classification,
        SafetyClassification::LowRiskCandidate,
        "Target directory with manifest and lockfile should be LowRiskCandidate"
    );
    assert_eq!(assessment.allowed_operation, AllowedOperation::Permitted);
    assert!(assessment.is_permitted);
    assert_eq!(assessment.recovery_possibility, "RegenerableViaBuildTool");
}

#[test]
fn test_regenerable_build_directory_missing_lockfile_demoted_to_high_risk() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create Cargo.toml WITHOUT Cargo.lock
    File::create(root.join("Cargo.toml")).unwrap().write_all(b"[package]\nname = \"demo\"\nversion = \"0.1.0\"\n").unwrap();

    // Create target directory
    let target_dir = root.join("target");
    fs::create_dir_all(&target_dir).unwrap();

    let assessment = SafetyGuardian::evaluate_target(&target_dir, "recycle");
    assert_eq!(
        assessment.classification,
        SafetyClassification::HighRiskOrUnknown,
        "Target directory without lockfile must be demoted to HighRiskOrUnknown (fail-closed)"
    );
    assert!(!assessment.is_permitted);
}

#[test]
fn test_build_directory_containing_secret_file_blocked() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create Cargo.toml and Cargo.lock
    File::create(root.join("Cargo.toml")).unwrap().write_all(b"[package]\nname = \"demo\"\n").unwrap();
    File::create(root.join("Cargo.lock")).unwrap().write_all(b"# lock\n").unwrap();

    // Create target directory containing a misplaced .env file
    let target_dir = root.join("target");
    fs::create_dir_all(&target_dir).unwrap();
    File::create(target_dir.join(".env")).unwrap().write_all(b"API_SECRET=12345\n").unwrap();

    let assessment = SafetyGuardian::evaluate_target(&target_dir, "recycle");
    assert_eq!(
        assessment.classification,
        SafetyClassification::ProtectedAndBlocked,
        "Build directory with secret must be ProtectedAndBlocked"
    );
    assert!(!assessment.is_permitted);
}

#[test]
fn test_toctou_handle_identity_revalidation_detects_mutation() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("toctou_test.txt");

    // Write initial file
    File::create(&file_path).unwrap().write_all(b"original content").unwrap();

    // Capture frozen snapshot
    let snapshot = capture_identity_snapshot(&file_path).unwrap();

    // Verification on unchanged file should pass cleanly
    assert!(revalidate_handle(&snapshot).is_ok());

    // Mutate file size and timestamp
    File::create(&file_path).unwrap().write_all(b"tampered content has changed!").unwrap();

    // TOCTOU revalidation must catch this modification
    let reval_result = revalidate_handle(&snapshot);
    assert!(reval_result.is_err(), "TOCTOU must fail when file content/size changes");
}

#[test]
fn test_safety_audit_trail_recorded_in_sqlite() {
    let conn = Database::open_in_memory().unwrap();

    let audit_record = SafetyAuditRecordDto {
        audit_id: 0,
        target_path: r"C:\test\target".to_string(),
        canonical_path: r"C:\test\target".to_string(),
        operation: "evaluate".to_string(),
        classification: SafetyClassification::LowRiskCandidate,
        is_permitted: true,
        rule_matches: vec!["DEV-REGENERABLE-TARGET".to_string()],
        evidence: vec![],
        block_reasons: vec![],
        toctou_verified: true,
        executed: false,
        execution_result: Some("EVALUATED_OK".to_string()),
        timestamp_epoch_ms: 1700000000000,
    };

    let inserted_id = AuditLogger::record_audit(&conn, &audit_record).unwrap();
    assert!(inserted_id > 0);

    let recent = AuditLogger::fetch_recent(&conn, 10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].target_path, r"C:\test\target");
    assert_eq!(recent[0].classification, SafetyClassification::LowRiskCandidate);
    assert!(recent[0].is_permitted);
    assert!(recent[0].toctou_verified);
}
