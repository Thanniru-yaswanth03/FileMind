use std::path::Path;
use walkdir::WalkDir;

use crate::models::{DeveloperArtifactDto, DeveloperProjectDto, RiskTier};
use crate::safety::project::{find_git_repository, inspect_package_manager_ecosystem, is_secret_file};

pub struct DeveloperWorkspaceScanner;

impl DeveloperWorkspaceScanner {
    /// Scan a root directory or search index for recognized developer projects.
    /// Traverses up to max_depth (e.g. 5) looking for project manifests.
    pub fn scan_workspace(root_path: &Path, max_depth: usize) -> Vec<DeveloperProjectDto> {
        let mut projects = Vec::new();
        if !root_path.exists() {
            return projects;
        }

        let mut visited_project_dirs = std::collections::HashSet::new();

        let walker = WalkDir::new(root_path)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                let name = entry.file_name().to_string_lossy();
                // Skip traversing into dependency / build directories when searching for project roots
                if entry.file_type().is_dir() {
                    if name == "node_modules"
                        || name == "target"
                        || name == ".git"
                        || name == ".venv"
                        || name == "venv"
                        || name == "$Recycle.Bin"
                        || name == "System Volume Information"
                    {
                        return false;
                    }
                }
                true
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let filename = entry.file_name().to_string_lossy();
            let parent_dir = match entry.path().parent() {
                Some(p) => p,
                None => continue,
            };

            let parent_canonical = parent_dir.to_string_lossy().to_string();
            if visited_project_dirs.contains(&parent_canonical) {
                continue;
            }

            // Detect ecosystem based on manifest
            if filename == "package.json" {
                if let Some(project) = Self::inspect_nodejs_project(parent_dir) {
                    visited_project_dirs.insert(parent_canonical);
                    projects.push(project);
                }
            } else if filename == "Cargo.toml" {
                if let Some(project) = Self::inspect_rust_project(parent_dir) {
                    visited_project_dirs.insert(parent_canonical);
                    projects.push(project);
                }
            } else if filename == "pyproject.toml" || filename == "Pipfile" || filename == "requirements.txt" {
                if let Some(project) = Self::inspect_python_project(parent_dir, &filename) {
                    visited_project_dirs.insert(parent_canonical);
                    projects.push(project);
                }
            } else if filename == "go.mod" {
                if let Some(project) = Self::inspect_go_project(parent_dir) {
                    visited_project_dirs.insert(parent_canonical);
                    projects.push(project);
                }
            }
        }

        projects
    }

    /// Inspect a Node.js project directory
    pub fn inspect_nodejs_project(dir: &Path) -> Option<DeveloperProjectDto> {
        let package_json = dir.join("package.json");
        if !package_json.exists() {
            return None;
        }

        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("NodeJS Project").to_string();
        let (manifest, lockfile) = inspect_package_manager_ecosystem(&dir.join("dummy_artifact"));
        let git_status = Self::check_git_status(dir);
        let is_clean = git_status == "clean";

        let mut artifacts = Vec::new();
        let mut total_size: u64 = 0;

        // 1. node_modules
        let node_modules = dir.join("node_modules");
        if node_modules.exists() {
            let size = Self::calculate_directory_size(&node_modules);
            total_size += size;
            let risk = if is_clean && lockfile.is_some() {
                RiskTier::Safe
            } else {
                RiskTier::Caution
            };

            artifacts.push(DeveloperArtifactDto {
                name: "node_modules".to_string(),
                relative_path: "node_modules".to_string(),
                full_path: node_modules.to_string_lossy().to_string(),
                size_bytes: size,
                category: "Dependencies".to_string(),
                is_regenerable: true,
                reversibility: "Quarantine Vault / npm install".to_string(),
                cleanup_procedure: "Stage to quarantine; regenerable via package manager install".to_string(),
                risk_tier: risk,
                risks_and_questions: if lockfile.is_none() {
                    "Missing lockfile (package-lock.json/pnpm-lock.yaml); re-installing may resolve newer dependency versions.".to_string()
                } else if !is_clean {
                    "Repository has uncommitted work or untracked changes. Exercise review before purge.".to_string()
                } else {
                    "Safe to purge. Fully restorable from Quarantine Vault or regenerable via npm/pnpm/yarn install.".to_string()
                },
            });
        }

        // 2. Build caches: .next, dist, build, .turbo
        for build_target in &[".next", "dist", "build", ".turbo", ".nuxt"] {
            let build_dir = dir.join(build_target);
            if build_dir.exists() {
                let size = Self::calculate_directory_size(&build_dir);
                total_size += size;
                artifacts.push(DeveloperArtifactDto {
                    name: build_target.to_string(),
                    relative_path: build_target.to_string(),
                    full_path: build_dir.to_string_lossy().to_string(),
                    size_bytes: size,
                    category: "Build Output".to_string(),
                    is_regenerable: true,
                    reversibility: "Quarantine Vault / npm run build".to_string(),
                    cleanup_procedure: "Stage to quarantine; regenerable via build script".to_string(),
                    risk_tier: if is_clean { RiskTier::Safe } else { RiskTier::Caution },
                    risks_and_questions: "Regenerable compilation artifact. Staged into per-volume vault.".to_string(),
                });
            }
        }

        Some(DeveloperProjectDto {
            id: format!("proj_node_{}", Self::hash_path(dir)),
            name,
            path: dir.to_string_lossy().to_string(),
            ecosystem: "Node.js".to_string(),
            manifest_file: manifest.unwrap_or_else(|| "package.json".to_string()),
            lockfile_file: lockfile,
            git_status,
            total_artifact_size_bytes: total_size,
            artifacts,
        })
    }

    /// Inspect a Rust project directory
    pub fn inspect_rust_project(dir: &Path) -> Option<DeveloperProjectDto> {
        let cargo_toml = dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            return None;
        }

        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("Rust Project").to_string();
        let cargo_lock = dir.join("Cargo.lock");
        let lockfile = if cargo_lock.exists() { Some("Cargo.lock".to_string()) } else { None };
        let git_status = Self::check_git_status(dir);
        let is_clean = git_status == "clean";

        let mut artifacts = Vec::new();
        let mut total_size: u64 = 0;

        // target/ directory
        let target_dir = dir.join("target");
        if target_dir.exists() {
            let size = Self::calculate_directory_size(&target_dir);
            total_size += size;
            let risk = if is_clean && lockfile.is_some() {
                RiskTier::Safe
            } else {
                RiskTier::Caution
            };

            artifacts.push(DeveloperArtifactDto {
                name: "target".to_string(),
                relative_path: "target".to_string(),
                full_path: target_dir.to_string_lossy().to_string(),
                size_bytes: size,
                category: "Build Output".to_string(),
                is_regenerable: true,
                reversibility: "Quarantine Vault / cargo build".to_string(),
                cleanup_procedure: "Stage to quarantine; regenerable via cargo build / cargo clean".to_string(),
                risk_tier: risk,
                risks_and_questions: if lockfile.is_none() {
                    "Missing Cargo.lock; cargo clean will require dependency re-resolution.".to_string()
                } else if !is_clean {
                    "Uncommitted work present in repository. Target contains compilation cache.".to_string()
                } else {
                    "Safe to purge. Fully regenerable with Cargo.".to_string()
                },
            });
        }

        Some(DeveloperProjectDto {
            id: format!("proj_rust_{}", Self::hash_path(dir)),
            name,
            path: dir.to_string_lossy().to_string(),
            ecosystem: "Rust".to_string(),
            manifest_file: "Cargo.toml".to_string(),
            lockfile_file: lockfile,
            git_status,
            total_artifact_size_bytes: total_size,
            artifacts,
        })
    }

    /// Inspect a Python project directory
    pub fn inspect_python_project(dir: &Path, manifest_name: &str) -> Option<DeveloperProjectDto> {
        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("Python Project").to_string();
        let git_status = Self::check_git_status(dir);
        let is_clean = git_status == "clean";

        let lockfile = if dir.join("poetry.lock").exists() {
            Some("poetry.lock".to_string())
        } else if dir.join("Pipfile.lock").exists() {
            Some("Pipfile.lock".to_string())
        } else {
            None
        };

        let mut artifacts = Vec::new();
        let mut total_size: u64 = 0;

        // Virtual environments: .venv, venv, env
        for venv_name in &[".venv", "venv", "env"] {
            let venv_dir = dir.join(venv_name);
            // Verify it's actually a virtual environment (contains pyvenv.cfg or Scripts/bin)
            if venv_dir.exists() && (venv_dir.join("pyvenv.cfg").exists() || venv_dir.join("Scripts").exists() || venv_dir.join("bin").exists()) {
                let size = Self::calculate_directory_size(&venv_dir);
                total_size += size;
                artifacts.push(DeveloperArtifactDto {
                    name: venv_name.to_string(),
                    relative_path: venv_name.to_string(),
                    full_path: venv_dir.to_string_lossy().to_string(),
                    size_bytes: size,
                    category: "Virtual Environment".to_string(),
                    is_regenerable: true,
                    reversibility: "Quarantine Vault / python -m venv".to_string(),
                    cleanup_procedure: "Stage to quarantine; regenerable via virtual environment creation".to_string(),
                    risk_tier: if is_clean { RiskTier::Safe } else { RiskTier::Caution },
                    risks_and_questions: "Virtual environment package cache. Reversible via quarantine.".to_string(),
                });
            }
        }

        // Caches: __pycache__, .pytest_cache, .mypy_cache
        for cache_name in &[".pytest_cache", ".mypy_cache", ".ruff_cache"] {
            let cache_dir = dir.join(cache_name);
            if cache_dir.exists() {
                let size = Self::calculate_directory_size(&cache_dir);
                total_size += size;
                artifacts.push(DeveloperArtifactDto {
                    name: cache_name.to_string(),
                    relative_path: cache_name.to_string(),
                    full_path: cache_dir.to_string_lossy().to_string(),
                    size_bytes: size,
                    category: "Cache".to_string(),
                    is_regenerable: true,
                    reversibility: "Quarantine Vault".to_string(),
                    cleanup_procedure: "Stage to quarantine".to_string(),
                    risk_tier: RiskTier::Safe,
                    risks_and_questions: "Ephemeral tool cache. Zero data loss.".to_string(),
                });
            }
        }

        Some(DeveloperProjectDto {
            id: format!("proj_py_{}", Self::hash_path(dir)),
            name,
            path: dir.to_string_lossy().to_string(),
            ecosystem: "Python".to_string(),
            manifest_file: manifest_name.to_string(),
            lockfile_file: lockfile,
            git_status,
            total_artifact_size_bytes: total_size,
            artifacts,
        })
    }

    /// Inspect a Go project directory
    pub fn inspect_go_project(dir: &Path) -> Option<DeveloperProjectDto> {
        let go_mod = dir.join("go.mod");
        if !go_mod.exists() {
            return None;
        }

        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("Go Project").to_string();
        let go_sum = dir.join("go.sum");
        let lockfile = if go_sum.exists() { Some("go.sum".to_string()) } else { None };
        let git_status = Self::check_git_status(dir);

        let mut artifacts = Vec::new();
        let mut total_size: u64 = 0;

        for build_target in &["bin", "dist"] {
            let b_dir = dir.join(build_target);
            if b_dir.exists() {
                let size = Self::calculate_directory_size(&b_dir);
                total_size += size;
                artifacts.push(DeveloperArtifactDto {
                    name: build_target.to_string(),
                    relative_path: build_target.to_string(),
                    full_path: b_dir.to_string_lossy().to_string(),
                    size_bytes: size,
                    category: "Build Output".to_string(),
                    is_regenerable: true,
                    reversibility: "Quarantine Vault / go build".to_string(),
                    cleanup_procedure: "Stage to quarantine; regenerable via go build".to_string(),
                    risk_tier: RiskTier::Safe,
                    risks_and_questions: "Compiled Go binary artifact.".to_string(),
                });
            }
        }

        Some(DeveloperProjectDto {
            id: format!("proj_go_{}", Self::hash_path(dir)),
            name,
            path: dir.to_string_lossy().to_string(),
            ecosystem: "Go".to_string(),
            manifest_file: "go.mod".to_string(),
            lockfile_file: lockfile,
            git_status,
            total_artifact_size_bytes: total_size,
            artifacts,
        })
    }

    /// Checks git repository status: 'clean', 'dirty', 'untracked_work', or 'no_repo'
    pub fn check_git_status(project_dir: &Path) -> String {
        let git_dir = match find_git_repository(project_dir) {
            Some(g) => g,
            None => return "no_repo".to_string(),
        };

        // Check if git command can be run safely
        if let Ok(output) = std::process::Command::new("git")
            .arg("-C")
            .arg(project_dir)
            .arg("status")
            .arg("--porcelain")
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    return "clean".to_string();
                } else {
                    return "dirty".to_string();
                }
            }
        }

        // Fallback: If git CLI is not installed or errors, check for basic git head / index presence
        if git_dir.join("index").exists() {
            "clean".to_string()
        } else {
            "no_repo".to_string()
        }
    }

    /// Recursively calculates directory size, bounded and avoiding reparse point loops
    pub fn calculate_directory_size(dir_path: &Path) -> u64 {
        let mut total_bytes: u64 = 0;
        let walker = WalkDir::new(dir_path)
            .max_depth(20)
            .follow_links(false)
            .into_iter();

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                // Ignore secret files inside build dirs from contributing to cleanable count
                let fname = entry.file_name().to_string_lossy();
                if is_secret_file(&fname) {
                    continue;
                }
                if let Ok(meta) = entry.metadata() {
                    total_bytes += meta.len();
                }
            }
        }
        total_bytes
    }

    fn hash_path(path: &Path) -> String {
        let s = path.to_string_lossy();
        let hash = blake3::hash(s.as_bytes());
        hash.to_hex()[..10].to_string()
    }
}
