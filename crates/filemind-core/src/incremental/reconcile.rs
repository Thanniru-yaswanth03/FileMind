use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use crate::adapter::metadata::inspect_file;
use crate::adapter::path::normalize_path;
use crate::adapter::volume::query_volume_info;
use crate::models::{FileCategory, RiskTier};
use crate::scanner::category::{categorize_file, evaluate_risk_tier};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileReport {
    pub added_files: u64,
    pub modified_files: u64,
    pub deleted_files: u64,
    pub renamed_files: u64,
    pub unchanged_files: u64,
    pub new_total_files: u64,
    pub new_total_logical_bytes: u64,
    pub session_id: String,
}

pub struct ReconciliationScanner;

impl ReconciliationScanner {
    /// Perform a conservative reconciliation scan between SQLite index and live filesystem.
    /// Does not rely on watcher events. Accurately detects additions, modifications, renames, and deletions.
    pub fn reconcile(conn: &mut Connection, session_id: &str, root_path: &Path) -> Result<ReconcileReport> {
        let norm_root = normalize_path(root_path);
        let vol_info = query_volume_info(root_path).unwrap_or_else(|_| crate::adapter::volume::VolumeInfo {
            root_path: norm_root.clone(),
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

        // Load existing indexed files into memory: path -> (file_id, size, ntfs_file_id)
        let mut indexed_by_path: HashMap<String, (i64, u64, Option<String>)> = HashMap::new();
        let mut indexed_by_ntfs_id: HashMap<String, (i64, String)> = HashMap::new();

        {
            let mut stmt = conn.prepare(
                "SELECT file_id, directory_path, filename, logical_size_bytes, ntfs_file_id
                 FROM files WHERE session_id = ?1",
            )?;

            let rows = stmt.query_map(params![session_id], |row| {
                let id: i64 = row.get(0)?;
                let dir: String = row.get(1)?;
                let name: String = row.get(2)?;
                let size: i64 = row.get(3)?;
                let ntfs_id: Option<String> = row.get(4)?;

                let full_path = if dir.ends_with('\\') {
                    format!("{}{}", dir, name)
                } else {
                    format!("{}\\{}", dir, name)
                };

                Ok((id, full_path, size as u64, ntfs_id))
            })?;

            for r in rows {
                let (id, full_path, size, ntfs_id) = r?;
                indexed_by_path.insert(full_path.clone(), (id, size, ntfs_id.clone()));
                if let Some(fid) = ntfs_id {
                    indexed_by_ntfs_id.insert(fid, (id, full_path));
                }
            }
        }

        let mut seen_paths: HashSet<String> = HashSet::new();
        let mut added = 0;
        let mut modified = 0;
        let mut renamed = 0;
        let mut unchanged = 0;

        let tx = conn.transaction()?;

        let mut insert_stmt = tx.prepare(
            r#"INSERT INTO files (
                session_id, directory_path, filename, extension, file_type,
                logical_size_bytes, physical_size_bytes, attributes, ctime, mtime, atime,
                ntfs_file_id, risk_tier, safety_rationale, is_locked
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"#,
        )?;

        let mut update_stmt = tx.prepare(
            r#"UPDATE files SET
                logical_size_bytes = ?1,
                physical_size_bytes = ?2,
                mtime = ?3,
                is_locked = ?4
               WHERE file_id = ?5"#,
        )?;

        let mut rename_stmt = tx.prepare(
            r#"UPDATE files SET
                directory_path = ?1,
                filename = ?2
               WHERE file_id = ?3"#,
        )?;

        // Walk the live directory tree
        let mut dir_queue = vec![root_path.to_path_buf()];

        while let Some(current_dir) = dir_queue.pop() {
            let entries = match std::fs::read_dir(&current_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let norm_current_dir = normalize_path(&current_dir);

            for entry in entries.flatten() {
                let entry_path = entry.path();
                let norm_path = normalize_path(&entry_path);

                let meta = match inspect_file(&entry_path, vol_info.cluster_size_bytes) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if meta.is_directory {
                    if !meta.is_reparse_point {
                        dir_queue.push(entry_path);
                    }
                } else {
                    seen_paths.insert(norm_path.clone());

                    let filename = entry_path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| norm_path.clone());

                    if let Some(&(file_id, old_size, _)) = indexed_by_path.get(&norm_path) {
                        // File exists at same path
                        if old_size != meta.logical_size_bytes {
                            // File modified
                            update_stmt.execute(params![
                                meta.logical_size_bytes as i64,
                                meta.physical_size_bytes as i64,
                                meta.mtime_epoch_ms as i64,
                                if meta.is_locked { 1 } else { 0 },
                                file_id,
                            ])?;
                            modified += 1;
                        } else {
                            unchanged += 1;
                        }
                    } else if let Some(ref fid) = meta.ntfs_file_id {
                        // Check if file was renamed/moved (same NTFS MFT record ID)
                        if let Some(&(file_id, ref _old_path)) = indexed_by_ntfs_id.get(fid) {
                            rename_stmt.execute(params![
                                norm_current_dir,
                                filename,
                                file_id,
                            ])?;
                            renamed += 1;
                        } else {
                            // New file
                            let ext = entry_path
                                .extension()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let cat = categorize_file(&entry_path, &ext);
                            let (risk, rationale) = evaluate_risk_tier(&entry_path, cat, meta.is_system);

                            let cat_str = match cat {
                                FileCategory::Code => "Code",
                                FileCategory::Media => "Media",
                                FileCategory::Archives => "Archives",
                                FileCategory::Documents => "Documents",
                                FileCategory::Executables => "Executables",
                                FileCategory::Caches => "Caches",
                                FileCategory::Other => "Other",
                            };

                            let risk_str = match risk {
                                RiskTier::Safe => "safe",
                                RiskTier::Caution => "caution",
                                RiskTier::High => "high",
                                RiskTier::Danger => "danger",
                            };

                            insert_stmt.execute(params![
                                session_id,
                                norm_current_dir,
                                filename,
                                if ext.is_empty() { String::new() } else { format!(".{}", ext.to_lowercase()) },
                                cat_str,
                                meta.logical_size_bytes as i64,
                                meta.physical_size_bytes as i64,
                                0i64,
                                meta.ctime_epoch_ms as i64,
                                meta.mtime_epoch_ms as i64,
                                meta.atime_epoch_ms as i64,
                                meta.ntfs_file_id,
                                risk_str,
                                rationale,
                                if meta.is_locked { 1 } else { 0 },
                            ])?;
                            added += 1;
                        }
                    } else {
                        // New file without NTFS ID
                        let ext = entry_path
                            .extension()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let cat = categorize_file(&entry_path, &ext);
                        let (_risk, rationale) = evaluate_risk_tier(&entry_path, cat, meta.is_system);

                        insert_stmt.execute(params![
                            session_id,
                            norm_current_dir,
                            filename,
                            if ext.is_empty() { String::new() } else { format!(".{}", ext.to_lowercase()) },
                            "Other",
                            meta.logical_size_bytes as i64,
                            meta.physical_size_bytes as i64,
                            0i64,
                            meta.ctime_epoch_ms as i64,
                            meta.mtime_epoch_ms as i64,
                            meta.atime_epoch_ms as i64,
                            meta.ntfs_file_id,
                            "caution",
                            rationale,
                            0i64,
                        ])?;
                        added += 1;
                    }
                }
            }
        }

        drop(insert_stmt);
        drop(update_stmt);
        drop(rename_stmt);

        // Delete records in DB that no longer exist on disk
        let mut delete_stmt = tx.prepare("DELETE FROM files WHERE file_id = ?1")?;
        let mut deleted = 0;

        for (path, (file_id, _, _)) in &indexed_by_path {
            if !seen_paths.contains(path) {
                delete_stmt.execute(params![file_id])?;
                deleted += 1;
            }
        }
        drop(delete_stmt);

        // Update session totals
        let (new_total_files, new_total_bytes): (i64, i64) = tx.query_row(
            "SELECT COUNT(*), COALESCE(SUM(logical_size_bytes), 0) FROM files WHERE session_id = ?1",
            params![session_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        tx.execute(
            r#"UPDATE scan_sessions SET
                total_files = ?1,
                total_logical_bytes = ?2,
                completed_at = ?3
               WHERE session_id = ?4"#,
            params![new_total_files, new_total_bytes, now_ms, session_id],
        )?;

        tx.commit()?;

        Ok(ReconcileReport {
            added_files: added,
            modified_files: modified,
            deleted_files: deleted,
            renamed_files: renamed,
            unchanged_files: unchanged,
            new_total_files: new_total_files as u64,
            new_total_logical_bytes: new_total_bytes as u64,
            session_id: session_id.to_string(),
        })
    }
}
