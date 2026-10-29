use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

use crate::adapter::metadata::epoch_ms_to_iso;
use crate::models::{
    CategoryBreakdownDto, ErrorKind, FileAttributesDto, FileCategory, FileItemDto, FileListQuery,
    FileListResponse, FolderRankingDto, RiskTier, ScanErrorDto, ScanSessionDto, SessionStatus,
};
use crate::scanner::walker::ScanResult;

pub struct Database {
    db_path: PathBuf,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Open a new connection with pragmas configured and automatic self-healing recovery
    pub fn get_connection(&self) -> Result<Connection> {
        if let Some(parent) = self.db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn_res = Connection::open(&self.db_path).and_then(|mut c| {
            crate::db::migrations::Migrator::migrate(&mut c)?;
            Ok(c)
        });

        match conn_res {
            Ok(conn) => {
                let _ = Self::seed_default_automation_rules_if_empty(&conn);
                Ok(conn)
            }
            Err(e) => {
                // Self-healing database recovery: If existing database file is corrupt or invalid,
                // backup to corrupt_{timestamp}.db and initialize a fresh database
                if self.db_path.exists() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0);
                    let corrupt_backup = self.db_path.with_extension(format!("corrupt_{}.db", now));
                    eprintln!(
                        "[Database Recovery] SQLite database corrupt ({:?}). Quarantining to {:?} and initializing fresh database.",
                        e, corrupt_backup
                    );
                    let _ = std::fs::rename(&self.db_path, &corrupt_backup);

                    // Preserve and quarantine any surviving WAL / SHM sidecars as recovery artifacts
                    let wal_candidates = [
                        std::path::PathBuf::from(format!("{}-wal", self.db_path.display())),
                        self.db_path.with_extension("db-wal"),
                    ];
                    for wal in &wal_candidates {
                        if wal.exists() {
                            let wal_backup = std::path::PathBuf::from(format!(
                                "{}.corrupt_{}.wal",
                                self.db_path.display(),
                                now
                            ));
                            let _ = std::fs::rename(wal, wal_backup);
                            break;
                        }
                    }

                    let shm_candidates = [
                        std::path::PathBuf::from(format!("{}-shm", self.db_path.display())),
                        self.db_path.with_extension("db-shm"),
                    ];
                    for shm in &shm_candidates {
                        if shm.exists() {
                            let shm_backup = std::path::PathBuf::from(format!(
                                "{}.corrupt_{}.shm",
                                self.db_path.display(),
                                now
                            ));
                            let _ = std::fs::rename(shm, shm_backup);
                            break;
                        }
                    }

                    let mut fresh_conn = Connection::open(&self.db_path)?;
                    crate::db::migrations::Migrator::migrate(&mut fresh_conn)?;
                    let _ = Self::seed_default_automation_rules_if_empty(&fresh_conn);
                    Ok(fresh_conn)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Open an in-memory database for testing
    pub fn open_in_memory() -> Result<Connection> {
        let mut conn = Connection::open_in_memory()?;
        crate::db::migrations::Migrator::migrate(&mut conn)?;
        let _ = Self::seed_default_automation_rules_if_empty(&conn);
        Ok(conn)
    }

    /// Verifies the structural and page integrity of the SQLite database
    pub fn verify_integrity(conn: &Connection) -> Result<bool> {
        let result: String = conn.query_row("PRAGMA integrity_check(1);", [], |r| r.get(0))?;
        Ok(result == "ok")
    }

    /// Get default Windows application data path.
    /// If `portable.flag` exists beside the executable or in the current working directory,
    /// redirects to `./data/filemind.db` for zero-install portability without `%LOCALAPPDATA%` pollution.
    /// Otherwise, defaults to `%LOCALAPPDATA%\FileMind\filemind.db`.
    pub fn default_app_data_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        if exe_dir.join("portable.flag").exists() || std::path::Path::new("portable.flag").exists()
        {
            let data_dir = exe_dir.join("data");
            let _ = std::fs::create_dir_all(&data_dir);
            return data_dir.join("filemind.db");
        }

        let local_app_data = std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
        let dir = PathBuf::from(local_app_data).join("FileMind");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("filemind.db")
    }

    /// Persist complete scan session, directories, files, and errors using batched WAL transactions
    pub fn save_scan_result(
        conn: &mut Connection,
        session_id: &str,
        scan_result: &ScanResult,
        started_at_epoch_ms: u64,
        completed_at_epoch_ms: u64,
    ) -> Result<()> {
        let tx = conn.transaction()?;

        let status_str = if scan_result.is_cancelled {
            "cancelled"
        } else if !scan_result.errors.is_empty() {
            "partial"
        } else {
            "completed"
        };

        // 1. Insert session record
        tx.execute(
            r#"INSERT OR REPLACE INTO scan_sessions (
                session_id, root_path, started_at, completed_at, status,
                total_files, total_directories, total_logical_bytes, total_physical_bytes, error_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                session_id,
                scan_result.root_path,
                started_at_epoch_ms as i64,
                completed_at_epoch_ms as i64,
                status_str,
                scan_result.total_files as i64,
                scan_result.total_directories as i64,
                scan_result.total_logical_bytes as i64,
                scan_result.total_physical_bytes as i64,
                scan_result.errors.len() as i64,
            ],
        )?;

        // 2. Upsert scan root
        tx.execute(
            r#"INSERT INTO scan_roots (
                path, volume_guid, volume_label, filesystem_type,
                cluster_size_bytes, total_bytes, free_bytes, last_scanned_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(path) DO UPDATE SET
                cluster_size_bytes = excluded.cluster_size_bytes,
                total_bytes = excluded.total_bytes,
                free_bytes = excluded.free_bytes,
                last_scanned_at = excluded.last_scanned_at"#,
            params![
                scan_result.root_path,
                "VOL_NTFS",
                "Local Disk",
                "NTFS",
                scan_result.cluster_size_bytes as i64,
                scan_result.volume_total_bytes as i64,
                scan_result.volume_free_bytes as i64,
                completed_at_epoch_ms as i64,
            ],
        )?;

        // 3. Batch insert directories
        {
            let mut stmt = tx.prepare(
                r#"INSERT INTO directories (
                    session_id, path, parent_path, depth,
                    total_size_bytes, file_count, dir_count, mtime
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            )?;

            for dir in &scan_result.directories {
                stmt.execute(params![
                    session_id,
                    dir.path,
                    dir.parent_path,
                    dir.depth as i64,
                    dir.total_size_bytes as i64,
                    dir.file_count as i64,
                    dir.dir_count as i64,
                    dir.mtime_epoch_ms as i64,
                ])?;
            }
        }

        // 4. Batch insert files in chunks of 1000
        {
            let mut stmt = tx.prepare(
                r#"INSERT INTO files (
                    session_id, directory_path, filename, extension, file_type,
                    logical_size_bytes, physical_size_bytes, attributes, ctime, mtime, atime,
                    ntfs_file_id, risk_tier, safety_rationale, is_locked
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"#,
            )?;

            for file in &scan_result.files {
                let category_str = match file.category {
                    FileCategory::Code => "Code",
                    FileCategory::Media => "Media",
                    FileCategory::Archives => "Archives",
                    FileCategory::Documents => "Documents",
                    FileCategory::Executables => "Executables",
                    FileCategory::Caches => "Caches",
                    FileCategory::Other => "Other",
                };

                let risk_str = match file.risk_tier {
                    RiskTier::Safe => "safe",
                    RiskTier::Caution => "caution",
                    RiskTier::High => "high",
                    RiskTier::Danger => "danger",
                };

                let raw_attr = if file.attributes.is_hidden { 2 } else { 0 }
                    | if file.attributes.is_system { 4 } else { 0 }
                    | if file.attributes.is_read_only { 1 } else { 0 }
                    | if file.attributes.is_reparse_point {
                        1024
                    } else {
                        0
                    };

                let mtime_ms = file.mtime_epoch_ms.unwrap_or(0);
                stmt.execute(params![
                    session_id,
                    file.parent_path,
                    file.name,
                    file.extension,
                    category_str,
                    file.size_bytes as i64,
                    file.physical_size_bytes as i64,
                    raw_attr,
                    mtime_ms as i64,
                    mtime_ms as i64,
                    mtime_ms as i64,
                    file.ntfs_file_id,
                    risk_str,
                    file.safety_rationale,
                    if file.is_locked { 1 } else { 0 },
                ])?;
            }
        }

        // 5. Batch insert errors
        {
            let mut err_stmt = tx.prepare(
                r#"INSERT INTO scan_errors (
                    session_id, path, error_kind, error_message, occurred_at
                ) VALUES (?1, ?2, ?3, ?4, ?5)"#,
            )?;

            for err in &scan_result.errors {
                let kind_str = match err.error_kind {
                    ErrorKind::PermissionDenied => "permission_denied",
                    ErrorKind::Locked => "locked",
                    ErrorKind::Disappeared => "disappeared",
                    ErrorKind::PathTooLong => "path_too_long",
                    ErrorKind::IoError => "io_error",
                    ErrorKind::ReparsePolicy => "reparse_policy",
                    ErrorKind::Other => "other",
                };

                err_stmt.execute(params![
                    session_id,
                    err.path,
                    kind_str,
                    err.message,
                    err.occurred_at_epoch_ms as i64,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Retrieve the most recent completed or partial scan session
    pub fn get_latest_session(
        conn: &Connection,
        root_path: Option<&str>,
    ) -> Result<Option<ScanSessionDto>> {
        let sql = match root_path {
            Some(_) => {
                "SELECT session_id, root_path, started_at, completed_at, status,
                        total_files, total_directories, total_logical_bytes, total_physical_bytes, error_count
                 FROM scan_sessions WHERE root_path = ?1 ORDER BY started_at DESC LIMIT 1"
            }
            None => {
                "SELECT session_id, root_path, started_at, completed_at, status,
                        total_files, total_directories, total_logical_bytes, total_physical_bytes, error_count
                 FROM scan_sessions ORDER BY started_at DESC LIMIT 1"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = match root_path {
            Some(p) => stmt.query(params![p])?,
            None => stmt.query([])?,
        };

        if let Some(row) = rows.next()? {
            let started_ms: i64 = row.get(2)?;
            let completed_ms: Option<i64> = row.get(3)?;
            let status_str: String = row.get(4)?;

            let status = match status_str.as_str() {
                "running" => SessionStatus::Running,
                "completed" => SessionStatus::Completed,
                "cancelled" => SessionStatus::Cancelled,
                "partial" => SessionStatus::Partial,
                _ => SessionStatus::Failed,
            };

            Ok(Some(ScanSessionDto {
                session_id: row.get(0)?,
                root_path: row.get(1)?,
                started_at: epoch_ms_to_iso(started_ms as u64),
                completed_at: completed_ms.map(|ms| epoch_ms_to_iso(ms as u64)),
                status,
                total_files: row.get::<_, i64>(5)? as u64,
                total_directories: row.get::<_, i64>(6)? as u64,
                logical_size_bytes: row.get::<_, i64>(7)? as u64,
                physical_size_bytes: row.get::<_, i64>(8)? as u64,
                error_count: row.get::<_, i64>(9)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all historical scan sessions
    pub fn list_sessions(conn: &Connection) -> Result<Vec<ScanSessionDto>> {
        let mut stmt = conn.prepare(
            "SELECT session_id, root_path, started_at, completed_at, status,
                    total_files, total_directories, total_logical_bytes, total_physical_bytes, error_count
             FROM scan_sessions ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let started_ms: i64 = row.get(2)?;
            let completed_ms: Option<i64> = row.get(3)?;
            let status_str: String = row.get(4)?;

            let status = match status_str.as_str() {
                "running" => SessionStatus::Running,
                "completed" => SessionStatus::Completed,
                "cancelled" => SessionStatus::Cancelled,
                "partial" => SessionStatus::Partial,
                _ => SessionStatus::Failed,
            };

            Ok(ScanSessionDto {
                session_id: row.get(0)?,
                root_path: row.get(1)?,
                started_at: epoch_ms_to_iso(started_ms as u64),
                completed_at: completed_ms.map(|ms| epoch_ms_to_iso(ms as u64)),
                status,
                total_files: row.get::<_, i64>(5)? as u64,
                total_directories: row.get::<_, i64>(6)? as u64,
                logical_size_bytes: row.get::<_, i64>(7)? as u64,
                physical_size_bytes: row.get::<_, i64>(8)? as u64,
                error_count: row.get::<_, i64>(9)? as u64,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Query indexed files with high-performance filtering and sorting
    pub fn query_files(conn: &Connection, query: &FileListQuery) -> Result<FileListResponse> {
        let mut conditions: Vec<String> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref session_id) = query.session_id {
            conditions.push("session_id = ?".to_string());
            params_vec.push(Box::new(session_id.clone()));
        }

        if let Some(ref prefix) = query.path_prefix {
            conditions.push("directory_path LIKE ?".to_string());
            params_vec.push(Box::new(format!("{}%", prefix)));
        }

        if let Some(ref category) = query.category {
            if category != "All" && !category.is_empty() {
                conditions.push("file_type = ?".to_string());
                params_vec.push(Box::new(category.clone()));
            }
        }

        if let Some(ref ext) = query.extension {
            if !ext.is_empty() {
                let dot_ext = if ext.starts_with('.') {
                    ext.to_lowercase()
                } else {
                    format!(".{}", ext.to_lowercase())
                };
                let raw_ext = ext.trim_start_matches('.').to_lowercase();
                conditions.push("(extension = ? OR extension = ?)".to_string());
                params_vec.push(Box::new(dot_ext));
                params_vec.push(Box::new(raw_ext));
            }
        }

        if let Some(ref search) = query.search {
            if !search.trim().is_empty() {
                conditions.push("(filename LIKE ? OR directory_path LIKE ?)".to_string());
                let search_pat = format!("%{}%", search.trim());
                params_vec.push(Box::new(search_pat.clone()));
                params_vec.push(Box::new(search_pat));
            }
        }

        if let Some(min_size) = query.min_size {
            conditions.push("logical_size_bytes >= ?".to_string());
            params_vec.push(Box::new(min_size as i64));
        }

        if let Some(max_size) = query.max_size {
            conditions.push("logical_size_bytes <= ?".to_string());
            params_vec.push(Box::new(max_size as i64));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Total count and total bytes query
        let count_sql = format!(
            "SELECT COUNT(*), COALESCE(SUM(logical_size_bytes), 0) FROM files {}",
            where_clause
        );

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let (total_count, total_bytes): (i64, i64) =
            conn.query_row(&count_sql, &param_refs[..], |r| Ok((r.get(0)?, r.get(1)?)))?;

        // Order by
        let sort_column = match query.sort_by.as_deref() {
            Some("name") => "filename",
            Some("mtime") => "mtime",
            Some("category") => "file_type",
            _ => "logical_size_bytes",
        };

        let sort_order = match query.sort_order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC",
        };

        let limit = query.limit.unwrap_or(2000).min(10000);
        let offset = query.offset.unwrap_or(0);

        let data_sql = format!(
            r#"SELECT file_id, directory_path, filename, extension, file_type,
                      logical_size_bytes, physical_size_bytes, attributes,
                      ctime, mtime, atime, ntfs_file_id, risk_tier, safety_rationale, is_locked
               FROM files {} ORDER BY {} {} LIMIT {} OFFSET {}"#,
            where_clause, sort_column, sort_order, limit, offset
        );

        let mut stmt = conn.prepare(&data_sql)?;
        let rows = stmt.query_map(&param_refs[..], |row| {
            let file_id: i64 = row.get(0)?;
            let dir_path: String = row.get(1)?;
            let filename: String = row.get(2)?;
            let ext: String = row.get(3)?;
            let file_type_str: String = row.get(4)?;
            let size_bytes: i64 = row.get(5)?;
            let phys_bytes: i64 = row.get(6)?;
            let attrs: i64 = row.get(7)?;
            let mtime_ms: i64 = row.get(9)?;
            let ntfs_id: Option<String> = row.get(11)?;
            let risk_str: String = row.get(12)?;
            let rationale: Option<String> = row.get(13)?;
            let is_locked: i64 = row.get(14)?;

            let category = match file_type_str.as_str() {
                "Code" => FileCategory::Code,
                "Media" => FileCategory::Media,
                "Archives" => FileCategory::Archives,
                "Documents" => FileCategory::Documents,
                "Executables" => FileCategory::Executables,
                "Caches" => FileCategory::Caches,
                _ => FileCategory::Other,
            };

            let risk_tier = match risk_str.as_str() {
                "danger" => RiskTier::Danger,
                "high" => RiskTier::High,
                "caution" => RiskTier::Caution,
                _ => RiskTier::Safe,
            };

            let full_path = if dir_path.ends_with('\\') {
                format!("{}{}", dir_path, filename)
            } else {
                format!("{}\\{}", dir_path, filename)
            };

            Ok(FileItemDto {
                id: format!("file_{}", file_id),
                name: filename,
                path: full_path,
                parent_path: dir_path,
                extension: ext,
                size_bytes: size_bytes as u64,
                physical_size_bytes: phys_bytes as u64,
                category,
                mtime: epoch_ms_to_iso(mtime_ms as u64),
                ctime: epoch_ms_to_iso(mtime_ms as u64),
                atime: epoch_ms_to_iso(mtime_ms as u64),
                mtime_epoch_ms: Some(mtime_ms as u64),
                attributes: FileAttributesDto {
                    is_hidden: (attrs & 2) != 0,
                    is_system: (attrs & 4) != 0,
                    is_read_only: (attrs & 1) != 0,
                    is_reparse_point: (attrs & 1024) != 0,
                },
                ntfs_file_id: ntfs_id,
                risk_tier,
                safety_rationale: rationale.unwrap_or_default(),
                is_locked: is_locked != 0,
                is_directory: false,
                child_count: None,
            })
        })?;

        let mut files = Vec::new();
        for r in rows {
            files.push(r?);
        }

        Ok(FileListResponse {
            total_count: total_count as u64,
            total_bytes: total_bytes as u64,
            files,
        })
    }

    /// Retrieve the top space-consuming folders for a session
    pub fn get_top_folders(
        conn: &Connection,
        session_id: &str,
        limit: u32,
    ) -> Result<Vec<FolderRankingDto>> {
        let total_size: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total_size_bytes), 1) FROM directories WHERE session_id = ?1 AND depth = 0",
                params![session_id],
                |r| r.get(0),
            )
            .unwrap_or(1);

        let mut stmt = conn.prepare(
            r#"SELECT path, total_size_bytes, file_count, dir_count
               FROM directories
               WHERE session_id = ?1 AND total_size_bytes > 0
               ORDER BY total_size_bytes DESC LIMIT ?2"#,
        )?;

        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            let path: String = row.get(0)?;
            let bytes: i64 = row.get(1)?;
            let files: i64 = row.get(2)?;
            let dirs: i64 = row.get(3)?;
            let pct = if total_size > 0 {
                (bytes as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            Ok(FolderRankingDto {
                path,
                total_size_bytes: bytes as u64,
                file_count: files as u64,
                dir_count: dirs as u64,
                percentage_of_total: (pct * 10.0).round() / 10.0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Retrieve storage breakdown aggregated by file category
    pub fn get_category_breakdown(
        conn: &Connection,
        session_id: &str,
    ) -> Result<Vec<CategoryBreakdownDto>> {
        let total_bytes: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(logical_size_bytes), 1) FROM files WHERE session_id = ?1",
                params![session_id],
                |r| r.get(0),
            )
            .unwrap_or(1);

        let mut stmt = conn.prepare(
            r#"SELECT file_type, SUM(logical_size_bytes), COUNT(*)
               FROM files
               WHERE session_id = ?1
               GROUP BY file_type
               ORDER BY SUM(logical_size_bytes) DESC"#,
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let cat_str: String = row.get(0)?;
            let bytes: i64 = row.get(1)?;
            let count: i64 = row.get(2)?;

            let category = match cat_str.as_str() {
                "Code" => FileCategory::Code,
                "Media" => FileCategory::Media,
                "Archives" => FileCategory::Archives,
                "Documents" => FileCategory::Documents,
                "Executables" => FileCategory::Executables,
                "Caches" => FileCategory::Caches,
                _ => FileCategory::Other,
            };

            let pct = if total_bytes > 0 {
                (bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            Ok(CategoryBreakdownDto {
                category,
                total_bytes: bytes as u64,
                file_count: count as u64,
                percentage: (pct * 10.0).round() / 10.0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Retrieve errors encountered during a specific scan session
    pub fn get_session_errors(
        conn: &Connection,
        session_id: &str,
        limit: u32,
    ) -> Result<Vec<ScanErrorDto>> {
        let mut stmt = conn.prepare(
            r#"SELECT error_id, session_id, path, error_kind, error_message, occurred_at
               FROM scan_errors
               WHERE session_id = ?1
               ORDER BY occurred_at DESC LIMIT ?2"#,
        )?;

        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            let id: i64 = row.get(0)?;
            let sid: String = row.get(1)?;
            let path: String = row.get(2)?;
            let kind_str: String = row.get(3)?;
            let msg: String = row.get(4)?;
            let occurred_ms: i64 = row.get(5)?;

            let error_kind = match kind_str.as_str() {
                "permission_denied" => ErrorKind::PermissionDenied,
                "locked" => ErrorKind::Locked,
                "disappeared" => ErrorKind::Disappeared,
                "path_too_long" => ErrorKind::PathTooLong,
                "reparse_policy" => ErrorKind::ReparsePolicy,
                _ => ErrorKind::IoError,
            };

            Ok(ScanErrorDto {
                id,
                session_id: sid,
                path,
                error_kind,
                message: msg,
                occurred_at: epoch_ms_to_iso(occurred_ms as u64),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Record a safety audit entry
    pub fn record_safety_audit(
        conn: &Connection,
        record: &crate::models::SafetyAuditRecordDto,
    ) -> Result<i64> {
        crate::safety::AuditLogger::record_audit(conn, record)
    }

    /// Retrieve recent safety audit entries
    pub fn get_recent_safety_audits(
        conn: &Connection,
        limit: u32,
    ) -> Result<Vec<crate::models::SafetyAuditRecordDto>> {
        crate::safety::AuditLogger::fetch_recent(conn, limit)
    }

    // ==========================================
    // Phase 7: Automation Repository Methods
    // ==========================================

    pub fn insert_automation_rule(
        conn: &Connection,
        rule: &crate::models::AutomationRuleDto,
    ) -> Result<()> {
        let trigger_type = match &rule.trigger {
            crate::models::AutomationTrigger::Schedule { .. } => "schedule",
            crate::models::AutomationTrigger::DiskSpaceThreshold { .. } => "disk_space_threshold",
            crate::models::AutomationTrigger::PostScan { .. } => "post_scan",
            crate::models::AutomationTrigger::FileEvent { .. } => "file_event",
            crate::models::AutomationTrigger::FolderGrowth { .. } => "folder_growth",
            crate::models::AutomationTrigger::Manual => "manual",
        };

        let trigger_json =
            serde_json::to_string(&rule.trigger).unwrap_or_else(|_| "{}".to_string());
        let conditions_json =
            serde_json::to_string(&rule.conditions).unwrap_or_else(|_| "[]".to_string());
        let actions_json =
            serde_json::to_string(&rule.actions).unwrap_or_else(|_| "[]".to_string());
        let scope_json =
            serde_json::to_string(&rule.scope_paths).unwrap_or_else(|_| "[]".to_string());
        let exclusion_json =
            serde_json::to_string(&rule.exclusion_paths).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            r#"INSERT OR REPLACE INTO automation_rules (
                rule_id, name, description, is_enabled, trigger_type, trigger_config_json,
                conditions_json, actions_json, scope_paths_json, exclusion_paths_json,
                schedule_expression, requires_explicit_approval, dry_run_mode,
                max_retries, cooldown_seconds, max_files_per_run, max_bytes_per_run,
                last_run_at, last_status, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21
            )"#,
            params![
                rule.rule_id,
                rule.name,
                rule.description,
                if rule.is_enabled { 1 } else { 0 },
                trigger_type,
                trigger_json,
                conditions_json,
                actions_json,
                scope_json,
                exclusion_json,
                rule.schedule_expression,
                if rule.requires_explicit_approval { 1 } else { 0 },
                if rule.dry_run_mode { 1 } else { 0 },
                rule.retry_policy.max_retries as i64,
                rule.execution_limits.cooldown_seconds as i64,
                rule.execution_limits.max_files_per_run as i64,
                rule.execution_limits.max_bytes_per_run as i64,
                rule.last_run_at.map(|v| v as i64),
                rule.last_status,
                rule.created_at as i64,
                rule.updated_at as i64,
            ],
        )?;

        Ok(())
    }

    pub fn list_automation_rules(
        conn: &Connection,
    ) -> Result<Vec<crate::models::AutomationRuleDto>> {
        let mut stmt = conn.prepare(
            r#"SELECT rule_id, name, description, is_enabled, trigger_config_json,
                      conditions_json, actions_json, scope_paths_json, exclusion_paths_json,
                      schedule_expression, requires_explicit_approval, dry_run_mode,
                      max_retries, cooldown_seconds, max_files_per_run, max_bytes_per_run,
                      last_run_at, last_status, created_at, updated_at
               FROM automation_rules
               ORDER BY created_at ASC"#,
        )?;

        let rows = stmt.query_map([], |row| {
            let rule_id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let is_enabled: bool = row.get::<_, i64>(3)? == 1;
            let trigger_json: String = row.get(4)?;
            let conditions_json: String = row.get(5)?;
            let actions_json: String = row.get(6)?;
            let scope_json: String = row.get(7)?;
            let exclusion_json: String = row.get(8)?;
            let schedule_expression: String = row.get(9)?;
            let requires_approval: bool = row.get::<_, i64>(10)? == 1;
            let dry_run: bool = row.get::<_, i64>(11)? == 1;
            let max_retries: i64 = row.get(12)?;
            let cooldown_sec: i64 = row.get(13)?;
            let max_files: i64 = row.get(14)?;
            let max_bytes: i64 = row.get(15)?;
            let last_run_at: Option<i64> = row.get(16)?;
            let last_status: Option<String> = row.get(17)?;
            let created_at: i64 = row.get(18)?;
            let updated_at: i64 = row.get(19)?;

            let trigger = serde_json::from_str(&trigger_json)
                .unwrap_or(crate::models::AutomationTrigger::Manual);
            let conditions = serde_json::from_str(&conditions_json).unwrap_or_default();
            let actions = serde_json::from_str(&actions_json).unwrap_or_default();
            let scope_paths = serde_json::from_str(&scope_json).unwrap_or_default();
            let exclusion_paths = serde_json::from_str(&exclusion_json).unwrap_or_default();

            Ok(crate::models::AutomationRuleDto {
                rule_id,
                name,
                description,
                is_enabled,
                trigger,
                conditions,
                actions,
                scope_paths,
                exclusion_paths,
                schedule_expression,
                requires_explicit_approval: requires_approval,
                dry_run_mode: dry_run,
                retry_policy: crate::models::RetryPolicy {
                    max_retries: max_retries as u32,
                    backoff_seconds: 10,
                },
                execution_limits: crate::models::ExecutionLimits {
                    max_files_per_run: max_files as u32,
                    max_bytes_per_run: max_bytes as u64,
                    cooldown_seconds: cooldown_sec as u32,
                },
                last_run_at: last_run_at.map(|v| v as u64),
                last_status,
                created_at: created_at as u64,
                updated_at: updated_at as u64,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_automation_rule(
        conn: &Connection,
        rule_id: &str,
    ) -> Result<Option<crate::models::AutomationRuleDto>> {
        let rules = Self::list_automation_rules(conn)?;
        Ok(rules.into_iter().find(|r| r.rule_id == rule_id))
    }

    pub fn toggle_automation_rule(conn: &Connection, rule_id: &str, enabled: bool) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        conn.execute(
            "UPDATE automation_rules SET is_enabled = ?1, updated_at = ?2 WHERE rule_id = ?3",
            params![if enabled { 1 } else { 0 }, now, rule_id],
        )?;
        Ok(())
    }

    pub fn delete_automation_rule(conn: &Connection, rule_id: &str) -> Result<()> {
        conn.execute(
            "DELETE FROM automation_rules WHERE rule_id = ?1",
            params![rule_id],
        )?;
        Ok(())
    }

    pub fn update_rule_last_run(
        conn: &Connection,
        rule_id: &str,
        status: &str,
        epoch_ms: u64,
    ) -> Result<()> {
        conn.execute(
            "UPDATE automation_rules SET last_run_at = ?1, last_status = ?2, updated_at = ?1 WHERE rule_id = ?3",
            params![epoch_ms as i64, status, rule_id],
        )?;
        Ok(())
    }

    pub fn record_automation_execution(
        conn: &Connection,
        hist: &crate::models::AutomationExecutionHistoryDto,
    ) -> Result<()> {
        let actions_json =
            serde_json::to_string(&hist.actions_attempted).unwrap_or_else(|_| "[]".to_string());
        let policies_json =
            serde_json::to_string(&hist.policy_decisions).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            r#"INSERT INTO automation_execution_history (
                run_id, rule_id, rule_name, trigger_source, started_at, completed_at,
                status, inputs_summary, actions_attempted_json, policy_decisions_json,
                result_summary, error_message, retries_attempted, files_affected_count,
                bytes_affected, plan_id
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
            )"#,
            params![
                hist.run_id,
                hist.rule_id,
                hist.rule_name,
                hist.trigger_source,
                hist.started_at as i64,
                hist.completed_at.map(|v| v as i64),
                hist.status,
                hist.inputs_summary,
                actions_json,
                policies_json,
                hist.result_summary,
                hist.error_message,
                hist.retries_attempted as i64,
                hist.files_affected_count as i64,
                hist.bytes_affected as i64,
                hist.plan_id,
            ],
        )?;

        Ok(())
    }

    pub fn list_automation_history(
        conn: &Connection,
        rule_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<crate::models::AutomationExecutionHistoryDto>> {
        let sql = if rule_id.is_some() {
            r#"SELECT run_id, rule_id, rule_name, trigger_source, started_at, completed_at,
                      status, inputs_summary, actions_attempted_json, policy_decisions_json,
                      result_summary, error_message, retries_attempted, files_affected_count,
                      bytes_affected, plan_id
               FROM automation_execution_history
               WHERE rule_id = ?1
               ORDER BY started_at DESC LIMIT ?2"#
        } else {
            r#"SELECT run_id, rule_id, rule_name, trigger_source, started_at, completed_at,
                      status, inputs_summary, actions_attempted_json, policy_decisions_json,
                      result_summary, error_message, retries_attempted, files_affected_count,
                      bytes_affected, plan_id
               FROM automation_execution_history
               ORDER BY started_at DESC LIMIT ?1"#
        };

        let mut list = Vec::new();

        if let Some(rid) = rule_id {
            let mut stmt = conn.prepare(sql)?;
            let rows =
                stmt.query_map(params![rid, limit as i64], |row| Self::map_history_row(row))?;
            for r in rows {
                list.push(r?);
            }
        } else {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(params![limit as i64], |row| Self::map_history_row(row))?;
            for r in rows {
                list.push(r?);
            }
        }

        Ok(list)
    }

    fn map_history_row(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<crate::models::AutomationExecutionHistoryDto> {
        let run_id: String = row.get(0)?;
        let rule_id: String = row.get(1)?;
        let rule_name: String = row.get(2)?;
        let trigger_source: String = row.get(3)?;
        let started_at: i64 = row.get(4)?;
        let completed_at: Option<i64> = row.get(5)?;
        let status: String = row.get(6)?;
        let inputs_summary: String = row.get(7)?;
        let actions_json: String = row.get(8)?;
        let policies_json: String = row.get(9)?;
        let result_summary: String = row.get(10)?;
        let error_message: Option<String> = row.get(11)?;
        let retries: i64 = row.get(12)?;
        let files_count: i64 = row.get(13)?;
        let bytes_count: i64 = row.get(14)?;
        let plan_id: Option<String> = row.get(15)?;

        let actions = serde_json::from_str(&actions_json).unwrap_or_default();
        let policies = serde_json::from_str(&policies_json).unwrap_or_default();

        Ok(crate::models::AutomationExecutionHistoryDto {
            run_id,
            rule_id,
            rule_name,
            trigger_source,
            started_at: started_at as u64,
            completed_at: completed_at.map(|v| v as u64),
            status,
            inputs_summary,
            actions_attempted: actions,
            policy_decisions: policies,
            result_summary,
            error_message,
            retries_attempted: retries as u32,
            files_affected_count: files_count as u32,
            bytes_affected: bytes_count as u64,
            plan_id,
        })
    }

    /// Seed predefined default automation rules if table is empty
    pub fn seed_default_automation_rules_if_empty(conn: &Connection) -> Result<()> {
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM automation_rules", [], |r| r.get(0))?;
        if count > 0 {
            return Ok(());
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let defaults = vec![
            crate::models::AutomationRuleDto {
                rule_id: "rule_low_disk_space".to_string(),
                name: "Low Disk Space Alert".to_string(),
                description: "Notifies immediately when free storage on system drive C: falls below 10 GB.".to_string(),
                is_enabled: true,
                trigger: crate::models::AutomationTrigger::DiskSpaceThreshold {
                    drive_letter: "C".to_string(),
                    threshold_gb: 10.0,
                },
                conditions: vec![crate::models::AutomationCondition::DiskFreeBelowBytes {
                    bytes: 10 * 1024 * 1024 * 1024,
                }],
                actions: vec![crate::models::AutomationAction::NotifyUser {
                    title: "Storage Alert: C: Drive Low Space".to_string(),
                    message: "Free space on C: is below 10 GB. Consider reviewing large files or running a cleanup plan.".to_string(),
                    severity: "warning".to_string(),
                }],
                scope_paths: vec!["C:\\".to_string()],
                exclusion_paths: vec![],
                schedule_expression: "Every 15 minutes".to_string(),
                requires_explicit_approval: false,
                dry_run_mode: true,
                retry_policy: crate::models::RetryPolicy::default(),
                execution_limits: crate::models::ExecutionLimits {
                    max_files_per_run: 0,
                    max_bytes_per_run: 0,
                    cooldown_seconds: 900,
                },
                last_run_at: None,
                last_status: None,
                created_at: now,
                updated_at: now,
            },
            crate::models::AutomationRuleDto {
                rule_id: "rule_post_scan_duplicates".to_string(),
                name: "Post-Scan Duplicate Review".to_string(),
                description: "Automatically analyzes newly completed scans and drafts a reviewable duplicate cleanup proposal.".to_string(),
                is_enabled: true,
                trigger: crate::models::AutomationTrigger::PostScan {
                    min_files_scanned: Some(100),
                },
                conditions: vec![],
                actions: vec![crate::models::AutomationAction::CreateCleanupPlan {
                    goal_type: "duplicates".to_string(),
                    target_bytes: None,
                }],
                scope_paths: vec!["C:\\".to_string()],
                exclusion_paths: vec![],
                schedule_expression: "Triggered on Scan Completion".to_string(),
                requires_explicit_approval: true,
                dry_run_mode: true,
                retry_policy: crate::models::RetryPolicy::default(),
                execution_limits: crate::models::ExecutionLimits {
                    max_files_per_run: 200,
                    max_bytes_per_run: 50 * 1024 * 1024 * 1024,
                    cooldown_seconds: 60,
                },
                last_run_at: None,
                last_status: None,
                created_at: now,
                updated_at: now,
            },
            crate::models::AutomationRuleDto {
                rule_id: "rule_weekly_developer_cache".to_string(),
                name: "Weekly Developer Cache Hygiene".to_string(),
                description: "Prepares a weekly dry-run cleanup plan for regenerable build targets (target/, node_modules/) across Git-clean projects.".to_string(),
                is_enabled: true,
                trigger: crate::models::AutomationTrigger::Schedule {
                    frequency: "weekly".to_string(),
                    interval_minutes: Some(10080),
                },
                conditions: vec![crate::models::AutomationCondition::GitCleanOnly],
                actions: vec![crate::models::AutomationAction::CreateCleanupPlan {
                    goal_type: "developer_artifacts".to_string(),
                    target_bytes: None,
                }],
                scope_paths: vec![],
                exclusion_paths: vec![],
                schedule_expression: "Every Monday 02:00 AM".to_string(),
                requires_explicit_approval: true,
                dry_run_mode: true,
                retry_policy: crate::models::RetryPolicy::default(),
                execution_limits: crate::models::ExecutionLimits::default(),
                last_run_at: None,
                last_status: None,
                created_at: now,
                updated_at: now,
            },
        ];

        for r in defaults {
            Self::insert_automation_rule(conn, &r)?;
        }

        Ok(())
    }
}
