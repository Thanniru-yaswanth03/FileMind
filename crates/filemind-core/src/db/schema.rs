pub const CURRENT_SCHEMA_VERSION: i32 = 5;

pub const PRAGMAS_INIT: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;
"#;

pub const MIGRATION_V1_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scan_sessions (
    session_id TEXT PRIMARY KEY,
    root_path TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    status TEXT NOT NULL, -- 'running', 'completed', 'cancelled', 'failed', 'partial'
    total_files INTEGER NOT NULL DEFAULT 0,
    total_directories INTEGER NOT NULL DEFAULT 0,
    total_logical_bytes INTEGER NOT NULL DEFAULT 0,
    total_physical_bytes INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS scan_roots (
    root_id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    volume_guid TEXT NOT NULL,
    volume_label TEXT,
    filesystem_type TEXT NOT NULL,
    cluster_size_bytes INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    free_bytes INTEGER NOT NULL,
    last_scanned_at INTEGER
);

CREATE TABLE IF NOT EXISTS directories (
    directory_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES scan_sessions(session_id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    parent_path TEXT NOT NULL,
    depth INTEGER NOT NULL,
    total_size_bytes INTEGER NOT NULL DEFAULT 0,
    file_count INTEGER NOT NULL DEFAULT 0,
    dir_count INTEGER NOT NULL DEFAULT 0,
    mtime INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS files (
    file_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES scan_sessions(session_id) ON DELETE CASCADE,
    directory_path TEXT NOT NULL,
    filename TEXT NOT NULL,
    extension TEXT NOT NULL,
    file_type TEXT NOT NULL, -- 'Code', 'Media', 'Archives', 'Documents', 'Executables', 'Caches', 'Other'
    logical_size_bytes INTEGER NOT NULL,
    physical_size_bytes INTEGER NOT NULL,
    attributes INTEGER NOT NULL DEFAULT 0,
    ctime INTEGER NOT NULL,
    mtime INTEGER NOT NULL,
    atime INTEGER NOT NULL,
    ntfs_file_id TEXT,
    risk_tier TEXT NOT NULL DEFAULT 'safe',
    safety_rationale TEXT,
    is_locked INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS scan_errors (
    error_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES scan_sessions(session_id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    error_kind TEXT NOT NULL,
    error_message TEXT NOT NULL,
    occurred_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS scan_exclusions (
    exclusion_id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL UNIQUE,
    is_regex INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    setting_key TEXT PRIMARY KEY,
    setting_value_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Fast UI lookup & filtering indexes
CREATE INDEX IF NOT EXISTS idx_files_session ON files(session_id);
CREATE INDEX IF NOT EXISTS idx_files_dir ON files(directory_path);
CREATE INDEX IF NOT EXISTS idx_files_type ON files(file_type);
CREATE INDEX IF NOT EXISTS idx_files_size ON files(logical_size_bytes);
CREATE INDEX IF NOT EXISTS idx_files_ext ON files(extension);
CREATE INDEX IF NOT EXISTS idx_files_mtime ON files(mtime);

CREATE INDEX IF NOT EXISTS idx_dirs_session ON directories(session_id);
CREATE INDEX IF NOT EXISTS idx_dirs_path ON directories(path);
CREATE INDEX IF NOT EXISTS idx_dirs_parent ON directories(parent_path);
CREATE INDEX IF NOT EXISTS idx_dirs_size ON directories(total_size_bytes DESC);

CREATE INDEX IF NOT EXISTS idx_errors_session ON scan_errors(session_id);
"#;

pub const MIGRATION_V2_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS document_content_index (
    file_id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    mtime_epoch_ms INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    indexed_at_epoch_ms INTEGER NOT NULL,
    char_count INTEGER NOT NULL,
    status TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS fts_documents USING fts5 (
    file_id UNINDEXED,
    path UNINDEXED,
    filename,
    extension,
    content,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE INDEX IF NOT EXISTS idx_files_size_session ON files(session_id, logical_size_bytes);
"#;

pub const MIGRATION_V3_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS safety_audit_log (
    audit_id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_path TEXT NOT NULL,
    canonical_path TEXT NOT NULL,
    operation TEXT NOT NULL,
    classification TEXT NOT NULL,
    is_permitted INTEGER NOT NULL,
    rule_matches_json TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    block_reasons_json TEXT NOT NULL,
    toctou_verified INTEGER NOT NULL DEFAULT 0,
    executed INTEGER NOT NULL DEFAULT 0,
    execution_result TEXT,
    timestamp_epoch_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_safety_audit_path ON safety_audit_log(canonical_path);
CREATE INDEX IF NOT EXISTS idx_safety_audit_time ON safety_audit_log(timestamp_epoch_ms DESC);
CREATE INDEX IF NOT EXISTS idx_safety_audit_class ON safety_audit_log(classification);
"#;

pub const MIGRATION_V4_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS cleanup_plans (
    plan_id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    goal_type TEXT NOT NULL,
    target_bytes INTEGER,
    status TEXT NOT NULL,
    total_planned_bytes INTEGER NOT NULL DEFAULT 0,
    total_reclaimed_bytes INTEGER NOT NULL DEFAULT 0,
    total_items_count INTEGER NOT NULL DEFAULT 0,
    completed_items_count INTEGER NOT NULL DEFAULT 0,
    failed_items_count INTEGER NOT NULL DEFAULT 0,
    explanation TEXT
);

CREATE TABLE IF NOT EXISTS cleanup_plan_items (
    item_id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES cleanup_plans(plan_id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    canonical_path TEXT NOT NULL,
    logical_size_bytes INTEGER NOT NULL,
    physical_size_bytes INTEGER NOT NULL,
    source TEXT NOT NULL,
    risk_tier TEXT NOT NULL,
    classification TEXT NOT NULL,
    operation TEXT NOT NULL,
    is_reversible INTEGER NOT NULL DEFAULT 1,
    state TEXT NOT NULL,
    is_selected INTEGER NOT NULL DEFAULT 1,
    is_excluded INTEGER NOT NULL DEFAULT 0,
    quarantine_id TEXT,
    error_message TEXT,
    evidence_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS quarantine_records (
    quarantine_id TEXT PRIMARY KEY,
    original_path TEXT NOT NULL,
    canonical_path TEXT NOT NULL,
    quarantine_path TEXT NOT NULL,
    filename TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    original_ctime INTEGER NOT NULL,
    original_mtime INTEGER NOT NULL,
    attributes INTEGER NOT NULL DEFAULT 0,
    blake3_hash TEXT,
    quarantined_at INTEGER NOT NULL,
    status TEXT NOT NULL,
    reason TEXT NOT NULL,
    restored_at INTEGER,
    restore_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_cleanup_plans_status ON cleanup_plans(status);
CREATE INDEX IF NOT EXISTS idx_cleanup_items_plan ON cleanup_plan_items(plan_id);
CREATE INDEX IF NOT EXISTS idx_cleanup_items_state ON cleanup_plan_items(state);
CREATE INDEX IF NOT EXISTS idx_quarantine_status ON quarantine_records(status);
CREATE INDEX IF NOT EXISTS idx_quarantine_path ON quarantine_records(original_path);
"#;

pub const MIGRATION_V5_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS automation_rules (
    rule_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    trigger_type TEXT NOT NULL,
    trigger_config_json TEXT NOT NULL,
    conditions_json TEXT NOT NULL,
    actions_json TEXT NOT NULL,
    scope_paths_json TEXT NOT NULL,
    exclusion_paths_json TEXT NOT NULL,
    schedule_expression TEXT NOT NULL,
    requires_explicit_approval INTEGER NOT NULL DEFAULT 1,
    dry_run_mode INTEGER NOT NULL DEFAULT 1,
    max_retries INTEGER NOT NULL DEFAULT 0,
    cooldown_seconds INTEGER NOT NULL DEFAULT 300,
    max_files_per_run INTEGER NOT NULL DEFAULT 100,
    max_bytes_per_run INTEGER NOT NULL DEFAULT 10737418240,
    last_run_at INTEGER,
    last_status TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS automation_execution_history (
    run_id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES automation_rules(rule_id) ON DELETE CASCADE,
    rule_name TEXT NOT NULL,
    trigger_source TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    status TEXT NOT NULL,
    inputs_summary TEXT NOT NULL,
    actions_attempted_json TEXT NOT NULL,
    policy_decisions_json TEXT NOT NULL,
    result_summary TEXT NOT NULL,
    error_message TEXT,
    retries_attempted INTEGER NOT NULL DEFAULT 0,
    files_affected_count INTEGER NOT NULL DEFAULT 0,
    bytes_affected INTEGER NOT NULL DEFAULT 0,
    plan_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_auto_rules_enabled ON automation_rules(is_enabled);
CREATE INDEX IF NOT EXISTS idx_auto_history_rule ON automation_execution_history(rule_id);
CREATE INDEX IF NOT EXISTS idx_auto_history_started ON automation_execution_history(started_at DESC);
"#;
