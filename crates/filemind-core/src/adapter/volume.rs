use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct VolumeInfo {
    pub root_path: String,
    pub drive_letter: String,
    pub volume_label: String,
    pub filesystem_type: String,
    pub cluster_size_bytes: u64,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub is_system_drive: bool,
    pub volume_serial: u32,
}

#[cfg(windows)]
mod win32_volume {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::MAX_PATH;
    use windows_sys::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDiskFreeSpaceW, GetLogicalDriveStringsW, GetVolumeInformationW,
    };

    pub fn to_wide_null(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn get_disk_space(root_str: &str) -> Option<(u64, u64, u64)> {
        let wide = to_wide_null(root_str);
        let mut free_available: u64 = 0;
        let mut total: u64 = 0;
        let mut total_free: u64 = 0;

        let ok = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_available as *mut u64 as *mut _,
                &mut total as *mut u64 as *mut _,
                &mut total_free as *mut u64 as *mut _,
            )
        };

        if ok != 0 {
            Some((total, total_free, free_available))
        } else {
            None
        }
    }

    pub fn get_cluster_size(root_str: &str) -> u64 {
        let wide = to_wide_null(root_str);
        let mut sectors_per_cluster: u32 = 0;
        let mut bytes_per_sector: u32 = 0;
        let mut number_of_free_clusters: u32 = 0;
        let mut total_number_of_clusters: u32 = 0;

        let ok = unsafe {
            GetDiskFreeSpaceW(
                wide.as_ptr(),
                &mut sectors_per_cluster,
                &mut bytes_per_sector,
                &mut number_of_free_clusters,
                &mut total_number_of_clusters,
            )
        };

        if ok != 0 {
            (sectors_per_cluster as u64) * (bytes_per_sector as u64)
        } else {
            4096 // Standard default cluster size for modern NTFS
        }
    }

    pub fn get_volume_metadata(root_str: &str) -> (String, String, u32) {
        let wide = to_wide_null(root_str);
        let mut volume_name = vec![0u16; (MAX_PATH + 1) as usize];
        let mut fs_name = vec![0u16; (MAX_PATH + 1) as usize];
        let mut serial: u32 = 0;
        let mut max_comp_len: u32 = 0;
        let mut flags: u32 = 0;

        let ok = unsafe {
            GetVolumeInformationW(
                wide.as_ptr(),
                volume_name.as_mut_ptr(),
                volume_name.len() as u32,
                &mut serial,
                &mut max_comp_len,
                &mut flags,
                fs_name.as_mut_ptr(),
                fs_name.len() as u32,
            )
        };

        if ok != 0 {
            let v_len = volume_name.iter().position(|&c| c == 0).unwrap_or(0);
            let f_len = fs_name.iter().position(|&c| c == 0).unwrap_or(0);
            let label = String::from_utf16_lossy(&volume_name[..v_len]);
            let fs = String::from_utf16_lossy(&fs_name[..f_len]);
            (label, fs, serial)
        } else {
            (String::new(), "NTFS".to_string(), 0)
        }
    }

    pub fn list_drives() -> Vec<String> {
        let mut buffer = vec![0u16; 512];
        let len = unsafe { GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr()) };
        if len == 0 || len > buffer.len() as u32 {
            // Fallback to testing common drive letters
            return vec!["C:\\".to_string()];
        }

        let mut drives = Vec::new();
        let mut start = 0;
        for i in 0..len as usize {
            if buffer[i] == 0 {
                if i > start {
                    let drive = String::from_utf16_lossy(&buffer[start..i]);
                    drives.push(drive);
                }
                start = i + 1;
            }
        }
        drives
    }
}

/// Extract drive root path from any path (e.g., "C:\Users\test" -> "C:\", "\\?\D:\data" -> "D:\")
pub fn get_drive_root(path: &Path) -> String {
    let clean = crate::adapter::path::strip_verbatim_prefix(&path.to_string_lossy()).replace('/', "\\");
    if clean.len() >= 2 && clean.chars().nth(1) == Some(':') {
        let letter = clean.chars().next().unwrap_or('C').to_ascii_uppercase();
        format!("{}:\\", letter)
    } else {
        "C:\\".to_string()
    }
}

/// Query complete volume information for a specific path or volume root
pub fn query_volume_info(path: &Path) -> Result<VolumeInfo, std::io::Error> {
    let root_str = get_drive_root(path);
    let drive_letter = root_str.chars().next().unwrap_or('C').to_ascii_uppercase().to_string();

    let sys_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let is_system_drive = sys_drive.to_uppercase().starts_with(&drive_letter);

    let (total, free, available) = win32_volume::get_disk_space(&root_str).unwrap_or((0, 0, 0));
    let cluster_size = win32_volume::get_cluster_size(&root_str);
    let (label, fs, serial) = win32_volume::get_volume_metadata(&root_str);

    let final_label = if label.is_empty() {
        if is_system_drive {
            "OS Disk".to_string()
        } else {
            format!("Local Disk ({}:)", drive_letter)
        }
    } else {
        label
    };

    Ok(VolumeInfo {
        root_path: root_str,
        drive_letter,
        volume_label: final_label,
        filesystem_type: if fs.is_empty() { "NTFS".to_string() } else { fs },
        cluster_size_bytes: cluster_size,
        total_bytes: total,
        free_bytes: free,
        available_bytes: available,
        is_system_drive,
        volume_serial: serial,
    })
}

/// Enumerate all active logical storage volumes on the Windows system
pub fn enumerate_all_volumes() -> Vec<VolumeInfo> {
    let drive_roots = win32_volume::list_drives();
    let mut volumes = Vec::new();

    for root in drive_roots {
        let p = PathBuf::from(&root);
        if let Ok(info) = query_volume_info(&p) {
            // Only include drives with non-zero total capacity
            if info.total_bytes > 0 {
                volumes.push(info);
            }
        }
    }

    if volumes.is_empty() {
        // Fallback default
        if let Ok(info) = query_volume_info(Path::new("C:\\")) {
            volumes.push(info);
        }
    }

    volumes
}
