use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug, Clone)]
pub struct CoalescedFileEvent {
    pub path: PathBuf,
    pub kind: String, // 'create' | 'modify' | 'delete' | 'overflow'
    pub first_seen_at: Instant,
    pub last_seen_at: Instant,
    pub event_count: usize,
}

pub struct AutomationEventWatcher {
    root_paths: Vec<PathBuf>,
    _watchers: Vec<RecommendedWatcher>,
    is_running: Arc<AtomicBool>,
    event_queue: Arc<Mutex<HashMap<PathBuf, CoalescedFileEvent>>>,
    overflow_detected: Arc<AtomicBool>,
    total_events_received: Arc<AtomicUsize>,
    debounce_duration: Duration,
    max_queue_size: usize,
}

impl AutomationEventWatcher {
    /// Creates a new background filesystem watcher with debouncing and coalescing
    pub fn new(
        roots: Vec<PathBuf>,
        debounce_ms: u64,
        max_queue_size: usize,
    ) -> Result<Self, notify::Error> {
        let is_running = Arc::new(AtomicBool::new(true));
        let event_queue: Arc<Mutex<HashMap<PathBuf, CoalescedFileEvent>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let overflow_detected = Arc::new(AtomicBool::new(false));
        let total_events_received = Arc::new(AtomicUsize::new(0));

        let mut watchers = Vec::new();

        for root in &roots {
            if !root.exists() {
                continue;
            }

            let (tx, rx) = mpsc::channel();
            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    let _ = tx.send(res);
                },
                Config::default(),
            )?;

            watcher.watch(root, RecursiveMode::Recursive)?;
            watchers.push(watcher);

            // Spawn background event drain thread
            let is_running_clone = Arc::clone(&is_running);
            let queue_clone = Arc::clone(&event_queue);
            let overflow_clone = Arc::clone(&overflow_detected);
            let count_clone = Arc::clone(&total_events_received);

            thread::spawn(move || {
                while is_running_clone.load(Ordering::Relaxed) {
                    match rx.recv_timeout(Duration::from_millis(200)) {
                        Ok(Ok(event)) => {
                            count_clone.fetch_add(1, Ordering::Relaxed);
                            Self::process_raw_event(event, &queue_clone, &overflow_clone, max_queue_size);
                        }
                        Ok(Err(_err)) => {
                            // Buffer overflow from OS notify queue
                            overflow_clone.store(true, Ordering::SeqCst);
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Heartbeat, continue
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }
            });
        }

        Ok(Self {
            root_paths: roots,
            _watchers: watchers,
            is_running,
            event_queue,
            overflow_detected,
            total_events_received,
            debounce_duration: Duration::from_millis(debounce_ms),
            max_queue_size,
        })
    }

    /// Process raw notify events and coalesce into queue with backpressure
    fn process_raw_event(
        event: Event,
        queue: &Arc<Mutex<HashMap<PathBuf, CoalescedFileEvent>>>,
        overflow_flag: &Arc<AtomicBool>,
        max_size: usize,
    ) {
        let kind_str = match event.kind {
            EventKind::Create(_) => "create",
            EventKind::Modify(_) => "modify",
            EventKind::Remove(_) => "delete",
            _ => return, // Ignore access, metadata-only noise
        };

        let now = Instant::now();

        if let Ok(mut map) = queue.lock() {
            if map.len() >= max_size {
                // Backpressure limit hit: signal overflow
                overflow_flag.store(true, Ordering::SeqCst);
                return;
            }

            for path in event.paths {
                // Filter out temporary and system lock files
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if filename.starts_with('~') || filename.ends_with(".tmp") || filename.ends_with(".crdownload") {
                    continue;
                }

                if let Some(entry) = map.get_mut(&path) {
                    entry.last_seen_at = now;
                    entry.event_count += 1;
                    if kind_str == "delete" {
                        entry.kind = "delete".to_string();
                    }
                } else {
                    map.insert(
                        path.clone(),
                        CoalescedFileEvent {
                            path,
                            kind: kind_str.to_string(),
                            first_seen_at: now,
                            last_seen_at: now,
                            event_count: 1,
                        },
                    );
                }
            }
        }
    }

    /// Drains debounced events that have settled for at least `debounce_duration`.
    /// Returns drained events and whether a buffer overflow occurred (requiring reconciliation).
    pub fn drain_settled_events(&self) -> (Vec<CoalescedFileEvent>, bool) {
        let now = Instant::now();
        let mut settled = Vec::new();
        let had_overflow = self.overflow_detected.swap(false, Ordering::SeqCst);

        if let Ok(mut map) = self.event_queue.lock() {
            let mut keys_to_remove = Vec::new();

            for (path, item) in map.iter() {
                if now.duration_since(item.last_seen_at) >= self.debounce_duration {
                    settled.push(item.clone());
                    keys_to_remove.push(path.clone());
                }
            }

            for k in keys_to_remove {
                map.remove(&k);
            }
        }

        (settled, had_overflow)
    }

    pub fn active_roots(&self) -> &[PathBuf] {
        &self.root_paths
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn total_events_count(&self) -> usize {
        self.total_events_received.load(Ordering::Relaxed)
    }

    pub fn max_queue_size(&self) -> usize {
        self.max_queue_size
    }

    pub fn pending_queue_count(&self) -> usize {
        self.event_queue.lock().map(|q| q.len()).unwrap_or(0)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

impl Drop for AutomationEventWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}
