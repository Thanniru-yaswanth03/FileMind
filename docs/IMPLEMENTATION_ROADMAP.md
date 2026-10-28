# FileMind — Implementation Roadmap & Execution Gates

**Document Version:** 1.0.0  
**Status:** Approved Roadmap Specification  
**Current Phase:** Phase 0 (Principal Engineering Discovery, Architecture & Impeccable Design Setup)  

---

## 1. Phased Delivery Strategy

To guarantee maximum engineering discipline, stability, and data safety, FileMind is implemented across eight sequential phases. 

> **CRITICAL SAFETY INVARIANT:** All filesystem functionality remains **100% READ-ONLY** through Phases 1, 2, and 3. Destructive and staging operations (quarantine moves) are prohibited until Phase 4, following the implementation and verification of the deterministic Safety Engine and automated Destructive Sandbox Suite.

```mermaid
gantt
    title FileMind Engineering Implementation Roadmap
    dateFormat  YYYY-MM-DD
    section Architectural Foundation
    Phase 0: Discovery & Architecture Gate        :done, p0, 2026-10-28, 1d
    section Read-Only Engine
    Phase 1: Toolchain Setup & Read-Only Scanner :active, p1, 2026-10-29, 3d
    Phase 2: Metadata Index, FTS5 & Duplicates   :p2, after p1, 4d
    Phase 3: Safety Engine & Read-Only Planner   :p3, after p2, 4d
    section Execution & UI
    Phase 4: Quarantine Vault & Recovery Engine   :p4, after p3, 5d
    Phase 5: Impeccable UI (React 19 + Tables)   :p5, after p4, 5d
    Phase 6: Developer Workspace & Automation    :p6, after p5, 4d
    section Copilot & Release
    Phase 7: Optional Copilot, Hardening & Ship  :p7, after p6, 4d
```

---

## 2. Detailed Phase Breakdown

### Phase 0: Principal Engineering Discovery, Architecture & Impeccable Setup (CURRENT)
- **Scope:** Complete workspace inspection, toolchain audit, Impeccable design integration, STRIDE threat model, database schema design, test strategy, and release plan.
- **Deliverables:**
  - `README.md`
  - `PRODUCT.md`
  - `docs/PRODUCT_REQUIREMENTS.md`
  - `docs/ARCHITECTURE.md`
  - `docs/DESIGN_SYSTEM.md`
  - `docs/THREAT_MODEL.md`
  - `docs/DATABASE_SCHEMA.md`
  - `docs/TEST_STRATEGY.md`
  - `docs/RELEASE_PLAN.md`
  - `docs/IMPLEMENTATION_ROADMAP.md`
  - `docs/DECISIONS.md`
- **Gate Check:** Architecture approved; missing prerequisites documented; read-only boundary confirmed.

---

### Phase 1: Toolchain Setup, Tauri 2 Scaffolding & Native Storage Scanner (READ-ONLY)
- **Scope:**
  - Install Rust toolchain (`rustup`) and Microsoft Visual Studio 2022 C++ Build Tools.
  - Scaffold Tauri 2 + Vite + React 19 + TypeScript workspace using `pnpm`.
  - Implement Module 4 (`Win32 Filesystem Adapter`) with long path (`\\?\`) and reparse point handling.
  - Implement Module 5 (`Parallel Storage Scanner`) using work-stealing threadpool and `THREAD_MODE_BACKGROUND_BEGIN`.
  - Expose typed Tauri command: `start_scan(path) -> Result<ScanSummary, AppError>`.
- **Safety Boundary:** Read-only directory traversal and attribute reading. Zero file mutation.
- **Definition of Done:** Native scanner compiles on Windows; scans a 100,000-file directory in < 1.5 seconds; reports correct byte sizes and cluster allocation.

---

### Phase 2: Metadata Indexing, SQLite FTS5 Search & Duplicate Pipeline (READ-ONLY)
- **Scope:**
  - Implement Module 6 (`File Metadata Indexer`) with SQLite WAL transactions.
  - Implement Module 14 (`FTS5 Search & Document Extractor`) for text, markdown, code, and PDF files.
  - Implement Module 7 (`Multi-Stage Duplicate Detector`) with size matching, 4KB sparse BLAKE3 hashing, and full cryptographic BLAKE3 stream hashing.
  - Implement NTFS hardlink detection to distinguish clones from hardlinks.
- **Safety Boundary:** Read-only disk reads, content hashing, and SQLite persistence.
- **Definition of Done:** 50GB mixed directory indexed into SQLite in < 3 seconds; duplicate detection correctly identifies exact file matches without flagging hardlinks.

---

### Phase 3: Safety Engine, Dependency Analyzer & Read-Only Planner (READ-ONLY)
- **Scope:**
  - Implement Module 8 (`Deterministic Safety Engine`) enforcing the **"Unknown is Unsafe"** invariant.
  - Implement Windows Restart Manager integration (`rstrtmgr.dll`) to detect active process file locks.
  - Implement Git repository status inspector (`git2` or direct `.git` index reading).
  - Implement Module 9 (`Cleanup Planner`) generating itemized plans with risk tier categorization (Safe, Caution, High Risk, Danger).
- **Safety Boundary:** Read-only inspection and dry-run plan generation. Zero file deletion.
- **Definition of Done:** Safety Engine scores 100+ simulated directory scenarios; correctly flags dirty git repos, active process locks, and system roots.

---

### Phase 4: Staged Quarantine Vault, Recovery Engine & Execution Manager (FIRST MUTATION)
- **Scope:**
  - Implement Module 11 (`Quarantine and Recovery Vault`) managing per-volume `.filemind_quarantine` roots.
  - Implement atomic file staging via `MoveFileExW(MOVEFILE_WRITE_THROUGH)`.
  - Implement quarantine manifest writer and SQLite ledger.
  - Implement complete one-click Rollback Engine with timestamp, attribute, and ACL preservation.
  - Implement Module 10 (`File Operation Manager`) with TOCTOU pre-flight verification.
  - Execute automated **Destructive Sandbox Suite** (`tests/safety_sandbox.rs`).
- **Safety Boundary:** All operations are reversible quarantine moves. Direct permanent deletion is prohibited.
- **Definition of Done:** 10,000 files staged into quarantine and 100% restored with byte-exact BLAKE3 hash parity; zero data loss in simulated power-cut or aborted batch scenarios.

---

### Phase 5: Impeccable Desktop UI Implementation
- **Scope:**
  - Build React 19 UI adhering to `docs/DESIGN_SYSTEM.md` in Impeccable *Operate Mode*.
  - Implement warm ivory light theme (`#FBF9F5`) and warm charcoal dark theme (`#18191A`).
  - Implement 60fps high-density virtualized table (`@tanstack/react-virtual`).
  - Implement interactive canvas-based treemap and sunburst storage visualizers.
  - Implement Property Inspector drawer, Breadcrumb bar, and Filter ribbons.
  - Integrate keyboard navigation (`Ctrl+F`, `Arrows`, `Space`, `Enter`, `Esc`).
- **Definition of Done:** Impeccable detector scan passes cleanly; UI renders 100,000 files at 60fps; full keyboard navigability.

---

### Phase 6: Developer Workspace Analyzer & Automation Engine
- **Scope:**
  - Implement Module 7 (`Developer Workspace Analyzer`) identifying `node_modules`, `target/`, `.venv`, `.gradle`, and Docker layers.
  - Correlate project inactive days and git cleanliness.
  - Implement Module 13 (`Automation Scheduler`) executing low-priority background cleaning rules on Tier 1 (Safe) items.
  - Implement Windows notification prompts for manual review of ambiguous items.
- **Definition of Done:** User can safely reclaim 50GB+ of stale developer caches across multiple repositories with two clicks.

---

### Phase 7: Optional Local Copilot, Hardening & Windows Release
- **Scope:**
  - Implement Module 15 (`Optional AI Provider`) connecting to local Ollama (`http://localhost:11434`).
  - Connect strictly typed, read-only tools; enforce prompt injection escaping.
  - Implement Module 16 (`Diagnostics, Logging & Health Checks`).
  - Configure GitHub Actions CI/CD release workflow for NSIS, WiX MSI, and portable ZIP builds.
  - Sign binaries with Microsoft Authenticode and Tauri Ed25519 updater keys.
- **Definition of Done:** E2E release build passes; MSI and NSIS installers install cleanly on Windows 11 and 10; auto-updater checks succeed.

---

## 3. High-Risk Engineering Decisions & Mitigations

| Risk Area | Description | Architectural Mitigation |
| :--- | :--- | :--- |
| **Accidental Deletion** | Deleting critical files causes permanent data loss or system corruption. | Destructive deletes are banned. All removals stage into `.filemind_quarantine`. The Safety Engine treats "unknown" as unsafe. |
| **TOCTOU Race Condition** | File modified between scan time and quarantine execution. | Atomic pre-flight check validates size, timestamp, and lock before moving. Aborts if discrepancy is detected. |
| **Junction Traversal** | Recursive scanner traverses NTFS junctions into parent or system disks. | Filesystem adapter inspects `FILE_ATTRIBUTE_REPARSE_POINT` and refuses to follow junctions during scans or deletions. |
| **UI Freeze on Large Disks** | Scanning millions of files causes IPC congestion or frontend lag. | Native scanner runs at `IDLE_PRIORITY_CLASS`. Progress events are throttled to 10/sec. Virtualized table renders only visible rows. |
| **AI Hallucination / Injection** | LLM hallucinates paths or is manipulated into suggesting system directory deletion. | Copilot has zero filesystem access. All proposals are independently verified by the native Rust Safety Engine. |

