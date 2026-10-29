use std::path::Path;

use crate::models::LocationTag;

/// Analyze location context of a file to provide intelligent, transparent recommendations.
pub struct LocationEvaluator;

impl LocationEvaluator {
    /// Determines whether the path is in a protected operating system or core application directory.
    pub fn is_protected_system_dir(path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.contains(r"\windows\")
            || lower.contains(r"\system32\")
            || lower.contains(r"\syswow64\")
            || lower.contains(r"\program files\")
            || lower.contains(r"\program files (x86)\")
            || lower.contains(r"\appdata\local\microsoft\")
            || lower.contains(r"\appdata\roaming\microsoft\")
            || lower.contains(r"\system volume information\")
            || lower.contains(r"\$recycle.bin\")
    }

    /// Evaluates if the file is part of an active software project or repository.
    pub fn is_project_member(path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.contains(r"\.git\")
            || lower.contains(r"\node_modules\")
            || lower.contains(r"\target\debug\")
            || lower.contains(r"\target\release\")
            || lower.contains(r"\src\")
            || lower.contains(r"\packages\")
            || lower.contains(r"\vendor\")
    }

    /// Checks if the file resides in an archive, backup, temp, or secondary download folder.
    pub fn is_backup_or_secondary(path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.contains(r"\backup")
            || lower.contains(r"\backups")
            || lower.contains(r"\old")
            || lower.contains(r"\archive")
            || lower.contains(r"\copy")
            || lower.contains(r"\downloads\")
            || lower.contains(r"\temp\")
            || lower.contains(r"\tmp\")
    }

    /// Checks if the file resides in a primary user documents or workspace folder.
    pub fn is_user_preferred_workspace(path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.contains(r"\documents\")
            || lower.contains(r"\desktop\")
            || lower.contains(r"\workspace\")
            || lower.contains(r"\projects\")
            || lower.contains(r"\vibecode\")
    }

    /// Assigns location tags for a file path.
    pub fn assign_tags(path: &str) -> Vec<LocationTag> {
        let mut tags = Vec::new();

        if Self::is_protected_system_dir(path) {
            tags.push(LocationTag::ProtectedSystem);
        }
        if Self::is_project_member(path) {
            tags.push(LocationTag::ProjectMember);
        }
        if Self::is_backup_or_secondary(path) {
            tags.push(LocationTag::BackupLocation);
        }
        if Self::is_user_preferred_workspace(path) {
            tags.push(LocationTag::UserPreferred);
        }

        tags
    }

    /// Compute relative score and recommendation rationale for a list of candidate copies.
    /// Higher score = safer and more desirable to retain as the primary copy.
    pub fn rank_copies(paths: &[&str]) -> (usize, String) {
        if paths.is_empty() {
            return (0, "No files in group".to_string());
        }

        let mut best_idx = 0;
        let mut highest_score = i32::MIN;
        let mut primary_reason = "Selected by location priority".to_string();

        for (idx, path) in paths.iter().enumerate() {
            let mut score: i32 = 0;
            let is_sys = Self::is_protected_system_dir(path);
            let is_proj = Self::is_project_member(path);
            let is_pref = Self::is_user_preferred_workspace(path);
            let is_sec = Self::is_backup_or_secondary(path);

            // Path depth: shallower paths generally indicate intentional primary placement
            let component_count = Path::new(path).components().count() as i32;
            score -= component_count * 2;

            if is_sys {
                // System files must never be recommended for deletion; if a duplicate exists, keep system copy
                score += 500;
            }
            if is_proj {
                score += 200;
            }
            if is_pref {
                score += 150;
            }
            if is_sec {
                score -= 100;
            }

            if score > highest_score {
                highest_score = score;
                best_idx = idx;

                primary_reason = if is_sys {
                    "Preserved in protected system directory to maintain application integrity.".to_string()
                } else if is_proj {
                    "Located in active development project repository.".to_string()
                } else if is_pref && is_sec {
                    "Located in primary workspace folder.".to_string()
                } else if is_sec {
                    "Located in temporary or download location.".to_string()
                } else {
                    "Standard primary location with clean path structure.".to_string()
                };
            }
        }

        (best_idx, primary_reason)
    }
}

