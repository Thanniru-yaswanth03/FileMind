use std::path::Path;
use std::process::Command;

use crate::adapter::path::{normalize_path, strip_verbatim_prefix};

/// Safely open Windows File Explorer with the specified item selected.
/// Controlled native action: Validates target exists, escapes arguments, and directly invokes explorer.exe.
pub fn open_containing_folder(target_path: &str) -> Result<(), String> {
    let clean_path = strip_verbatim_prefix(target_path);
    let p = Path::new(&clean_path);

    if !p.exists() {
        return Err(format!("Cannot open containing folder: Path does not exist '{}'", clean_path));
    }

    let norm_path = normalize_path(p);

    // Directly invoke explorer.exe with /select,"path" without cmd.exe or shell execution
    let status = Command::new("explorer.exe")
        .arg(format!("/select,{}", norm_path))
        .spawn()
        .map_err(|e| format!("Failed to launch Windows Explorer: {}", e))?;

    drop(status);
    Ok(())
}

