use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

pub struct FileSystemWatcher {
    root_path: PathBuf,
    _watcher: RecommendedWatcher,
    event_rx: Receiver<notify::Result<Event>>,
    is_running: Arc<AtomicBool>,
}

impl FileSystemWatcher {
    pub fn new(root_path: PathBuf) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(&root_path, RecursiveMode::Recursive)?;

        Ok(Self {
            root_path,
            _watcher: watcher,
            event_rx: rx,
            is_running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Try to receive pending events without blocking. Returns true if changes were detected.
    pub fn poll_events(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(res) = self.event_rx.try_recv() {
            if let Ok(evt) = res {
                events.push(evt);
            }
        }
        events
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
