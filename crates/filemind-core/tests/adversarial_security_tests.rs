use filemind_core::adapter::path::{is_dos_device_name, is_has_alternate_data_stream};
use filemind_core::db::repository::Database;
use filemind_core::models::SafetyClassification;
use filemind_core::safety::toctou::{capture_identity_snapshot, revalidate_handle, ToctouError};
use filemind_core::safety::SafetyGuardian;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_short_name_sfn_protection_blocked() {
    let sfn_paths = [
        r"C:\PROGRA~1\Common Files\system.dll",
        r"C:\PROGRA~2\Application\app.exe",
        r"C:\WINDOW~1\System32\ntoskrnl.exe",
        r"C:\WINDOW~1\explorer.exe",
    ];

    for path_str in sfn_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "SFN Path '{}' MUST be ProtectedAndBlocked",
            path_str
        );
        assert!(!assessment.is_permitted);
    }
}

#[test]
fn test_trailing_dots_and_spaces_sanitization() {
    let trailing_paths = [
        r"C:\Windows \System32",
        r"C:\Program Files. \App",
        r"C:\Windows.",
    ];

    for path_str in trailing_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "Trailing dot/space path '{}' MUST be ProtectedAndBlocked",
            path_str
        );
    }
}

#[test]
fn test_volume_guid_and_verbatim_syntax_blocked() {
    let verbatim_paths = [
        r"\\?\Volume{4c1b02c2-0000-0000-0000-100000000000}\Windows\System32\cmd.exe",
        r"//?/C:/Windows/System32/calc.exe",
        r"\\.\C:\Windows\System32\drivers\pci.sys",
    ];

    for path_str in verbatim_paths {
        let assessment = SafetyGuardian::evaluate_target(Path::new(path_str), "delete");
        assert_eq!(
            assessment.classification,
            SafetyClassification::ProtectedAndBlocked,
            "Verbatim/Volume GUID path '{}' MUST be ProtectedAndBlocked",
            path_str
        );
    }
}

#[test]
fn test_dos_device_names_detected() {
    assert!(is_dos_device_name(Path::new("CON")));
    assert!(is_dos_device_name(Path::new("con.txt")));
    assert!(is_dos_device_name(Path::new("NUL")));
    assert!(is_dos_device_name(Path::new("nul.dat")));
    assert!(is_dos_device_name(Path::new("AUX")));
    assert!(is_dos_device_name(Path::new("COM1")));
    assert!(is_dos_device_name(Path::new("LPT3")));
    assert!(is_dos_device_name(Path::new(r"C:\safe\CON")));
    assert!(is_dos_device_name(Path::new(r"C:\safe\nul.tmp")));

    assert!(!is_dos_device_name(Path::new("normal_file.txt")));
    assert!(!is_dos_device_name(Path::new("constant.rs")));
    assert!(!is_dos_device_name(Path::new("auxiliary.cpp")));
}

#[test]
fn test_alternate_data_streams_detected() {
    assert!(is_has_alternate_data_stream(Path::new(r"C:\folder\file.txt:evil.exe")));
    assert!(is_has_alternate_data_stream(Path::new(r"C:\folder\file.txt:$DATA")));
    assert!(is_has_alternate_data_stream(Path::new(r"C:\folder\file.txt:stream")));

    assert!(!is_has_alternate_data_stream(Path::new(r"C:\folder\file.txt")));
    assert!(!is_has_alternate_data_stream(Path::new(r"D:\data\archive.zip")));
}

#[test]
fn test_toctou_mutation_after_planning_strictly_aborts_execution() {
    let tmp = tempdir().unwrap();
    let target_file = tmp.path().join("victim_file.txt");
    fs::write(&target_file, b"Original approved content 12345").unwrap();

    // 1. Capture snapshot at planning time
    let planned_snapshot = capture_identity_snapshot(&target_file).expect("Failed to snapshot original file");
    assert_eq!(planned_snapshot.logical_size_bytes, 31);

    // 2. Simulate adversarial mutation between planning and execution (e.g. file content replaced)
    // Sleep briefly to ensure filesystem timestamp ticks
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(&target_file, b"Adversarial malicious content replaced after user approval! Extra payload bytes here!").unwrap();

    // 3. Revalidation against planned snapshot MUST detect mutation and abort
    let reval_result = revalidate_handle(&planned_snapshot);
    assert!(
        reval_result.is_err(),
        "Revalidation MUST fail when file content/size changes after planning"
    );
    match reval_result.unwrap_err() {
        ToctouError::SizeChanged { expected_size, actual_size } => {
            assert_eq!(expected_size, 31);
            assert!(actual_size > 31);
        }
        ToctouError::TargetModified { .. } => {}
        other => panic!("Expected SizeChanged or TargetModified, got: {:?}", other),
    }

    // Target file must remain intact
    assert!(target_file.exists());
}

#[test]
fn test_database_integrity_verification() {
    let conn = Database::open_in_memory().unwrap();
    let is_healthy = Database::verify_integrity(&conn).expect("Integrity check query failed");
    assert!(is_healthy, "Fresh database must pass integrity check");
}
