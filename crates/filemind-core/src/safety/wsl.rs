use std::path::Path;
use crate::adapter::path::{normalize_path, strip_verbatim_prefix};
use crate::models::{SafetyClassification, EvidenceSource, TypedEvidenceItem};

#[derive(Debug, Clone)]
pub struct WslRuleMatch {
    pub rule_id: &'static str,
    pub description: &'static str,
    pub classification: SafetyClassification,
    pub block_reason: &'static str,
    pub evidence: TypedEvidenceItem,
}

/// Evaluates if a given path corresponds to a Windows Subsystem for Linux (WSL)
/// distribution, virtual hard disk, or mounted Linux filesystem.
pub fn evaluate_wsl_protection(path: &Path) -> Option<WslRuleMatch> {
    let clean_str = strip_verbatim_prefix(&path.to_string_lossy());
    let normalized = normalize_path(Path::new(&clean_str));
    let norm_lower = normalized.to_lowercase().replace('/', "\\");

    // UNC WSL share paths (\\wsl$\ or \\wsl.localhost\)
    if norm_lower.starts_with("\\\\wsl$\\") || norm_lower.starts_with("\\\\wsl.localhost\\") || norm_lower == "\\\\wsl$" || norm_lower == "\\\\wsl.localhost" {
        return Some(WslRuleMatch {
            rule_id: "WSL-UNC-MOUNT-BLOCK",
            description: "Direct Windows Subsystem for Linux network mount",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Direct modification or deletion of live WSL mounted filesystem targets can cause corruption of Linux distribution metadata",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WslDetector,
                claim: "Target is a WSL UNC root or network share path".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: format!("Target '{}' addresses live WSL distribution via Plan9/9P filesystem provider", clean_str),
            },
        });
    }

    // Virtual Hard Disk (VHDX) detection
    let is_vhdx = path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("vhdx")).unwrap_or(false);
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();

    if filename == "ext4.vhdx" || (is_vhdx && (norm_lower.contains("wsl") || norm_lower.contains("canonical") || norm_lower.contains("docker"))) {
        return Some(WslRuleMatch {
            rule_id: "WSL-VHDX-DISK-IMAGE",
            description: "WSL distribution ext4 virtual hard disk image",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Virtual hard disk contains the entire Linux rootfs, user homes, and package database. Deletion results in complete loss of the Linux environment",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WslDetector,
                claim: "Target is an active WSL2 ext4 virtual disk image".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: format!("Target '{}' houses virtualized Linux distribution filesystem", clean_str),
            },
        });
    }

    // Known WSL distribution package storage directories
    let is_in_packages = norm_lower.contains("appdata\\local\\packages");
    if is_in_packages {
        let is_known_distro = norm_lower.contains("canonicalgrouplimited")
            || norm_lower.contains("ubuntugroup")
            || norm_lower.contains("thedebianproject")
            || norm_lower.contains("opensuse")
            || norm_lower.contains("kalilinux")
            || norm_lower.contains("archlinux")
            || norm_lower.contains("alpinelinux")
            || norm_lower.contains("wsl");

        if is_known_distro {
            return Some(WslRuleMatch {
                rule_id: "WSL-DISTRO-STORAGE",
                description: "WSL UWP Package container storage for Linux distribution",
                classification: SafetyClassification::ProtectedAndBlocked,
                block_reason: "WSL distribution packages must be unregistered or uninstalled via 'wsl.exe --unregister <Distro>', never manually deleted",
                evidence: TypedEvidenceItem {
                    source: EvidenceSource::WslDetector,
                    claim: "Target is within a recognized WSL distribution package storage container".to_string(),
                    supports_safety: false,
                    verification_status: "Verified".to_string(),
                    details: format!("Target matched WSL distro storage directory: '{}'", clean_str),
                },
            });
        }
    }

    // Docker Desktop WSL backend
    if norm_lower.contains("docker\\wsl") {
        return Some(WslRuleMatch {
            rule_id: "WSL-DOCKER-BACKEND",
            description: "Docker Desktop WSL integration storage",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Docker WSL backend houses container images, volumes, and runtime state",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WslDetector,
                claim: "Target is within Docker Desktop WSL data storage".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Direct deletion corrupts the Docker Engine WSL daemon".to_string(),
            },
        });
    }

    None
}

