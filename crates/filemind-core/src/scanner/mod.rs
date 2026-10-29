pub mod category;
pub mod error;
pub mod filter;
pub mod progress;
pub mod walker;

pub use category::{categorize_file, evaluate_risk_tier};
pub use error::ScanError;
pub use filter::{ReparsePointPolicy, ScanFilter};
pub use progress::ProgressTracker;
pub use walker::{execute_scan, InternalDirectoryRecord, ScanResult};
