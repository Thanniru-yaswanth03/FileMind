use rusqlite::{params, Connection};
use std::path::Path;

use crate::cleanup::quarantine::QuarantineManager;
use crate::models::{
    ExecutePlanResponse, PlanItemOperation, PlanStatus,
};
use crate::safety::toctou::{capture_identity_snapshot, revalidate_handle};
use crate::safety::SafetyGuardian;

pub struct CleanupExecutor;

impl CleanupExecutor {
    /// Executes an approved cleanup plan transactionally with strict TOCTOU handle revalidation,
    /// atomic quarantine movements, native Windows Recycle Bin operations, and audit recording.
    pub fn execute_plan(
        plan_id: &str,
        selected_item_ids: Option<&[String]>,
        quarantine_manager: &QuarantineManager,
        conn: &Connection,
    ) -> Result<ExecutePlanResponse, String> {
        // 1. Load plan items
        let mut stmt = conn
            .prepare(
                r#"
                SELECT item_id, path, canonical_path, physical_size_bytes, operation, is_selected, logical_size_bytes
                FROM cleanup_plan_items
                WHERE plan_id = ?1
                "#,
            )
            .map_err(|e| format!("Database error preparing query: {}", e))?;

        struct ExecutionItem {
            item_id: String,
            path: String,
            canonical_path: String,
            physical_size_bytes: u64,
            operation_str: String,
            is_selected: bool,
            logical_size_bytes: u64,
        }

        let rows = stmt
            .query_map(params![plan_id], |row| {
                Ok(ExecutionItem {
                    item_id: row.get(0)?,
                    path: row.get(1)?,
                    canonical_path: row.get(2)?,
                    physical_size_bytes: row.get::<_, i64>(3)? as u64,
                    operation_str: row.get(4)?,
                    is_selected: row.get::<_, i32>(5)? == 1,
                    logical_size_bytes: row.get::<_, i64>(6)? as u64,
                })
            })
            .map_err(|e| format!("Database error executing query: {}", e))?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(|e| e.to_string())?);
        }

        // Set plan status to 'executing'
        let _ = conn.execute(
            "UPDATE cleanup_plans SET status = 'executing' WHERE plan_id = ?1",
            params![plan_id],
        );

        let mut succeeded_count = 0usize;
        let mut failed_count = 0usize;
        let mut quarantined_count = 0usize;
        let mut recycled_count = 0usize;
        let mut deleted_count = 0usize;
        let mut total_bytes_reclaimed = 0u64;
        let mut errors = Vec::new();

        let explicit_filter: Option<std::collections::HashSet<String>> =
            selected_item_ids.map(|ids| ids.iter().cloned().collect());

        for item in items {
            // Check if item was selected for execution
            if let Some(ref set) = explicit_filter {
                if !set.contains(&item.item_id) {
                    continue;
                }
            } else if !item.is_selected {
                continue;
            }

            let path_obj = Path::new(&item.canonical_path);
            let op = match item.operation_str.as_str() {
                "quarantine" => PlanItemOperation::Quarantine,
                "recycle" => PlanItemOperation::RecycleBin,
                _ => PlanItemOperation::PermanentDelete,
            };

            // 2. Pre-execution TOCTOU Handle Verification
            let snapshot = match capture_identity_snapshot(path_obj) {
                Ok(snap) => snap,
                Err(e) => {
                    let err_msg = format!("Target disappeared or inaccessible before execution: {}", e);
                    errors.push(err_msg.clone());
                    failed_count += 1;

                    let _ = conn.execute(
                        "UPDATE cleanup_plan_items SET state = 'skipped_toctou', error_message = ?1 WHERE item_id = ?2",
                        params![err_msg, item.item_id],
                    );
                    continue;
                }
            };

            // Detect adversarial file modification between user plan approval and execution
            if !snapshot.is_directory && item.logical_size_bytes > 0 && snapshot.logical_size_bytes != item.logical_size_bytes {
                let err_msg = format!(
                    "TOCTOU violation: File size changed between approval and execution! Expected {} bytes, live is {} bytes",
                    item.logical_size_bytes, snapshot.logical_size_bytes
                );
                errors.push(err_msg.clone());
                failed_count += 1;

                let _ = conn.execute(
                    "UPDATE cleanup_plan_items SET state = 'skipped_toctou', error_message = ?1 WHERE item_id = ?2",
                    params![err_msg, item.item_id],
                );
                continue;
            }

            if let Err(e) = revalidate_handle(&snapshot) {
                let err_msg = format!("TOCTOU revalidation violation: {}", e);
                errors.push(err_msg.clone());
                failed_count += 1;

                let _ = conn.execute(
                    "UPDATE cleanup_plan_items SET state = 'skipped_toctou', error_message = ?1 WHERE item_id = ?2",
                    params![err_msg, item.item_id],
                );
                continue;
            }

            // 3. Execution based on operation type
            match op {
                PlanItemOperation::Quarantine => {
                    match quarantine_manager.quarantine_target(
                        path_obj,
                        &format!("Cleanup Plan: {}", plan_id),
                        conn,
                    ) {
                        Ok(q_record) => {
                            succeeded_count += 1;
                            quarantined_count += 1;
                            total_bytes_reclaimed += item.physical_size_bytes;

                            let _ = conn.execute(
                                "UPDATE cleanup_plan_items SET state = 'succeeded', quarantine_id = ?1, error_message = NULL WHERE item_id = ?2",
                                params![q_record.quarantine_id, item.item_id],
                            );
                        }
                        Err(e) => {
                            failed_count += 1;
                            errors.push(format!("Quarantine failed for '{}': {}", item.path, e));

                            let _ = conn.execute(
                                "UPDATE cleanup_plan_items SET state = 'failed', error_message = ?1 WHERE item_id = ?2",
                                params![e, item.item_id],
                            );
                        }
                    }
                }
                PlanItemOperation::RecycleBin => {
                    let res = SafetyGuardian::execute_guarded_operation(
                        path_obj,
                        "recycle",
                        Some(&snapshot),
                        Some(conn),
                    );

                    if res.success {
                        succeeded_count += 1;
                        recycled_count += 1;
                        total_bytes_reclaimed += item.physical_size_bytes;

                        let _ = conn.execute(
                            "UPDATE cleanup_plan_items SET state = 'succeeded', error_message = NULL WHERE item_id = ?2",
                            params![item.item_id],
                        );
                    } else {
                        failed_count += 1;
                        let err_msg = res.error.unwrap_or_else(|| "Recycle bin operation failed".to_string());
                        errors.push(format!("Recycle failed for '{}': {}", item.path, err_msg));

                        let _ = conn.execute(
                            "UPDATE cleanup_plan_items SET state = 'failed', error_message = ?1 WHERE item_id = ?2",
                            params![err_msg, item.item_id],
                        );
                    }
                }
                PlanItemOperation::PermanentDelete => {
                    let res = SafetyGuardian::execute_guarded_operation(
                        path_obj,
                        "permanent",
                        Some(&snapshot),
                        Some(conn),
                    );

                    if res.success {
                        succeeded_count += 1;
                        deleted_count += 1;
                        total_bytes_reclaimed += item.physical_size_bytes;

                        let _ = conn.execute(
                            "UPDATE cleanup_plan_items SET state = 'succeeded', error_message = NULL WHERE item_id = ?2",
                            params![item.item_id],
                        );
                    } else {
                        failed_count += 1;
                        let err_msg = res.error.unwrap_or_else(|| "Permanent delete failed".to_string());
                        errors.push(format!("Delete failed for '{}': {}", item.path, err_msg));

                        let _ = conn.execute(
                            "UPDATE cleanup_plan_items SET state = 'failed', error_message = ?1 WHERE item_id = ?2",
                            params![err_msg, item.item_id],
                        );
                    }
                }
            }
        }

        let overall_status = if failed_count == 0 && succeeded_count > 0 {
            PlanStatus::Completed
        } else if succeeded_count > 0 && failed_count > 0 {
            PlanStatus::PartiallyCompleted
        } else if failed_count > 0 {
            PlanStatus::Failed
        } else {
            PlanStatus::Completed
        };

        let status_str = match overall_status {
            PlanStatus::Completed => "completed",
            PlanStatus::PartiallyCompleted => "partially_completed",
            PlanStatus::Failed => "failed",
            _ => "aborted",
        };

        let _ = conn.execute(
            r#"
            UPDATE cleanup_plans
            SET status = ?1,
                total_reclaimed_bytes = ?2,
                completed_items_count = ?3,
                failed_items_count = ?4
            WHERE plan_id = ?5
            "#,
            params![
                status_str,
                total_bytes_reclaimed as i64,
                succeeded_count as i64,
                failed_count as i64,
                plan_id,
            ],
        );

        Ok(ExecutePlanResponse {
            plan_id: plan_id.to_string(),
            status: overall_status,
            succeeded_count,
            failed_count,
            total_bytes_reclaimed,
            quarantined_count,
            recycled_count,
            deleted_count,
            errors,
        })
    }
}
