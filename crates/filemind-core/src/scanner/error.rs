use std::io::ErrorKind as StdErrorKind;
use std::path::Path;
use crate::adapter::metadata::epoch_ms_to_iso;
use crate::adapter::path::normalize_path;
use crate::models::{ErrorKind, ScanErrorDto};

#[derive(Debug, Clone)]
pub struct ScanError {
    pub path: String,
    pub error_kind: ErrorKind,
    pub message: String,
    pub occurred_at_epoch_ms: u64,
}

impl ScanError {
    pub fn from_io_error(path: &Path, err: &std::io::Error) -> Self {
        let norm_path = normalize_path(path);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let (kind, msg) = match err.kind() {
            StdErrorKind::PermissionDenied => (
                ErrorKind::PermissionDenied,
                format!("Access denied to '{}': System or user permissions restricted.", norm_path),
            ),
            StdErrorKind::NotFound => (
                ErrorKind::Disappeared,
                format!("File or directory disappeared during scan: '{}'", norm_path),
            ),
            _ => {
                let raw_code = err.raw_os_error().unwrap_or(0);
                if raw_code == 32 || raw_code == 33 {
                    (
                        ErrorKind::Locked,
                        format!("File is currently locked by another process (Sharing Violation): '{}'", norm_path),
                    )
                } else if raw_code == 206 {
                    (
                        ErrorKind::PathTooLong,
                        format!("Path exceeds Windows maximum path length (260 characters): '{}'", norm_path),
                    )
                } else {
                    (
                        ErrorKind::IoError,
                        format!("I/O Error on '{}': {} (OS code: {})", norm_path, err, raw_code),
                    )
                }
            }
        };

        Self {
            path: norm_path,
            error_kind: kind,
            message: msg,
            occurred_at_epoch_ms: now_ms,
        }
    }

    pub fn new_reparse_policy(path: &Path, message: String) -> Self {
        let norm_path = normalize_path(path);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            path: norm_path,
            error_kind: ErrorKind::ReparsePolicy,
            message,
            occurred_at_epoch_ms: now_ms,
        }
    }

    pub fn to_dto(&self, id: i64, session_id: &str) -> ScanErrorDto {
        ScanErrorDto {
            id,
            session_id: session_id.to_string(),
            path: self.path.clone(),
            error_kind: self.error_kind,
            message: self.message.clone(),
            occurred_at: epoch_ms_to_iso(self.occurred_at_epoch_ms),
        }
    }
}

