# FileMind

<div align="center">

**Local-First Windows Desktop Storage Intelligence, Safety Guardian, and Clean Planning**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011%20(x64)-0078D6.svg)](docs/DISTRIBUTION_GUIDE.md)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/react-19-61DAFB.svg)](https://react.dev/)
[![SQLite](https://img.shields.io/badge/sqlite-WAL%20Mode-003B57.svg)](https://www.sqlite.org/)
[![Telemetry](https://img.shields.io/badge/telemetry-0%25%20Offline-green.svg)](docs/PRIVACY_AND_DATA.md)

</div>

---

## 1. Overview

**FileMind** is a high-performance, local-first Windows desktop storage manager and file safety guardian. It gives users complete visibility into disk allocation, locates exact duplicate files through cryptographic verification, identifies developer cache bloat, and executes safe, recoverable cleanups backed by a two-stage quarantine vault.

FileMind is designed from first principles with **zero mandatory cloud services, zero analytics telemetry, and zero remote dependencies**. Everything runs entirely on your local machine.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        FILEMIND ARCHITECTURE                           │
├────────────────────────────────┬───────────────────────────────────────┤
│        FRONTEND LAYER          │            BACKEND RUNTIME            │
│  React 19 + TypeScript + Vite  │    Rust (Native x86_64 Executable)    │
│   Impeccable Design Standard   │        Tauri 2 / Embedded HTTP        │
├────────────────────────────────┴───────────────────────────────────────┤
│                          CORE SUBSYSTEMS                               │
│  ┌───────────────────────────┐         ┌────────────────────────────┐  │
│  │   Multi-Threaded Scanner  │ ──────> │  SQLite WAL Storage Index  │  │
│  └───────────────────────────┘         └────────────────────────────┘  │
│                │                                     │                 │
│                ▼                                     ▼                 │
│  ┌───────────────────────────┐         ┌────────────────────────────┐  │
│  │    Duplicate Detection    │         │     FTS5 Search Engine     │  │
│  │  (Size + 4KB + BLAKE3)    │         │   (Filenames & Documents)  │  │
│  └───────────────────────────┘         └────────────────────────────┘  │
│                │                                     │                 │
│                ▼                                     ▼                 │
│  ┌───────────────────────────┐         ┌────────────────────────────┐  │
│  │  Deterministic Guardian   │ ──────> │  Goal-Based Cleanup Plan   │  │
│  │  (Evidence-Based Safety)  │         │  (Impact & Risk Grouping)  │  │
│  └───────────────────────────┘         └────────────────────────────┘  │
│                │                                     │                 │
│                ▼                                     ▼                 │
│  ┌───────────────────────────┐         ┌────────────────────────────┐  │
│  │ Recoverable Quarantine    │         │ Local Automation Engine    │  │
│  │ (Rollback Manifest Vault) │         │ (Rules & Audit Ledger)     │  │
│  └───────────────────────────┘         └────────────────────────────┘  │
│                │                                     │                 │
│                ▼                                     ▼                 │
│  ┌───────────────────────────┐         ┌────────────────────────────┐  │
│  │  Advisory Local Copilot   │         │ Developer Workspace Bloat  │  │
│  │  (Read-Only Ollama LLM)   │         │ (node_modules, target, venv)│  │
│  └───────────────────────────┘         └────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Key Features

- **Blazing Fast Read-Only Scanner:** Multi-threaded filesystem enumeration using Rayon work-stealing and crossbeam channels. Resilient against inaccessible NTFS directories, junctions, and reparse points without hanging.
- **SQLite Storage Index (WAL Mode):** High-throughput persistent index supporting sub-millisecond metadata lookups, extension categorization, directory aggregation, and historical scan tracking.
- **Multi-Stage Duplicate Pipeline:** 
  1. *Candidate Grouping:* Groups files by exact byte size.
  2. *Sparse Block Hashing:* Reads 4KB header/footer blocks to prune non-duplicates.
  3. *Cryptographic Verification:* Memory-bounded streaming [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) hashes with hardlink deduplication.
- **FTS5 Document & Metadata Search:** Real-time search across millions of indexed records, including extracted text from PDFs, plain text, and source code.
- **Deterministic Safety Guardian:** Conservative risk engine evaluating target paths against 5 security classifications. **Unknown files are never assumed safe.** Detects Windows system paths, active process locks, and uncommitted Git changes.
- **Recoverable Two-Stage Quarantine:** Deleted files are staged into an isolated vault with SHA-256 manifests, original timestamps, and permissions preserved for one-click rollback before any permanent purge.
- **Developer Workspace Intelligence:** Identifies orphaned build directories (`node_modules`, Rust `target/`, Python `.venv`, Docker layers, Gradle/Maven caches).
- **Local Automation Engine:** Typed background rules with disk thresholds, schedule triggers, and an append-only audit ledger.
- **Advisory Local Copilot (Offline AI):** Optional integration with local [Ollama](https://ollama.com/) models (`qwen2.5-coder`, `llama3.2`). Grounded exclusively in your SQLite index with **zero write permissions**.
- **Impeccable Design Standard:** Professional, distraction-free desktop aesthetic using warm paper-neutral palettes, high-contrast typography, and strict tabular numerals.

---

## 3. Installation & Distribution

FileMind is distributed in two formats:

### Option A: Zero-Install Portable Edition (Recommended)

Ideal for running from any folder, external SSD, or USB drive without administrative privileges:

1. Download `FileMind-v0.2.0-windows-x64-portable.zip` from [GitHub Releases](https://github.com/vibe-filemind/filemind/releases).
2. Extract the archive into any folder of your choice.
3. Double-click `FileMind.exe`.
4. FileMind launches and automatically opens its interface in your default browser at `http://localhost:41732`.

> **Zero Registry Pollution:** The presence of `portable.flag` configures FileMind to store all SQLite databases, logs, and quarantine vaults in `./data/`. No files are written to `%LOCALAPPDATA%` or the Windows Registry. To uninstall, simply delete the extracted folder.

### Option B: Native Windows Installer (Planned GA Release)

*(Staged for future release pending brand icon finalization; v0.2.0 Beta is distributed primarily via the Zero-Install Portable Edition).*

- **NSIS Installer (`.exe`):** Guided installation wizard with Desktop and Start Menu shortcuts, per-user or machine-wide installation, and an uninstaller in Windows *Settings > Installed apps*.
- **WiX MSI Package (`.msi`):** Enterprise-ready installer supporting silent administrative deployments via:
  ```powershell
  msiexec /i FileMind.msi /qn /norestart
  ```

---

## 4. Cryptographic Verification & SmartScreen

### Verifying Release Checksums

To verify the integrity of your download, open PowerShell in your download folder:

```powershell
# Automated verification via included script:
.\Verify-Checksums.ps1

# Or manual verification:
(Get-FileHash .\FileMind-v0.2.0-windows-x64-portable.zip -Algorithm SHA256).Hash
```

Compare the output hash with the published hash in [GitHub Releases](https://github.com/vibe-filemind/filemind/releases) or `SHA256SUMS.txt`.

### Windows Defender SmartScreen Notice

> [!NOTE]
> FileMind is open-source software built locally and published on GitHub without commercial Extended Validation (EV) code signing certificates (which cost several hundred dollars annually).
> 
> When running FileMind for the first time, Windows Defender SmartScreen may display an *"Unknown Publisher"* warning:
> 1. Verify the cryptographic SHA-256 hash using the command above.
> 2. Click **More info**.
> 3. Click **Run anyway**.
>
> If you prefer not to bypass SmartScreen, you can inspect the source code and compile the binary locally using the build instructions below.

---

## 5. System Requirements

| Specification | Requirement |
| :--- | :--- |
| **Operating System** | Windows 10 (version 1809+) or Windows 11 (x64) |
| **Processor** | 64-bit Intel / AMD x86_64 processor (2+ cores recommended) |
| **RAM** | 512 MB minimum (1 GB recommended for large scans >1M files) |
| **Storage** | 35 MB disk space for executable; variable space for index database |
| **Browser Runtime** | Microsoft Edge WebView2 (pre-installed on Windows 10/11) or modern browser |
| **Network** | **None.** Operates 100% offline |

---

## 6. Building from Source

### Prerequisites

Ensure you have the following installed on your Windows development environment:
- **Git:** For version control.
- **Node.js 20+ & pnpm:** `npm install -g pnpm`
- **Rust Toolchain:** `rustup default stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`.
- **C/C++ Compiler:** MinGW-w64 GCC or Visual Studio C++ Build Tools.

### Build Steps

1. **Clone the repository:**
   ```powershell
   git clone https://github.com/vibe-filemind/filemind.git
   cd FileMind
   ```

2. **Install frontend dependencies & build static assets:**
   ```powershell
   pnpm install
   pnpm run build
   ```

3. **Run the full test suite (85 tests across 19 suites):**
   ```powershell
   cargo test --all
   ```

4. **Compile the optimized release binary:**
   ```powershell
   cargo build --release --bin filemind-core
   ```

5. **Package the distribution bundle:**
   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts/package-release.ps1 -Version "0.2.0"
   ```

---

## 7. Privacy & Data Handling

FileMind's core engineering principles guarantee user privacy:

- **Zero Remote Telemetry:** No tracking, pingbacks, analytics, or error crash reporting sent across the network.
- **Local Storage Index:** Metadata indexes and scan histories are stored in a standard local SQLite database (`filemind.db`).
- **Protected Paths & Folders:** Sensitive user paths can be explicitly excluded from scans and search indexing.
- **Offline AI Isolation:** If Ollama local copilot is enabled, prompts are routed exclusively to `http://localhost:11434`. No data ever leaves your computer.

For complete privacy specifications, read [`docs/PRIVACY_AND_DATA.md`](docs/PRIVACY_AND_DATA.md).

---

## 8. Documentation Index

Detailed architectural specifications and engineering guides:

- [**Distribution & Packaging Guide**](docs/DISTRIBUTION_GUIDE.md) — Artifact formats, silent installations, and disaster recovery.
- [**Privacy & Data Policy**](docs/PRIVACY_AND_DATA.md) — Local-first privacy guarantees and diagnostic scrubbing.
- [**Product Requirements**](docs/PRODUCT_REQUIREMENTS.md) — User personas, functional requirements, and core constraints.
- [**Architecture & Contracts**](docs/ARCHITECTURE.md) — Multi-process model, native IPC contracts, and module boundaries.
- [**Impeccable Design Standard**](docs/DESIGN_SYSTEM.md) — Visual tokens, tabular typography, and accessible color palettes.
- [**Threat Model & Mitigations**](docs/THREAT_MODEL.md) — STRIDE security analysis, TOCTOU race mitigation, and reparse security.
- [**Database Schema Specification**](docs/DATABASE_SCHEMA.md) — SQLite schema, WAL configuration, and FTS5 search tables.
- [**Security & QA Audit**](docs/TEST_STRATEGY.md) — Test strategy, sandboxed execution tests, and benchmark performance.
- [**Changelog**](CHANGELOG.md) — Chronological history of all releases and improvements.
- [**Engineering Operating Instructions**](docs/OPERATING_INSTRUCTIONS.md) — Implementation, design, security, and verification disciplines.
- [**Contributing Guidelines**](CONTRIBUTING.md) — Development workflow, code style, and safety contribution invariants.

---

## 9. Contributing

We welcome community contributions that uphold FileMind's safety and privacy principles. Before submitting a pull request, please review [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 10. License

FileMind is licensed under the **Apache License, Version 2.0**. See [LICENSE](LICENSE) for details.
