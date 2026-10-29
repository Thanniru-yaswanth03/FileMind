use rusqlite::{Connection, Result};
use std::time::SystemTime;

use crate::db::schema::{MIGRATION_V1_DDL, PRAGMAS_INIT};

pub struct Migrator;

impl Migrator {
    /// Apply initial pragmatic optimizations and run pending schema migrations
    pub fn migrate(conn: &mut Connection) -> Result<()> {
        conn.execute_batch(PRAGMAS_INIT)?;

        let current_version: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

        if current_version < 1 {
            let tx = conn.transaction()?;
            tx.execute_batch(MIGRATION_V1_DDL)?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            tx.execute(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    1,
                    now,
                    "V1: Base storage scanner, sessions, directories, files, and errors schema"
                ],
            )?;

            tx.pragma_update(None, "user_version", 1)?;
            tx.commit()?;
        }

        let version_after_v1: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

        if version_after_v1 < 2 {
            let tx = conn.transaction()?;
            tx.execute_batch(crate::db::schema::MIGRATION_V2_DDL)?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            tx.execute(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    2,
                    now,
                    "V2: FTS5 full-text document search and duplicate detection indexing"
                ],
            )?;

            tx.pragma_update(None, "user_version", 2)?;
            tx.commit()?;
        }

        let version_after_v2: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

        if version_after_v2 < 3 {
            let tx = conn.transaction()?;
            tx.execute_batch(crate::db::schema::MIGRATION_V3_DDL)?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            tx.execute(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    3,
                    now,
                    "V3: Smart Delete Guardian and Dependency Intelligence safety audit log"
                ],
            )?;

            tx.pragma_update(None, "user_version", 3)?;
            tx.commit()?;
        }

        let version_after_v3: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

        if version_after_v3 < 4 {
            let tx = conn.transaction()?;
            tx.execute_batch(crate::db::schema::MIGRATION_V4_DDL)?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            tx.execute(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![4, now, "V4: Intelligent cleanup plans, dry runs, transactions, and quarantine recovery"],
            )?;

            tx.pragma_update(None, "user_version", 4)?;
            tx.commit()?;
        }

        let version_after_v4: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

        if version_after_v4 < 5 {
            let tx = conn.transaction()?;
            tx.execute_batch(crate::db::schema::MIGRATION_V5_DDL)?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            tx.execute(
                "INSERT INTO schema_version (version, applied_at, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![5, now, "V5: Local automation rules engine, execution history, and scheduler persistence"],
            )?;

            tx.pragma_update(None, "user_version", 5)?;
            tx.commit()?;
        }

        Ok(())
    }

    /// Retrieve the current applied schema version
    pub fn get_version(conn: &Connection) -> Result<i32> {
        conn.query_row("PRAGMA user_version;", [], |r| r.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::CURRENT_SCHEMA_VERSION;

    #[test]
    fn test_migrations_apply_cleanly() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert_eq!(Migrator::get_version(&conn).unwrap(), 0);

        Migrator::migrate(&mut conn).unwrap();
        assert_eq!(
            Migrator::get_version(&conn).unwrap(),
            CURRENT_SCHEMA_VERSION
        );

        // Re-running migration should be an idempotent no-op
        Migrator::migrate(&mut conn).unwrap();
        assert_eq!(
            Migrator::get_version(&conn).unwrap(),
            CURRENT_SCHEMA_VERSION
        );
    }
}
