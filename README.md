# FileMind

> **Local-First Windows Desktop Storage Intelligence, Safety Analysis, and Clean Planning**

FileMind is a local-first, privacy-respecting Windows desktop utility built with **Tauri 2**, **Rust**, **React 19**, and **SQLite**. It brings together high-performance storage visualization, metadata indexing, duplicate detection, dependency-aware safety scoring, intelligent cleanup planning, and recoverable quarantine operations.

FileMind runs completely offline with **zero mandatory cloud services, zero paid APIs, and zero telemetry**. It operates with complete utility even if its optional local AI copilot is entirely disabled.

---

## 1. Product Vision & Core Capabilities

FileMind unifies 10 core desktop storage capabilities into a cohesive, evidence-based application:

1. **Storage Analysis & Visualization:** Interactive treemaps, sunburst visualizations, and compact breakdown tables analyzing disk allocation down to individual clusters.
2. **File Indexing & Search:** High-speed SQLite-backed indexing with FTS5 full-text search across filenames, metadata, and extracted document text.
3. **Multi-Stage Duplicate Detection:** Deterministic duplicate identification pipeline using file size matching, 4KB sparse block hashing, full BLAKE3 cryptographic verification, and hardlink detection.
4. **Evidence-Based File Safety Engine:** Deterministic rule engine evaluating system directories, running processes, open file locks, package manifests, and repository dirty states. **Unknown safety is never treated as safe.**
5. **Intelligent Cleanup Planning:** Impact-assessed cleanup plans with projected space reclamation, grouped by risk tier (Safe, Caution, High Risk, Danger), requiring explicit user review.
6. **Quarantine & Recoverable Operations:** Two-stage quarantine vaulting system with atomic file staging, metadata preservation, and complete zero-loss rollback manifests before any permanent purge.
7. **Developer Workspace & Cache Analysis:** Deep inspection of developer bloat (`node_modules`, Python virtualenvs, Rust `target/`, Gradle/Maven caches, Docker layers, IDE indexes).
8. **Configurable File Automation:** User-defined background rules (e.g., auto-archiving temporary downloads, clearing stale build caches beyond age thresholds) running via a native scheduler.
9. **Optional Local Copilot:** An isolated, advisory JARVIS-style assistant connecting to a local Ollama instance. The model has **zero direct filesystem access** and communicates strictly via read-only inspection tools.
10. **Local Diagnostics & Release Management:** Append-only local audit logs, crash diagnostics, database health checks, and secure Ed25519-signed update validation.

---

## 2. Environment Status & Prerequisites

During **Phase 0 Engineering Discovery**, the host Windows environment was inspected:

| Component | Status | Details |
| :--- | :--- | :--- |
| **Operating System** | Supported | Windows 11 / Windows 10 (x64) |
| **Node.js** | Available | `v22.19.0` |
| **Package Manager** | Available | `npm 11.2.0`, `pnpm 10.13.1` (pnpm is the preferred package manager) |
| **WebView2 Runtime** | Available | Microsoft Edge WebView2 installed in Program Files |
| **Impeccable Design Engine** | Available | Installed globally at `C:\Users\batma\.gemini\config\skills\impeccable\` |
| **Rust Toolchain** | **MISSING** | `rustc` and `cargo` not detected in PATH or user profile |
| **C++ Build Tools** | **MISSING** | MSVC v143 toolchain & Windows SDK not detected by `vswhere` |

### Required Setup Instructions for Native Build

Before compiling the native Tauri/Rust desktop binary, execute the following setup commands in an elevated PowerShell session:

```powershell
# 1. Install Microsoft Visual Studio 2022 C++ Build Tools & Windows 11 SDK
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --config C:\Users\batma\.gemini\antigravity\brain\vs_config.json --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# 2. Install the Rust Toolchain via rustup
winget install --id Rustlang.Rustup

# 3. Configure default MSVC host target and restart terminal
rustup default stable-x86_64-pc-windows-msvc
```

> **Read-Only Gate:** Native destructive filesystem actions remain strictly blocked until the complete safety and quarantine architecture is implemented and verified through automated sandbox tests.

---

## 3. Impeccable Design Integration

FileMind rejects generic AI dashboard tropes—there are no neon glows, purple gradients, glassmorphism, floating AI orbs, or fake terminal windows. 

Instead, the UI follows the **Impeccable** design standard in **Operate Mode**:
- **Surfaces:** Warm ivory (`#FBF9F5`) and paper-like neutral layers (`#F3EFEA`, `#E8E2D9`).
- **Typography:** High-contrast graphite (`#1F2022`) and warm charcoal (`#2C2D30`) with strict tabular numerals (`font-variant-numeric: tabular-nums`) for data tables and measurements.
- **Accents:** Restrained muted sage green (`#4A6B5B`) for primary actions and subtle indicators; muted terracotta (`#B4533C`) for warnings and destructive prompts.
- **Dark Mode:** Complementary warm-charcoal (`#18191A`, `#222325`) palette preserving identical semantic contrast ratios.

---

## 4. Documentation Index

The complete architectural specification is maintained under the [`docs/`](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs) directory:

- [**PRODUCT_REQUIREMENTS.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/PRODUCT_REQUIREMENTS.md) — Comprehensive product vision, personas, user journeys, functional and non-functional specifications.
- [**ARCHITECTURE.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/ARCHITECTURE.md) — Multi-process Tauri 2 architecture, 18 native module designs, data flows, and IPC contracts.
- [**DESIGN_SYSTEM.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/DESIGN_SYSTEM.md) — Impeccable design system tokens, layout grids, components, dark mode, and accessibility benchmarks.
- [**THREAT_MODEL.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/THREAT_MODEL.md) — STRIDE security evaluation, TOCTOU race mitigation, NTFS junction handling, and AI prompt isolation.
- [**DATABASE_SCHEMA.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/DATABASE_SCHEMA.md) — SQLite schema, WAL configuration, FTS5 virtual tables, indexes, and migration strategy.
- [**TEST_STRATEGY.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/TEST_STRATEGY.md) — Unit, integration, and E2E testing strategy, including destructive test sandboxes and benchmarks.
- [**RELEASE_PLAN.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/RELEASE_PLAN.md) — CI/CD matrix, Ed25519 code signing, MSI/NSIS bundling, and update verification.
- [**IMPLEMENTATION_ROADMAP.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/IMPLEMENTATION_ROADMAP.md) — Phased rollout plan, definition of done, and implementation gates.
- [**DECISIONS.md**](file:///c:/Users/batma/Desktop/VibeCode/FileMind/docs/DECISIONS.md) — Architecture Decision Records (ADRs) explaining core design trade-offs.

---

## 5. Security & Safety Guarantee

The safety of user files is an absolute invariant:
- **No Direct AI Filesystem Control:** The optional LLM acts purely as an advisory reader. Every operation must translate into a typed proposal evaluated by the Rust safety engine.
- **Two-Stage Quarantine Vault:** Files scheduled for deletion are moved into an isolated staging vault with an index manifest before any purge can occur.
- **Audit Ledger:** Every quarantine, restore, and purge action is immutably logged into a local SQLite ledger.

