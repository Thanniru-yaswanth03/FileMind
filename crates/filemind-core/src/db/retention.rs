use rusqlite::{params, Connection, Result};
use std::time::SystemTime;

pub struct RetentionManager;

impl RetentionManager {
    /// Prune scan sessions older than `retention_days`
    pub fn prune_older_than_days(conn: &mut Connection, retention_days: u32) -> Result<usize> {
        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let cutoff_ms = now_ms - (retention_days as i64 * 86_400_000);

        let deleted_count = conn.execute(
            "DELETE FROM scan_sessions WHERE started_at < ?1",
            params![cutoff_ms],
        )?;

        // Reclaim unused pages incrementally
        let _ = conn.execute_batch("PRAGMA incremental_vacuum(500);");

        Ok(deleted_count)
    }

    /// Keep only the latest `max_sessions` sessions for each root path
    pub fn keep_latest_sessions_per_root(
        conn: &mut Connection,
        max_sessions: usize,
    ) -> Result<usize> {
        let mut stmt = conn.prepare("SELECT DISTINCT root_path FROM scan_sessions")?;
        let roots: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut total_deleted = 0;

        for root in roots {
            let deleted = conn.execute(
                r#"DELETE FROM scan_sessions
                   WHERE root_path = ?1 AND session_id NOT IN (
                       SELECT session_id FROM scan_sessions
                       WHERE root_path = ?1
                       ORDER BY started_at DESC
                       LIMIT ?2
                   )"#,
                params![root, max_sessions as i64],
            )?;
            total_deleted += deleted;
        }

        let _ = conn.execute_batch("PRAGMA incremental_vacuum(500);");
        Ok(total_deleted)
    }
}
