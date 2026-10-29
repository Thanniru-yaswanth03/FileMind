use std::fs;
use tempfile::tempdir;

use filemind_core::cleanup::quarantine::verified_quarantine_move;

#[test]
fn test_cross_volume_quarantine_streaming_verification() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let src_dir = temp_root.path().join("source_vol");
    let vault_dir = temp_root.path().join("vault_vol");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let test_file = src_dir.join("critical_report.docx");
    let content = b"Confidential Quarterly Storage Audit Data 2026";
    fs::write(&test_file, content).unwrap();

    let expected_hash = blake3::hash(content).to_hex().to_string();
    let dest_file = vault_dir.join("quar_target.docx");

    // Move using verified quarantine
    let res = verified_quarantine_move(&test_file, &dest_file, false, Some(&expected_hash));
    assert!(res.is_ok(), "Verified quarantine move should succeed: {:?}", res);

    assert!(!test_file.exists(), "Original file should be deleted after successful verification");
    assert!(dest_file.exists(), "Quarantined destination must exist in vault");

    let dest_content = fs::read(&dest_file).unwrap();
    assert_eq!(dest_content, content, "Restored content must match byte-for-byte");
}

#[test]
fn test_quarantine_destination_collision_rejection() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let src = temp_root.path().join("source.txt");
    let dest = temp_root.path().join("dest.txt");

    fs::write(&src, "Source File Data").unwrap();
    fs::write(&dest, "Existing Destination File").unwrap();

    // Destination already exists! Must reject with collision error without destroying dest
    let res = verified_quarantine_move(&src, &dest, false, None);
    assert!(res.is_err(), "Must reject destination collision");
    assert!(res.unwrap_err().contains("collision"), "Error should report collision");

    assert!(src.exists(), "Source file must not be modified or deleted on collision");
    assert_eq!(fs::read_to_string(&dest).unwrap(), "Existing Destination File");
}

#[test]
fn test_quarantine_hash_mismatch_failure_injection() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let src = temp_root.path().join("untrusted.bin");
    let dest = temp_root.path().join("vault_target.bin");

    let content = b"Original Uncorrupted Payload";
    fs::write(&src, content).unwrap();

    let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

    // Expected hash does not match
    let res = verified_quarantine_move(&src, &dest, false, Some(wrong_hash));
    assert!(res.is_err(), "Must abort when cryptographic hash does not match expected");
    assert!(res.unwrap_err().contains("cryptographic verification failed"));

    // INVARIANT: Source file must remain 100% intact!
    assert!(src.exists(), "Source file must never be deleted if verification fails");
    assert!(!dest.exists(), "Destination file must be purged on failed verification");
}

#[test]
fn test_quarantine_directory_structure_verification() {
    let temp_root = tempdir().expect("Failed to create tempdir");
    let src_tree = temp_root.path().join("nested_project");
    let vault_tree = temp_root.path().join("vault_nested_project");

    fs::create_dir_all(src_tree.join("subfolder")).unwrap();
    fs::write(src_tree.join("file1.txt"), "File 1 content").unwrap();
    fs::write(src_tree.join("subfolder").join("file2.txt"), "File 2 content inside subfolder").unwrap();

    let res = verified_quarantine_move(&src_tree, &vault_tree, true, None);
    assert!(res.is_ok(), "Directory quarantine move should succeed: {:?}", res);

    assert!(!src_tree.exists(), "Original directory must be removed after successful verified move");
    assert!(vault_tree.join("file1.txt").exists());
    assert!(vault_tree.join("subfolder").join("file2.txt").exists());
}
