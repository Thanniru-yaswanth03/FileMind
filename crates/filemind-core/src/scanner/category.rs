use crate::models::{FileCategory, RiskTier};
use std::path::Path;

/// Categorizes a file based on its extension and directory context.
pub fn categorize_file(path: &Path, extension: &str) -> FileCategory {
    let ext_lower = extension.to_lowercase();
    let ext = ext_lower.as_str();

    // Check directory context first (e.g., node_modules, target, .venv)
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains(r"\node_modules\")
        || path_str.contains(r"\target\")
        || path_str.contains(r"\.gradle\")
        || path_str.contains(r"\.venv\")
        || path_str.contains(r"\.cache\")
        || path_str.contains(r"\obj\")
        || path_str.contains(r"\__pycache__\")
    {
        return FileCategory::Caches;
    }

    match ext {
        // Code and developer assets
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "c" | "cpp" | "h" | "hpp" | "cs" | "go"
        | "java" | "html" | "css" | "scss" | "sass" | "less" | "json" | "yaml" | "yml" | "toml"
        | "md" | "sql" | "sh" | "ps1" | "bat" | "cmd" | "xml" | "proto" | "graphql" | "vue"
        | "svelte" | "dart" | "kt" | "swift" | "rb" | "php" => FileCategory::Code,

        // Media (images, videos, audio)
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        | "psd" | "ai" | "raw" | "cr2" | "nef" | "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv"
        | "webm" | "m4v" | "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => {
            FileCategory::Media
        }

        // Compressed archives
        "zip" | "tar" | "gz" | "7z" | "rar" | "bz2" | "xz" | "iso" | "tgz" | "zst" | "cab"
        | "dmg" | "wim" | "pkg" => FileCategory::Archives,

        // Office & text documents
        "pdf" | "docx" | "doc" | "xlsx" | "xls" | "pptx" | "ppt" | "txt" | "csv" | "tsv"
        | "rtf" | "epub" | "odt" | "ods" | "odp" | "pages" | "numbers" | "key" => {
            FileCategory::Documents
        }

        // Executables and native binaries
        "exe" | "dll" | "sys" | "msi" | "com" | "scr" | "vbs" | "wsf" | "jar" => {
            FileCategory::Executables
        }

        // Disposable and cache extensions
        "tmp" | "temp" | "log" | "cache" | "bak" | "old" | "dmp" | "swp" | "lock" | "chk"
        | "part" | "crdownload" => FileCategory::Caches,

        _ => FileCategory::Other,
    }
}

/// Evaluates default safety risk tier and rationale for a file.
/// Enforces the architectural invariant: "Unknown is Unsafe".
pub fn evaluate_risk_tier(
    path: &Path,
    category: FileCategory,
    is_system: bool,
) -> (RiskTier, String) {
    let path_str = path.to_string_lossy().to_lowercase();

    // Danger: Operating system core directories or system files
    if is_system
        || path_str.contains(r"c:\windows")
        || path_str.contains(r"\system volume information")
        || path_str.contains(r"\$recycle.bin")
        || path_str.contains(r"\boot")
    {
        return (
            RiskTier::Danger,
            "Protected OS System File: Deletion risks unrecoverable Windows failure".to_string(),
        );
    }

    // Safe (Tier 1): Known build caches and disposable logs
    if path_str.contains(r"\target\debug\")
        || path_str.contains(r"\target\release\")
        || path_str.contains(r"\node_modules\")
        || path_str.contains(r"\.gradle\caches\")
        || path_str.contains(r"\__pycache__\")
        || category == FileCategory::Caches
    {
        return (
            RiskTier::Safe,
            "Disposable Cache: Verified build artifact or temporary cache".to_string(),
        );
    }

    // Caution (Tier 2): Executables, DLLs, and archives outside build directories
    if category == FileCategory::Executables || category == FileCategory::Archives {
        return (
            RiskTier::Caution,
            "Binary or Archive: Requires manual review to avoid deleting installed software"
                .to_string(),
        );
    }

    // High Risk (Tier 3): Source code and personal documents
    if category == FileCategory::Code
        || category == FileCategory::Documents
        || category == FileCategory::Media
    {
        return (
            RiskTier::High,
            "User Document or Code: Critical user asset. Never scheduled for auto-cleanup"
                .to_string(),
        );
    }

    // Default: "Unknown is Unsafe"
    (
        RiskTier::Caution,
        "Unknown File Type: Requires explicit confirmation before any cleanup".to_string(),
    )
}
