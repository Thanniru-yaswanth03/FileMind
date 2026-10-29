use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::adapter::path::to_long_path;
use crate::search::security::SecretFilter;

pub const MAX_INDEXABLE_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB max document size
pub const MAX_EXTRACTED_CHARACTERS: usize = 100_000;      // 100K char limit

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExtractionStatus {
    Complete,
    Truncated,
    SensitiveExcluded,
    UnsupportedFormat,
    FileTooLarge,
    Failed(String),
}

pub struct ExtractionResult {
    pub text: String,
    pub status: ExtractionStatus,
    pub char_count: usize,
}

pub struct DocumentExtractor;

impl DocumentExtractor {
    /// Supported text and code extensions
    pub const TEXT_EXTENSIONS: &'static [&'static str] = &[
        "txt", "md", "markdown", "json", "yaml", "yml", "toml", "xml", "csv", "tsv", "log", "ini",
        "cfg", "rs", "ts", "tsx", "js", "jsx", "py", "c", "cpp", "h", "hpp", "go", "java", "sh",
        "ps1", "sql", "html", "htm", "css", "scss",
    ];

    /// Determines if a file extension is supported for content extraction.
    pub fn is_supported(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "pdf" {
                return true;
            }
            return Self::TEXT_EXTENSIONS.contains(&ext_str.as_str());
        }
        false
    }

    /// Safely extracts text content from a document without executing macros or external code.
    pub fn extract(path: &Path) -> ExtractionResult {
        // 1. Check if path is blacklisted as sensitive/credential
        if SecretFilter::is_sensitive(path) {
            return ExtractionResult {
                text: String::new(),
                status: ExtractionStatus::SensitiveExcluded,
                char_count: 0,
            };
        }

        // 2. Check extension support
        let ext = match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
            Some(e) => e,
            None => {
                return ExtractionResult {
                    text: String::new(),
                    status: ExtractionStatus::UnsupportedFormat,
                    char_count: 0,
                }
            }
        };

        if !Self::is_supported(path) {
            return ExtractionResult {
                text: String::new(),
                status: ExtractionStatus::UnsupportedFormat,
                char_count: 0,
            };
        }

        // 3. Check physical file size
        let long_path = to_long_path(path);
        let metadata = match std::fs::metadata(&long_path) {
            Ok(m) => m,
            Err(e) => {
                return ExtractionResult {
                    text: String::new(),
                    status: ExtractionStatus::Failed(format!("Failed to read metadata: {}", e)),
                    char_count: 0,
                }
            }
        };

        if metadata.len() > MAX_INDEXABLE_FILE_SIZE {
            return ExtractionResult {
                text: String::new(),
                status: ExtractionStatus::FileTooLarge,
                char_count: 0,
            };
        }

        // 4. Extract based on format
        if ext == "pdf" {
            Self::extract_pdf(&long_path)
        } else {
            Self::extract_text_file(&long_path)
        }
    }

    fn extract_text_file(path: &Path) -> ExtractionResult {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return ExtractionResult {
                    text: String::new(),
                    status: ExtractionStatus::Failed(format!("Open error: {}", e)),
                    char_count: 0,
                }
            }
        };

        let mut buffer = String::new();
        // Read up to limit + 1024 bytes to detect truncation
        let mut reader = (&mut file).take((MAX_EXTRACTED_CHARACTERS * 2) as u64);
        if let Err(e) = reader.read_to_string(&mut buffer) {
            return ExtractionResult {
                text: String::new(),
                status: ExtractionStatus::Failed(format!("UTF-8 read error: {}", e)),
                char_count: 0,
            };
        }

        // Clean null characters or control chars that may disrupt SQLite FTS tokenizer
        let sanitized: String = buffer
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect();

        if sanitized.len() > MAX_EXTRACTED_CHARACTERS {
            let truncated: String = sanitized.chars().take(MAX_EXTRACTED_CHARACTERS).collect();
            let count = truncated.len();
            ExtractionResult {
                text: truncated,
                status: ExtractionStatus::Truncated,
                char_count: count,
            }
        } else {
            let count = sanitized.len();
            ExtractionResult {
                text: sanitized,
                status: ExtractionStatus::Complete,
                char_count: count,
            }
        }
    }

    fn extract_pdf(path: &Path) -> ExtractionResult {
        // Load PDF document using lopdf (pure Rust, safe, no script execution)
        let doc = match lopdf::Document::load(path) {
            Ok(d) => d,
            Err(e) => {
                return ExtractionResult {
                    text: String::new(),
                    status: ExtractionStatus::Failed(format!("PDF parse error: {}", e)),
                    char_count: 0,
                }
            }
        };

        let mut extracted_text = String::new();
        let pages = doc.get_pages();

        for (_page_num, page_id) in pages {
            if let Ok(text) = doc.extract_text(&[page_id.0]) {
                extracted_text.push_str(&text);
                extracted_text.push('\n');

                if extracted_text.len() >= MAX_EXTRACTED_CHARACTERS {
                    break;
                }
            }
        }

        let sanitized: String = extracted_text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect();

        if sanitized.len() > MAX_EXTRACTED_CHARACTERS {
            let truncated: String = sanitized.chars().take(MAX_EXTRACTED_CHARACTERS).collect();
            let count = truncated.len();
            ExtractionResult {
                text: truncated,
                status: ExtractionStatus::Truncated,
                char_count: count,
            }
        } else {
            let count = sanitized.len();
            ExtractionResult {
                text: sanitized,
                status: ExtractionStatus::Complete,
                char_count: count,
            }
        }
    }
}

