use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

use crate::adapter::path::to_long_path;

#[derive(Debug)]
pub enum HashingError {
    Io(std::io::Error),
    ModifiedDuringStream {
        initial_mtime: u64,
        current_mtime: u64,
    },
    SizeChangedDuringStream {
        initial_size: u64,
        current_size: u64,
    },
    NotFound,
    PermissionDenied,
}

impl From<std::io::Error> for HashingError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => HashingError::NotFound,
            std::io::ErrorKind::PermissionDenied => HashingError::PermissionDenied,
            _ => HashingError::Io(err),
        }
    }
}

/// Compute a streaming cryptographic BLAKE3 hash for a file using a bounded 64KB buffer.
/// Detects if the file is modified or truncated while streaming.
pub fn compute_file_blake3_streaming(path: &Path) -> Result<(String, u64, u64), HashingError> {
    let long_path = to_long_path(path);
    let mut file = File::open(&long_path)?;

    let meta = file.metadata()?;
    let initial_size = meta.len();
    let initial_mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536]; // 64KB bounded buffer
    let mut total_bytes_read = 0u64;

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total_bytes_read += n as u64;
    }

    // Re-verify metadata after streaming to guarantee file did not mutate during read
    let post_meta = file.metadata()?;
    let post_mtime = post_meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let post_size = post_meta.len();

    if post_size != initial_size || total_bytes_read != initial_size {
        return Err(HashingError::SizeChangedDuringStream {
            initial_size,
            current_size: post_size,
        });
    }

    if post_mtime != initial_mtime {
        return Err(HashingError::ModifiedDuringStream {
            initial_mtime,
            current_mtime: post_mtime,
        });
    }

    let hash_hex = hasher.finalize().to_hex().to_string();
    Ok((hash_hex, total_bytes_read, initial_mtime))
}

/// Compute a quick sparse/prefix hash (e.g., first 4KB) for early candidate divergence.
pub fn compute_prefix_hash(path: &Path, prefix_bytes: usize) -> Result<String, HashingError> {
    let long_path = to_long_path(path);
    let mut file = File::open(&long_path)?;

    let mut buffer = vec![0u8; prefix_bytes];
    let n = file.read(&mut buffer)?;

    let mut hasher = blake3::Hasher::new();
    hasher.update(&buffer[..n]);
    Ok(hasher.finalize().to_hex().to_string())
}

/// High-assurance byte-for-byte streaming comparison between two candidate files.
/// Guarantees 100% certainty that files are identical down to every single bit.
pub fn verify_byte_by_byte(path_a: &Path, path_b: &Path) -> Result<bool, HashingError> {
    let long_a = to_long_path(path_a);
    let long_b = to_long_path(path_b);

    let mut file_a = File::open(&long_a)?;
    let mut file_b = File::open(&long_b)?;

    let meta_a = file_a.metadata()?;
    let meta_b = file_b.metadata()?;

    if meta_a.len() != meta_b.len() {
        return Ok(false);
    }

    let mut buf_a = [0u8; 65536];
    let mut buf_b = [0u8; 65536];

    loop {
        let na = file_a.read(&mut buf_a)?;
        let nb = file_b.read(&mut buf_b)?;

        if na != nb {
            return Ok(false);
        }
        if na == 0 {
            break;
        }
        if buf_a[..na] != buf_b[..nb] {
            return Ok(false);
        }
    }

    Ok(true)
}
