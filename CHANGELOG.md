# Changelog

All notable changes to **FileMind** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] — 2026-10-28

### Added
- **Native Storage Scanning Engine:**
  - High-performance, read-only multi-threaded filesystem crawler built with Rayon and crossbeam channels.
  - SQLite metadata storage index configured with Write-Ahead Logging (WAL) and synchronous normal mode for concurrent read-write workloads.
  - Comprehensive MIME and extension categorization (Audio, Video, Images, Documents, Archives, Executables, Developer Bloat, System, Temporary, Other).
  - Directory subtree aggregation tracking file counts, total allocation, and largest file hierarchies.
  - Graceful handling of inaccessible NTFS directories, junctions, and permission-denied errors without terminating scans.

- **Multi-Stage Duplicate Detection & Fast Search:**
  - 3-stage duplicate verification pipeline:
    1. Size and extension grouping (O(1) candidate identification).
    2. 4KB sparse block hashing (rapid false-positive pruning).
    3. Full streaming BLAKE3 cryptographic verification with memory-bounded chunking and hardlink identity checking.
  - SQLite FTS5 full-text search indexing across filenames, metadata, and extracted document text (PDF, plain text, code files).

- **Deterministic Safety Guardian (Smart Delete Engine):**
  - Conservative, evidence-based safety evaluation engine. **Unknown safety is never treated as safe.**
  - 5 deterministic risk tiers: `Protected System Path`, `High Risk`, `Caution / Dependencies Present`, `Review Required`, and `Low Risk / Validated Cleanup`.
  - Automatic detection of Windows system roots (`C:\Windows`, `System32`, `Program Files`, `WinSxS`, system volume information).
  - Detection of running process locks via Windows toolhelp snapshot API.
  - Detection of developer project state (uncommitted Git modifications, dirty worktrees).

- **Recoverable Quarantine Vault & Cleanup Planning:**
  - Two-stage recoverable quarantine vault preserving original paths, timestamps, permissions, and BLAKE3 hashes in an encrypted/isolated manifest.
  - Atomic rename-based quarantine with cross-volume copy-and-verify fallback.
  - Complete zero-loss restoration mechanism with manifest verification.
  - Goal-oriented cleanup planner (e.g. "Reclaim 10 GB") prioritizing validated low-risk candidates and reversible actions.
  - Permanent purge requiring explicit secondary confirmation.

- **JARVIS Advisory Copilot (Local AI):**
  - Offline LLM integration with local Ollama instances (`qwen2.5-coder`, `llama3.2`, etc.).
  - Complete prompt grounding in live SQLite database records.
  - Strictly read-only tool orchestration: the copilot has **zero write permissions** and cannot execute filesystem operations directly.

- **Local Automation Engine & Developer Intelligence:**
  - Typed rule evaluation engine responding to file triggers, disk space thresholds, and scheduled intervals.
  - Developer bloat scanner detecting orphaned `node_modules`, Rust `target/` directories, Python `.venv`, Docker caches, and Gradle/Maven artifacts.
  - Immutable SQLite audit ledger recording all automation triggers, evaluations, and user interventions.

- **Security & Adversarial QA Hardening:**
  - Patched static asset path traversal vulnerability (`..` escaping sandbox roots) via strict canonicalization and path segment sanitization.
  - Patched loose CORS validation (`http://localhost*` substring) with exact host and numeric port checking to block malicious origins (`http://localhost.attacker.com`).
  - Implemented cryptographic session capability token (`X-FileMind-Session-Token`) to prevent unauthenticated rogue processes or CSRF attacks from invoking mutating endpoints.
  - Hardened cross-volume quarantine and restore operations with `verified_quarantine_move`: pre-move hash checks, streaming BLAKE3 hashing, post-copy source mutation checks, and automatic rollback.
  - Added self-healing SQLite database recovery: automatically quarantines corrupted `.db` files and initializes a fresh clean database to prevent crash loops.
  - Expanded automated test suite: **85 tests across 19 suites passing with zero failures**, including real-world synthetic scale (25,000 files in SQLite WAL), 35-level deep directory nesting, and 624 MB/s streaming hash benchmarks.

- **Distribution & Release Engineering:**
  - Portable Zero-Install Windows release (`FileMind-v0.2.0-windows-x64-portable.zip`):
    - Activated by `portable.flag`; redirects all SQLite databases and quarantine storage to local `./data/`.
    - Zero registry modifications, zero `%LOCALAPPDATA%` pollution.
  - Native embedded SPA server: serves compiled React 19 UI assets (`dist/`) directly over local HTTP (`http://localhost:41732`).
  - Automatic browser launching on startup with `--no-open` flag for headless execution.
  - Tauri 2 bundle configuration supporting NSIS (`.exe`) and WiX (`.msi`) installers.
  - GitHub Actions CI/CD workflows (`ci.yml` and `release.yml`) for automated building, linting, testing, and multi-artifact packaging.
  - Cryptographic verification via `SHA256SUMS.txt` and automated PowerShell verification script `Verify-Checksums.ps1`.
  - Comprehensive documentation: `DISTRIBUTION_GUIDE.md`, `PRIVACY_AND_DATA.md`, `CONTRIBUTING.md`, and Apache-2.0 `LICENSE`.

---

## [0.1.0] — 2026-10-28

### Added
- Initial project architecture and specifications:
  - Product Requirements Document (`docs/PRODUCT_REQUIREMENTS.md`).
  - System Architecture & IPC Contract (`docs/ARCHITECTURE.md`).
  - Impeccable Design System specification (`docs/DESIGN_SYSTEM.md`).
  - STRIDE Threat Model (`docs/THREAT_MODEL.md`).
  - Database Schema & Migration Strategy (`docs/DATABASE_SCHEMA.md`).
  - Test Strategy & Benchmark Suite (`docs/TEST_STRATEGY.md`).
  - Release & Packaging Strategy (`docs/RELEASE_PLAN.md`).
  - Implementation Roadmap (`docs/IMPLEMENTATION_ROADMAP.md`).
  - Architecture Decision Records (`docs/DECISIONS.md`).
- React 19 + TypeScript + Vite frontend scaffold with Tailwind CSS, Lucide icons, and Impeccable design palette.
- Cargo workspace with `filemind-core` library and CLI binary.

