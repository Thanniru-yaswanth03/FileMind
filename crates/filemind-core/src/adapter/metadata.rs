use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

use crate::adapter::path::to_long_path;

#[derive(Debug, Clone)]
pub struct NativeFileMetadata {
    pub logical_size_bytes: u64,
    pub physical_size_bytes: u64,
    pub ctime_epoch_ms: u64,
    pub mtime_epoch_ms: u64,
    pub atime_epoch_ms: u64,
    pub is_hidden: bool,
    pub is_system: bool,
    pub is_read_only: bool,
    pub is_reparse_point: bool,
    pub is_directory: bool,
    pub is_locked: bool,
    pub ntfs_file_id: Option<String>,
    pub volume_serial: Option<u32>,
    pub number_of_links: u32,
}

#[cfg(windows)]
mod win32_api {
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, GetCompressedFileSizeW, GetFileInformationByHandle,
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SYSTEM, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    };

    pub fn to_wide_null(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub struct Win32FileHandle {
        pub handle: HANDLE,
    }

    impl Drop for Win32FileHandle {
        fn drop(&mut self) {
            if self.handle != INVALID_HANDLE_VALUE && self.handle != std::ptr::null_mut() {
                unsafe { CloseHandle(self.handle) };
            }
        }
    }

    pub fn open_for_attributes(path: &Path) -> Option<Win32FileHandle> {
        let wide = to_wide_null(path);
        // Using FILE_FLAG_BACKUP_SEMANTICS allows opening directories as well as files.
        // Using FILE_FLAG_OPEN_REPARSE_POINT opens the junction/symlink itself, not the target.
        // We request FILE_READ_ATTRIBUTES (0x0080) to minimize permission requirements.
        let flags = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                0x0080, // FILE_READ_ATTRIBUTES
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                std::ptr::null(),
                OPEN_EXISTING,
                flags,
                std::ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            None
        } else {
            Some(Win32FileHandle { handle })
        }
    }

    pub fn query_file_information(handle: HANDLE) -> Option<BY_HANDLE_FILE_INFORMATION> {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        let success = unsafe { GetFileInformationByHandle(handle, &mut info) };
        if success != 0 {
            Some(info)
        } else {
            None
        }
    }

    pub fn get_compressed_size(path: &Path) -> Option<u64> {
        let wide = to_wide_null(path);
        let mut high: u32 = 0;
        let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };
        if low == 0xFFFFFFFF && unsafe { windows_sys::Win32::Foundation::GetLastError() } != 0 {
            None
        } else {
            Some(((high as u64) << 32) | (low as u64))
        }
    }

    pub fn check_is_locked(path: &Path) -> bool {
        let wide = to_wide_null(path);
        // Attempt to open with exclusive share mode (dwShareMode = 0)
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ,
                0, // exclusive
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            // Error 32 = ERROR_SHARING_VIOLATION, Error 33 = ERROR_LOCK_VIOLATION
            let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
            err == 32 || err == 33
        } else {
            unsafe { CloseHandle(handle) };
            false
        }
    }

    pub const ATTR_READONLY: u32 = FILE_ATTRIBUTE_READONLY;
    pub const ATTR_HIDDEN: u32 = FILE_ATTRIBUTE_HIDDEN;
    pub const ATTR_SYSTEM: u32 = FILE_ATTRIBUTE_SYSTEM;
    pub const ATTR_REPARSE_POINT: u32 = FILE_ATTRIBUTE_REPARSE_POINT;
}

/// Convert a `SystemTime` to milliseconds since UNIX epoch
pub fn system_time_to_epoch_ms(time: SystemTime) -> u64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Format an epoch millisecond timestamp as an ISO-8601 string
pub fn epoch_ms_to_iso(epoch_ms: u64) -> String {
    // If 0, return fallback
    if epoch_ms == 0 {
        return "1970-01-01T00:00:00Z".to_string();
    }
    let total_secs = epoch_ms / 1000;
    let millis = epoch_ms % 1000;

    let days = total_secs / 86400;
    let rem_secs = total_secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    // Approximate date from days since 1970-01-01
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let days_in_months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1;
    for &dim in &days_in_months {
        if d < dim {
            break;
        }
        d -= dim;
        m += 1;
    }
    let day = d + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y, m, day, hours, minutes, seconds, millis
    )
}

/// Query native file metadata with deep Win32 inspection.
/// Calculates logical size, physical allocated size, attributes, and NTFS ID.
pub fn inspect_file(
    path: &Path,
    cluster_size_bytes: u64,
) -> Result<NativeFileMetadata, std::io::Error> {
    let long_path = to_long_path(path);

    // Standard std::fs::symlink_metadata to avoid traversing links
    let std_meta = std::fs::symlink_metadata(&long_path)?;
    let logical_size = std_meta.len();
    let is_dir = std_meta.is_dir();

    let ctime_ms = std_meta.created().map(system_time_to_epoch_ms).unwrap_or(0);
    let mtime_ms = std_meta
        .modified()
        .map(system_time_to_epoch_ms)
        .unwrap_or(0);
    let atime_ms = std_meta
        .accessed()
        .map(system_time_to_epoch_ms)
        .unwrap_or(0);

    let raw_attributes = std_meta.file_attributes();
    let is_hidden = (raw_attributes & win32_api::ATTR_HIDDEN) != 0;
    let is_system = (raw_attributes & win32_api::ATTR_SYSTEM) != 0;
    let is_read_only = (raw_attributes & win32_api::ATTR_READONLY) != 0;
    let is_reparse_point = (raw_attributes & win32_api::ATTR_REPARSE_POINT) != 0;

    let mut physical_size = logical_size;
    let mut ntfs_file_id = None;
    let mut volume_serial = None;
    let mut number_of_links = 1;

    // Deep Win32 inspection using open handle
    if let Some(file_handle) = win32_api::open_for_attributes(&long_path) {
        if let Some(info) = win32_api::query_file_information(file_handle.handle) {
            let high = info.nFileIndexHigh as u64;
            let low = info.nFileIndexLow as u64;
            let file_id_u64 = (high << 32) | low;
            ntfs_file_id = Some(format!(
                "{:08X}:{:016X}",
                info.dwVolumeSerialNumber, file_id_u64
            ));
            volume_serial = Some(info.dwVolumeSerialNumber);
            number_of_links = info.nNumberOfLinks;
        }
    }

    // Attempt to get physical size via GetCompressedFileSizeW
    if !is_dir {
        if let Some(compressed_size) = win32_api::get_compressed_size(&long_path) {
            physical_size = compressed_size;
        } else if cluster_size_bytes > 0 && logical_size > 0 {
            // Cluster alignment: align up to cluster boundary
            let rem = logical_size % cluster_size_bytes;
            physical_size = if rem == 0 {
                logical_size
            } else {
                logical_size + (cluster_size_bytes - rem)
            };
        }
    } else {
        physical_size = 0;
    }

    let is_locked = if !is_dir && !is_reparse_point && logical_size > 0 {
        win32_api::check_is_locked(&long_path)
    } else {
        false
    };

    Ok(NativeFileMetadata {
        logical_size_bytes: logical_size,
        physical_size_bytes: physical_size,
        ctime_epoch_ms: ctime_ms,
        mtime_epoch_ms: mtime_ms,
        atime_epoch_ms: atime_ms,
        is_hidden,
        is_system,
        is_read_only,
        is_reparse_point,
        is_directory: is_dir,
        is_locked,
        ntfs_file_id,
        volume_serial,
        number_of_links,
    })
}
