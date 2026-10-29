use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::adapter::path::{is_descendant_of, normalize_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReparsePointPolicy {
    /// Never traverse into reparse points (symlinks, junctions). Record them as leaf entries only.
    NeverTraverse,
    /// Traverse only if target remains strictly within the approved root directory.
    TraverseWithinRootOnly,
}

impl Default for ReparsePointPolicy {
    fn default() -> Self {
        ReparsePointPolicy::NeverTraverse
    }
}

pub struct ScanFilter {
    root_path: PathBuf,
    reparse_policy: ReparsePointPolicy,
    custom_exclusions: Vec<String>,
    visited_directories: HashSet<(u32, u64)>, // (volume_serial, file_index)
}

impl ScanFilter {
    pub fn new(root_path: PathBuf, reparse_policy: ReparsePointPolicy, custom_exclusions: Vec<String>) -> Self {
        Self {
            root_path,
            reparse_policy,
            custom_exclusions,
            visited_directories: HashSet::new(),
        }
    }

    /// Check if path matches default OS or custom exclusion rules
    pub fn is_excluded(&self, path: &Path) -> bool {
        // Exclude DOS device names to prevent driver hangs
        if crate::adapter::path::is_dos_device_name(path) {
            return true;
        }

        let norm = normalize_path(path).to_lowercase();

        // Built-in protected system metadata and virtual directories
        if norm.contains(r"\$recycle.bin")
            || norm.contains(r"\system volume information")
            || norm.ends_with(r"\pagefile.sys")
            || norm.ends_with(r"\hiberfil.sys")
            || norm.ends_with(r"\swapfile.sys")
            || norm.ends_with(r"\dumpstack.log")
        {
            return true;
        }

        // Custom user exclusions
        for pattern in &self.custom_exclusions {
            let pat_lower = pattern.to_lowercase();
            if norm.contains(&pat_lower) {
                return true;
            }
        }

        false
    }

    /// Check reparse point policy: should we traverse into this directory reparse point?
    pub fn should_traverse_reparse_point(&self, target_path: &Path) -> bool {
        match self.reparse_policy {
            ReparsePointPolicy::NeverTraverse => false,
            ReparsePointPolicy::TraverseWithinRootOnly => {
                is_descendant_of(target_path, &self.root_path)
            }
        }
    }

    /// Track directory device/file ID to prevent infinite traversal cycles.
    /// Returns true if this is the first time seeing this directory, false if loop detected.
    pub fn mark_visited_or_loop(&mut self, volume_serial: u32, file_id: u64) -> bool {
        if volume_serial == 0 && file_id == 0 {
            // Cannot reliably identify, allow first-time traversal
            return true;
        }
        self.visited_directories.insert((volume_serial, file_id))
    }
}

