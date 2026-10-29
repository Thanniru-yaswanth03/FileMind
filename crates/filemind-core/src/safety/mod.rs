pub mod audit;
pub mod engine;
pub mod project;
pub mod rules;
pub mod toctou;
pub mod wsl;

pub use audit::AuditLogger;
pub use engine::{send_to_recycle_bin, SafetyGuardian};
pub use project::{evaluate_project_dependencies, is_secret_file, is_source_code_file, ProjectEvidence};
pub use rules::{evaluate_windows_protection, WindowsRuleMatch};
pub use toctou::{capture_identity_snapshot, revalidate_handle, HandleIdentitySnapshot, ToctouError};
pub use wsl::{evaluate_wsl_protection, WslRuleMatch};

