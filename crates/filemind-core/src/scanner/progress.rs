use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use crate::models::ScanProgressState;

pub struct ProgressTracker {
    files_count: AtomicU64,
    bytes_count: AtomicU64,
    dirs_count: AtomicU64,
    errors_count: AtomicU64,
    start_time: Instant,
    last_emit_time: Mutex<Instant>,
    current_path: Mutex<String>,
    session_id: Option<String>,
}

impl ProgressTracker {
    pub fn new(session_id: Option<String>) -> Self {
        let now = Instant::now();
        Self {
            files_count: AtomicU64::new(0),
            bytes_count: AtomicU64::new(0),
            dirs_count: AtomicU64::new(0),
            errors_count: AtomicU64::new(0),
            start_time: now,
            last_emit_time: Mutex::new(now),
            current_path: Mutex::new(String::new()),
            session_id,
        }
    }

    pub fn record_file(&self, bytes: u64, path: &str) {
        self.files_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_count.fetch_add(bytes, Ordering::Relaxed);

        if let Ok(mut cp) = self.current_path.try_lock() {
            *cp = path.to_string();
        }
    }

    pub fn record_directory(&self, path: &str) {
        self.dirs_count.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut cp) = self.current_path.try_lock() {
            *cp = path.to_string();
        }
    }

    pub fn record_error(&self) {
        self.errors_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if at least 100ms has elapsed since the last progress update (rate limiting to max 10/sec)
    pub fn should_emit(&self) -> bool {
        if let Ok(mut last) = self.last_emit_time.try_lock() {
            if last.elapsed().as_millis() >= 100 {
                *last = Instant::now();
                return true;
            }
        }
        false
    }

    pub fn snapshot(&self, is_scanning: bool) -> ScanProgressState {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let files = self.files_count.load(Ordering::Relaxed);
        let bytes = self.bytes_count.load(Ordering::Relaxed);
        let dirs = self.dirs_count.load(Ordering::Relaxed);
        let errors = self.errors_count.load(Ordering::Relaxed);
        let path = self
            .current_path
            .lock()
            .map(|p| p.clone())
            .unwrap_or_default();

        // Percent calculation: If active, simulate logarithmic progress curve until completion
        let percent = if !is_scanning {
            100.0
        } else {
            let p = 100.0 * (1.0 - (-0.00015 * (files as f64)).exp());
            p.min(99.0)
        };

        ScanProgressState {
            percent: (percent * 10.0).round() / 10.0,
            current_path: path,
            files_count: files,
            bytes_count: bytes,
            directories_count: dirs,
            errors_count: errors,
            elapsed_ms: elapsed,
            is_scanning,
            session_id: self.session_id.clone(),
        }
    }
}
