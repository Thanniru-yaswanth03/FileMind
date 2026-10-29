pub mod engine;
pub mod scheduler;
pub mod watcher;

pub use engine::AutomationEngine;
pub use scheduler::AutomationScheduler;
pub use watcher::{AutomationEventWatcher, CoalescedFileEvent};

