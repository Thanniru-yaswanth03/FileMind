use crate::adapter::path::{
    is_dos_device_name, is_has_alternate_data_stream, normalize_path, strip_verbatim_prefix,
};
use crate::models::{EvidenceSource, SafetyClassification, TypedEvidenceItem};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct WindowsRuleMatch {
    pub rule_id: &'static str,
    pub description: &'static str,
    pub classification: SafetyClassification,
    pub block_reason: &'static str,
    pub evidence: TypedEvidenceItem,
}

/// Evaluates if a given path violates Windows OS safety rules.
/// Returns a rule match if the path is protected or blocked by Windows OS policy.
pub fn evaluate_windows_protection(path: &Path) -> Option<WindowsRuleMatch> {
    let raw_str = path.to_string_lossy();

    // Check for null byte injection
    if raw_str.contains('\0') {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-NULL-BYTE-BLOCK",
            description: "Null byte '\\0' injection detected in path",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Paths containing embedded null bytes are strictly rejected to prevent string truncation attacks",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Path contains embedded null byte".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Null bytes can truncate paths in Win32 system APIs".to_string(),
            },
        });
    }

    // Check for DOS reserved device names (CON, PRN, AUX, NUL, COM1-9, LPT1-9)
    if is_dos_device_name(path) {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-DOS-DEVICE-BLOCK",
            description: "DOS reserved device name detected (CON, PRN, AUX, NUL, COM, LPT)",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Target references a legacy Windows DOS device, which can hang OS I/O or cause kernel faults",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target matches DOS reserved device name".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: format!("Target '{}' is a reserved Win32 device name", raw_str),
            },
        });
    }

    // Check for NTFS Alternate Data Stream (ADS)
    if is_has_alternate_data_stream(path) {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-NTFS-ADS-BLOCK",
            description: "NTFS Alternate Data Stream (ADS) syntax detected",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Alternate Data Streams can conceal secondary payloads or evade extension safeguards",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target contains ':' stream separator indicative of NTFS ADS".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: format!("Target '{}' references an alternate stream", raw_str),
            },
        });
    }

    let clean_str = strip_verbatim_prefix(&raw_str);
    let normalized = normalize_path(Path::new(&clean_str));
    let norm_lower = normalized.to_lowercase().replace('/', "\\");

    // Check for path traversal attempts
    if clean_str.contains("..") {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-TRAVERSAL-BLOCK",
            description: "Path traversal character sequence '..' detected",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Paths containing traversal sequences are strictly prohibited to prevent jailbreaking roots",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Path contains '..' parent directory traversal".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: format!("Target '{}' contains forbidden traversal components", clean_str),
            },
        });
    }

    // Windows Root and System directories (including 8.3 short name WINDOW~1 and Volume GUID syntax)
    let is_windows_dir = norm_lower.contains(":\\windows")
        || norm_lower.starts_with("c:\\windows")
        || norm_lower.contains(":\\window~")
        || norm_lower.starts_with("c:\\window~")
        || (norm_lower.contains("volume{")
            && (norm_lower.contains("\\windows") || norm_lower.contains("\\window~")));

    if is_windows_dir {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-SYS-ROOT",
            description: "Core Windows OS directory protection",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Modification or deletion of Windows OS files can render the operating system unbootable",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target is within the Windows installation directory".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: format!("Target matches Windows root tree: '{}'", normalized),
            },
        });
    }

    // Program Files directories (including 8.3 short names PROGRA~1, PROGRA~2 and Volume GUID syntax)
    let is_program_files = norm_lower.contains(":\\program files")
        || norm_lower.contains(":\\program files (x86)")
        || norm_lower.contains(":\\progra~")
        || norm_lower.starts_with("c:\\progra~")
        || (norm_lower.contains("volume{")
            && (norm_lower.contains("\\program files") || norm_lower.contains("\\progra~")));

    if is_program_files {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-PROGRAM-FILES",
            description: "Installed Application Binary Directory",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Application installation directories must be managed through official uninstallers or package managers",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target is located within Program Files".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Program Files contents must not be deleted through generic cleanup".to_string(),
            },
        });
    }

    // System Volume Information and Recycle Bin
    if norm_lower.contains("system volume information")
        || norm_lower.contains("$recycle.bin")
        || norm_lower.contains(":\\recovery")
    {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-VOL-METADATA",
            description: "Filesystem metadata, restore points, or Recycle Bin root",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "Volume metadata and system restore points are reserved for NTFS filesystem operations",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target is a volume metadata or system recovery directory".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Target path intersects with NTFS metadata or recovery data".to_string(),
            },
        });
    }

    // ProgramData Microsoft infrastructure
    if norm_lower.contains("programdata\\microsoft") {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-PROGRAMDATA-MS",
            description: "Windows Defender, Cryptographic, and System Service Data",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "ProgramData\\Microsoft houses active cryptographic keys, defender definitions, and service state",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target is within ProgramData\\Microsoft infrastructure".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "System services depend on this data for security and boot validation".to_string(),
            },
        });
    }

    // Critical AppData Microsoft Windows locations
    if norm_lower.contains("appdata\\local\\microsoft\\windows")
        || norm_lower.contains("appdata\\roaming\\microsoft\\windows")
    {
        return Some(WindowsRuleMatch {
            rule_id: "WIN-USER-OS-DATA",
            description: "Per-user Windows shell, start menu, and credential state",
            classification: SafetyClassification::ProtectedAndBlocked,
            block_reason: "User Windows profile directory contains explorer shortcuts, shell databases, and credential vaults",
            evidence: TypedEvidenceItem {
                source: EvidenceSource::WindowsPolicy,
                claim: "Target is within user Windows shell profile".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Crucial for current user session stability".to_string(),
            },
        });
    }

    // Critical system files (bootmgr, pagefile, hiberfil, swapfile, registry hives)
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    match filename.as_str() {
        "pagefile.sys" | "swapfile.sys" | "hiberfil.sys" | "bootmgr" | "bootnxt"
        | "bootsect.bak" | "bcd" => {
            return Some(WindowsRuleMatch {
                rule_id: "WIN-CRITICAL-BOOT-FILE",
                description: "Windows kernel virtual memory or bootloader configuration file",
                classification: SafetyClassification::ProtectedAndBlocked,
                block_reason:
                    "Kernel virtual memory and bootloader files are critical for machine operation",
                evidence: TypedEvidenceItem {
                    source: EvidenceSource::WindowsPolicy,
                    claim: format!(
                        "Target filename '{}' is an essential Windows kernel or boot asset",
                        filename
                    ),
                    supports_safety: false,
                    verification_status: "Verified".to_string(),
                    details: "Deletion would cause a BSOD or boot failure".to_string(),
                },
            });
        }
        "sam" | "security" | "software" | "system" | "default" | "ntuser.dat" | "usrclass.dat" => {
            return Some(WindowsRuleMatch {
                rule_id: "WIN-REGISTRY-HIVE",
                description: "Windows Registry Hive or User Profile Database",
                classification: SafetyClassification::ProtectedAndBlocked,
                block_reason: "Registry hives contain the fundamental configuration of the operating system and user profile",
                evidence: TypedEvidenceItem {
                    source: EvidenceSource::WindowsPolicy,
                    claim: format!("Target filename '{}' is a Windows Registry hive", filename),
                    supports_safety: false,
                    verification_status: "Verified".to_string(),
                    details: "Registry corruption causes immediate session breakdown".to_string(),
                },
            });
        }
        _ => {}
    }

    // Critical kernel driver extensions
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if ext_lower == "sys" || ext_lower == "inf" || ext_lower == "cat" {
            return Some(WindowsRuleMatch {
                rule_id: "WIN-KERNEL-DRIVER",
                description: "Kernel-mode driver, installation information, or security catalog",
                classification: SafetyClassification::ProtectedAndBlocked,
                block_reason:
                    "Device drivers and security catalogs are essential for hardware communication",
                evidence: TypedEvidenceItem {
                    source: EvidenceSource::WindowsPolicy,
                    claim: format!(
                        "Target extension '.{}' corresponds to device driver / security catalog",
                        ext_lower
                    ),
                    supports_safety: false,
                    verification_status: "Verified".to_string(),
                    details: "Drivers must not be removed by file cleanup tools".to_string(),
                },
            });
        }
    }

    None
}
