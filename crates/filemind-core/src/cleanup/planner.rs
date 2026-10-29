use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::SystemTime;

use crate::adapter::metadata::{inspect_file, system_time_to_epoch_ms};
use crate::adapter::path::{is_descendant_of, normalize_path, strip_verbatim_prefix};
use crate::duplicate::pipeline::DuplicatePipeline;
use crate::models::{
    CandidateSource, CleanupPlanDto, CleanupPlanItemDto, CreatePlanRequest, PlanItemOperation,
    PlanItemState, PlanStatus, RiskTier, SafetyClassification,
};
use crate::safety::project::evaluate_project_dependencies;
use crate::safety::SafetyGuardian;

pub struct CleanupPlanner;

impl CleanupPlanner {
    /// Generates a comprehensive, defensible cleanup plan based on verified data sources,
    /// strict non-double-counting, hierarchy pruning, and safety classifications.
    pub fn create_plan(
        req: &CreatePlanRequest,
        conn: &Connection,
    ) -> Result<CleanupPlanDto, String> {
        let now_epoch_ms = system_time_to_epoch_ms(SystemTime::now());
        let plan_id = format!("plan_{}", now_epoch_ms);

        let mut raw_candidates: Vec<(String, CandidateSource, PlanItemOperation)> = Vec::new();

        // 1. Gather candidates according to requested goal
        match req.goal.as_str() {
            "clean_developer_caches" => {
                Self::gather_developer_artifacts(
                    conn,
                    req.session_id.as_deref(),
                    &mut raw_candidates,
                );
            }
            "review_duplicates" => {
                Self::gather_duplicate_candidates(
                    conn,
                    req.session_id.as_deref(),
                    &mut raw_candidates,
                );
            }
            "clean_folder" => {
                if let Some(ref folder) = req.folder_path {
                    Self::gather_folder_candidates(Path::new(folder), &mut raw_candidates);
                }
            }
            "user_custom" => {
                if let Some(ref paths) = req.selected_paths {
                    for p in paths {
                        raw_candidates.push((
                            p.clone(),
                            CandidateSource::UserSelected,
                            PlanItemOperation::Quarantine,
                        ));
                    }
                }
            }
            _ => {
                // "free_bytes", "maximize_safe", or general plan: combine developer artifacts, verified duplicates, and large temp files
                Self::gather_developer_artifacts(
                    conn,
                    req.session_id.as_deref(),
                    &mut raw_candidates,
                );
                Self::gather_duplicate_candidates(
                    conn,
                    req.session_id.as_deref(),
                    &mut raw_candidates,
                );
                Self::gather_large_files(conn, req.session_id.as_deref(), &mut raw_candidates);
            }
        }

        // If explicitly requested selected_paths are provided, ensure they are incorporated
        if let Some(ref paths) = req.selected_paths {
            for p in paths {
                let p_obj = Path::new(p);
                let (source, op) = if p_obj.is_dir() {
                    (
                        CandidateSource::DeveloperArtifact,
                        PlanItemOperation::Quarantine,
                    )
                } else {
                    (CandidateSource::UserSelected, PlanItemOperation::Quarantine)
                };
                raw_candidates.push((p.clone(), source, op));
            }
        }

        // 2. Overlap Pruning & Deduplication: Prevent counting same bytes multiple times!
        // Sort candidate paths by length ascending so parent directories come before their child files
        raw_candidates.sort_by_key(|(p, _, _)| p.len());

        let mut unique_candidates = Vec::new();
        let mut included_parent_dirs: Vec<String> = Vec::new();
        let mut seen_canonical_paths: HashSet<String> = HashSet::new();

        for (path_str, source, op) in raw_candidates {
            let clean = strip_verbatim_prefix(&path_str);
            let norm = normalize_path(Path::new(&clean));

            // Check if already included
            if seen_canonical_paths.contains(&norm) {
                continue;
            }

            // Check if path is a child of an already-included candidate directory
            let is_child_of_included = included_parent_dirs
                .iter()
                .any(|parent| is_descendant_of(Path::new(&norm), Path::new(parent)));

            if is_child_of_included {
                continue; // Skip child to prevent double counting and double deletion!
            }

            let path_obj = Path::new(&norm);
            if path_obj.is_dir() {
                included_parent_dirs.push(norm.clone());
            }

            seen_canonical_paths.insert(norm.clone());
            unique_candidates.push((clean, norm, source, op));
        }

        // 3. Safety Guardian Assessment & Ranking
        let mut evaluated_items: Vec<CleanupPlanItemDto> = Vec::new();
        let mut risk_counts: HashMap<String, usize> = HashMap::new();
        risk_counts.insert("safe".to_string(), 0);
        risk_counts.insert("caution".to_string(), 0);
        risk_counts.insert("high".to_string(), 0);
        risk_counts.insert("danger".to_string(), 0);

        let mut item_counter = 1usize;
        for (clean_path, norm_path, source, op) in unique_candidates {
            let path_obj = Path::new(&norm_path);
            if !path_obj.exists() {
                continue;
            }

            let assessment = SafetyGuardian::evaluate_target(path_obj, "evaluate");

            // Filter out targets that are strictly blocked or high risk
            if assessment.classification == SafetyClassification::ProtectedAndBlocked
                || assessment.classification == SafetyClassification::HighRiskOrUnknown
            {
                continue;
            }

            let is_dir = path_obj.is_dir();
            let (logical_size, physical_size) = if is_dir {
                // Directory physical size estimation
                let (log_s, phys_s) = compute_dir_size(path_obj);
                (log_s, phys_s)
            } else {
                let meta = inspect_file(path_obj, 4096).ok();
                (
                    meta.as_ref().map(|m| m.logical_size_bytes).unwrap_or(0),
                    meta.as_ref().map(|m| m.physical_size_bytes).unwrap_or(0),
                )
            };

            let risk_tier = match assessment.classification {
                SafetyClassification::LowRiskCandidate => RiskTier::Safe,
                SafetyClassification::ReviewRequired => RiskTier::Caution,
                _ => RiskTier::High,
            };

            let tier_str = match risk_tier {
                RiskTier::Safe => "safe",
                RiskTier::Caution => "caution",
                RiskTier::High => "high",
                RiskTier::Danger => "danger",
            };
            *risk_counts.entry(tier_str.to_string()).or_insert(0) += 1;

            let is_selected_by_default =
                assessment.classification == SafetyClassification::LowRiskCandidate;
            let item_id = format!("{}_{}", plan_id, item_counter);
            item_counter += 1;

            let evidence_summary = assessment
                .evidence
                .first()
                .map(|e| e.claim.clone())
                .unwrap_or_else(|| "Inspected via native filesystem engine".to_string());

            let potential_impact = if assessment.classification
                == SafetyClassification::LowRiskCandidate
            {
                "Zero data loss. Rebuildable on demand from project lockfile.".to_string()
            } else {
                "Requires manual user review. File is restorable from Quarantine Vault.".to_string()
            };

            evaluated_items.push(CleanupPlanItemDto {
                item_id,
                plan_id: plan_id.clone(),
                path: clean_path,
                canonical_path: norm_path,
                logical_size_bytes: logical_size,
                physical_size_bytes: physical_size,
                source,
                risk_tier,
                classification: assessment.classification,
                operation: op,
                is_reversible: true,
                state: PlanItemState::Pending,
                is_selected: is_selected_by_default,
                is_excluded: false,
                quarantine_id: None,
                error_message: None,
                evidence_summary,
                potential_impact,
            });
        }

        // 4. Rank Candidates: Safe first, then by physical recoverable size descending
        evaluated_items.sort_by(|a, b| {
            let a_is_safe = a.risk_tier == RiskTier::Safe;
            let b_is_safe = b.risk_tier == RiskTier::Safe;
            if a_is_safe != b_is_safe {
                return b_is_safe.cmp(&a_is_safe);
            }
            b.physical_size_bytes.cmp(&a.physical_size_bytes)
        });

        // 5. Target Budget Accounting & Attainability
        let mut total_planned_bytes = 0u64;
        let mut total_physical_reclaimable_bytes = 0u64;

        for item in &evaluated_items {
            total_physical_reclaimable_bytes += item.physical_size_bytes;
            if item.is_selected {
                total_planned_bytes += item.physical_size_bytes;
            }
        }

        let target_bytes = req.target_bytes;
        let mut is_target_attainable_safely = true;
        let mut unmet_target_explanation = None;

        if let Some(target) = target_bytes {
            if total_physical_reclaimable_bytes < target {
                is_target_attainable_safely = false;
                unmet_target_explanation = Some(format!(
                    "Available safe candidates total {:.2} MB ({:.2} GB). The target goal of {:.2} GB cannot be reached safely without endangering user files or operating system integrity.",
                    total_physical_reclaimable_bytes as f64 / (1024.0 * 1024.0),
                    total_physical_reclaimable_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                    target as f64 / (1024.0 * 1024.0 * 1024.0)
                ));
            }
        }

        let plan_dto = CleanupPlanDto {
            plan_id: plan_id.clone(),
            created_at_epoch_ms: now_epoch_ms,
            goal_type: req.goal.clone(),
            target_bytes,
            status: PlanStatus::Draft,
            total_planned_bytes,
            total_physical_reclaimable_bytes,
            total_reclaimed_bytes: 0,
            items_count: evaluated_items.len(),
            completed_count: 0,
            failed_count: 0,
            is_target_attainable_safely,
            unmet_target_explanation: unmet_target_explanation.clone(),
            risk_counts,
            items: evaluated_items,
        };

        // 6. Persist plan and items in SQLite
        conn.execute(
            r#"
            INSERT INTO cleanup_plans (
                plan_id, created_at, goal_type, target_bytes, status,
                total_planned_bytes, total_reclaimed_bytes, total_items_count,
                completed_items_count, failed_items_count, explanation
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                plan_dto.plan_id,
                plan_dto.created_at_epoch_ms as i64,
                plan_dto.goal_type,
                plan_dto.target_bytes.map(|t| t as i64),
                "draft",
                plan_dto.total_planned_bytes as i64,
                0i64,
                plan_dto.items_count as i64,
                0i64,
                0i64,
                unmet_target_explanation,
            ],
        )
        .map_err(|e| format!("Failed to persist cleanup plan: {}", e))?;

        for item in &plan_dto.items {
            let class_str = format!("{:?}", item.classification);
            let risk_str = format!("{:?}", item.risk_tier);
            let op_str = match item.operation {
                PlanItemOperation::Quarantine => "quarantine",
                PlanItemOperation::RecycleBin => "recycle",
                PlanItemOperation::PermanentDelete => "permanent",
            };

            conn.execute(
                r#"
                INSERT INTO cleanup_plan_items (
                    item_id, plan_id, path, canonical_path,
                    logical_size_bytes, physical_size_bytes, source,
                    risk_tier, classification, operation, is_reversible,
                    state, is_selected, is_excluded, quarantine_id, error_message, evidence_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                "#,
                params![
                    item.item_id,
                    item.plan_id,
                    item.path,
                    item.canonical_path,
                    item.logical_size_bytes as i64,
                    item.physical_size_bytes as i64,
                    format!("{:?}", item.source),
                    risk_str,
                    class_str,
                    op_str,
                    if item.is_reversible { 1 } else { 0 },
                    "pending",
                    if item.is_selected { 1 } else { 0 },
                    if item.is_excluded { 1 } else { 0 },
                    item.quarantine_id,
                    item.error_message,
                    item.evidence_summary,
                ],
            )
            .map_err(|e| format!("Failed to persist cleanup item: {}", e))?;
        }

        Ok(plan_dto)
    }

    /// Helper to gather developer build artifact directories from scan index and disk
    fn gather_developer_artifacts(
        conn: &Connection,
        session_id: Option<&str>,
        candidates: &mut Vec<(String, CandidateSource, PlanItemOperation)>,
    ) {
        // Query indexed directories named target, node_modules, build, dist, __pycache__, .gradle, .next
        let sid_clause = if let Some(sid) = session_id {
            format!("WHERE session_id = '{}'", sid)
        } else {
            String::new()
        };

        let sql = format!(
            r#"
            SELECT path FROM directories
            {}
            AND (
                path LIKE '%\target' OR path LIKE '%/target' OR
                path LIKE '%\node_modules' OR path LIKE '%/node_modules' OR
                path LIKE '%\build' OR path LIKE '%/build' OR
                path LIKE '%\dist' OR path LIKE '%/dist' OR
                path LIKE '%\__pycache__' OR path LIKE '%/__pycache__' OR
                path LIKE '%\.gradle' OR path LIKE '%/.gradle' OR
                path LIKE '%\.next' OR path LIKE '%/.next'
            )
            LIMIT 50
            "#,
            if sid_clause.is_empty() {
                "WHERE 1=1"
            } else {
                &sid_clause
            }
        );

        if let Ok(mut stmt) = conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for r in rows.flatten() {
                    let p = Path::new(&r);
                    let ev = evaluate_project_dependencies(p);
                    if ev.classification == SafetyClassification::LowRiskCandidate {
                        candidates.push((
                            r,
                            CandidateSource::DeveloperArtifact,
                            PlanItemOperation::Quarantine,
                        ));
                    }
                }
            }
        }
    }

    /// Helper to gather duplicate redundant copies from duplicate pipeline
    fn gather_duplicate_candidates(
        conn: &Connection,
        session_id: Option<&str>,
        candidates: &mut Vec<(String, CandidateSource, PlanItemOperation)>,
    ) {
        let req = crate::models::DuplicateScanRequest {
            session_id: session_id.map(String::from),
            min_size_bytes: None,
            max_groups: Some(30),
            verify_byte_by_byte: Some(false),
        };

        if let Ok(dup_resp) = DuplicatePipeline::execute(conn, &req) {
            for group in dup_resp.groups {
                for file in group.files {
                    // Only target redundant copies (never the recommended original!)
                    if !file.is_recommended_keep && !file.is_protected {
                        candidates.push((
                            file.path,
                            CandidateSource::DuplicateGroup,
                            PlanItemOperation::RecycleBin,
                        ));
                    }
                }
            }
        }
    }

    /// Helper to gather large files
    fn gather_large_files(
        conn: &Connection,
        session_id: Option<&str>,
        candidates: &mut Vec<(String, CandidateSource, PlanItemOperation)>,
    ) {
        let sid_clause = if let Some(sid) = session_id {
            format!("WHERE session_id = '{}' AND", sid)
        } else {
            "WHERE".to_string()
        };

        let sql = format!(
            r#"
            SELECT directory_path, filename FROM files
            {} logical_size_bytes > 52428800 AND risk_tier IN ('safe', 'caution')
            ORDER BY logical_size_bytes DESC
            LIMIT 25
            "#,
            sid_clause
        );

        if let Ok(mut stmt) = conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map([], |row| {
                let dir: String = row.get(0)?;
                let file: String = row.get(1)?;
                Ok(format!("{}\\{}", dir.trim_end_matches('\\'), file))
            }) {
                for p in rows.flatten() {
                    candidates.push((p, CandidateSource::LargeFile, PlanItemOperation::RecycleBin));
                }
            }
        }
    }

    /// Helper to gather folder candidates
    fn gather_folder_candidates(
        folder: &Path,
        candidates: &mut Vec<(String, CandidateSource, PlanItemOperation)>,
    ) {
        if let Ok(entries) = std::fs::read_dir(folder) {
            for entry in entries.flatten() {
                let p = entry.path();
                candidates.push((
                    p.to_string_lossy().to_string(),
                    CandidateSource::UserSelected,
                    PlanItemOperation::RecycleBin,
                ));
            }
        }
    }
}

/// Recursively computes directory size
fn compute_dir_size(dir: &Path) -> (u64, u64) {
    let mut logical = 0u64;
    let mut physical = 0u64;

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let (l, ph) = compute_dir_size(&p);
                logical += l;
                physical += ph;
            } else if let Ok(meta) = inspect_file(&p, 4096) {
                logical += meta.logical_size_bytes;
                physical += meta.physical_size_bytes;
            }
        }
    }

    (logical, physical)
}
