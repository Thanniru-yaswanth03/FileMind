use std::path::{Component, Path, PathBuf};

/// Normalize a path string for Windows presentation and filesystem lookups.
/// Handles forward slashes, redundant separators, trailing slashes, trailing dots/spaces, and null bytes.
pub fn normalize_path(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\0', "");
    let converted = raw.replace('/', "\\");
    let stripped = strip_verbatim_prefix(&converted);
    let p = Path::new(&stripped);

    let mut parts: Vec<String> = Vec::new();
    let mut has_prefix = false;
    let mut prefix_str = String::new();

    for comp in p.components() {
        match comp {
            Component::Prefix(prefix) => {
                has_prefix = true;
                prefix_str = prefix.as_os_str().to_string_lossy().to_string();
            }
            Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(name) => {
                let s = name.to_string_lossy();
                let trimmed = s.trim_end_matches(|c| c == ' ' || c == '.').to_string();
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
            }
        }
    }

    let mut out = if has_prefix {
        if parts.is_empty() {
            format!("{}\\", prefix_str)
        } else {
            format!("{}\\{}", prefix_str, parts.join("\\"))
        }
    } else if parts.is_empty() {
        "\\".to_string()
    } else {
        parts.join("\\")
    };

    if out.len() == 2 && out.ends_with(':') {
        out.push('\\');
    } else if out.len() > 3 && out.ends_with('\\') {
        out.pop();
    }
    out
}

/// Prepends the Windows Extended-Length Path prefix `\\?\` if needed.
/// This enables accessing paths up to 32,767 characters without hitting MAX_PATH (260).
pub fn to_long_path(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();

    // If already has verbatim prefix, return as-is
    if s.starts_with(r"\\?\") || s.starts_with("//?/") {
        return path.to_path_buf();
    }

    // Handle UNC paths: \\server\share -> \\?\UNC\server\share
    if s.starts_with(r"\\") || s.starts_with("//") {
        let clean = &s[2..].replace('/', "\\");
        return PathBuf::from(format!(r"\\?\UNC\{}", clean));
    }

    // Handle absolute drive paths: C:\folder -> \\?\C:\folder
    if s.len() >= 2 && s.chars().nth(1) == Some(':') {
        let clean = s.replace('/', "\\");
        return PathBuf::from(format!(r"\\?\{}", clean));
    }

    path.to_path_buf()
}

/// Strip any verbatim or extended-length prefix (`\\?\`, `//?/`, `\\.\`, or `\\?\UNC\`) for safe display.
pub fn strip_verbatim_prefix(path_str: &str) -> String {
    let s = path_str.replace('/', "\\");
    if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", stripped)
    } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else if let Some(stripped) = s.strip_prefix(r"\\.\") {
        stripped.to_string()
    } else {
        path_str.to_string()
    }
}

/// Checks whether `candidate` is a descendant of `ancestor_root`.
/// Prevents traversal outside allowed roots or escaping into parent directories.
pub fn is_descendant_of(candidate: &Path, ancestor_root: &Path) -> bool {
    let norm_candidate = normalize_path(candidate).to_lowercase();
    let norm_ancestor = normalize_path(ancestor_root).to_lowercase();

    if norm_candidate == norm_ancestor {
        return true;
    }

    let ancestor_prefix = if norm_ancestor.ends_with('\\') {
        norm_ancestor
    } else {
        format!("{}\\", norm_ancestor)
    };

    norm_candidate.starts_with(&ancestor_prefix)
}

/// Verifies whether the path components contain suspicious parent directory traversals ("..")
pub fn is_safe_relative_path(path: &Path) -> bool {
    for comp in path.components() {
        if comp == Component::ParentDir {
            return false;
        }
    }
    true
}

/// Detects DOS reserved device names (CON, PRN, AUX, NUL, COM1-9, LPT1-9).
/// Opening or scanning these names on Windows can cause driver hangs or unexpected kernel device errors.
pub fn is_dos_device_name(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let filename = match path.file_name().and_then(|f| f.to_str()) {
        Some(name) => name,
        None => s.as_ref(),
    };

    // Extract the primary name stem before the first dot
    let stem = filename.split('.').next().unwrap_or("").trim();
    let upper = stem.to_ascii_uppercase();

    matches!(
        upper.as_str(),
        "CON" | "PRN" | "AUX" | "NUL"
            | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9"
            | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9"
    )
}

/// Detects whether a path targets an NTFS Alternate Data Stream (ADS).
/// Formats like `file.txt:hidden` or `folder:$DATA`.
pub fn is_has_alternate_data_stream(path: &Path) -> bool {
    let clean = strip_verbatim_prefix(&path.to_string_lossy()).replace('/', "\\");
    // Ignore volume GUID format: Volume{...}\
    if clean.to_ascii_lowercase().contains("volume{") {
        return false;
    }

    // If path has a drive letter like C:\, skip the drive letter colon at index 1
    let slice_to_check = if clean.len() >= 2 && clean.chars().nth(1) == Some(':') {
        &clean[2..]
    } else {
        &clean[..]
    };

    slice_to_check.contains(':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("C:/Windows/System32")), r"C:\Windows\System32");
        assert_eq!(normalize_path(Path::new("C:/")), r"C:\");
        assert_eq!(normalize_path(Path::new("C:")), r"C:\");
        assert_eq!(normalize_path(Path::new(r"\\?\C:\Users\test\")), r"C:\Users\test");
        assert_eq!(normalize_path(Path::new(r"\\?\UNC\server\share\")), r"\\server\share");
        assert_eq!(normalize_path(Path::new("C:/Windows /System32")), r"C:\Windows\System32");
        assert_eq!(normalize_path(Path::new("C:/Windows.")), r"C:\Windows");
    }

    #[test]
    fn test_to_long_path() {
        let p = Path::new(r"C:\very\long\path");
        let long_p = to_long_path(p);
        assert_eq!(long_p.to_string_lossy(), r"\\?\C:\very\long\path");

        let unc = Path::new(r"\\server\share\file.txt");
        let long_unc = to_long_path(unc);
        assert_eq!(long_unc.to_string_lossy(), r"\\?\UNC\server\share\file.txt");
    }

    #[test]
    fn test_is_descendant_of() {
        let root = Path::new(r"C:\Projects");
        assert!(is_descendant_of(Path::new(r"C:\Projects\file.rs"), root));
        assert!(is_descendant_of(Path::new(r"C:\Projects\subdir\file.rs"), root));
        assert!(!is_descendant_of(Path::new(r"C:\Windows\System32"), root));
        assert!(!is_descendant_of(Path::new(r"C:\ProjectsOther"), root));
    }
}
