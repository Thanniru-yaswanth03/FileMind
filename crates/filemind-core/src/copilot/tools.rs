use rusqlite::Connection;
use serde_json::json;
use std::path::Path;

use crate::adapter::path::{normalize_path, strip_verbatim_prefix};
use crate::cleanup::planner::CleanupPlanner;
use crate::duplicate::pipeline::DuplicatePipeline;
use crate::models::{
    CopilotToolName, CopilotToolResult, CreatePlanRequest, DuplicateScanRequest, FileListQuery,
    SafetyClassification,
};
use crate::safety::SafetyGuardian;
use crate::search::MetadataSearchEngine;

pub struct CopilotToolRunner;

impl CopilotToolRunner {
    /// Validates path argument to guard against path traversal attacks.
    pub fn sanitize_path(path_str: &str) -> Result<String, String> {
        let trimmed = path_str.trim();
        if trimmed.is_empty() {
            return Err("Target path cannot be empty".to_string());
        }
        if trimmed.contains("..") {
            return Err("Path traversal (..) is strictly prohibited".to_string());
        }
        let clean = strip_verbatim_prefix(trimmed);
        Ok(normalize_path(Path::new(&clean)))
    }

    /// Executes a typed tool against the FileMind core and SQLite database.
    pub fn execute(
        tool: CopilotToolName,
        args: &serde_json::Value,
        conn: &Connection,
    ) -> CopilotToolResult {
        match tool {
            CopilotToolName::SearchFiles => Self::tool_search_files(args, conn),
            CopilotToolName::InspectFile => Self::tool_inspect_file(args),
            CopilotToolName::SummarizeScan => Self::tool_summarize_scan(conn),
            CopilotToolName::FindDuplicates => Self::tool_find_duplicates(args, conn),
            CopilotToolName::ExplainSafety => Self::tool_explain_safety(args),
            CopilotToolName::GenerateCleanupPlan => Self::tool_generate_cleanup_plan(args, conn),
            CopilotToolName::StartReadOnlyScan => Self::tool_start_scan(args),
            CopilotToolName::GetOperationHistory => Self::tool_get_operation_history(args, conn),
            CopilotToolName::CheckActionPermitted => Self::tool_check_action_permitted(args),
        }
    }

    fn tool_search_files(args: &serde_json::Value, conn: &Connection) -> CopilotToolResult {
        let query = args.get("query").and_then(|v| v.as_str());
        let extension = args.get("extension").and_then(|v| v.as_str());
        let category = args.get("category").and_then(|v| v.as_str());
        let prefix = args.get("prefix").and_then(|v| v.as_str());
        let min_size = args.get("min_size").and_then(|v| v.as_u64());
        let max_size = args.get("max_size").and_then(|v| v.as_u64());
        let sort_by = args.get("sort_by").and_then(|v| v.as_str()).unwrap_or("size_desc");
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25).min(100) as usize;

        let (sort_field, sort_order) = match sort_by {
            "mtime_desc" => ("mtime", "desc"),
            "size_desc" => ("size", "desc"),
            "name_asc" => ("name", "asc"),
            _ => ("size", "desc"),
        };

        let params = FileListQuery {
            session_id: None,
            path_prefix: prefix.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
            extension: extension.map(|s| s.to_string()),
            search: query.map(|s| s.to_string()),
            min_size,
            max_size,
            sort_by: Some(sort_field.to_string()),
            sort_order: Some(sort_order.to_string()),
            limit: Some(limit as u32),
            offset: Some(0),
        };

        match MetadataSearchEngine::search(conn, &params) {
            Ok(resp) => {
                let count = resp.files.len();
                let summary = format!("Found {} matching files in local index.", count);
                CopilotToolResult {
                    tool: CopilotToolName::SearchFiles,
                    success: true,
                    summary,
                    data: json!(resp),
                    error: None,
                }
            }
            Err(e) => CopilotToolResult {
                tool: CopilotToolName::SearchFiles,
                success: false,
                summary: "File search failed".to_string(),
                data: json!({}),
                error: Some(e.to_string()),
            },
        }
    }

    fn tool_inspect_file(args: &serde_json::Value) -> CopilotToolResult {
        let raw_path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return CopilotToolResult {
                    tool: CopilotToolName::InspectFile,
                    success: false,
                    summary: "Missing 'path' argument".to_string(),
                    data: json!({}),
                    error: Some("Parameter 'path' is required".to_string()),
                }
            }
        };

        let norm_path = match Self::sanitize_path(raw_path) {
            Ok(p) => p,
            Err(e) => {
                return CopilotToolResult {
                    tool: CopilotToolName::InspectFile,
                    success: false,
                    summary: "Invalid path specification".to_string(),
                    data: json!({}),
                    error: Some(e.to_string()),
                }
            }
        };

        let path_obj = Path::new(&norm_path);
        if !path_obj.exists() {
            return CopilotToolResult {
                tool: CopilotToolName::InspectFile,
                success: false,
                summary: format!("Target '{}' does not exist on disk", norm_path),
                data: json!({ "path": norm_path, "exists": false }),
                error: Some("File not found".to_string()),
            };
        }

        let assessment = SafetyGuardian::evaluate_target(path_obj, "inspect");
        let meta = std::fs::metadata(path_obj).ok();
        let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);

        let data = json!({
            "path": norm_path,
            "filename": path_obj.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "sizeBytes": size_bytes,
            "isDirectory": is_dir,
            "classification": assessment.classification,
            "isPermitted": assessment.is_permitted,
            "ruleMatches": assessment.rule_matches,
            "evidence": assessment.evidence,
        });

        CopilotToolResult {
            tool: CopilotToolName::InspectFile,
            success: true,
            summary: format!(
                "Inspected target '{}' ({}, classification: {:?})",
                norm_path,
                if is_dir { "Directory" } else { "File" },
                assessment.classification
            ),
            data,
            error: None,
        }
    }

    fn tool_summarize_scan(conn: &Connection) -> CopilotToolResult {
        // Query latest scan session
        let session_res = conn.query_row(
            r#"
            SELECT session_id, root_path, started_at, completed_at, status,
                   total_files, total_directories, total_logical_bytes, total_physical_bytes, error_count
            FROM scan_sessions
            ORDER BY started_at DESC
            LIMIT 1
            "#,
            [],
            |row| {
                Ok(json!({
                    "sessionId": row.get::<_, String>(0)?,
                    "rootPath": row.get::<_, String>(1)?,
                    "startedAt": row.get::<_, i64>(2)?,
                    "completedAt": row.get::<_, Option<i64>>(3)?,
                    "status": row.get::<_, String>(4)?,
                    "totalFiles": row.get::<_, i64>(5)?,
                    "totalDirectories": row.get::<_, i64>(6)?,
                    "totalLogicalBytes": row.get::<_, i64>(7)?,
                    "totalPhysicalBytes": row.get::<_, i64>(8)?,
                    "errorCount": row.get::<_, i64>(9)?,
                }))
            },
        );

        let session_info = match session_res {
            Ok(info) => info,
            Err(_) => {
                return CopilotToolResult {
                    tool: CopilotToolName::SummarizeScan,
                    success: true,
                    summary: "No storage scans have been performed yet. Index is currently empty."
                        .to_string(),
                    data: json!({
                        "hasScanned": false,
                        "message": "Storage index is empty. Please run a scan on a selected root folder.",
                    }),
                    error: None,
                }
            }
        };

        // Query top largest folders
        let mut top_folders = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            r#"
            SELECT path, total_size_bytes, file_count
            FROM directories
            ORDER BY total_size_bytes DESC
            LIMIT 6
            "#,
        ) {
            let rows = stmt.query_map([], |row| {
                Ok(json!({
                    "path": row.get::<_, String>(0)?,
                    "sizeBytes": row.get::<_, i64>(1)?,
                    "fileCount": row.get::<_, i64>(2)?,
                }))
            });
            if let Ok(iter) = rows {
                for r in iter.flatten() {
                    top_folders.push(r);
                }
            }
        }

        // Query category breakdown
        let mut categories = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            r#"
            SELECT file_type, COUNT(*), SUM(logical_size_bytes)
            FROM files
            GROUP BY file_type
            ORDER BY SUM(logical_size_bytes) DESC
            "#,
        ) {
            let rows = stmt.query_map([], |row| {
                Ok(json!({
                    "category": row.get::<_, String>(0)?,
                    "fileCount": row.get::<_, i64>(1)?,
                    "totalBytes": row.get::<_, i64>(2)?,
                }))
            });
            if let Ok(iter) = rows {
                for r in iter.flatten() {
                    categories.push(r);
                }
            }
        }

        let total_files = session_info["totalFiles"].as_i64().unwrap_or(0);
        let total_bytes = session_info["totalLogicalBytes"].as_i64().unwrap_or(0);
        let root = session_info["rootPath"].as_str().unwrap_or("C:\\");

        let summary = format!(
            "Scan on '{}' recorded {} files totaling {:.2} GB.",
            root,
            total_files,
            total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        CopilotToolResult {
            tool: CopilotToolName::SummarizeScan,
            success: true,
            summary,
            data: json!({
                "session": session_info,
                "topFolders": top_folders,
                "categories": categories,
            }),
            error: None,
        }
    }

    fn tool_find_duplicates(args: &serde_json::Value, conn: &Connection) -> CopilotToolResult {
        let max_groups = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
        let verify_bytes = args.get("verify_bytes").and_then(|v| v.as_bool()).unwrap_or(false);

        let req = DuplicateScanRequest {
            session_id: None,
            min_size_bytes: Some(1024), // 1 KB min
            verify_byte_by_byte: Some(verify_bytes),
            max_groups: Some(max_groups),
        };

        match DuplicatePipeline::execute(conn, &req) {
            Ok(result) => {
                let summary = format!(
                    "Identified {} duplicate groups with {:.2} MB wasted storage.",
                    result.groups.len(),
                    result.total_wasted_bytes as f64 / (1024.0 * 1024.0)
                );
                CopilotToolResult {
                    tool: CopilotToolName::FindDuplicates,
                    success: true,
                    summary,
                    data: json!(result),
                    error: None,
                }
            }
            Err(e) => CopilotToolResult {
                tool: CopilotToolName::FindDuplicates,
                success: false,
                summary: "Duplicate detection failed".to_string(),
                data: json!({}),
                error: Some(e.to_string()),
            },
        }
    }

    fn tool_explain_safety(args: &serde_json::Value) -> CopilotToolResult {
        let raw_path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return CopilotToolResult {
                    tool: CopilotToolName::ExplainSafety,
                    success: false,
                    summary: "Path parameter is missing".to_string(),
                    data: json!({}),
                    error: Some("Missing 'path' parameter".to_string()),
                }
            }
        };

        let norm_path = match Self::sanitize_path(raw_path) {
            Ok(p) => p,
            Err(e) => {
                return CopilotToolResult {
                    tool: CopilotToolName::ExplainSafety,
                    success: false,
                    summary: "Invalid path".to_string(),
                    data: json!({}),
                    error: Some(e.to_string()),
                }
            }
        };

        let op = args.get("operation").and_then(|v| v.as_str()).unwrap_or("delete");
        let path_obj = Path::new(&norm_path);
        let assessment = SafetyGuardian::evaluate_target(path_obj, op);

        let summary = format!(
            "Target '{}': Classification={:?}, Permitted={}, Reason={}",
            norm_path,
            assessment.classification,
            assessment.is_permitted,
            assessment.block_reason.as_deref().unwrap_or("No policy restriction violated")
        );

        CopilotToolResult {
            tool: CopilotToolName::ExplainSafety,
            success: true,
            summary,
            data: json!(assessment),
            error: None,
        }
    }

    fn tool_generate_cleanup_plan(args: &serde_json::Value, conn: &Connection) -> CopilotToolResult {
        let goal = args.get("goal").and_then(|v| v.as_str()).unwrap_or("maximize_safe");
        let target_bytes = args.get("target_bytes").and_then(|v| v.as_u64());
        let folder_path = args.get("folder_path").and_then(|v| v.as_str()).map(|s| s.to_string());
        let selected_paths: Option<Vec<String>> = args.get("selected_paths").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
        });

        let req = CreatePlanRequest {
            goal: goal.to_string(),
            target_bytes,
            folder_path,
            selected_paths,
            session_id: None,
        };

        match CleanupPlanner::create_plan(&req, conn) {
            Ok(plan) => {
                let summary = format!(
                    "Generated cleanup plan '{}' with {} candidates (projected {:.2} MB recoverable). Safe attainability: {}.",
                    plan.plan_id,
                    plan.items_count,
                    plan.total_physical_reclaimable_bytes as f64 / (1024.0 * 1024.0),
                    plan.is_target_attainable_safely
                );
                CopilotToolResult {
                    tool: CopilotToolName::GenerateCleanupPlan,
                    success: true,
                    summary,
                    data: json!(plan),
                    error: None,
                }
            }
            Err(e) => CopilotToolResult {
                tool: CopilotToolName::GenerateCleanupPlan,
                success: false,
                summary: "Failed to create cleanup plan".to_string(),
                data: json!({}),
                error: Some(e.to_string()),
            },
        }
    }

    fn tool_start_scan(args: &serde_json::Value) -> CopilotToolResult {
        let target_path = args.get("path").and_then(|v| v.as_str()).unwrap_or("C:\\");
        let clean = match Self::sanitize_path(target_path) {
            Ok(p) => p,
            Err(e) => {
                return CopilotToolResult {
                    tool: CopilotToolName::StartReadOnlyScan,
                    success: false,
                    summary: "Invalid scan root path".to_string(),
                    data: json!({}),
                    error: Some(e.to_string()),
                }
            }
        };

        // StorageScanner triggers a non-destructive read-only scan
        CopilotToolResult {
            tool: CopilotToolName::StartReadOnlyScan,
            success: true,
            summary: format!(
                "Read-only storage scan initiated for '{}'. The index will update in real time.",
                clean
            ),
            data: json!({ "targetPath": clean, "mode": "read_only" }),
            error: None,
        }
    }

    fn tool_get_operation_history(args: &serde_json::Value, conn: &Connection) -> CopilotToolResult {
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(15).min(50) as usize;

        let mut quarantine_items = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            r#"
            SELECT quarantine_id, original_path, size_bytes, quarantined_at, status, reason
            FROM quarantine_records
            ORDER BY quarantined_at DESC
            LIMIT ?1
            "#,
        ) {
            let rows = stmt.query_map([limit], |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "originalPath": row.get::<_, String>(1)?,
                    "sizeBytes": row.get::<_, i64>(2)?,
                    "quarantinedAtEpochMs": row.get::<_, i64>(3)?,
                    "status": row.get::<_, String>(4)?,
                    "reason": row.get::<_, String>(5)?,
                }))
            });
            if let Ok(iter) = rows {
                for r in iter.flatten() {
                    quarantine_items.push(r);
                }
            }
        }

        let mut audit_entries = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            r#"
            SELECT audit_id, target_path, operation, classification, is_permitted, timestamp_epoch_ms
            FROM safety_audit_log
            ORDER BY timestamp_epoch_ms DESC
            LIMIT ?1
            "#,
        ) {
            let rows = stmt.query_map([limit], |row| {
                Ok(json!({
                    "auditId": row.get::<_, i64>(0)?,
                    "targetPath": row.get::<_, String>(1)?,
                    "operation": row.get::<_, String>(2)?,
                    "classification": row.get::<_, String>(3)?,
                    "isPermitted": row.get::<_, i32>(4)? == 1,
                    "timestampEpochMs": row.get::<_, i64>(5)?,
                }))
            });
            if let Ok(iter) = rows {
                for r in iter.flatten() {
                    audit_entries.push(r);
                }
            }
        }

        let summary = format!(
            "Retrieved {} quarantine items and {} safety audit records.",
            quarantine_items.len(),
            audit_entries.len()
        );

        CopilotToolResult {
            tool: CopilotToolName::GetOperationHistory,
            success: true,
            summary,
            data: json!({
                "quarantine": quarantine_items,
                "auditLog": audit_entries,
            }),
            error: None,
        }
    }

    fn tool_check_action_permitted(args: &serde_json::Value) -> CopilotToolResult {
        let raw_path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return CopilotToolResult {
                    tool: CopilotToolName::CheckActionPermitted,
                    success: false,
                    summary: "Missing 'path' argument".to_string(),
                    data: json!({}),
                    error: Some("Parameter 'path' is required".to_string()),
                }
            }
        };

        let norm_path = match Self::sanitize_path(raw_path) {
            Ok(p) => p,
            Err(e) => {
                return CopilotToolResult {
                    tool: CopilotToolName::CheckActionPermitted,
                    success: false,
                    summary: "Invalid path specification".to_string(),
                    data: json!({}),
                    error: Some(e.to_string()),
                }
            }
        };

        let operation = args.get("operation").and_then(|v| v.as_str()).unwrap_or("delete");
        let path_obj = Path::new(&norm_path);
        let assessment = SafetyGuardian::evaluate_target(path_obj, operation);

        let is_blocked = !assessment.is_permitted;
        let requires_confirmation = assessment.classification == SafetyClassification::ReviewRequired;

        let summary = if is_blocked {
            format!(
                "Operation '{}' on '{}' is STRICTLY BLOCKED by safety rule: {}",
                operation,
                norm_path,
                assessment.block_reason.as_deref().unwrap_or("Protected path")
            )
        } else if requires_confirmation {
            format!(
                "Operation '{}' on '{}' requires explicit user review and confirmation (Review Required).",
                operation, norm_path
            )
        } else {
            format!("Operation '{}' on '{}' is permitted as low-risk.", operation, norm_path)
        };

        CopilotToolResult {
            tool: CopilotToolName::CheckActionPermitted,
            success: true,
            summary,
            data: json!({
                "path": norm_path,
                "operation": operation,
                "isPermitted": assessment.is_permitted,
                "requiresExplicitConfirmation": requires_confirmation,
                "classification": assessment.classification,
                "blockReason": assessment.block_reason,
            }),
            error: None,
        }
    }
}
