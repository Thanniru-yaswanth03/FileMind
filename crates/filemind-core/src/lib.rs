pub mod accounting;
pub mod adapter;
pub mod automation;
pub mod cleanup;
pub mod copilot;
pub mod db;
pub mod developer;
pub mod duplicate;
pub mod incremental;
pub mod models;
pub mod safety;
pub mod scanner;
pub mod search;
pub mod server;

pub use accounting::StorageAccountant;
pub use adapter::{
    enumerate_all_volumes, inspect_file, is_descendant_of, normalize_path, query_volume_info,
    strip_verbatim_prefix, to_long_path, NativeFileMetadata, VolumeInfo,
};
pub use automation::{AutomationEngine, AutomationEventWatcher, AutomationScheduler};
pub use cleanup::{CleanupExecutor, CleanupPlanner, DryRunEngine, QuarantineManager};
pub use copilot::{CopilotDispatcher, CopilotToolRunner, OllamaClient};
pub use db::{Database, Migrator, RetentionManager};
pub use developer::DeveloperWorkspaceScanner;
pub use duplicate::{
    compute_file_blake3_streaming, compute_prefix_hash, verify_byte_by_byte, DuplicatePipeline,
    LocationEvaluator,
};
pub use incremental::{FileSystemWatcher, ReconcileReport, ReconciliationScanner};
pub use models::*;
pub use safety::{
    capture_identity_snapshot, evaluate_project_dependencies, evaluate_windows_protection,
    evaluate_wsl_protection, revalidate_handle, send_to_recycle_bin, AuditLogger,
    HandleIdentitySnapshot, ProjectEvidence, SafetyGuardian, ToctouError, WindowsRuleMatch,
    WslRuleMatch,
};
pub use scanner::{
    categorize_file, evaluate_risk_tier, execute_scan, ReparsePointPolicy, ScanError, ScanFilter,
    ScanResult,
};
pub use search::{DocumentExtractor, FtsEngine, MetadataSearchEngine, SecretFilter};
pub use server::{open_containing_folder, start_server, AppState};
