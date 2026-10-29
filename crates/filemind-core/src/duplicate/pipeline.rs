use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::adapter::path::normalize_path;
use crate::duplicate::hashing::{
    compute_file_blake3_streaming, compute_prefix_hash, verify_byte_by_byte, HashingError,
};
use crate::duplicate::heuristics::LocationEvaluator;
use crate::models::{
    DuplicateConfidence, DuplicateFileDto, DuplicateFileStatus, DuplicateGroupDto,
    DuplicateScanRequest, DuplicateScanResponse, LocationTag,
};

#[derive(Debug, Clone)]
struct CandidateRecord {
    file_id: i64,
    directory_path: String,
    filename: String,
    size_bytes: u64,
    mtime: u64,
}

pub struct DuplicatePipeline;

impl DuplicatePipeline {
    /// Execute the staged duplicate detection pipeline.
    ///
    /// Stage 1: Candidate grouping by matching byte size.
    /// Stage 2: Streaming BLAKE3 cryptographic hash comparison with prefix pre-filter.
    /// Stage 3: Metadata re-verification, optional byte-for-byte check, and location heuristics.
    pub fn execute(
        conn: &Connection,
        request: &DuplicateScanRequest,
    ) -> Result<DuplicateScanResponse, rusqlite::Error> {
        let min_size = request.min_size_bytes.unwrap_or(1); // Exclude 0-byte files by default
        let max_groups = request.max_groups.unwrap_or(200);
        let verify_byte = request.verify_byte_by_byte.unwrap_or(false);

        // -------------------------------------------------------------------
        // STAGE 1: Candidate Grouping from Metadata Index
        // -------------------------------------------------------------------
        let mut candidates_by_size: HashMap<u64, Vec<CandidateRecord>> = HashMap::new();

        let sql = match &request.session_id {
            Some(_) => {
                "SELECT file_id, directory_path, filename, logical_size_bytes, mtime
                 FROM files
                 WHERE session_id = ?1 AND logical_size_bytes >= ?2 AND is_locked = 0
                 ORDER BY logical_size_bytes DESC"
            }
            None => {
                "SELECT file_id, directory_path, filename, logical_size_bytes, mtime
                 FROM files
                 WHERE logical_size_bytes >= ?1 AND is_locked = 0
                 ORDER BY logical_size_bytes DESC"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = match &request.session_id {
            Some(sid) => stmt.query(rusqlite::params![sid, min_size as i64])?,
            None => stmt.query(rusqlite::params![min_size as i64])?,
        };

        while let Some(row) = rows.next()? {
            let file_id: i64 = row.get(0)?;
            let directory_path: String = row.get(1)?;
            let filename: String = row.get(2)?;
            let size_bytes: i64 = row.get(3)?;
            let mtime: i64 = row.get(4)?;

            let size = size_bytes as u64;
            candidates_by_size
                .entry(size)
                .or_default()
                .push(CandidateRecord {
                    file_id,
                    directory_path,
                    filename,
                    size_bytes: size,
                    mtime: mtime as u64,
                });
        }

        // Filter to sizes with 2 or more files (potential duplicates)
        let candidate_groups: Vec<(u64, Vec<CandidateRecord>)> = candidates_by_size
            .into_iter()
            .filter(|(_size, list)| list.len() >= 2)
            .take(max_groups * 2)
            .collect();

        let unverified_candidates_count = candidate_groups.iter().map(|(_, list)| list.len()).sum();

        // -------------------------------------------------------------------
        // STAGE 2: Content Comparison via Streaming Cryptographic Hashing
        // -------------------------------------------------------------------
        let mut verified_groups: Vec<DuplicateGroupDto> = Vec::new();

        for (size_bytes, records) in candidate_groups {
            // Optional fast pre-filter: if files > 64KB, check 4KB prefix hash
            let mut prefix_buckets: HashMap<String, Vec<CandidateRecord>> = HashMap::new();

            for rec in records {
                let full_path = PathBuf::from(&rec.directory_path).join(&rec.filename);
                if !full_path.exists() {
                    continue;
                }

                if size_bytes > 65536 {
                    match compute_prefix_hash(&full_path, 4096) {
                        Ok(p_hash) => {
                            prefix_buckets.entry(p_hash).or_default().push(rec);
                        }
                        Err(_) => {
                            // Inaccessible or read error; skip candidate
                        }
                    }
                } else {
                    prefix_buckets
                        .entry("small_file".to_string())
                        .or_default()
                        .push(rec);
                }
            }

            // For candidates sharing the same prefix hash, compute full streaming BLAKE3 hash
            for (_prefix, potential_dupes) in prefix_buckets {
                if potential_dupes.len() < 2 {
                    continue;
                }

                let mut hash_buckets: HashMap<
                    String,
                    Vec<(CandidateRecord, String, DuplicateFileStatus)>,
                > = HashMap::new();

                for rec in potential_dupes {
                    let full_path = PathBuf::from(&rec.directory_path).join(&rec.filename);
                    match compute_file_blake3_streaming(&full_path) {
                        Ok((hash, _read_bytes, _mtime)) => {
                            hash_buckets.entry(hash.clone()).or_default().push((
                                rec,
                                hash,
                                DuplicateFileStatus::Ok,
                            ));
                        }
                        Err(HashingError::ModifiedDuringStream { .. }) => {
                            // File modified during scan; record as changed
                            hash_buckets
                                .entry("modified".to_string())
                                .or_default()
                                .push((rec, String::new(), DuplicateFileStatus::ChangedDuringHash));
                        }
                        Err(HashingError::NotFound) => {
                            // Disappeared
                        }
                        Err(HashingError::PermissionDenied) => {
                            // Inaccessible
                        }
                        Err(_) => {}
                    }
                }

                // -----------------------------------------------------------
                // STAGE 3: Verification & Contextual Recommendation
                // -----------------------------------------------------------
                for (hash, items) in hash_buckets {
                    if hash == "modified" || items.len() < 2 {
                        continue;
                    }

                    // Optional byte-for-byte verification
                    let mut confidence = DuplicateConfidence::CryptographicHashMatch;
                    let mut is_byte_verified = false;

                    if verify_byte && items.len() >= 2 {
                        let base_path =
                            PathBuf::from(&items[0].0.directory_path).join(&items[0].0.filename);
                        let mut all_match = true;

                        for other in items.iter().skip(1) {
                            let other_path =
                                PathBuf::from(&other.0.directory_path).join(&other.0.filename);
                            match verify_byte_by_byte(&base_path, &other_path) {
                                Ok(true) => {}
                                _ => {
                                    all_match = false;
                                    break;
                                }
                            }
                        }

                        if all_match {
                            confidence = DuplicateConfidence::ByteForByteVerified;
                            is_byte_verified = true;
                        }
                    }

                    let paths: Vec<String> = items
                        .iter()
                        .map(|(r, _, _)| {
                            normalize_path(&PathBuf::from(&r.directory_path).join(&r.filename))
                        })
                        .collect();

                    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
                    let (recommended_idx, primary_rationale) =
                        LocationEvaluator::rank_copies(&path_refs);

                    let mut file_dtos: Vec<DuplicateFileDto> = Vec::new();

                    for (idx, (rec, f_hash, status)) in items.into_iter().enumerate() {
                        let full_str = paths[idx].clone();
                        let is_rec = idx == recommended_idx;
                        let is_sys = LocationEvaluator::is_protected_system_dir(&full_str);
                        let mut tags = LocationEvaluator::assign_tags(&full_str);

                        if is_rec {
                            tags.insert(0, LocationTag::RecommendedKeep);
                        } else {
                            tags.push(LocationTag::RedundantCopy);
                        }

                        let rationale = if is_rec {
                            primary_rationale.clone()
                        } else if is_sys {
                            "Protected system path. Do not delete.".to_string()
                        } else {
                            "Redundant copy. Secondary location.".to_string()
                        };

                        let formatted_mtime = format_timestamp(rec.mtime);

                        file_dtos.push(DuplicateFileDto {
                            id: rec.file_id,
                            path: full_str,
                            filename: rec.filename,
                            directory_path: rec.directory_path,
                            size_bytes: rec.size_bytes,
                            mtime_epoch_ms: rec.mtime,
                            formatted_mtime,
                            blake3_hash: Some(f_hash),
                            status,
                            location_tags: tags,
                            is_recommended_keep: is_rec,
                            recommendation_rationale: rationale,
                            is_selected_for_retention: is_rec,
                            is_protected: is_sys,
                        });
                    }

                    let group_id = format!("grp_{}_{}", size_bytes, &hash[..8.min(hash.len())]);
                    let wasted_bytes = (file_dtos.len() as u64 - 1) * size_bytes;

                    let group_rationale = if is_byte_verified {
                        format!(
                            "100% byte-for-byte bitstream match verified across {} identical copies (BLAKE3: {}...)",
                            file_dtos.len(),
                            &hash[..12.min(hash.len())]
                        )
                    } else {
                        format!(
                            "Cryptographic BLAKE3 stream hash match across {} identical copies (Hash: {}...)",
                            file_dtos.len(),
                            &hash[..12.min(hash.len())]
                        )
                    };

                    verified_groups.push(DuplicateGroupDto {
                        group_id,
                        size_bytes,
                        file_count: file_dtos.len(),
                        wasted_bytes,
                        confidence,
                        hash: Some(hash),
                        group_rationale,
                        files: file_dtos,
                    });

                    if verified_groups.len() >= max_groups {
                        break;
                    }
                }
            }

            if verified_groups.len() >= max_groups {
                break;
            }
        }

        // Sort verified groups by wasted bytes descending (highest recoverable space first)
        verified_groups.sort_by(|a, b| b.wasted_bytes.cmp(&a.wasted_bytes));

        let total_wasted_bytes: u64 = verified_groups.iter().map(|g| g.wasted_bytes).sum();
        let total_duplicate_files: usize = verified_groups.iter().map(|g| g.file_count).sum();
        let verified_groups_count = verified_groups.len();

        Ok(DuplicateScanResponse {
            groups: verified_groups,
            total_duplicate_files,
            total_wasted_bytes,
            unverified_candidates_count,
            verified_groups_count,
        })
    }
}

fn format_timestamp(epoch_ms: u64) -> String {
    if epoch_ms == 0 {
        return "Unknown".to_string();
    }
    let secs = epoch_ms / 1000;
    // Format timestamp as YYYY-MM-DD HH:MM
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    // Approximation for display
    let approx_year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        approx_year,
        month.min(12),
        day.min(31),
        hours,
        minutes
    )
}
