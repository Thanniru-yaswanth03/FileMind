use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FileCategory {
    Code,
    Media,
    Archives,
    Documents,
    Executables,
    Caches,
    Other,
}

impl Default for FileCategory {
    fn default() -> Self {
        FileCategory::Other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskTier {
    Safe,
    Caution,
    High,
    Danger,
}

impl Default for RiskTier {
    fn default() -> Self {
        RiskTier::Safe
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileAttributesDto {
    pub is_hidden: bool,
    pub is_system: bool,
    pub is_read_only: bool,
    pub is_reparse_point: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileItemDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub parent_path: String,
    pub extension: String,
    pub size_bytes: u64,
    pub physical_size_bytes: u64,
    pub category: FileCategory,
    pub mtime: String,
    pub ctime: String,
    pub atime: String,
    pub mtime_epoch_ms: Option<u64>,
    pub attributes: FileAttributesDto,
    pub ntfs_file_id: Option<String>,
    pub risk_tier: RiskTier,
    pub safety_rationale: String,
    pub is_locked: bool,
    pub is_directory: bool,
    pub child_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryItemDto {
    pub id: String,
    pub path: String,
    pub parent_path: String,
    pub depth: u32,
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub mtime: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Running,
    Completed,
    Cancelled,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSessionDto {
    pub session_id: String,
    pub root_path: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: SessionStatus,
    pub total_files: u64,
    pub total_directories: u64,
    pub logical_size_bytes: u64,
    pub physical_size_bytes: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    PermissionDenied,
    Locked,
    Disappeared,
    PathTooLong,
    IoError,
    ReparsePolicy,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanErrorDto {
    pub id: i64,
    pub session_id: String,
    pub path: String,
    pub error_kind: ErrorKind,
    pub message: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageAccountingDto {
    pub volume_total_bytes: u64,
    pub volume_free_bytes: u64,
    pub volume_used_bytes: u64,
    pub logical_size_bytes: u64,
    pub physical_size_bytes: u64,
    pub indexed_files_total: u64,
    pub inaccessible_regions_count: u64,
    pub exact_figures_note: String,
    pub estimated_figures_note: String,
    pub hardlinks_deduplicated_count: u64,
    pub hardlinks_bytes_saved: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummaryDto {
    pub session: ScanSessionDto,
    pub storage_accounting: StorageAccountingDto,
    pub recent_errors: Vec<ScanErrorDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressState {
    pub percent: f64,
    pub current_path: String,
    pub files_count: u64,
    pub bytes_count: u64,
    pub directories_count: u64,
    pub errors_count: u64,
    pub elapsed_ms: u64,
    pub is_scanning: bool,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRankingDto {
    pub path: String,
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub percentage_of_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryBreakdownDto {
    pub category: FileCategory,
    pub total_bytes: u64,
    pub file_count: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeRootDto {
    pub id: String,
    pub drive_letter: String,
    pub label: String,
    pub filesystem: String,
    pub cluster_size_bytes: u64,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub is_system_drive: bool,
    pub last_scanned_at: String,
    pub scan_freshness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileListQuery {
    pub session_id: Option<String>,
    pub path_prefix: Option<String>,
    pub category: Option<String>,
    pub extension: Option<String>,
    pub search: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub sort_by: Option<String>,   // "size", "name", "mtime", "category"
    pub sort_order: Option<String>,// "asc", "desc"
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListResponse {
    pub total_count: u64,
    pub total_bytes: u64,
    pub files: Vec<FileItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartScanRequest {
    pub path: String,
    pub max_depth: Option<u32>,
    pub follow_symlinks: Option<bool>,
    pub excluded_patterns: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Phase 3: Duplicate Detection Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateConfidence {
    Unverified,
    SizeMatchOnly,
    PrefixHashMatch,
    CryptographicHashMatch,
    ByteForByteVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateFileStatus {
    Ok,
    ChangedDuringHash,
    Inaccessible,
    LockError,
    Disappeared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationTag {
    RecommendedKeep,
    ProtectedSystem,
    ProjectMember,
    UserPreferred,
    BackupLocation,
    RedundantCopy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFileDto {
    pub id: i64,
    pub path: String,
    pub filename: String,
    pub directory_path: String,
    pub size_bytes: u64,
    pub mtime_epoch_ms: u64,
    pub formatted_mtime: String,
    pub blake3_hash: Option<String>,
    pub status: DuplicateFileStatus,
    pub location_tags: Vec<LocationTag>,
    pub is_recommended_keep: bool,
    pub recommendation_rationale: String,
    pub is_selected_for_retention: bool,
    pub is_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupDto {
    pub group_id: String,
    pub size_bytes: u64,
    pub file_count: usize,
    pub wasted_bytes: u64,
    pub confidence: DuplicateConfidence,
    pub hash: Option<String>,
    pub group_rationale: String,
    pub files: Vec<DuplicateFileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateScanRequest {
    pub session_id: Option<String>,
    pub min_size_bytes: Option<u64>,
    pub max_groups: Option<usize>,
    pub verify_byte_by_byte: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateScanResponse {
    pub groups: Vec<DuplicateGroupDto>,
    pub total_duplicate_files: usize,
    pub total_wasted_bytes: u64,
    pub unverified_candidates_count: usize,
    pub verified_groups_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyGroupRequest {
    pub group_id: String,
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyGroupResponse {
    pub group_id: String,
    pub confidence: DuplicateConfidence,
    pub verified_byte_for_byte: bool,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Phase 3: Content Search & Indexing Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchQuery {
    pub query: String,
    pub extension: Option<String>,
    pub path_prefix: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResultItem {
    pub file_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub mtime_epoch_ms: u64,
    pub snippet: String,
    pub is_stale: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResponse {
    pub total_matches: u64,
    pub results: Vec<ContentSearchResultItem>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentIndexProgress {
    pub indexed_documents: u64,
    pub total_documents: u64,
    pub current_file: String,
    pub skipped_sensitive: u64,
    pub truncated_count: u64,
    pub failed_count: u64,
    pub is_indexing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentIndexStatusResponse {
    pub total_indexed_documents: u64,
    pub total_extracted_characters: u64,
    pub skipped_sensitive_count: u64,
    pub truncated_documents_count: u64,
    pub is_active: bool,
}

// ---------------------------------------------------------------------------
// Phase 4: Smart Delete Guardian and Dependency Intelligence Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SafetyClassification {
    LowRiskCandidate,
    ReviewRequired,
    HighRiskOrUnknown,
    ProtectedAndBlocked,
}

impl Default for SafetyClassification {
    fn default() -> Self {
        SafetyClassification::HighRiskOrUnknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllowedOperation {
    Permitted,
    RequiresExplicitUserConfirmation,
    Blocked,
}

impl Default for AllowedOperation {
    fn default() -> Self {
        AllowedOperation::Blocked
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    WindowsPolicy,
    WslDetector,
    DeveloperManifest,
    GitRepository,
    FileSystemAttributes,
    LockInspection,
    ToctouVerification,
    UserExclusion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedEvidenceItem {
    pub source: EvidenceSource,
    pub claim: String,
    pub supports_safety: bool,
    pub verification_status: String, // "Verified", "Unverified", "Inferred", "Failed"
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyAssessmentDto {
    pub target_path: String,
    pub canonical_path: String,
    pub file_identity: Option<String>,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub classification: SafetyClassification,
    pub allowed_operation: AllowedOperation,
    pub is_permitted: bool,
    pub rule_matches: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub consequences: Vec<String>,
    pub recovery_possibility: String,
    pub block_reason: Option<String>,
    pub evidence: Vec<TypedEvidenceItem>,
    pub evaluated_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyAuditRecordDto {
    pub audit_id: i64,
    pub target_path: String,
    pub canonical_path: String,
    pub operation: String,
    pub classification: SafetyClassification,
    pub is_permitted: bool,
    pub rule_matches: Vec<String>,
    pub evidence: Vec<TypedEvidenceItem>,
    pub block_reasons: Vec<String>,
    pub toctou_verified: bool,
    pub executed: bool,
    pub execution_result: Option<String>,
    pub timestamp_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyEvaluationRequest {
    pub paths: Vec<String>,
    pub operation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyEvaluationResponse {
    pub assessments: Vec<SafetyAssessmentDto>,
    pub total_evaluated: usize,
    pub permitted_count: usize,
    pub review_required_count: usize,
    pub blocked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCleanupRequest {
    pub paths: Vec<String>,
    pub operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupExecutionItemResult {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
    pub toctou_verified: bool,
    pub method_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCleanupResponse {
    pub results: Vec<CleanupExecutionItemResult>,
    pub total_succeeded: usize,
    pub total_failed: usize,
    pub total_bytes_reclaimed: u64,
}

// ---------------------------------------------------------------------------
// Phase 5: Intelligent Cleanup Planning & Quarantine Recovery Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateSource {
    DeveloperArtifact,
    DuplicateGroup,
    LargeFile,
    TemporaryArtifact,
    UserSelected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanItemOperation {
    Quarantine,
    RecycleBin,
    PermanentDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanItemState {
    Pending,
    DryRunOk,
    Executing,
    Succeeded,
    Failed,
    SkippedToctou,
    SkippedConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    DryRunValidated,
    Executing,
    Completed,
    PartiallyCompleted,
    Aborted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanItemDto {
    pub item_id: String,
    pub plan_id: String,
    pub path: String,
    pub canonical_path: String,
    pub logical_size_bytes: u64,
    pub physical_size_bytes: u64,
    pub source: CandidateSource,
    pub risk_tier: RiskTier,
    pub classification: SafetyClassification,
    pub operation: PlanItemOperation,
    pub is_reversible: bool,
    pub state: PlanItemState,
    pub is_selected: bool,
    pub is_excluded: bool,
    pub quarantine_id: Option<String>,
    pub error_message: Option<String>,
    pub evidence_summary: String,
    pub potential_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanDto {
    pub plan_id: String,
    pub created_at_epoch_ms: u64,
    pub goal_type: String,
    pub target_bytes: Option<u64>,
    pub status: PlanStatus,
    pub total_planned_bytes: u64,
    pub total_physical_reclaimable_bytes: u64,
    pub total_reclaimed_bytes: u64,
    pub items_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub is_target_attainable_safely: bool,
    pub unmet_target_explanation: Option<String>,
    pub risk_counts: std::collections::HashMap<String, usize>,
    pub items: Vec<CleanupPlanItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanRequest {
    pub goal: String,
    pub target_bytes: Option<u64>,
    pub folder_path: Option<String>,
    pub selected_paths: Option<Vec<String>>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunRequest {
    pub plan_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunItemPrediction {
    pub item_id: String,
    pub path: String,
    pub operation: PlanItemOperation,
    pub can_proceed: bool,
    pub reversibility: String,
    pub expected_freed_bytes: u64,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunResponse {
    pub plan_id: String,
    pub can_execute: bool,
    pub validated_items_count: usize,
    pub total_expected_freed_bytes: u64,
    pub conflicts_detected: Vec<String>,
    pub predictions: Vec<DryRunItemPrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanRequest {
    pub plan_id: String,
    pub selected_item_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanResponse {
    pub plan_id: String,
    pub status: PlanStatus,
    pub succeeded_count: usize,
    pub failed_count: usize,
    pub total_bytes_reclaimed: u64,
    pub quarantined_count: usize,
    pub recycled_count: usize,
    pub deleted_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineRecordDto {
    pub quarantine_id: String,
    pub original_path: String,
    pub canonical_path: String,
    pub quarantine_path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub original_ctime: u64,
    pub original_mtime: u64,
    pub attributes: u32,
    pub blake3_hash: Option<String>,
    pub quarantined_at_epoch_ms: u64,
    pub status: String,
    pub reason: String,
    pub restored_at_epoch_ms: Option<u64>,
    pub restore_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRequest {
    pub quarantine_ids: Vec<String>,
    pub alternate_destination: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreItemResult {
    pub quarantine_id: String,
    pub original_path: String,
    pub restored_path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResponse {
    pub results: Vec<RestoreItemResult>,
    pub total_restored: usize,
    pub total_failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeQuarantineRequest {
    pub quarantine_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeResponse {
    pub purged_count: usize,
    pub bytes_permanently_freed: u64,
}

// ---------------------------------------------------------------------------
// Phase 6: JARVIS Desktop Copilot & Local AI Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CopilotToolName {
    SearchFiles,
    InspectFile,
    SummarizeScan,
    FindDuplicates,
    ExplainSafety,
    GenerateCleanupPlan,
    StartReadOnlyScan,
    GetOperationHistory,
    CheckActionPermitted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotToolCall {
    pub tool: CopilotToolName,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotToolResult {
    pub tool: CopilotToolName,
    pub success: bool,
    pub summary: String,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotEvidenceItem {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub risk_tier: String,
    pub reason: String,
    pub action_type: String, // 'inspect', 'open_folder', 'plan_review'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotOperationProposal {
    pub plan_id: Option<String>,
    pub title: String,
    pub description: String,
    pub estimated_freed_bytes: u64,
    pub target_paths_count: usize,
    pub reversibility: String,
    pub requires_explicit_approval: bool,
    pub screen_action: String, // 'cleanup-planner', 'safety-center', 'activity-recovery'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotMessageDto {
    pub id: String,
    pub sender: String, // 'user' | 'assistant'
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotQueryRequest {
    pub query: String,
    pub history: Option<Vec<CopilotMessageDto>>,
    pub allow_ai: Option<bool>,
    pub ollama_endpoint: Option<String>,
    pub ollama_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotQueryResponse {
    pub answer: String,
    pub tool_executions: Vec<CopilotToolResult>,
    pub evidence: Vec<CopilotEvidenceItem>,
    pub proposal: Option<CopilotOperationProposal>,
    pub provider: String, // 'ollama' | 'local_rules'
    pub is_stale_index: bool,
    pub suggested_followups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub estimated_ram_mb: u64,
    pub parameter_size: Option<String>,
    pub family: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatusResponse {
    pub is_available: bool,
    pub endpoint: String,
    pub models: Vec<OllamaModelInfo>,
    pub active_model: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteToolRequest {
    pub tool: CopilotToolName,
    pub arguments: serde_json::Value,
}

// ==========================================
// Phase 7: Local Automation Engine Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationTrigger {
    Schedule {
        frequency: String, // 'hourly' | 'daily' | 'weekly' | 'interval'
        interval_minutes: Option<u32>,
    },
    DiskSpaceThreshold {
        drive_letter: String,
        threshold_gb: f64,
    },
    PostScan {
        min_files_scanned: Option<usize>,
    },
    FileEvent {
        watch_path: String,
        event_kinds: Vec<String>, // 'create' | 'modify' | 'delete'
    },
    FolderGrowth {
        folder_path: String,
        threshold_mb: f64,
    },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationCondition {
    DiskFreeBelowBytes { bytes: u64 },
    FileSizeAboveBytes { bytes: u64 },
    FileTypeMatches { file_type: String },
    PathPatternMatches { pattern: String },
    ArtifactAgeDays { min_days: u32 },
    GitCleanOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationAction {
    NotifyUser {
        title: String,
        message: String,
        severity: String, // 'info' | 'warning' | 'alert'
    },
    GenerateReport {
        output_path: Option<String>,
        format: String, // 'json' | 'markdown'
    },
    CreateCleanupPlan {
        goal_type: String, // 'free_bytes' | 'developer_artifacts' | 'duplicates'
        target_bytes: Option<u64>,
    },
    StageToQuarantine {
        dry_run: bool,
    },
    OrganizeToFolder {
        target_directory: String,
        subfolder_pattern: String, // 'by_extension' | 'by_year_month'
    },
    RunScan {
        root_path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLimits {
    pub max_files_per_run: u32,
    pub max_bytes_per_run: u64,
    pub cooldown_seconds: u32,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_files_per_run: 100,
            max_bytes_per_run: 10 * 1024 * 1024 * 1024, // 10 GB
            cooldown_seconds: 300,                       // 5 minutes
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_seconds: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            backoff_seconds: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRuleDto {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub is_enabled: bool,
    pub trigger: AutomationTrigger,
    pub conditions: Vec<AutomationCondition>,
    pub actions: Vec<AutomationAction>,
    pub scope_paths: Vec<String>,
    pub exclusion_paths: Vec<String>,
    pub schedule_expression: String,
    pub requires_explicit_approval: bool,
    pub dry_run_mode: bool,
    pub retry_policy: RetryPolicy,
    pub execution_limits: ExecutionLimits,
    pub last_run_at: Option<u64>,
    pub last_status: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationExecutionHistoryDto {
    pub run_id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub trigger_source: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub status: String, // 'success' | 'partial' | 'blocked' | 'failed' | 'dry_run_completed'
    pub inputs_summary: String,
    pub actions_attempted: Vec<String>,
    pub policy_decisions: Vec<String>,
    pub result_summary: String,
    pub error_message: Option<String>,
    pub retries_attempted: u32,
    pub files_affected_count: u32,
    pub bytes_affected: u64,
    pub plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAutomationRuleRequest {
    pub name: String,
    pub description: String,
    pub trigger: AutomationTrigger,
    pub conditions: Vec<AutomationCondition>,
    pub actions: Vec<AutomationAction>,
    pub scope_paths: Vec<String>,
    pub exclusion_paths: Option<Vec<String>>,
    pub schedule_expression: Option<String>,
    pub requires_explicit_approval: Option<bool>,
    pub dry_run_mode: Option<bool>,
    pub retry_policy: Option<RetryPolicy>,
    pub execution_limits: Option<ExecutionLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerRuleRequest {
    pub dry_run_override: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerStatusResponse {
    pub is_running: bool,
    pub active_watchers_count: usize,
    pub watched_paths: Vec<String>,
    pub scheduled_rules_count: usize,
    pub windows_scheduled_task_configured: bool,
    pub windows_scheduled_task_name: String,
    pub operating_mode: String,
}

// ========================================================
// Phase 7: Developer Workspace Intelligence Models
// ========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperArtifactDto {
    pub name: String,
    pub relative_path: String,
    pub full_path: String,
    pub size_bytes: u64,
    pub category: String, // 'Dependencies' | 'Build Output' | 'Virtual Environment' | 'Cache'
    pub is_regenerable: bool,
    pub reversibility: String,
    pub cleanup_procedure: String,
    pub risk_tier: RiskTier,
    pub risks_and_questions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperProjectDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub ecosystem: String, // 'Node.js' | 'Rust' | 'Python' | 'Go' | 'Java' | 'Docker' | 'Git'
    pub manifest_file: String,
    pub lockfile_file: Option<String>,
    pub git_status: String, // 'clean' | 'dirty' | 'untracked_work' | 'no_repo'
    pub total_artifact_size_bytes: u64,
    pub artifacts: Vec<DeveloperArtifactDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProjectArtifactsRequest {
    pub project_ids: Vec<String>,
    pub dry_run: Option<bool>,
    pub target_plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProjectArtifactsResponse {
    pub plan_id: String,
    pub staged_artifacts_count: usize,
    pub total_bytes: u64,
    pub requires_explicit_approval: bool,
}





