use std::path::Path;

/// Security inspector to exclude sensitive files, credentials, secrets, and private keys.
pub struct SecretFilter;

impl SecretFilter {
    /// Sensitive file extensions that should never be indexed into full-text search.
    pub const SENSITIVE_EXTENSIONS: &'static [&'static str] = &[
        "pem",
        "key",
        "pfx",
        "p12",
        "crt",
        "cer",
        "kdbx",
        "token",
        "id_rsa",
        "id_ed25519",
        "jks",
        "keystore",
        "pkcs12",
        "der",
        "p8",
    ];

    /// Sensitive filenames or substring patterns that indicate secrets or credentials.
    pub const SENSITIVE_FILENAMES: &'static [&'static str] = &[
        ".env",
        ".env.local",
        ".env.production",
        ".env.development",
        "credentials.json",
        "service-account.json",
        "secrets.yaml",
        "secrets.json",
        "id_rsa",
        "id_ed25519",
        "known_hosts",
        "authorized_keys",
        ".npmrc",
        ".dockercfg",
        "shadow",
        "passwd",
    ];

    /// Sensitive directories that should never have document content indexed.
    pub const SENSITIVE_DIRS: &'static [&'static str] = &[
        r"\.ssh\",
        r"\.gnupg\",
        r"\.aws\",
        r"\.azure\",
        r"\.kube\",
        r"\.git\",
        r"\appdata\local\google\chrome\user data\",
    ];

    /// Returns true if the file path is sensitive and must be excluded from content indexing.
    pub fn is_sensitive(path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let lower = path_str.to_lowercase();

        // Check sensitive directories
        for dir in Self::SENSITIVE_DIRS {
            if lower.contains(dir) {
                return true;
            }
        }

        // Check filename
        if let Some(file_name) = path.file_name() {
            let fname_lower = file_name.to_string_lossy().to_lowercase();

            for s_name in Self::SENSITIVE_FILENAMES {
                if fname_lower == *s_name || fname_lower.starts_with(".env.") {
                    return true;
                }
            }

            // Check sensitive extensions
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                for s_ext in Self::SENSITIVE_EXTENSIONS {
                    if ext_lower == *s_ext {
                        return true;
                    }
                }
            }
        }

        false
    }
}
