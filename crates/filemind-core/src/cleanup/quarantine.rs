use rusqlite::{params, Connection, Result as SqlResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::adapter::metadata::{inspect_file, system_time_to_epoch_ms};
use crate::adapter::path::{normalize_path, strip_verbatim_prefix, to_long_path};
use crate::duplicate::compute_file_blake3_streaming;
use crate::models::{QuarantineRecordDto, RestoreItemResult, SafetyClassification};
use crate::safety::SafetyGuardian;

static QUARANTINE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct QuarantineManager {
    vault_dir: PathBuf,
}

impl QuarantineManager {
    pub fn new(vault_dir: Option<PathBuf>) -> Self {
        let dir = vault_dir.unwrap_or_else(|| {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));

            if exe_dir.join("portable.flag").exists() || Path::new("portable.flag").exists() {
                return exe_dir.join("data").join("Quarantine");
            }

            let local_app_data = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
            PathBuf::from(local_app_data).join("FileMind").join("Quarantine")
        });

        let _ = fs::create_dir_all(&dir);
        Self { vault_dir: dir }
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_dir
    }

    /// Moves a candidate file or directory safely into the application-managed Quarantine Vault.
    /// Preserves original file attributes, timestamps, and metadata in SQLite.
    pub fn quarantine_target(
        &self,
        path: &Path,
        reason: &str,
        conn: &Connection,
    ) -> Result<QuarantineRecordDto, String> {
        let clean_path = strip_verbatim_prefix(&path.to_string_lossy());
        let norm_path = normalize_path(Path::new(&clean_path));
        let path_obj = Path::new(&norm_path);

        if !path_obj.exists() {
            return Err(format!("Target does not exist: '{}'", norm_path));
        }

        // 1. Mandatory Safety Engine Evaluation: Quarantine cannot bypass OS or WSL protection
        let assessment = SafetyGuardian::evaluate_target(path_obj, "quarantine");
        if assessment.classification == SafetyClassification::ProtectedAndBlocked {
            return Err(format!(
                "Quarantine blocked by security policy: {}",
                assessment.block_reason.unwrap_or_default()
            ));
        }

        // 2. Capture deep metadata
        let long_path = to_long_path(path_obj);
        let meta = inspect_file(&long_path, 4096)
            .map_err(|e| format!("Failed to inspect target metadata: {}", e))?;

        if meta.is_locked {
            return Err("Target file is actively locked by another process".to_string());
        }

        let now_epoch_ms = system_time_to_epoch_ms(SystemTime::now());
        let counter = QUARANTINE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let quarantine_id = format!("quar_{}_{}", now_epoch_ms, counter);

        let filename = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed_target")
            .to_string();

        let vault_filename = format!("{}_{}", quarantine_id, filename);
        let quarantine_dest = self.vault_dir.join(&vault_filename);

        // Calculate cryptographic hash if regular file under 100MB
        let blake3_hash = if !meta.is_directory && meta.logical_size_bytes > 0 && meta.logical_size_bytes < 100 * 1024 * 1024 {
            compute_file_blake3_streaming(path_obj).ok().map(|(hash, _, _)| hash)
        } else {
            None
        };

        // 3. Move target to quarantine vault using verified atomic rename or streaming copy
        verified_quarantine_move(
            path_obj,
            &quarantine_dest,
            meta.is_directory,
            blake3_hash.as_deref(),
        ).map_err(|e| format!("Quarantine transfer failed: {}", e))?;

        let raw_attributes = if meta.is_read_only { 1 } else { 0 }
            | if meta.is_hidden { 2 } else { 0 }
            | if meta.is_system { 4 } else { 0 };

        let record = QuarantineRecordDto {
            quarantine_id: quarantine_id.clone(),
            original_path: clean_path.to_string(),
            canonical_path: norm_path,
            quarantine_path: quarantine_dest.to_string_lossy().to_string(),
            filename,
            size_bytes: meta.logical_size_bytes,
            original_ctime: meta.ctime_epoch_ms,
            original_mtime: meta.mtime_epoch_ms,
            attributes: raw_attributes,
            blake3_hash,
            quarantined_at_epoch_ms: now_epoch_ms,
            status: "quarantined".to_string(),
            reason: reason.to_string(),
            restored_at_epoch_ms: None,
            restore_error: None,
        };

        // 4. Record entry in SQLite database
        conn.execute(
            r#"
            INSERT INTO quarantine_records (
                quarantine_id,
                original_path,
                canonical_path,
                quarantine_path,
                filename,
                size_bytes,
                original_ctime,
                original_mtime,
                attributes,
                blake3_hash,
                quarantined_at,
                status,
                reason,
                restored_at,
                restore_error
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                record.quarantine_id,
                record.original_path,
                record.canonical_path,
                record.quarantine_path,
                record.filename,
                record.size_bytes as i64,
                record.original_ctime as i64,
                record.original_mtime as i64,
                record.attributes as i64,
                record.blake3_hash,
                record.quarantined_at_epoch_ms as i64,
                record.status,
                record.reason,
                record.restored_at_epoch_ms.map(|t| t as i64),
                record.restore_error,
            ],
        )
        .map_err(|e| format!("Failed to record quarantine entry in database: {}", e))?;

        Ok(record)
    }

    /// Restores a quarantined item back to its original path or an alternate destination.
    /// Invariant: Refuses to overwrite any existing destination file silently.
    pub fn restore_target(
        &self,
        quarantine_id: &str,
        alternate_dest: Option<&Path>,
        conn: &Connection,
    ) -> RestoreItemResult {
        let record = match self.get_record(quarantine_id, conn) {
            Ok(Some(r)) => r,
            Ok(None) => {
                return RestoreItemResult {
                    quarantine_id: quarantine_id.to_string(),
                    original_path: String::new(),
                    restored_path: String::new(),
                    success: false,
                    error: Some("Quarantine record not found".to_string()),
                }
            }
            Err(e) => {
                return RestoreItemResult {
                    quarantine_id: quarantine_id.to_string(),
                    original_path: String::new(),
                    restored_path: String::new(),
                    success: false,
                    error: Some(format!("Database query error: {}", e)),
                }
            }
        };

        let quar_path = Path::new(&record.quarantine_path);
        if !quar_path.exists() {
            return RestoreItemResult {
                quarantine_id: quarantine_id.to_string(),
                original_path: record.original_path.clone(),
                restored_path: String::new(),
                success: false,
                error: Some(format!("Quarantined file missing from vault: '{}'", record.quarantine_path)),
            };
        }

        let destination = alternate_dest
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&record.original_path));

        // CONFLICT CHECK: Never overwrite an existing destination file silently!
        if destination.exists() {
            return RestoreItemResult {
                quarantine_id: quarantine_id.to_string(),
                original_path: record.original_path.clone(),
                restored_path: destination.to_string_lossy().to_string(),
                success: false,
                error: Some(format!(
                    "Restore conflict: Destination '{}' already exists. Silently overwriting is prohibited.",
                    destination.display()
                )),
            };
        }

        // Recreate parent directories if needed
        if let Some(parent) = destination.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Move back to destination using verified atomic rename or streaming copy
        let is_dir = quar_path.is_dir();
        if let Err(e) = verified_quarantine_move(quar_path, &destination, is_dir, record.blake3_hash.as_deref()) {
            return RestoreItemResult {
                quarantine_id: quarantine_id.to_string(),
                original_path: record.original_path.clone(),
                restored_path: destination.to_string_lossy().to_string(),
                success: false,
                error: Some(format!("Failed to restore item: {}", e)),
            };
        }

        let now_ms = system_time_to_epoch_ms(SystemTime::now());
        let _ = conn.execute(
            "UPDATE quarantine_records SET status = 'restored', restored_at = ?1, restore_error = NULL WHERE quarantine_id = ?2",
            params![now_ms as i64, quarantine_id],
        );

        RestoreItemResult {
            quarantine_id: quarantine_id.to_string(),
            original_path: record.original_path,
            restored_path: destination.to_string_lossy().to_string(),
            success: true,
            error: None,
        }
    }

    /// Permanently purges a record and file from the Quarantine Vault
    pub fn purge_target(&self, quarantine_id: &str, conn: &Connection) -> Result<u64, String> {
        let record = match self.get_record(quarantine_id, conn) {
            Ok(Some(r)) => r,
            Ok(None) => return Err("Quarantine record not found".to_string()),
            Err(e) => return Err(format!("Database error: {}", e)),
        };

        let quar_path = Path::new(&record.quarantine_path);
        if quar_path.exists() {
            if quar_path.is_dir() {
                let _ = fs::remove_dir_all(quar_path);
            } else {
                let _ = fs::remove_file(quar_path);
            }
        }

        let _ = conn.execute(
            "UPDATE quarantine_records SET status = 'purged' WHERE quarantine_id = ?1",
            params![quarantine_id],
        );

        Ok(record.size_bytes)
    }

    /// Retrieves an individual quarantine record
    pub fn get_record(&self, quarantine_id: &str, conn: &Connection) -> SqlResult<Option<QuarantineRecordDto>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                quarantine_id, original_path, canonical_path, quarantine_path,
                filename, size_bytes, original_ctime, original_mtime,
                attributes, blake3_hash, quarantined_at, status, reason,
                restored_at, restore_error
            FROM quarantine_records
            WHERE quarantine_id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![quarantine_id])?;
        if let Some(row) = rows.next()? {
            let q_id: String = row.get(0)?;
            let orig: String = row.get(1)?;
            let canon: String = row.get(2)?;
            let q_path: String = row.get(3)?;
            let fname: String = row.get(4)?;
            let size: i64 = row.get(5)?;
            let ctime: i64 = row.get(6)?;
            let mtime: i64 = row.get(7)?;
            let attr: i64 = row.get(8)?;
            let hash: Option<String> = row.get(9)?;
            let q_at: i64 = row.get(10)?;
            let status: String = row.get(11)?;
            let reason: String = row.get(12)?;
            let restored_at: Option<i64> = row.get(13)?;
            let restore_err: Option<String> = row.get(14)?;

            Ok(Some(QuarantineRecordDto {
                quarantine_id: q_id,
                original_path: orig,
                canonical_path: canon,
                quarantine_path: q_path,
                filename: fname,
                size_bytes: size as u64,
                original_ctime: ctime as u64,
                original_mtime: mtime as u64,
                attributes: attr as u32,
                blake3_hash: hash,
                quarantined_at_epoch_ms: q_at as u64,
                status,
                reason,
                restored_at_epoch_ms: restored_at.map(|t| t as u64),
                restore_error: restore_err,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists all quarantine records
    pub fn list_all(&self, conn: &Connection) -> SqlResult<Vec<QuarantineRecordDto>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                quarantine_id, original_path, canonical_path, quarantine_path,
                filename, size_bytes, original_ctime, original_mtime,
                attributes, blake3_hash, quarantined_at, status, reason,
                restored_at, restore_error
            FROM quarantine_records
            ORDER BY quarantined_at DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let q_id: String = row.get(0)?;
            let orig: String = row.get(1)?;
            let canon: String = row.get(2)?;
            let q_path: String = row.get(3)?;
            let fname: String = row.get(4)?;
            let size: i64 = row.get(5)?;
            let ctime: i64 = row.get(6)?;
            let mtime: i64 = row.get(7)?;
            let attr: i64 = row.get(8)?;
            let hash: Option<String> = row.get(9)?;
            let q_at: i64 = row.get(10)?;
            let status: String = row.get(11)?;
            let reason: String = row.get(12)?;
            let restored_at: Option<i64> = row.get(13)?;
            let restore_err: Option<String> = row.get(14)?;

            Ok(QuarantineRecordDto {
                quarantine_id: q_id,
                original_path: orig,
                canonical_path: canon,
                quarantine_path: q_path,
                filename: fname,
                size_bytes: size as u64,
                original_ctime: ctime as u64,
                original_mtime: mtime as u64,
                attributes: attr as u32,
                blake3_hash: hash,
                quarantined_at_epoch_ms: q_at as u64,
                status,
                reason,
                restored_at_epoch_ms: restored_at.map(|t| t as u64),
                restore_error: restore_err,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_child = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_child)?;
        } else {
            fs::copy(entry.path(), &dest_child)?;
        }
    }
    Ok(())
}

fn count_dir_entries(dir: &Path) -> (u64, u64) {
    let mut files = 0u64;
    let mut bytes = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    let (sub_files, sub_bytes) = count_dir_entries(&entry.path());
                    files += sub_files;
                    bytes += sub_bytes;
                } else {
                    files += 1;
                    bytes += meta.len();
                }
            }
        }
    }
    (files, bytes)
}

fn copy_and_hash_file(source: &Path, dest: &Path) -> std::io::Result<(String, u64)> {
    use std::io::{Read, Write};
    let mut src = fs::File::open(source)?;
    let mut dst = fs::File::create(dest)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total_bytes = 0u64;

    loop {
        let n = src.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        dst.write_all(&buffer[..n])?;
        total_bytes += n as u64;
    }
    dst.flush()?;
    Ok((hasher.finalize().to_hex().to_string(), total_bytes))
}

pub fn verified_quarantine_move(
    source: &Path,
    destination: &Path,
    is_dir: bool,
    expected_hash: Option<&str>,
) -> Result<(), String> {
    if destination.exists() {
        return Err(format!(
            "Quarantine destination collision: '{}' already exists in vault",
            destination.display()
        ));
    }

    // Pre-move cryptographic verification: if expected_hash is provided, verify upfront
    if !is_dir {
        if let Some(expected) = expected_hash {
            let (actual_hash, _, _) = compute_file_blake3_streaming(source)
                .map_err(|e| format!("Failed to compute source hash for verification: {:?}", e))?;
            if actual_hash != expected {
                return Err(format!(
                    "Pre-move cryptographic verification failed: expected {}, actual {}",
                    expected, actual_hash
                ));
            }
        }
    }

    // Fast-path: Same filesystem volume atomic rename
    if fs::rename(source, destination).is_ok() {
        return Ok(());
    }

    // Fallback: Cross-volume safe copy & verification
    if is_dir {
        let staging_dir = destination.with_extension(format!("tmp_stage_{}", std::process::id()));
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }

        if let Err(e) = copy_dir_recursive(source, &staging_dir) {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(format!("Failed to copy directory to quarantine staging: {}", e));
        }

        let (src_files, src_bytes) = count_dir_entries(source);
        let (stage_files, stage_bytes) = count_dir_entries(&staging_dir);
        if src_files != stage_files || src_bytes != stage_bytes {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err("Cross-volume directory copy verification failed: file count or size mismatch".to_string());
        }

        if let Err(e) = fs::rename(&staging_dir, destination) {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(format!("Failed to finalize quarantine directory move: {}", e));
        }

        if let Err(e) = fs::remove_dir_all(source) {
            return Err(format!("Quarantine copied successfully, but failed to remove original source directory: {}", e));
        }

        Ok(())
    } else {
        let staging_file = destination.with_extension(format!("tmp_stage_{}", std::process::id()));
        if staging_file.exists() {
            let _ = fs::remove_file(&staging_file);
        }

        let pre_meta = fs::metadata(source).map_err(|e| format!("Failed to read source metadata: {}", e))?;
        let pre_size = pre_meta.len();
        let pre_mtime = pre_meta.modified().ok();

        let (dest_hash, copied_bytes) = match copy_and_hash_file(source, &staging_file) {
            Ok(res) => res,
            Err(e) => {
                let _ = fs::remove_file(&staging_file);
                return Err(format!("Cross-volume copy failed: {}", e));
            }
        };

        let post_meta = fs::metadata(source).map_err(|e| format!("Failed to re-verify source metadata: {}", e))?;
        if post_meta.len() != pre_size || (pre_mtime.is_some() && post_meta.modified().ok() != pre_mtime) {
            let _ = fs::remove_file(&staging_file);
            return Err("Aborting quarantine: Source file was modified concurrently during copy".to_string());
        }

        if copied_bytes != pre_size {
            let _ = fs::remove_file(&staging_file);
            return Err(format!("Cross-volume copy byte count mismatch: expected {}, copied {}", pre_size, copied_bytes));
        }

        if let Some(expected) = expected_hash {
            if dest_hash != expected {
                let _ = fs::remove_file(&staging_file);
                return Err(format!("Cross-volume cryptographic verification failed: expected {}, actual {}", expected, dest_hash));
            }
        }

        if let Err(e) = fs::rename(&staging_file, destination) {
            let _ = fs::remove_file(&staging_file);
            return Err(format!("Failed to finalize quarantine vault file rename: {}", e));
        }

        if let Err(e) = fs::remove_file(source) {
            return Err(format!("Quarantine copied successfully, but failed to remove original file: {}", e));
        }

        Ok(())
    }
}
