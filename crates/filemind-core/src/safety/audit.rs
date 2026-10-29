use rusqlite::{params, Connection, Result};
use crate::models::{SafetyAuditRecordDto, SafetyClassification, TypedEvidenceItem};

pub struct AuditLogger;

impl AuditLogger {
    /// Inserts a new safety audit entry into SQLite database
    pub fn record_audit(conn: &Connection, record: &SafetyAuditRecordDto) -> Result<i64> {
        let rule_matches_json = serde_json::to_string(&record.rule_matches).unwrap_or_else(|_| "[]".to_string());
        let evidence_json = serde_json::to_string(&record.evidence).unwrap_or_else(|_| "[]".to_string());
        let block_reasons_json = serde_json::to_string(&record.block_reasons).unwrap_or_else(|_| "[]".to_string());
        let classification_str = format!("{:?}", record.classification);

        conn.execute(
            r#"
            INSERT INTO safety_audit_log (
                target_path,
                canonical_path,
                operation,
                classification,
                is_permitted,
                rule_matches_json,
                evidence_json,
                block_reasons_json,
                toctou_verified,
                executed,
                execution_result,
                timestamp_epoch_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                record.target_path,
                record.canonical_path,
                record.operation,
                classification_str,
                if record.is_permitted { 1 } else { 0 },
                rule_matches_json,
                evidence_json,
                block_reasons_json,
                if record.toctou_verified { 1 } else { 0 },
                if record.executed { 1 } else { 0 },
                record.execution_result,
                record.timestamp_epoch_ms as i64,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Queries the most recent safety audit records
    pub fn fetch_recent(conn: &Connection, limit: u32) -> Result<Vec<SafetyAuditRecordDto>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                audit_id,
                target_path,
                canonical_path,
                operation,
                classification,
                is_permitted,
                rule_matches_json,
                evidence_json,
                block_reasons_json,
                toctou_verified,
                executed,
                execution_result,
                timestamp_epoch_ms
            FROM safety_audit_log
            ORDER BY audit_id DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            let audit_id: i64 = row.get(0)?;
            let target_path: String = row.get(1)?;
            let canonical_path: String = row.get(2)?;
            let operation: String = row.get(3)?;
            let class_str: String = row.get(4)?;
            let is_permitted_int: i32 = row.get(5)?;
            let rule_matches_raw: String = row.get(6)?;
            let evidence_raw: String = row.get(7)?;
            let block_reasons_raw: String = row.get(8)?;
            let toctou_verified_int: i32 = row.get(9)?;
            let executed_int: i32 = row.get(10)?;
            let execution_result: Option<String> = row.get(11)?;
            let ts: i64 = row.get(12)?;

            let classification = match class_str.as_str() {
                "LowRiskCandidate" => SafetyClassification::LowRiskCandidate,
                "ReviewRequired" => SafetyClassification::ReviewRequired,
                "ProtectedAndBlocked" => SafetyClassification::ProtectedAndBlocked,
                _ => SafetyClassification::HighRiskOrUnknown,
            };

            let rule_matches: Vec<String> = serde_json::from_str(&rule_matches_raw).unwrap_or_default();
            let evidence: Vec<TypedEvidenceItem> = serde_json::from_str(&evidence_raw).unwrap_or_default();
            let block_reasons: Vec<String> = serde_json::from_str(&block_reasons_raw).unwrap_or_default();

            Ok(SafetyAuditRecordDto {
                audit_id,
                target_path,
                canonical_path,
                operation,
                classification,
                is_permitted: is_permitted_int == 1,
                rule_matches,
                evidence,
                block_reasons,
                toctou_verified: toctou_verified_int == 1,
                executed: executed_int == 1,
                execution_result,
                timestamp_epoch_ms: ts as u64,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }

        Ok(results)
    }
}

