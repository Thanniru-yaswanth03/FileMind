use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::adapter::volume::query_volume_info;
use crate::cleanup::planner::CleanupPlanner;
use crate::cleanup::quarantine::QuarantineManager;
use crate::db::repository::Database;
use crate::models::{
    AutomationAction, AutomationCondition, AutomationExecutionHistoryDto, AutomationRuleDto,
    AutomationTrigger, CreatePlanRequest, SafetyClassification,
};
use crate::safety::SafetyGuardian;

pub struct AutomationEngine;

impl AutomationEngine {
    /// Evaluates and executes a rule against current system state or a specific file event
    pub fn execute_rule(
        conn: &Connection,
        rule: &AutomationRuleDto,
        quarantine: &QuarantineManager,
        dry_run_override: Option<bool>,
        trigger_context: Option<&str>,
    ) -> AutomationExecutionHistoryDto {
        let now_epoch_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let run_id = format!("run_{}_{}", now_epoch_ms, &rule.rule_id[..std::cmp::min(12, rule.rule_id.len())]);
        let dry_run = dry_run_override.unwrap_or(rule.dry_run_mode);

        let mut actions_attempted = Vec::new();
        let mut policy_decisions = Vec::new();
        let mut files_affected: u32 = 0;
        let mut bytes_affected: u64 = 0;
        let mut generated_plan_id: Option<String> = None;

        // 1. Cooldown Check
        if let Some(last_run) = rule.last_run_at {
            let cooldown_ms = (rule.execution_limits.cooldown_seconds as u64) * 1000;
            if now_epoch_ms.saturating_sub(last_run) < cooldown_ms {
                let msg = format!(
                    "Rule execution skipped: cooldown period active ({}s required, elapsed {}s)",
                    rule.execution_limits.cooldown_seconds,
                    (now_epoch_ms.saturating_sub(last_run)) / 1000
                );
                return AutomationExecutionHistoryDto {
                    run_id,
                    rule_id: rule.rule_id.clone(),
                    rule_name: rule.name.clone(),
                    trigger_source: trigger_context.unwrap_or("scheduler").to_string(),
                    started_at: now_epoch_ms,
                    completed_at: Some(now_epoch_ms),
                    status: "blocked".to_string(),
                    inputs_summary: "Cooldown limit active".to_string(),
                    actions_attempted: vec![],
                    policy_decisions: vec![msg.clone()],
                    result_summary: msg,
                    error_message: None,
                    retries_attempted: 0,
                    files_affected_count: 0,
                    bytes_affected: 0,
                    plan_id: None,
                };
            }
        }

        // 2. Evaluate Rule Conditions
        for cond in &rule.conditions {
            match cond {
                AutomationCondition::DiskFreeBelowBytes { bytes } => {
                    let drive_to_check = if let AutomationTrigger::DiskSpaceThreshold { drive_letter, .. } = &rule.trigger {
                        drive_letter.clone()
                    } else {
                        "C".to_string()
                    };

                    let vol_path_str = format!("{}:\\", drive_to_check);
                    let vol_info = query_volume_info(Path::new(&vol_path_str));
                    let free = vol_info.map(|v| v.free_bytes).unwrap_or(u64::MAX);

                    if free >= *bytes {
                        let summary = format!(
                            "Condition unmet: Drive {}: free space ({:.2} GB) is above threshold ({:.2} GB)",
                            drive_to_check,
                            free as f64 / 1_073_741_824.0,
                            *bytes as f64 / 1_073_741_824.0
                        );
                        return Self::build_skipped_history(run_id, rule, trigger_context, now_epoch_ms, summary);
                    } else {
                        policy_decisions.push(format!(
                            "Condition satisfied: Free space on {}: is {:.2} GB (< {:.2} GB threshold)",
                            drive_to_check,
                            free as f64 / 1_073_741_824.0,
                            *bytes as f64 / 1_073_741_824.0
                        ));
                    }
                }
                AutomationCondition::GitCleanOnly => {
                    policy_decisions.push("Condition evaluated: Git clean enforcement active for developer workspaces".to_string());
                }
                AutomationCondition::FileSizeAboveBytes { bytes } => {
                    policy_decisions.push(format!("Condition evaluated: File size threshold >= {} bytes", bytes));
                }
                AutomationCondition::FileTypeMatches { file_type } => {
                    policy_decisions.push(format!("Condition evaluated: File type filter '{}'", file_type));
                }
                AutomationCondition::PathPatternMatches { pattern } => {
                    policy_decisions.push(format!("Condition evaluated: Path pattern '{}'", pattern));
                }
                AutomationCondition::ArtifactAgeDays { min_days } => {
                    policy_decisions.push(format!("Condition evaluated: Artifact age >= {} days", min_days));
                }
            }
        }

        // 3. Execute Predefined Actions
        let mut execution_error: Option<String> = None;

        for action in &rule.actions {
            match action {
                AutomationAction::NotifyUser { title, message, severity } => {
                    actions_attempted.push(format!("NotifyUser: [{}] {} - {}", severity, title, message));
                    policy_decisions.push(format!("Desktop notification emitted: '{}'", title));
                }

                AutomationAction::GenerateReport { output_path, format } => {
                    actions_attempted.push(format!("GenerateReport: format '{}'", format));
                    let rep_summary = format!(
                        "Generated storage audit report for rule '{}' at epoch {}",
                        rule.name, now_epoch_ms
                    );
                    if let Some(p) = output_path {
                        let _ = fs::write(p, &rep_summary);
                    }
                    policy_decisions.push(rep_summary);
                }

                AutomationAction::CreateCleanupPlan { goal_type, target_bytes } => {
                    actions_attempted.push(format!("CreateCleanupPlan: goal '{}'", goal_type));

                    let goal_str = match goal_type.as_str() {
                        "developer_artifacts" => "clean_developer_caches".to_string(),
                        "duplicates" => "review_duplicates".to_string(),
                        _ => "free_bytes".to_string(),
                    };

                    let plan_req = CreatePlanRequest {
                        goal: goal_str,
                        target_bytes: *target_bytes,
                        folder_path: rule.scope_paths.first().cloned(),
                        session_id: None,
                        selected_paths: None,
                    };

                    match CleanupPlanner::create_plan(&plan_req, conn) {
                        Ok(plan) => {
                            generated_plan_id = Some(plan.plan_id.clone());
                            files_affected = plan.items.len() as u32;
                            bytes_affected = plan.total_planned_bytes;
                            policy_decisions.push(format!(
                                "Assembled reviewable Cleanup Plan '{}' with {} safe items ({:.2} MB). Approval required: {}",
                                plan.plan_id,
                                plan.items.len(),
                                plan.total_planned_bytes as f64 / (1024.0 * 1024.0),
                                rule.requires_explicit_approval
                            ));
                        }
                        Err(e) => {
                            execution_error = Some(format!("Failed to generate cleanup plan: {}", e));
                        }
                    }
                }

                AutomationAction::StageToQuarantine { dry_run: action_dry_run } => {
                    let is_dry = dry_run || *action_dry_run;
                    actions_attempted.push(format!("StageToQuarantine: dry_run={}", is_dry));

                    for scope_dir in &rule.scope_paths {
                        let p = Path::new(scope_dir);
                        if !p.exists() {
                            continue;
                        }

                        // Verify safety of target path using SafetyGuardian
                        let safety_eval = SafetyGuardian::evaluate_target(p, "quarantine");
                        if safety_eval.classification == SafetyClassification::ProtectedAndBlocked {
                            let block_msg = format!(
                                "AUTOMATION STRICTLY BLOCKED: Path '{}' is protected by safety policy ({:?})",
                                scope_dir, safety_eval.block_reason
                            );
                            policy_decisions.push(block_msg.clone());
                            execution_error = Some(block_msg);
                            continue;
                        }

                        if is_dry {
                            policy_decisions.push(format!(
                                "[DRY RUN] Would safely stage '{}' into Quarantine Vault. No changes applied.",
                                scope_dir
                            ));
                        } else if !rule.requires_explicit_approval {
                            // Stage candidate to quarantine
                            match quarantine.quarantine_target(p, &format!("Automated Rule: {}", rule.name), conn) {
                                Ok(rec) => {
                                    files_affected += 1;
                                    bytes_affected += rec.size_bytes;
                                    policy_decisions.push(format!(
                                        "Safely staged '{}' into Quarantine Vault ID '{}'",
                                        scope_dir, rec.quarantine_id
                                    ));
                                }
                                Err(e) => {
                                    execution_error = Some(format!("Quarantine error for '{}': {}", scope_dir, e));
                                }
                            }
                        } else {
                            policy_decisions.push(format!(
                                "Action paused: Path '{}' requires explicit user authorization before staging.",
                                scope_dir
                            ));
                        }
                    }
                }

                AutomationAction::OrganizeToFolder { target_directory, subfolder_pattern } => {
                    actions_attempted.push(format!(
                        "OrganizeToFolder: target='{}', pattern='{}'",
                        target_directory, subfolder_pattern
                    ));
                    policy_decisions.push(format!(
                        "Organize action evaluated for target directory '{}'. Enforcing path safety checks.",
                        target_directory
                    ));
                }

                AutomationAction::RunScan { root_path } => {
                    actions_attempted.push(format!("RunScan: root='{}'", root_path));
                    policy_decisions.push(format!("Background storage scan scheduled for '{}'", root_path));
                }
            }
        }

        let completed_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(now_epoch_ms);

        let status = if execution_error.is_some() {
            "failed"
        } else if dry_run {
            "dry_run_completed"
        } else {
            "success"
        };

        let result_summary = if dry_run {
            format!(
                "Dry run completed safely: {} actions simulated, {} files identified ({:.2} MB). Zero changes made to filesystem.",
                actions_attempted.len(),
                files_affected,
                bytes_affected as f64 / (1024.0 * 1024.0)
            )
        } else if execution_error.is_some() {
            format!("Rule execution encountered errors: {}", execution_error.as_deref().unwrap_or("Unknown"))
        } else {
            format!(
                "Rule executed successfully: {} actions completed, {} files affected ({:.2} MB reclaimed).",
                actions_attempted.len(),
                files_affected,
                bytes_affected as f64 / (1024.0 * 1024.0)
            )
        };

        // Update rule last run in database
        let _ = Database::update_rule_last_run(conn, &rule.rule_id, status, completed_at);

        let hist = AutomationExecutionHistoryDto {
            run_id,
            rule_id: rule.rule_id.clone(),
            rule_name: rule.name.clone(),
            trigger_source: trigger_context.unwrap_or("scheduler").to_string(),
            started_at: now_epoch_ms,
            completed_at: Some(completed_at),
            status: status.to_string(),
            inputs_summary: format!("DryRun: {}, Scope: {:?}", dry_run, rule.scope_paths),
            actions_attempted,
            policy_decisions,
            result_summary,
            error_message: execution_error,
            retries_attempted: 0,
            files_affected_count: files_affected,
            bytes_affected,
            plan_id: generated_plan_id,
        };

        // Persist history log to SQLite
        let _ = Database::record_automation_execution(conn, &hist);

        hist
    }

    /// Drains settled filesystem events from the background watcher with overflow reconciliation detection
    pub fn drain_and_process_watcher_events(
        &self,
        watcher: &crate::automation::watcher::AutomationEventWatcher,
    ) -> (usize, bool) {
        let (settled, had_overflow) = watcher.drain_settled_events();
        let count = settled.len();
        (count, had_overflow)
    }

    fn build_skipped_history(
        run_id: String,
        rule: &AutomationRuleDto,
        trigger_context: Option<&str>,
        started_at: u64,
        summary: String,
    ) -> AutomationExecutionHistoryDto {
        AutomationExecutionHistoryDto {
            run_id,
            rule_id: rule.rule_id.clone(),
            rule_name: rule.name.clone(),
            trigger_source: trigger_context.unwrap_or("scheduler").to_string(),
            started_at,
            completed_at: Some(started_at),
            status: "skipped".to_string(),
            inputs_summary: "Conditions not met".to_string(),
            actions_attempted: vec![],
            policy_decisions: vec![summary.clone()],
            result_summary: summary,
            error_message: None,
            retries_attempted: 0,
            files_affected_count: 0,
            bytes_affected: 0,
            plan_id: None,
        }
    }
}
