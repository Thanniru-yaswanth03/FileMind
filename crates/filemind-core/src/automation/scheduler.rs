use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::automation::engine::AutomationEngine;
use crate::automation::watcher::AutomationEventWatcher;
use crate::cleanup::quarantine::QuarantineManager;
use crate::db::repository::Database;
use crate::models::{AutomationTrigger, SchedulerStatusResponse};

pub struct AutomationScheduler {
    db_path: PathBuf,
    quarantine: QuarantineManager,
    is_running: Arc<AtomicBool>,
    watcher: Arc<Mutex<Option<AutomationEventWatcher>>>,
}

impl AutomationScheduler {
    pub fn new(db_path: PathBuf, quarantine: QuarantineManager) -> Self {
        Self {
            db_path,
            quarantine,
            is_running: Arc::new(AtomicBool::new(false)),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts the background scheduler loop and file event watchers
    pub fn start(&self) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return; // Already running
        }

        let is_running_clone = Arc::clone(&self.is_running);
        let db_path_clone = self.db_path.clone();
        let quarantine_clone = self.quarantine.clone();
        let watcher_clone = Arc::clone(&self.watcher);

        // Initialize file event watcher for active roots if any exist
        let db = Database::new(db_path_clone.clone());
        if let Ok(conn) = db.get_connection() {
            if let Ok(rules) = Database::list_automation_rules(&conn) {
                let mut watch_paths = Vec::new();
                for r in rules {
                    if r.is_enabled {
                        if let AutomationTrigger::FileEvent { watch_path, .. } = &r.trigger {
                            let p = PathBuf::from(watch_path);
                            if p.exists() {
                                watch_paths.push(p);
                            }
                        }
                    }
                }

                if !watch_paths.is_empty() {
                    if let Ok(w) = AutomationEventWatcher::new(watch_paths, 1500, 5000) {
                        if let Ok(mut lock) = watcher_clone.lock() {
                            *lock = Some(w);
                        }
                    }
                }
            }
        }

        thread::spawn(move || {
            let db = Database::new(db_path_clone);

            while is_running_clone.load(Ordering::Relaxed) {
                // 1. Drain file events from watcher
                if let Ok(w_lock) = watcher_clone.lock() {
                    if let Some(ref w) = *w_lock {
                        let (settled, had_overflow) = w.drain_settled_events();
                        if had_overflow {
                            // Watcher queue overflowed: a reconciliation scan is recommended
                            println!("[AutomationScheduler] Watcher overflow detected; reconciliation hint logged.");
                        }

                        if !settled.is_empty() {
                            if let Ok(conn) = db.get_connection() {
                                if let Ok(rules) = Database::list_automation_rules(&conn) {
                                    for event in settled {
                                        for r in &rules {
                                            if r.is_enabled {
                                                if let AutomationTrigger::FileEvent { watch_path, event_kinds } = &r.trigger {
                                                    if event.path.starts_with(watch_path) && event_kinds.contains(&event.kind) {
                                                        let _ = AutomationEngine::execute_rule(
                                                            &conn,
                                                            r,
                                                            &quarantine_clone,
                                                            None,
                                                            Some(&format!("file_event:{}", event.kind)),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. Evaluate scheduled and threshold rules
                if let Ok(conn) = db.get_connection() {
                    if let Ok(rules) = Database::list_automation_rules(&conn) {
                        let now_epoch_ms = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);

                        for rule in rules {
                            if !rule.is_enabled {
                                continue;
                            }

                            let should_execute = match &rule.trigger {
                                AutomationTrigger::Schedule { interval_minutes, .. } => {
                                    let interval_ms = (interval_minutes.unwrap_or(60) as u64) * 60 * 1000;
                                    match rule.last_run_at {
                                        Some(last) => now_epoch_ms.saturating_sub(last) >= interval_ms,
                                        None => true,
                                    }
                                }
                                AutomationTrigger::DiskSpaceThreshold { .. } => {
                                    // Check threshold on every scheduler tick (subject to rule cooldown)
                                    true
                                }
                                _ => false,
                            };

                            if should_execute {
                                let _ = AutomationEngine::execute_rule(
                                    &conn,
                                    &rule,
                                    &quarantine_clone,
                                    None,
                                    Some("scheduler"),
                                );
                            }
                        }
                    }
                }

                // Sleep for scheduler interval (10 seconds)
                for _ in 0..10 {
                    if !is_running_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });
    }

    /// Stops the scheduler and terminates background threads
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        if let Ok(mut lock) = self.watcher.lock() {
            if let Some(w) = lock.take() {
                w.stop();
            }
        }
    }

    /// Inspects current status and Windows Scheduled Task configuration
    pub fn get_status(&self) -> SchedulerStatusResponse {
        let is_running = self.is_running.load(Ordering::Relaxed);
        let mut active_watchers_count = 0;
        let mut watched_paths = Vec::new();

        if let Ok(lock) = self.watcher.lock() {
            if let Some(ref w) = *lock {
                active_watchers_count = w.active_roots().len();
                watched_paths = w.active_roots().iter().map(|p| p.to_string_lossy().to_string()).collect();
            }
        }

        let db = Database::new(self.db_path.clone());
        let scheduled_rules_count = db
            .get_connection()
            .and_then(|c| Database::list_automation_rules(&c))
            .map(|rules| rules.into_iter().filter(|r| r.is_enabled).count())
            .unwrap_or(0);

        let (win_task_configured, task_name) = Self::check_windows_scheduled_task();

        SchedulerStatusResponse {
            is_running,
            active_watchers_count,
            watched_paths,
            scheduled_rules_count,
            windows_scheduled_task_configured: win_task_configured,
            windows_scheduled_task_name: task_name,
            operating_mode: if win_task_configured {
                "windows_task_and_daemon".to_string()
            } else if is_running {
                "native_daemon_background".to_string()
            } else {
                "in_process_ui_only".to_string()
            },
        }
    }

    /// Checks if a Windows Scheduled Task exists for FileMind via schtasks.exe
    fn check_windows_scheduled_task() -> (bool, String) {
        let task_name = "FileMindStorageScheduler";
        if let Ok(output) = std::process::Command::new("schtasks")
            .arg("/query")
            .arg("/tn")
            .arg(task_name)
            .output()
        {
            if output.status.success() {
                return (true, task_name.to_string());
            }
        }
        (false, task_name.to_string())
    }
}

