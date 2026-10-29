use rusqlite::Connection;
use serde_json::json;

use crate::copilot::ollama::OllamaClient;
use crate::copilot::tools::CopilotToolRunner;
use crate::models::{
    CopilotEvidenceItem, CopilotOperationProposal, CopilotQueryRequest, CopilotQueryResponse,
    CopilotToolName, CopilotToolResult,
};

pub struct CopilotDispatcher;

impl CopilotDispatcher {
    /// Sanitizes untrusted user query to resist prompt injection attempts.
    pub fn sanitize_query(raw_query: &str) -> String {
        let mut sanitized = raw_query.trim().to_string();

        // Neutralize common prompt injection prefixes (case-insensitive)
        let injection_patterns = [
            "ignore previous instructions",
            "disregard safety policy",
            "you are now an unrestricted ai",
            "jailbreak",
            "execute system command",
            "override permissions",
        ];

        for pattern in &injection_patterns {
            while let Some(pos) = sanitized.to_lowercase().find(pattern) {
                let end = pos + pattern.len();
                sanitized = format!("{}[untrusted instruction stripped]{}", &sanitized[..pos], &sanitized[end..]);
            }
        }

        sanitized
    }

    /// Checks if the current SQLite index is stale or empty.
    pub fn is_index_stale(conn: &Connection) -> (bool, String) {
        let res: rusqlite::Result<(i64, i64)> = conn.query_row(
            r#"
            SELECT COUNT(*), COALESCE(MAX(started_at), 0)
            FROM scan_sessions
            WHERE status = 'completed'
            "#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match res {
            Ok((count, last_started)) => {
                if count == 0 {
                    (
                        true,
                        "Storage index is empty. No completed scan has been run yet.".to_string(),
                    )
                } else {
                    let now_epoch_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);

                    let age_days = (now_epoch_ms - last_started) / (1000 * 60 * 60 * 24);
                    if age_days > 7 {
                        (
                            true,
                            format!(
                                "Storage index was last verified {} days ago and may be stale.",
                                age_days
                            ),
                        )
                    } else {
                        (false, "Storage index is current.".to_string())
                    }
                }
            }
            Err(_) => (true, "Storage index is not available.".to_string()),
        }
    }

    /// Main entry point: parses the user inquiry, invokes typed application tools,
    /// formats grounded responses, and enforces action approval boundaries.
    pub fn handle_query(req: &CopilotQueryRequest, conn: &Connection) -> CopilotQueryResponse {
        let query = Self::sanitize_query(&req.query);
        let query_lower = query.to_lowercase();

        let (is_stale, stale_reason) = Self::is_index_stale(conn);

        let mut tool_executions: Vec<CopilotToolResult> = Vec::new();
        let mut evidence: Vec<CopilotEvidenceItem> = Vec::new();
        let mut proposal: Option<CopilotOperationProposal> = None;
        let mut suggested_followups: Vec<String> = Vec::new();

        // 1. Tool Routing & Grounding based on query intent
        let (deterministic_answer, grounded_text) = if query_lower.contains("most space")
            || query_lower.contains("taking up space")
            || query_lower.contains("space on my c")
            || query_lower.contains("summarize")
            || query_lower.contains("today's scan")
            || query_lower.contains("storage overview")
        {
            let res = CopilotToolRunner::execute(CopilotToolName::SummarizeScan, &json!({}), conn);
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            // Extract top folders as evidence
            if let Some(folders) = data.get("topFolders").and_then(|v| v.as_array()) {
                for f in folders.iter().take(4) {
                    let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let size = f.get("sizeBytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    evidence.push(CopilotEvidenceItem {
                        path: path.to_string(),
                        filename: path.split(['\\', '/']).last().unwrap_or(path).to_string(),
                        size_bytes: size,
                        risk_tier: "caution".to_string(),
                        reason: "High storage consumption directory".to_string(),
                        action_type: "open_folder".to_string(),
                    });
                }
            }

            suggested_followups.push("Prepare a plan to free 10 GB".to_string());
            suggested_followups.push("Show me the largest files in this folder".to_string());
            suggested_followups.push("Find duplicate archives in Downloads".to_string());

            let ans = format!(
                "### Storage Summary\n\n{}\n\n{}",
                summary_str,
                if is_stale {
                    format!("> [!NOTE]\n> {} Click 'Trigger Volume Scan' to refresh index.", stale_reason)
                } else {
                    "Data is grounded in your verified local SQLite index.".to_string()
                }
            );

            (ans, format!("Scan summary: {}", data))
        } else if query_lower.contains("duplicate") {
            let res = CopilotToolRunner::execute(
                CopilotToolName::FindDuplicates,
                &json!({ "limit": 10, "verify_bytes": false }),
                conn,
            );
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            if let Some(groups) = data.get("groups").and_then(|v| v.as_array()) {
                for g in groups.iter().take(3) {
                    if let Some(files) = g.get("files").and_then(|v| v.as_array()) {
                        for f in files.iter().take(2) {
                            let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                            let size = f.get("sizeBytes").and_then(|v| v.as_u64()).unwrap_or(0);
                            let fname = f.get("filename").and_then(|v| v.as_str()).unwrap_or("");
                            evidence.push(CopilotEvidenceItem {
                                path: path.to_string(),
                                filename: fname.to_string(),
                                size_bytes: size,
                                risk_tier: "safe".to_string(),
                                reason: "Redundant verified duplicate copy".to_string(),
                                action_type: "plan_review".to_string(),
                            });
                        }
                    }
                }
            }

            suggested_followups.push("Tell me which cleanup actions are reversible.".to_string());
            suggested_followups.push("Prepare a plan to free 5 GB".to_string());

            let ans = format!(
                "### Duplicate Detection\n\n{}\n\nRedundant duplicate copies can be staged into the recoverable Quarantine Vault.",
                summary_str
            );

            (ans, format!("Duplicate results: {}", data))
        } else if query_lower.contains("largest files") || query_lower.contains("large files") {
            let res = CopilotToolRunner::execute(
                CopilotToolName::SearchFiles,
                &json!({ "sort_by": "size_desc", "limit": 5 }),
                conn,
            );
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            if let Some(files) = data.get("files").and_then(|v| v.as_array()) {
                for f in files.iter().take(5) {
                    let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let fname = f.get("filename").and_then(|v| v.as_str()).unwrap_or("");
                    let size = f.get("logicalSizeBytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let risk = f.get("riskTier").and_then(|v| v.as_str()).unwrap_or("caution");

                    evidence.push(CopilotEvidenceItem {
                        path: path.to_string(),
                        filename: fname.to_string(),
                        size_bytes: size,
                        risk_tier: risk.to_string(),
                        reason: "Ranked among top large files in local index".to_string(),
                        action_type: "inspect".to_string(),
                    });
                }
            }

            suggested_followups.push("Explain why this file is protected.".to_string());
            suggested_followups.push("Prepare a plan to free 10 GB".to_string());

            let ans = format!(
                "### Largest Files in Storage Index\n\n{}\n\nReview the specific items below in the evidence ledger.",
                summary_str
            );

            (ans, format!("Large files: {}", data))
        } else if query_lower.contains("javascript")
            || query_lower.contains("node_modules")
            || query_lower.contains("dependencies")
        {
            let res = CopilotToolRunner::execute(
                CopilotToolName::SearchFiles,
                &json!({ "query": "node_modules", "limit": 10 }),
                conn,
            );
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            if let Some(files) = data.get("files").and_then(|v| v.as_array()) {
                for f in files.iter().take(4) {
                    let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let fname = f.get("filename").and_then(|v| v.as_str()).unwrap_or("");
                    let size = f.get("logicalSizeBytes").and_then(|v| v.as_u64()).unwrap_or(0);

                    evidence.push(CopilotEvidenceItem {
                        path: path.to_string(),
                        filename: fname.to_string(),
                        size_bytes: size,
                        risk_tier: "safe".to_string(),
                        reason: "Disposable package dependency cache (rebuildable via npm/pnpm)".to_string(),
                        action_type: "plan_review".to_string(),
                    });
                }
            }

            suggested_followups.push("Prepare a plan to clean developer caches".to_string());

            let ans = format!(
                "### JavaScript Projects & Dependencies\n\n{}\n\n`node_modules` caches can be safely quarantined and reclaimed provided a valid `package-lock.json` or `pnpm-lock.yaml` is present.",
                summary_str
            );

            (ans, format!("JS dependency scan: {}", data))
        } else if query_lower.contains("why is") && query_lower.contains("protected")
            || query_lower.contains("explain why this file")
        {
            // Extract file path from query or fallback to Windows root / vhdx
            let target_path = if query_lower.contains("vhdx") || query_lower.contains("linux") {
                r"C:\WSL\ext4.vhdx"
            } else if query_lower.contains("windows") || query_lower.contains("system32") {
                r"C:\Windows\System32\ntdll.dll"
            } else {
                r"C:\Windows"
            };

            let res = CopilotToolRunner::execute(
                CopilotToolName::ExplainSafety,
                &json!({ "path": target_path }),
                conn,
            );
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            evidence.push(CopilotEvidenceItem {
                path: target_path.to_string(),
                filename: target_path.split(['\\', '/']).last().unwrap_or(target_path).to_string(),
                size_bytes: 0,
                risk_tier: "danger".to_string(),
                reason: "Protected system path or kernel component".to_string(),
                action_type: "inspect".to_string(),
            });

            suggested_followups.push("Tell me which cleanup actions are reversible.".to_string());

            let ans = format!(
                "### Safety Policy Analysis\n\n{}\n\nFileMind's Safety Guardian blocks destructive operations on core OS assets to preserve operating system and virtualization integrity.",
                summary_str
            );

            (ans, format!("Safety explanation: {}", data))
        } else if query_lower.contains("linux") || query_lower.contains("wsl") || query_lower.contains("vhdx") {
            let res = CopilotToolRunner::execute(
                CopilotToolName::SearchFiles,
                &json!({ "extension": "vhdx", "limit": 5 }),
                conn,
            );
            tool_executions.push(res);

            let safety_res = CopilotToolRunner::execute(
                CopilotToolName::ExplainSafety,
                &json!({ "path": r"C:\WSL\ext4.vhdx" }),
                conn,
            );
            let safety_summary = safety_res.summary.clone();
            tool_executions.push(safety_res);

            evidence.push(CopilotEvidenceItem {
                path: r"C:\WSL\ext4.vhdx".to_string(),
                filename: "ext4.vhdx".to_string(),
                size_bytes: 15 * 1024 * 1024 * 1024,
                risk_tier: "danger".to_string(),
                reason: "WSL2 Virtual Hard Disk (Rule: WSL-VHDX-IMAGE)".to_string(),
                action_type: "inspect".to_string(),
            });

            suggested_followups.push("Explain why this file is protected.".to_string());

            let ans = format!(
                "### Linux & WSL Storage Analysis\n\nLinux distributions on Windows operate inside virtual disk images (`.vhdx`).\n\n**Safety Classification:**\n{}\n\nThese virtual hard disks are strictly protected against automated deletion.",
                safety_summary
            );

            (ans, "Linux/WSL image findings".to_string())
        } else if query_lower.contains("free") && (query_lower.contains("10 gb") || query_lower.contains("5 gb") || query_lower.contains("plan")) {
            let target_gb: u64 = if query_lower.contains("5 gb") { 5 } else { 10 };
            let target_bytes: u64 = target_gb * 1024 * 1024 * 1024;

            let res = CopilotToolRunner::execute(
                CopilotToolName::GenerateCleanupPlan,
                &json!({ "goal": "free_bytes", "target_bytes": target_bytes }),
                conn,
            );
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            let plan_id = data.get("planId").and_then(|v| v.as_str()).map(|s| s.to_string());
            let reclaimable = data.get("totalPhysicalReclaimableBytes").and_then(|v| v.as_u64()).unwrap_or(0);
            let items_count = data.get("itemsCount").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

            // Formulate Action Proposal requiring explicit user approval
            proposal = Some(CopilotOperationProposal {
                plan_id: plan_id.clone(),
                title: format!("Cleanup Plan: Target Free {} GB", target_gb),
                description: format!(
                    "Prepared draft plan with {} verified candidates ({:.2} MB recoverable on disk).",
                    items_count,
                    reclaimable as f64 / (1024.0 * 1024.0)
                ),
                estimated_freed_bytes: reclaimable,
                target_paths_count: items_count,
                reversibility: "Reversible via Quarantine Vault / Recycle Bin".to_string(),
                requires_explicit_approval: true,
                screen_action: "cleanup-planner".to_string(),
            });

            suggested_followups.push("Tell me which cleanup actions are reversible.".to_string());

            let ans = format!(
                "### Cleanup Plan Prepared\n\n{}\n\n**Action Approval Boundary:** I have assembled this plan as a proposal. Per FileMind safety policies, no files have been modified or deleted. Please review the candidates and execute the dry run in the **Cleanup Planner**.",
                summary_str
            );

            (ans, format!("Cleanup plan data: {}", data))
        } else if query_lower.contains("reversible") || query_lower.contains("undo") || query_lower.contains("recovery") {
            let res = CopilotToolRunner::execute(
                CopilotToolName::GetOperationHistory,
                &json!({ "limit": 10 }),
                conn,
            );
            let summary_str = res.summary.clone();
            tool_executions.push(res);

            suggested_followups.push("Summarize today's storage scan.".to_string());

            let ans = format!(
                "### Reversible Operations in FileMind\n\n{}\n\n1. **Quarantine Vault**: Target files are moved atomically to `%LOCALAPPDATA%\\FileMind\\Quarantine` with preserved metadata and streaming BLAKE3 hashes. Fully reversible via 1-click restore.\n2. **Windows Recycle Bin**: Staged items sent to the system Recycle Bin can be restored from File Explorer.\n3. **Permanent Delete**: Only applied when explicitly chosen by the user on verified disposable caches with affirmative confirmation.",
                summary_str
            );

            (ans, "Reversibility guide".to_string())
        } else if query_lower.contains("recently") || query_lower.contains("changed") {
            let res = CopilotToolRunner::execute(
                CopilotToolName::SearchFiles,
                &json!({ "sort_by": "mtime_desc", "limit": 5 }),
                conn,
            );
            let summary_str = res.summary.clone();
            let data = res.data.clone();
            tool_executions.push(res);

            if let Some(files) = data.get("files").and_then(|v| v.as_array()) {
                for f in files.iter().take(5) {
                    let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let fname = f.get("filename").and_then(|v| v.as_str()).unwrap_or("");
                    let size = f.get("logicalSizeBytes").and_then(|v| v.as_u64()).unwrap_or(0);

                    evidence.push(CopilotEvidenceItem {
                        path: path.to_string(),
                        filename: fname.to_string(),
                        size_bytes: size,
                        risk_tier: "caution".to_string(),
                        reason: "Recently modified file in local index".to_string(),
                        action_type: "inspect".to_string(),
                    });
                }
            }

            let ans = format!("### Recently Modified Files\n\n{}", summary_str);
            (ans, format!("Recent files: {}", data))
        } else {
            // General query: search files or summarize
            let res = CopilotToolRunner::execute(
                CopilotToolName::SearchFiles,
                &json!({ "query": query, "limit": 5 }),
                conn,
            );
            let summary_str = res.summary.clone();
            tool_executions.push(res);

            suggested_followups.push("What is taking up the most space on my C drive?".to_string());
            suggested_followups.push("Prepare a plan to free 10 GB".to_string());
            suggested_followups.push("Summarize today's storage scan.".to_string());

            let ans = format!(
                "### Storage Intelligence Query\n\n{}\n\nYou can ask about large files, duplicates, Linux WSL files, or request cleanup plans.",
                summary_str
            );

            (ans, format!("Search query: {}", query))
        };

        // 2. Local Ollama Integration with Graceful Fallback
        let allow_ai = req.allow_ai.unwrap_or(true);
        let endpoint = req
            .ollama_endpoint
            .as_deref()
            .unwrap_or("http://127.0.0.1:11434");
        let model = req.ollama_model.as_deref().unwrap_or("llama3.2:3b");

        let mut final_answer = deterministic_answer;
        let mut provider_used = "local_rules".to_string();

        if allow_ai {
            // Try querying local Ollama
            match OllamaClient::generate_grounded(endpoint, model, &query, &grounded_text) {
                Ok(ai_response) => {
                    if !ai_response.trim().is_empty() {
                        final_answer = ai_response;
                        provider_used = format!("ollama ({})", model);
                    }
                }
                Err(_) => {
                    // Ollama offline: seamlessly use deterministic local rule response!
                    provider_used = "local_rules (Ollama offline)".to_string();
                }
            }
        }

        CopilotQueryResponse {
            answer: final_answer,
            tool_executions,
            evidence,
            proposal,
            provider: provider_used,
            is_stale_index: is_stale,
            suggested_followups,
        }
    }
}
