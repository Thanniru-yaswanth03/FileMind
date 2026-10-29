use std::collections::HashSet;
use crate::models::StorageAccountingDto;
use crate::scanner::walker::ScanResult;

pub struct StorageAccountant;

impl StorageAccountant {
    pub fn compute(scan_result: &ScanResult) -> StorageAccountingDto {
        let mut seen_ntfs_ids: HashSet<String> = HashSet::new();
        let mut hardlinks_count: u64 = 0;
        let mut hardlinks_saved_bytes: u64 = 0;

        for file in &scan_result.files {
            if let Some(ref fid) = file.ntfs_file_id {
                if !seen_ntfs_ids.insert(fid.clone()) {
                    // Already seen this physical MFT record, this is a hard link clone
                    hardlinks_count += 1;
                    hardlinks_saved_bytes += file.size_bytes;
                }
            }
        }

        let volume_used = if scan_result.volume_total_bytes >= scan_result.volume_free_bytes {
            scan_result.volume_total_bytes - scan_result.volume_free_bytes
        } else {
            0
        };

        let exact_note = format!(
            "Indexed {} files totaling {} logical bytes across {} directories. File sizes reflect exact uncompressed content lengths.",
            scan_result.total_files,
            scan_result.total_logical_bytes,
            scan_result.total_directories
        );

        let est_note = format!(
            "Reported allocated storage is {} bytes (cluster size: {} bytes). Reclaimable space upon deletion is estimated: NTFS sparse allocation, filesystem journal metadata, snapshots, and locked handles can affect actual recovered disk space. {} hardlinks detected ({} bytes shared).",
            scan_result.total_physical_bytes,
            scan_result.cluster_size_bytes,
            hardlinks_count,
            hardlinks_saved_bytes
        );

        StorageAccountingDto {
            volume_total_bytes: scan_result.volume_total_bytes,
            volume_free_bytes: scan_result.volume_free_bytes,
            volume_used_bytes: volume_used,
            logical_size_bytes: scan_result.total_logical_bytes,
            physical_size_bytes: scan_result.total_physical_bytes,
            indexed_files_total: scan_result.total_files,
            inaccessible_regions_count: scan_result.errors.len() as u64,
            exact_figures_note: exact_note,
            estimated_figures_note: est_note,
            hardlinks_deduplicated_count: hardlinks_count,
            hardlinks_bytes_saved: hardlinks_saved_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileAttributesDto, FileCategory, FileItemDto, RiskTier};

    #[test]
    fn test_hardlink_deduplication() {
        let f1 = FileItemDto {
            id: "1".into(),
            name: "file1.txt".into(),
            path: "C:\\test\\file1.txt".into(),
            parent_path: "C:\\test".into(),
            extension: ".txt".into(),
            size_bytes: 1000,
            physical_size_bytes: 4096,
            category: FileCategory::Documents,
            mtime: "".into(),
            ctime: "".into(),
            atime: "".into(),
            mtime_epoch_ms: None,
            attributes: FileAttributesDto::default(),
            ntfs_file_id: Some("00112233:0000000000000042".into()),
            risk_tier: RiskTier::Safe,
            safety_rationale: "".into(),
            is_locked: false,
            is_directory: false,
            child_count: None,
        };

        // f2 is a hardlink to f1
        let mut f2 = f1.clone();
        f2.id = "2".into();
        f2.name = "file2.txt".into();
        f2.path = "C:\\test\\file2.txt".into();

        let result = ScanResult {
            root_path: "C:\\test".into(),
            is_cancelled: false,
            files: vec![f1, f2],
            directories: vec![],
            errors: vec![],
            total_files: 2,
            total_directories: 1,
            total_logical_bytes: 2000,
            total_physical_bytes: 8192,
            cluster_size_bytes: 4096,
            volume_total_bytes: 1000000,
            volume_free_bytes: 500000,
        };

        let accounting = StorageAccountant::compute(&result);
        assert_eq!(accounting.hardlinks_deduplicated_count, 1);
        assert_eq!(accounting.hardlinks_bytes_saved, 1000);
    }
}
