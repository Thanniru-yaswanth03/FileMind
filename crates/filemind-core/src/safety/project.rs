use std::path::{Path, PathBuf};
use crate::models::{SafetyClassification, EvidenceSource, TypedEvidenceItem};

#[derive(Debug, Clone)]
pub struct ProjectEvidence {
    pub classification: SafetyClassification,
    pub is_regenerable: bool,
    pub manifest_found: Option<String>,
    pub lockfile_found: Option<String>,
    pub git_repo_detected: bool,
    pub has_secrets: bool,
    pub explanation: String,
    pub evidence_items: Vec<TypedEvidenceItem>,
}

/// Identifies if a file is an untracked credential, private key, or secret
pub fn is_secret_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    if lower == ".env" || lower.starts_with(".env.") {
        return true;
    }
    if lower.contains("id_rsa") || lower.contains("id_ed25519") || lower.contains("id_ecdsa") {
        return true;
    }
    if lower.ends_with(".pem") || lower.ends_with(".key") || lower.ends_with(".pkcs12") || lower.ends_with(".pfx") || lower.ends_with(".p12") || lower.ends_with(".kdbx") {
        return true;
    }
    if lower == "credentials.json" || lower.starts_with("service-account") && lower.ends_with(".json") {
        return true;
    }
    false
}

/// Identifies if a file is source code or project configuration
pub fn is_source_code_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "c" | "cpp" | "h" | "hpp"
            | "cs" | "java" | "kt" | "swift" | "sql" | "sh" | "ps1" | "toml" | "yaml" | "yml" => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Looks for a Git repository (.git directory) by traversing upwards from the given path
pub fn find_git_repository(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent()
    } else {
        Some(start_path)
    };

    while let Some(dir) = current {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            return Some(git_dir);
        }
        current = dir.parent();
    }
    None
}

/// Inspects parent and ancestor directories for package manifests and corresponding lockfiles
pub fn inspect_package_manager_ecosystem(candidate_dir: &Path) -> (Option<String>, Option<String>) {
    let parent = match candidate_dir.parent() {
        Some(p) => p,
        None => return (None, None),
    };

    // Check Cargo (Rust)
    let cargo_toml = parent.join("Cargo.toml");
    let cargo_lock = parent.join("Cargo.lock");
    if cargo_toml.exists() {
        let lock = if cargo_lock.exists() { Some("Cargo.lock".to_string()) } else { None };
        return (Some("Cargo.toml".to_string()), lock);
    }

    // Check Node / JavaScript
    let package_json = parent.join("package.json");
    if package_json.exists() {
        let lock = if parent.join("package-lock.json").exists() {
            Some("package-lock.json".to_string())
        } else if parent.join("pnpm-lock.yaml").exists() {
            Some("pnpm-lock.yaml".to_string())
        } else if parent.join("yarn.lock").exists() {
            Some("yarn.lock".to_string())
        } else if parent.join("bun.lockb").exists() {
            Some("bun.lockb".to_string())
        } else {
            None
        };
        return (Some("package.json".to_string()), lock);
    }

    // Check Python (pyproject.toml, Pipfile, requirements.txt)
    let pyproject = parent.join("pyproject.toml");
    if pyproject.exists() {
        let lock = if parent.join("poetry.lock").exists() {
            Some("poetry.lock".to_string())
        } else if parent.join("Pipfile.lock").exists() {
            Some("Pipfile.lock".to_string())
        } else {
            None
        };
        return (Some("pyproject.toml".to_string()), lock);
    }

    let requirements = parent.join("requirements.txt");
    if requirements.exists() {
        return (Some("requirements.txt".to_string()), None);
    }

    // Check Go
    let go_mod = parent.join("go.mod");
    if go_mod.exists() {
        let lock = if parent.join("go.sum").exists() {
            Some("go.sum".to_string())
        } else {
            None
        };
        return (Some("go.mod".to_string()), lock);
    }

    // Check Gradle / Maven
    let pom_xml = parent.join("pom.xml");
    if pom_xml.exists() {
        return (Some("pom.xml".to_string()), None);
    }
    let build_gradle = parent.join("build.gradle");
    if build_gradle.exists() || parent.join("build.gradle.kts").exists() {
        return (Some("build.gradle".to_string()), None);
    }

    (None, None)
}

/// Evaluates developer project safety, analyzing manifests, lockfiles, git tracking, and secrets
pub fn evaluate_project_dependencies(path: &Path) -> ProjectEvidence {
    let mut evidence_items = Vec::new();
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // 1. Secret / Credential Inspection
    if is_secret_file(filename) {
        evidence_items.push(TypedEvidenceItem {
            source: EvidenceSource::DeveloperManifest,
            claim: format!("Filename '{}' matches known credential, private key, or secret pattern", filename),
            supports_safety: false,
            verification_status: "Verified".to_string(),
            details: "Secrets and credentials must never be automatically or carelessly deleted".to_string(),
        });

        return ProjectEvidence {
            classification: SafetyClassification::ProtectedAndBlocked,
            is_regenerable: false,
            manifest_found: None,
            lockfile_found: None,
            git_repo_detected: false,
            has_secrets: true,
            explanation: "File is a sensitive credential, API key, or private key".to_string(),
            evidence_items,
        };
    }

    // 2. Git Directory Inspection
    if filename == ".git" || path.to_string_lossy().contains("\\.git\\") || path.to_string_lossy().contains("/.git/") {
        evidence_items.push(TypedEvidenceItem {
            source: EvidenceSource::GitRepository,
            claim: "Target is within the internal Git database (.git)".to_string(),
            supports_safety: false,
            verification_status: "Verified".to_string(),
            details: "Direct deletion of .git database destroys version control history and branches".to_string(),
        });

        return ProjectEvidence {
            classification: SafetyClassification::ProtectedAndBlocked,
            is_regenerable: false,
            manifest_found: None,
            lockfile_found: None,
            git_repo_detected: true,
            has_secrets: false,
            explanation: "Target is part of internal Git version control database".to_string(),
            evidence_items,
        };
    }

    // 3. Git Repo Presence for Context
    let git_repo = find_git_repository(path);
    let git_detected = git_repo.is_some();
    if git_detected {
        evidence_items.push(TypedEvidenceItem {
            source: EvidenceSource::GitRepository,
            claim: "Enclosing Git repository detected".to_string(),
            supports_safety: true,
            verification_status: "Verified".to_string(),
            details: format!("Repository root found at '{:?}'", git_repo.as_ref().unwrap().parent().unwrap_or(Path::new(""))),
        });
    }

    // 4. Source Code File Protection
    if is_source_code_file(path) {
        evidence_items.push(TypedEvidenceItem {
            source: EvidenceSource::DeveloperManifest,
            claim: "Target is primary source code or project configuration".to_string(),
            supports_safety: false,
            verification_status: "Verified".to_string(),
            details: "Source code cannot be regenerated from build manifests; requires manual review".to_string(),
        });

        return ProjectEvidence {
            classification: SafetyClassification::ReviewRequired,
            is_regenerable: false,
            manifest_found: None,
            lockfile_found: None,
            git_repo_detected: git_detected,
            has_secrets: false,
            explanation: "Source code or configuration file requires explicit user review".to_string(),
            evidence_items,
        };
    }

    // 5. Regenerable Build Artifact Directories
    let dir_name_lower = filename.to_lowercase();
    let is_candidate_dir = dir_name_lower == "target"
        || dir_name_lower == "node_modules"
        || dir_name_lower == "build"
        || dir_name_lower == "dist"
        || dir_name_lower == "out"
        || dir_name_lower == "__pycache__"
        || dir_name_lower == ".pytest_cache"
        || dir_name_lower == ".gradle"
        || dir_name_lower == ".next"
        || dir_name_lower == ".nuxt"
        || dir_name_lower == ".turbo";

    if is_candidate_dir && path.is_dir() {
        let (manifest, lockfile) = inspect_package_manager_ecosystem(path);

        if let Some(ref m) = manifest {
            evidence_items.push(TypedEvidenceItem {
                source: EvidenceSource::DeveloperManifest,
                claim: format!("Parent package manifest '{}' confirmed", m),
                supports_safety: true,
                verification_status: "Verified".to_string(),
                details: "Establishes that the directory belongs to an active, identifiable software project".to_string(),
            });
        } else {
            evidence_items.push(TypedEvidenceItem {
                source: EvidenceSource::DeveloperManifest,
                claim: "No parent package manifest discovered in enclosing directory".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Cannot verify project origin; could be user files named 'build' or 'target'".to_string(),
            });
        }

        if let Some(ref l) = lockfile {
            evidence_items.push(TypedEvidenceItem {
                source: EvidenceSource::DeveloperManifest,
                claim: format!("Deterministic lockfile '{}' confirmed", l),
                supports_safety: true,
                verification_status: "Verified".to_string(),
                details: "Ensures that rebuilding the project produces reproducible dependencies".to_string(),
            });
        } else if manifest.is_some() {
            evidence_items.push(TypedEvidenceItem {
                source: EvidenceSource::DeveloperManifest,
                claim: "Lockfile missing: project dependencies are not strictly pinned".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Re-installing dependencies might produce non-deterministic versions".to_string(),
            });
        }

        // Quick check inside the candidate folder for misplaced secrets (.env)
        let env_file = path.join(".env");
        if env_file.exists() {
            evidence_items.push(TypedEvidenceItem {
                source: EvidenceSource::DeveloperManifest,
                claim: "Misplaced secret '.env' file detected inside candidate build directory".to_string(),
                supports_safety: false,
                verification_status: "Verified".to_string(),
                details: "Contains environment secret file; cannot be safely cleaned without backup".to_string(),
            });

            return ProjectEvidence {
                classification: SafetyClassification::HighRiskOrUnknown,
                is_regenerable: false,
                manifest_found: manifest,
                lockfile_found: lockfile,
                git_repo_detected: git_detected,
                has_secrets: true,
                explanation: "Build directory contains misplaced sensitive '.env' file".to_string(),
                evidence_items,
            };
        }

        // Evaluate regenerability:
        // Requires: manifest present + lockfile present
        if manifest.is_some() && lockfile.is_some() {
            return ProjectEvidence {
                classification: SafetyClassification::LowRiskCandidate,
                is_regenerable: true,
                manifest_found: manifest,
                lockfile_found: lockfile,
                git_repo_detected: git_detected,
                has_secrets: false,
                explanation: format!(
                    "Regenerable build directory verified by parent manifest and deterministic lockfile"
                ),
                evidence_items,
            };
        } else if manifest.is_some() {
            // Manifest exists but lockfile missing -> Demote to HighRiskOrUnknown (fail closed)
            return ProjectEvidence {
                classification: SafetyClassification::HighRiskOrUnknown,
                is_regenerable: false,
                manifest_found: manifest,
                lockfile_found: None,
                git_repo_detected: git_detected,
                has_secrets: false,
                explanation: "Parent manifest exists but lockfile is missing; rebuild cannot be verified as deterministic".to_string(),
                evidence_items,
            };
        } else {
            // No manifest found -> Demote to ReviewRequired
            return ProjectEvidence {
                classification: SafetyClassification::ReviewRequired,
                is_regenerable: false,
                manifest_found: None,
                lockfile_found: None,
                git_repo_detected: git_detected,
                has_secrets: false,
                explanation: "Directory name matches build directory, but no project manifest exists to prove ownership".to_string(),
                evidence_items,
            };
        }
    }

    // Default for personal files, archives, documents
    ProjectEvidence {
        classification: SafetyClassification::ReviewRequired,
        is_regenerable: false,
        manifest_found: None,
        lockfile_found: None,
        git_repo_detected: git_detected,
        has_secrets: false,
        explanation: "Standard user file or project item; requires user review".to_string(),
        evidence_items,
    }
}

