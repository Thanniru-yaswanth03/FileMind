use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use crate::adapter::path::normalize_path;
use crate::search::extractor::{DocumentExtractor, ExtractionStatus};
use crate::models::{
    ContentIndexProgress, ContentIndexStatusResponse, ContentSearchQuery, ContentSearchResponse,
    ContentSearchResultItem,
};

pub struct FtsEngine;

impl FtsEngine {
    /// Incremental content indexer.
    /// Safely indexes supported document formats into the SQLite FTS5 index.
    /// Excludes credentials and sensitive files, and detects if file content changed.
    pub fn index_documents(
        conn: &mut Connection,
        session_id: Option<&str>,
        max_files: usize,
    ) -> Result<ContentIndexProgress> {
        let mut progress = ContentIndexProgress {
            indexed_documents: 0,
            total_documents: 0,
            current_file: String::new(),
            skipped_sensitive: 0,
            truncated_count: 0,
            failed_count: 0,
            is_indexing: true,
        };

        // Query candidate documents from files table
        let sql = match session_id {
            Some(_) => {
                "SELECT file_id, directory_path, filename, extension, logical_size_bytes, mtime
                 FROM files
                 WHERE session_id = ?1 AND logical_size_bytes > 0 AND logical_size_bytes <= 5242880"
            }
            None => {
                "SELECT file_id, directory_path, filename, extension, logical_size_bytes, mtime
                 FROM files
                 WHERE logical_size_bytes > 0 AND logical_size_bytes <= 5242880"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = match session_id {
            Some(s) => stmt.query(params![s])?,
            None => stmt.query([])?,
        };

        let mut candidate_list = Vec::new();
        while let Some(row) = rows.next()? {
            let file_id: i64 = row.get(0)?;
            let dir: String = row.get(1)?;
            let fname: String = row.get(2)?;
            let ext: String = row.get(3)?;
            let size: i64 = row.get(4)?;
            let mtime: i64 = row.get(5)?;

            let full = PathBuf::from(&dir).join(&fname);
            if DocumentExtractor::is_supported(&full) || crate::search::security::SecretFilter::is_sensitive(&full) {
                candidate_list.push((file_id, dir, fname, ext, size as u64, mtime as u64));
            }
        }

        progress.total_documents = candidate_list.len().min(max_files) as u64;

        let now_epoch_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        for (file_id, dir, fname, ext, size_bytes, mtime_ms) in candidate_list.into_iter().take(max_files) {
            let full_path = PathBuf::from(&dir).join(&fname);
            let full_norm = normalize_path(&full_path);
            progress.current_file = fname.clone();

            // Check if already indexed with matching mtime and size (incremental skip)
            let existing: Option<(i64, i64, String)> = conn
                .query_row(
                    "SELECT mtime_epoch_ms, size_bytes, status FROM document_content_index WHERE file_id = ?1",
                    params![file_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();

            if let Some((stored_mtime, stored_size, _stored_status)) = existing {
                if stored_mtime as u64 == mtime_ms && stored_size as u64 == size_bytes {
                    // Up-to-date; skip
                    progress.indexed_documents += 1;
                    continue;
                }
            }

            // Extract content
            let extraction = DocumentExtractor::extract(&full_path);

            match extraction.status {
                ExtractionStatus::SensitiveExcluded => {
                    progress.skipped_sensitive += 1;
                    conn.execute(
                        r#"INSERT OR REPLACE INTO document_content_index (
                            file_id, path, mtime_epoch_ms, size_bytes, indexed_at_epoch_ms, char_count, status
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'excluded_sensitive')"#,
                        params![file_id, full_norm, mtime_ms as i64, size_bytes as i64, now_epoch_ms, 0],
                    )?;
                }
                ExtractionStatus::Complete | ExtractionStatus::Truncated => {
                    if extraction.status == ExtractionStatus::Truncated {
                        progress.truncated_count += 1;
                    }

                    let status_str = if extraction.status == ExtractionStatus::Truncated {
                        "truncated"
                    } else {
                        "complete"
                    };

                    // Update FTS virtual table
                    conn.execute("DELETE FROM fts_documents WHERE file_id = ?1", params![file_id])?;
                    conn.execute(
                        "INSERT INTO fts_documents (file_id, path, filename, extension, content) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![file_id, full_norm, fname, ext, extraction.text],
                    )?;

                    // Record tracking status
                    conn.execute(
                        r#"INSERT OR REPLACE INTO document_content_index (
                            file_id, path, mtime_epoch_ms, size_bytes, indexed_at_epoch_ms, char_count, status
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                        params![
                            file_id,
                            full_norm,
                            mtime_ms as i64,
                            size_bytes as i64,
                            now_epoch_ms,
                            extraction.char_count as i64,
                            status_str
                        ],
                    )?;

                    progress.indexed_documents += 1;
                }
                ExtractionStatus::Failed(_) => {
                    progress.failed_count += 1;
                    conn.execute(
                        r#"INSERT OR REPLACE INTO document_content_index (
                            file_id, path, mtime_epoch_ms, size_bytes, indexed_at_epoch_ms, char_count, status
                        ) VALUES (?1, ?2, ?3, ?4, ?5, 0, 'failed')"#,
                        params![file_id, full_norm, mtime_ms as i64, size_bytes as i64, now_epoch_ms],
                    )?;
                }
                _ => {}
            }
        }

        progress.is_indexing = false;
        Ok(progress)
    }

    /// Full-text search with highlight snippets and staleness detection.
    pub fn search_content(conn: &Connection, query: &ContentSearchQuery) -> Result<ContentSearchResponse> {
        let start_time = Instant::now();
        let clean_query = query.query.trim();

        if clean_query.is_empty() {
            return Ok(ContentSearchResponse {
                total_matches: 0,
                results: Vec::new(),
                execution_time_ms: 0,
            });
        }

        // Prepare FTS query string (quote bare terms or match prefix)
        let fts_query = if clean_query.contains('"') || clean_query.contains('*') {
            clean_query.to_string()
        } else {
            // Match token prefix: e.g. "storage engine" -> "storage* engine*"
            clean_query
                .split_whitespace()
                .map(|token| format!("{}*", token))
                .collect::<Vec<_>>()
                .join(" ")
        };

        let limit = query.limit.unwrap_or(50).min(200);
        let offset = query.offset.unwrap_or(0);

        let sql = r#"
            SELECT f.file_id, f.path, f.filename, f.extension,
                   c.size_bytes, c.mtime_epoch_ms, c.status,
                   snippet(fts_documents, 4, '<mark>', '</mark>', '...', 28) AS match_snippet
            FROM fts_documents f
            JOIN document_content_index c ON f.file_id = c.file_id
            WHERE fts_documents MATCH ?1
            LIMIT ?2 OFFSET ?3
        "#;

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(params![fts_query, limit, offset])?;

        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let file_id: i64 = row.get(0)?;
            let path_str: String = row.get(1)?;
            let filename: String = row.get(2)?;
            let extension: String = row.get(3)?;
            let size_bytes: i64 = row.get(4)?;
            let mtime_ms: i64 = row.get(5)?;
            let status: String = row.get(6)?;
            let snippet: String = row.get(7)?;

            // Check if result is stale relative to physical file on disk
            let mut is_stale = false;
            let physical_path = PathBuf::from(&path_str);
            if let Ok(meta) = physical_path.metadata() {
                let current_size = meta.len();
                let current_mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                if current_size != size_bytes as u64 || (current_mtime as i64 - mtime_ms).abs() > 100 {
                    is_stale = true;
                }
            } else {
                is_stale = true; // File disappeared or inaccessible
            }

            results.push(ContentSearchResultItem {
                file_id,
                path: path_str,
                filename,
                extension,
                size_bytes: size_bytes as u64,
                mtime_epoch_ms: mtime_ms as u64,
                snippet,
                is_stale,
                status,
            });
        }

        let elapsed = start_time.elapsed().as_millis() as u64;
        let total_matches = results.len() as u64;

        Ok(ContentSearchResponse {
            total_matches,
            results,
            execution_time_ms: elapsed,
        })
    }

    /// Retrieve content index status metrics.
    pub fn get_status(conn: &Connection) -> Result<ContentIndexStatusResponse> {
        let (total_indexed, total_chars): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(char_count), 0) FROM document_content_index WHERE status IN ('complete', 'truncated')",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap_or((0, 0));

        let skipped_sensitive: i64 = conn.query_row(
            "SELECT COUNT(*) FROM document_content_index WHERE status = 'excluded_sensitive'",
            [],
            |r| r.get(0),
        ).unwrap_or(0);

        let truncated_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM document_content_index WHERE status = 'truncated'",
            [],
            |r| r.get(0),
        ).unwrap_or(0);

        Ok(ContentIndexStatusResponse {
            total_indexed_documents: total_indexed as u64,
            total_extracted_characters: total_chars as u64,
            skipped_sensitive_count: skipped_sensitive as u64,
            truncated_documents_count: truncated_count as u64,
            is_active: true,
        })
    }

    /// Clear and purge the document content index without affecting the base files index.
    pub fn clear(conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM fts_documents", [])?;
        tx.execute("DELETE FROM document_content_index", [])?;
        tx.commit()?;
        Ok(())
    }
}
