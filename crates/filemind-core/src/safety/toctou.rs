use std::path::Path;
use crate::adapter::metadata::{inspect_file, system_time_to_epoch_ms};
use crate::adapter::path::to_long_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleIdentitySnapshot {
    pub canonical_path: String,
    pub volume_serial: Option<u32>,
    pub ntfs_file_id: Option<String>,
    pub logical_size_bytes: u64,
    pub mtime_epoch_ms: u64,
    pub is_directory: bool,
    pub is_reparse_point: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToctouError {
    TargetDisappeared(String),
    IdentityChanged {
        expected_id: Option<String>,
        actual_id: Option<String>,
    },
    TargetModified {
        expected_mtime: u64,
        actual_mtime: u64,
    },
    SizeChanged {
        expected_size: u64,
        actual_size: u64,
    },
    ReparsePointSwapped,
    AccessDenied(String),
}

impl std::fmt::Display for ToctouError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToctouError::TargetDisappeared(path) => {
                write!(f, "Target disappeared between evaluation and execution: '{}'", path)
            }
            ToctouError::IdentityChanged { expected_id, actual_id } => {
                write!(
                    f,
                    "TOCTOU violation: File identity / NTFS file ID changed! Expected {:?}, got {:?}",
                    expected_id, actual_id
                )
            }
            ToctouError::TargetModified { expected_mtime, actual_mtime } => {
                write!(
                    f,
                    "TOCTOU violation: Target was modified after evaluation! Expected mtime {}, got {}",
                    expected_mtime, actual_mtime
                )
            }
            ToctouError::SizeChanged { expected_size, actual_size } => {
                write!(
                    f,
                    "TOCTOU violation: Target size changed! Expected {} bytes, got {} bytes",
                    expected_size, actual_size
                )
            }
            ToctouError::ReparsePointSwapped => {
                write!(f, "TOCTOU violation: Reparse point / junction swap detected!")
            }
            ToctouError::AccessDenied(msg) => {
                write!(f, "Access denied during pre-execution handle validation: {}", msg)
            }
        }
    }
}

impl std::error::Error for ToctouError {}

/// Captures a frozen handle identity snapshot for a given path during evaluation
pub fn capture_identity_snapshot(path: &Path) -> Result<HandleIdentitySnapshot, ToctouError> {
    let long_path = to_long_path(path);
    let meta = inspect_file(&long_path, 4096)
        .map_err(|e| ToctouError::TargetDisappeared(format!("{}: {}", path.display(), e)))?;

    Ok(HandleIdentitySnapshot {
        canonical_path: path.to_string_lossy().to_string(),
        volume_serial: meta.volume_serial,
        ntfs_file_id: meta.ntfs_file_id,
        logical_size_bytes: meta.logical_size_bytes,
        mtime_epoch_ms: meta.mtime_epoch_ms,
        is_directory: meta.is_directory,
        is_reparse_point: meta.is_reparse_point,
    })
}

/// Re-opens the target right before any destructive action using safe non-following flags
/// and validates that the file identity, NTFS file ID, size, and timestamps are unchanged.
pub fn revalidate_handle(snapshot: &HandleIdentitySnapshot) -> Result<(), ToctouError> {
    let path = Path::new(&snapshot.canonical_path);
    let long_path = to_long_path(path);

    // Standard symlink_metadata check to verify existence without traversing links
    let std_meta = match std::fs::symlink_metadata(&long_path) {
        Ok(m) => m,
        Err(e) => return Err(ToctouError::TargetDisappeared(format!("{}: {}", snapshot.canonical_path, e))),
    };

    let cur_mtime = std_meta.modified().map(system_time_to_epoch_ms).unwrap_or(0);
    let cur_size = std_meta.len();

    // Inspect live handle
    let live_meta = match inspect_file(&long_path, 4096) {
        Ok(m) => m,
        Err(e) => return Err(ToctouError::AccessDenied(e.to_string())),
    };

    // 1. Verify Reparse Point status (must not have been converted to/from a junction)
    if live_meta.is_reparse_point != snapshot.is_reparse_point {
        return Err(ToctouError::ReparsePointSwapped);
    }

    // 2. Verify NTFS File ID (Volume Serial + 64-bit File Index)
    if snapshot.ntfs_file_id.is_some() && live_meta.ntfs_file_id.is_some() {
        if snapshot.ntfs_file_id != live_meta.ntfs_file_id {
            return Err(ToctouError::IdentityChanged {
                expected_id: snapshot.ntfs_file_id.clone(),
                actual_id: live_meta.ntfs_file_id,
            });
        }
    }

    // 3. For regular files, check for modifications or size mutations
    if !snapshot.is_directory {
        // Tolerating up to 500ms filesystem clock jitter
        if cur_mtime > snapshot.mtime_epoch_ms + 500 || (snapshot.mtime_epoch_ms > 500 && cur_mtime < snapshot.mtime_epoch_ms - 500) {
            return Err(ToctouError::TargetModified {
                expected_mtime: snapshot.mtime_epoch_ms,
                actual_mtime: cur_mtime,
            });
        }

        if cur_size != snapshot.logical_size_bytes {
            return Err(ToctouError::SizeChanged {
                expected_size: snapshot.logical_size_bytes,
                actual_size: cur_size,
            });
        }
    }

    Ok(())
}

