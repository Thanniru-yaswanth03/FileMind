use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::adapter::metadata::{epoch_ms_to_iso, inspect_file};
use crate::adapter::path::{normalize_path, to_long_path};
use crate::adapter::volume::query_volume_info;
use crate::models::{FileAttributesDto, FileItemDto, ScanProgressState};
use crate::scanner::category::{categorize_file, evaluate_risk_tier};
use crate::scanner::error::ScanError;
use crate::scanner::filter::{ReparsePointPolicy, ScanFilter};
use crate::scanner::progress::ProgressTracker;

pub struct InternalDirectoryRecord {
    pub path: String,
    pub parent_path: String,
    pub depth: u32,
    pub direct_size: u64,
    pub direct_files: u64,
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub mtime_epoch_ms: u64,
}

pub struct ScanResult {
    pub root_path: String,
    pub is_cancelled: bool,
    pub files: Vec<FileItemDto>,
    pub directories: Vec<InternalDirectoryRecord>,
    pub errors: Vec<ScanError>,
    pub total_files: u64,
    pub total_directories: u64,
    pub total_logical_bytes: u64,
    pub total_physical_bytes: u64,
    pub cluster_size_bytes: u64,
    pub volume_total_bytes: u64,
    pub volume_free_bytes: u64,
}

pub fn execute_scan(
    root_path: &Path,
    custom_exclusions: Vec<String>,
    cancel_token: Arc<AtomicBool>,
    progress_callback: Option<Arc<dyn Fn(ScanProgressState) + Send + Sync>>,
    session_id: Option<String>,
) -> ScanResult {
    let normalized_root = normalize_path(root_path);
    let vol_info =
        query_volume_info(root_path).unwrap_or_else(|_| crate::adapter::volume::VolumeInfo {
            root_path: normalized_root.clone(),
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

    let cluster_size = vol_info.cluster_size_bytes;
    let progress = Arc::new(ProgressTracker::new(session_id.clone()));
    let mut filter = ScanFilter::new(
        root_path.to_path_buf(),
        ReparsePointPolicy::NeverTraverse,
        custom_exclusions,
    );

    let mut collected_files: Vec<FileItemDto> = Vec::new();
    let mut dir_map: HashMap<String, InternalDirectoryRecord> = HashMap::new();
    let mut collected_errors: Vec<ScanError> = Vec::new();

    let mut total_logical: u64 = 0;
    let mut total_physical: u64 = 0;
    let mut is_cancelled = false;

    // Use a work queue for directory traversal
    let mut dir_queue: Vec<(PathBuf, u32)> = vec![(root_path.to_path_buf(), 0)];

    // Initialize root directory in directory map
    dir_map.insert(
        normalized_root.clone(),
        InternalDirectoryRecord {
            path: normalized_root.clone(),
            parent_path: String::new(),
            depth: 0,
            direct_size: 0,
            direct_files: 0,
            total_size_bytes: 0,
            file_count: 0,
            dir_count: 0,
            mtime_epoch_ms: 0,
        },
    );

    while let Some((current_dir, depth)) = dir_queue.pop() {
        if cancel_token.load(Ordering::Relaxed) {
            is_cancelled = true;
            break;
        }

        let norm_current_dir = normalize_path(&current_dir);
        progress.record_directory(&norm_current_dir);

        if let Some(ref cb) = progress_callback {
            if progress.should_emit() {
                cb(progress.snapshot(true));
            }
        }

        let long_dir = to_long_path(&current_dir);
        let entries = match std::fs::read_dir(&long_dir) {
            Ok(entries) => entries,
            Err(e) => {
                let scan_err = ScanError::from_io_error(&current_dir, &e);
                collected_errors.push(scan_err);
                progress.record_error();
                continue;
            }
        };

        for entry_res in entries {
            if cancel_token.load(Ordering::Relaxed) {
                is_cancelled = true;
                break;
            }

            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    let scan_err = ScanError::from_io_error(&current_dir, &e);
                    collected_errors.push(scan_err);
                    progress.record_error();
                    continue;
                }
            };

            let entry_path = entry.path();
            let norm_entry_path = normalize_path(&entry_path);

            if filter.is_excluded(&entry_path) {
                continue;
            }

            // Inspect file/directory metadata with Win32 adapter
            let meta = match inspect_file(&entry_path, cluster_size) {
                Ok(m) => m,
                Err(e) => {
                    let scan_err = ScanError::from_io_error(&entry_path, &e);
                    collected_errors.push(scan_err);
                    progress.record_error();
                    continue;
                }
            };

            let filename = entry_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| norm_entry_path.clone());

            if meta.is_directory {
                // Check if directory is a reparse point (junction or symlink)
                if meta.is_reparse_point {
                    if !filter.should_traverse_reparse_point(&entry_path) {
                        collected_errors.push(ScanError::new_reparse_policy(
                            &entry_path,
                            format!(
                                "Reparse point skipped by safety policy to prevent infinite recursion: '{}'",
                                norm_entry_path
                            ),
                        ));
                        continue;
                    }
                }

                // Cycle detection for directory loops
                if let Some(v_serial) = meta.volume_serial {
                    if let Some(ref file_id_str) = meta.ntfs_file_id {
                        let parsed_id = file_id_str
                            .split(':')
                            .nth(1)
                            .and_then(|h| u64::from_str_radix(h, 16).ok())
                            .unwrap_or(0);

                        if !filter.mark_visited_or_loop(v_serial, parsed_id) {
                            collected_errors.push(ScanError::new_reparse_policy(
                                &entry_path,
                                format!(
                                    "Circular directory junction loop detected and skipped: '{}'",
                                    norm_entry_path
                                ),
                            ));
                            continue;
                        }
                    }
                }

                // Register directory in directory map
                dir_map
                    .entry(norm_entry_path.clone())
                    .or_insert_with(|| InternalDirectoryRecord {
                        path: norm_entry_path.clone(),
                        parent_path: norm_current_dir.clone(),
                        depth: depth + 1,
                        direct_size: 0,
                        direct_files: 0,
                        total_size_bytes: 0,
                        file_count: 0,
                        dir_count: 0,
                        mtime_epoch_ms: meta.mtime_epoch_ms,
                    });

                // Enqueue subdirectory for scanning
                dir_queue.push((entry_path, depth + 1));
            } else {
                // File entry
                let ext = entry_path
                    .extension()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                let category = categorize_file(&entry_path, &ext);
                let (risk_tier, rationale) =
                    evaluate_risk_tier(&entry_path, category, meta.is_system);

                let file_dto = FileItemDto {
                    id: format!("file_{}_{}", meta.ctime_epoch_ms, collected_files.len() + 1),
                    name: filename,
                    path: norm_entry_path.clone(),
                    parent_path: norm_current_dir.clone(),
                    extension: if ext.is_empty() {
                        String::new()
                    } else {
                        format!(".{}", ext.to_lowercase())
                    },
                    size_bytes: meta.logical_size_bytes,
                    physical_size_bytes: meta.physical_size_bytes,
                    category,
                    mtime: epoch_ms_to_iso(meta.mtime_epoch_ms),
                    ctime: epoch_ms_to_iso(meta.ctime_epoch_ms),
                    atime: epoch_ms_to_iso(meta.atime_epoch_ms),
                    mtime_epoch_ms: Some(meta.mtime_epoch_ms),
                    attributes: FileAttributesDto {
                        is_hidden: meta.is_hidden,
                        is_system: meta.is_system,
                        is_read_only: meta.is_read_only,
                        is_reparse_point: meta.is_reparse_point,
                    },
                    ntfs_file_id: meta.ntfs_file_id,
                    risk_tier,
                    safety_rationale: rationale,
                    is_locked: meta.is_locked,
                    is_directory: false,
                    child_count: None,
                };

                total_logical += meta.logical_size_bytes;
                total_physical += meta.physical_size_bytes;

                progress.record_file(meta.logical_size_bytes, &norm_entry_path);

                // Add to parent directory's direct count and size
                if let Some(parent_dir) = dir_map.get_mut(&norm_current_dir) {
                    parent_dir.direct_size += meta.logical_size_bytes;
                    parent_dir.direct_files += 1;
                }

                collected_files.push(file_dto);
            }
        }
    }

    // Bottom-up aggregation of directory total sizes, file counts, and dir counts
    let mut sorted_dirs: Vec<String> = dir_map.keys().cloned().collect();
    // Sort by depth descending (deepest directories first)
    sorted_dirs.sort_by(|a, b| {
        let depth_a = dir_map[a].depth;
        let depth_b = dir_map[b].depth;
        depth_b.cmp(&depth_a)
    });

    for dir_path in &sorted_dirs {
        let (direct_size, direct_files, parent_path) = {
            let d = &dir_map[dir_path];
            (d.direct_size, d.direct_files, d.parent_path.clone())
        };

        // Initialize total size with direct size
        let current_total_size = if let Some(d) = dir_map.get_mut(dir_path) {
            d.total_size_bytes += direct_size;
            d.file_count += direct_files;
            d.total_size_bytes
        } else {
            direct_size
        };
        let current_file_count = dir_map[dir_path].file_count;
        let current_dir_count = dir_map[dir_path].dir_count;

        // Propagate to immediate parent
        if !parent_path.is_empty() {
            if let Some(parent) = dir_map.get_mut(&parent_path) {
                parent.total_size_bytes += current_total_size;
                parent.file_count += current_file_count;
                parent.dir_count += current_dir_count + 1;
            }
        }
    }

    let mut final_dirs: Vec<InternalDirectoryRecord> = dir_map.into_values().collect();
    final_dirs.sort_by(|a, b| b.total_size_bytes.cmp(&a.total_size_bytes));

    if let Some(ref cb) = progress_callback {
        cb(progress.snapshot(false));
    }

    ScanResult {
        root_path: normalized_root,
        is_cancelled,
        total_files: collected_files.len() as u64,
        total_directories: final_dirs.len() as u64,
        total_logical_bytes: total_logical,
        total_physical_bytes: total_physical,
        cluster_size_bytes: cluster_size,
        volume_total_bytes: vol_info.total_bytes,
        volume_free_bytes: vol_info.free_bytes,
        files: collected_files,
        directories: final_dirs,
        errors: collected_errors,
    }
}
