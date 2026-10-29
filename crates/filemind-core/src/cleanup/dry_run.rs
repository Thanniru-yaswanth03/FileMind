use rusqlite::{params, Connection};
use std::path::Path;

use crate::adapter::metadata::inspect_file;
use crate::adapter::path::to_long_path;
use crate::models::{
    DryRunItemPrediction, DryRunResponse, PlanItemOperation, SafetyClassification,
};
use crate::safety::SafetyGuardian;

pub struct DryRunEngine;

impl DryRunEngine {
    /// Executes a genuine, non-destructive dry run for a proposed cleanup plan.
    /// Validates permissions, checks protected paths, detects active locks and conflicts,
    /// and projects expected space recovery without moving, renaming, or deleting any file.
    pub fn execute_dry_run(plan_id: &str, conn: &Connection) -> Result<DryRunResponse, String> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT item_id, path, canonical_path, logical_size_bytes, physical_size_bytes,
                       operation, is_selected, is_reversible
                FROM cleanup_plan_items
                WHERE plan_id = ?1 AND is_excluded = 0
                "#,
            )
            .map_err(|e| format!("Database error preparing items query: {}", e))?;

        struct ItemRow {
            item_id: String,
            path: String,
            canonical_path: String,
            _logical_size: i64,
            _physical_size: i64,
            operation_str: String,
            is_reversible: i32,
        }

        let rows = stmt
            .query_map(params![plan_id], |row| {
                let item_id: String = row.get(0)?;
                let path: String = row.get(1)?;
                let canonical_path: String = row.get(2)?;
                let _logical_size: i64 = row.get(3)?;
                let _physical_size: i64 = row.get(4)?;
                let operation_str: String = row.get(5)?;
                let is_reversible: i32 = row.get(7)?;

                Ok(ItemRow {
                    item_id,
                    path,
                    canonical_path,
                    _logical_size,
                    _physical_size,
                    operation_str,
                    is_reversible,
                })
            })
            .map_err(|e| format!("Database error executing query: {}", e))?;

        let mut predictions = Vec::new();
        let mut conflicts_detected = Vec::new();
        let mut total_expected_freed = 0u64;
        let mut can_execute = true;

        for r in rows {
            let item = r.map_err(|e| e.to_string())?;
            let path_obj = Path::new(&item.canonical_path);
            let op = match item.operation_str.as_str() {
                "quarantine" => PlanItemOperation::Quarantine,
                "recycle" => PlanItemOperation::RecycleBin,
                _ => PlanItemOperation::PermanentDelete,
            };

            let reversibility = if item.is_reversible == 1 {
                if op == PlanItemOperation::Quarantine {
                    "Reversible via Quarantine Vault".to_string()
                } else {
                    "Reversible via Windows Recycle Bin".to_string()
                }
            } else {
                "Irreversible Permanent Destruction".to_string()
            };

            // 1. Existence check
            if !path_obj.exists() {
                can_execute = false;
                conflicts_detected
                    .push(format!("File disappeared or inaccessible: '{}'", item.path));
                predictions.push(DryRunItemPrediction {
                    item_id: item.item_id,
                    path: item.path,
                    operation: op,
                    can_proceed: false,
                    reversibility,
                    expected_freed_bytes: 0,
                    warning: Some(
                        "Target disappeared from disk between planning and dry-run".to_string(),
                    ),
                });
                continue;
            }

            // 2. Permission and Handle validation
            let long_p = to_long_path(path_obj);
            let meta_res = inspect_file(&long_p, 4096);
            if let Err(ref e) = meta_res {
                can_execute = false;
                conflicts_detected.push(format!(
                    "Permission denied opening handle: '{}' ({})",
                    item.path, e
                ));
                predictions.push(DryRunItemPrediction {
                    item_id: item.item_id,
                    path: item.path,
                    operation: op,
                    can_proceed: false,
                    reversibility,
                    expected_freed_bytes: 0,
                    warning: Some(format!("Failed to open handle: {}", e)),
                });
                continue;
            }

            let meta = match meta_res {
                Ok(m) => m,
                Err(_) => continue,
            };

            // 3. Process Lock check
            if meta.is_locked {
                can_execute = false;
                conflicts_detected.push(format!(
                    "Process lock active: '{}' is held open by another process",
                    item.path
                ));
                predictions.push(DryRunItemPrediction {
                    item_id: item.item_id,
                    path: item.path,
                    operation: op,
                    can_proceed: false,
                    reversibility,
                    expected_freed_bytes: 0,
                    warning: Some(
                        "Target is actively locked with exclusive sharing flags".to_string(),
                    ),
                });
                continue;
            }

            // 4. Safety Guardian Re-evaluation
            let assessment = SafetyGuardian::evaluate_target(path_obj, "dry_run");
            if assessment.classification == SafetyClassification::ProtectedAndBlocked {
                can_execute = false;
                conflicts_detected.push(format!(
                    "Security policy violation: '{}' matches protected policy ({})",
                    item.path,
                    assessment.block_reason.as_deref().unwrap_or("Protected")
                ));
                predictions.push(DryRunItemPrediction {
                    item_id: item.item_id,
                    path: item.path,
                    operation: op,
                    can_proceed: false,
                    reversibility,
                    expected_freed_bytes: 0,
                    warning: Some(format!(
                        "Security block triggered: {}",
                        assessment.block_reason.unwrap_or_default()
                    )),
                });
                continue;
            }

            // Validated item
            total_expected_freed += meta.physical_size_bytes;
            predictions.push(DryRunItemPrediction {
                item_id: item.item_id,
                path: item.path,
                operation: op,
                can_proceed: true,
                reversibility,
                expected_freed_bytes: meta.physical_size_bytes,
                warning: None,
            });
        }

        // Update plan state in SQLite
        let new_status = if can_execute {
            "dry_run_validated"
        } else {
            "draft"
        };
        let _ = conn.execute(
            "UPDATE cleanup_plans SET status = ?1 WHERE plan_id = ?2",
            params![new_status, plan_id],
        );

        Ok(DryRunResponse {
            plan_id: plan_id.to_string(),
            can_execute,
            validated_items_count: predictions.iter().filter(|p| p.can_proceed).count(),
            total_expected_freed_bytes: total_expected_freed,
            conflicts_detected,
            predictions,
        })
    }
}
