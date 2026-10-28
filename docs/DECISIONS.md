# FileMind — Architecture Decision Records (ADRs)

**Document Version:** 1.0.0  
**Status:** Approved Architectural Decisions  

---

## ADR-001: Tauri 2 over Electron for Native Windows Desktop Architecture

### Context
FileMind is a system-level utility for Windows workstations requiring deep filesystem access, low resource consumption, and rapid startup. Electron bundles an entire Chromium browser and Node.js runtime with each binary, resulting in large distribution packages (150MB+ installer), high memory consumption (300MB+ idle), and complex multi-threaded native C++ addon bindings.

### Decision
Adopt **Tauri 2** (Rust native core + Windows WebView2 runtime).

### Consequences
- **Pros:**
  - Distribution binary size is under 15MB (compared to 150MB+ for Electron).
  - Memory consumption is significantly lower (~80MB idle vs. 300MB+).
  - Native filesystem, NTFS APIs, Win32 calls, and multi-threaded BLAKE3 hashing are written directly in idiomatic, memory-safe Rust without node-gyp or FFI overhead.
  - Granular security isolation between the WebView and Rust core via Tauri's permission capabilities.
- **Cons:**
  - Requires Windows WebView2 runtime (pre-installed on Windows 10/11, but requires bootstrapper on rare barebones systems).
  - Requires Rust toolchain for native compilation.

---

## ADR-002: Embedded SQLite with Bundled FTS5 over External Indexers or Cloud DBs

### Context
FileMind must index millions of files and full-text documents while remaining 100% offline and local-first. Integrating external search engines (such as Tantivy or Meilisearch) or cloud databases adds significant operational complexity, background daemon bloat, and port-binding security vulnerabilities.

### Decision
Embed **SQLite 3.45+** directly into the Rust core using `rusqlite` with the `bundled` and `bundled-fts5` feature flags, configured with Write-Ahead Logging (`WAL`) and `PRAGMA synchronous = NORMAL`.

### Consequences
- **Pros:**
  - Zero external processes, zero background daemons, zero network port binding.
  - Entire database resides in a single, robust file (`%LOCALAPPDATA%\FileMind\filemind.db`).
  - Full-text search with BM25 ranking is natively supported via SQLite's built-in FTS5 module.
  - Supports millions of rows with sub-millisecond B-tree index queries.
- **Cons:**
  - Requires careful WAL transaction batching (20,000 rows/batch) to avoid lock contention during multi-threaded scanning.

---

## ADR-003: Purely Local Advisory AI Copilot with Zero Direct Filesystem Privileges

### Context
The product vision includes an optional JARVIS-style assistant to help users understand disk usage. However, giving an LLM direct filesystem privileges (e.g., shell access or deletion tools) introduces catastrophic security risks, including hallucinated deletions and prompt injection attacks from indexed untrusted files.

### Decision
Integrate an optional connection to local **Ollama** instances (`http://127.0.0.1:11434`), operating strictly as an **advisory-only copilot**. The copilot cannot execute filesystem write operations. When it suggests cleanups, it outputs typed JSON proposals that must pass through the deterministic Rust Safety Engine and receive explicit UI confirmation from the user.

### Consequences
- **Pros:**
  - Eliminates prompt injection risk: malicious text inside indexed files cannot cause unauthorized deletion.
  - Eliminates cloud dependency: zero API keys, zero monthly fees, zero telemetry.
  - Complete graceful degradation: if Ollama is absent, the application remains 100% functional without degraded core features.
- **Cons:**
  - User must independently install and run Ollama if they desire natural-language copilot interactions.

---

## ADR-004: Two-Stage Per-Volume Quarantine Vault with Rollback over Direct Deletion

### Context
Accidental file deletion is the primary fear users experience when running disk cleanup software. Direct unrecoverable deletion (`std::fs::remove_file`) carries extreme liability. Windows Recycle Bin (`IFileOperation`) can be unpredictable, has strict size caps, and often fails silently on large developer folders.

### Decision
Implement a **Two-Stage Atomic Quarantine Engine**:
1. Files scheduled for cleanup are moved via Win32 `MoveFileExW` into a dedicated `.filemind_quarantine` vault created at the root of the respective volume (`C:\.filemind_quarantine`, `D:\.filemind_quarantine`).
2. An atomic manifest records original path, timestamps, ACLs, and cryptographic hash.
3. Users can restore any item or entire batches with one click.
4. Permanent purges only occur after explicit user instruction.

### Consequences
- **Pros:**
  - Zero data loss liability: operations are 100% reversible.
  - Instantaneous performance: moves within the same NTFS volume are zero-copy metadata updates.
  - Complete audit trail preserved in SQLite.
- **Cons:**
  - Disk space is not immediately freed until the user explicitly purges the quarantine vault or retention policy expires.

---

## ADR-005: Deterministic Safety Rule Engine with "Unknown is Unsafe" Default

### Context
Traditional cleaners use probabilistic heuristics (e.g., "if folder contains 'temp', delete it"). Such heuristics regularly destroy critical developer assets, game modifications, or uncommitted project data.

### Decision
Implement an evidence-based, deterministic Safety Engine where **"Unknown is Unsafe"**. If a file does not match a verified positive rule (e.g., `target/` with a valid `Cargo.toml` in a clean Git repo), it cannot be categorized as Safe (Tier 1).

### Consequences
- **Pros:**
  - Guarantees zero false-positive deletions of user documents or uncommitted code.
  - Predictable, explainable rationale provided for every file evaluation.
- **Cons:**
  - May classify some obscure cache directories as "Caution" or "High Risk", requiring manual user confirmation. This is a deliberate and desired trade-off.

---

## ADR-006: Warm Ivory & Paper Neutral Impeccable Design System over AI Dashboard Tropes

### Context
Modern web and desktop AI applications overwhelmingly gravitate toward a monotonous aesthetic: dark navy or near-black backgrounds, electric purple and cyan gradients, glowing borders, floating holographic orbs, and glassmorphism. This aesthetic communicates gimmickry, causes visual fatigue during detailed file inspection, and fails accessibility contrast standards.

### Decision
Adopt the **Impeccable Design Standard** in **Operate Mode**:
- Surfaces: Warm ivory paper (`#FBF9F5`) and warm neutral layers (`#F3EFEA`, `#EAE4DC`).
- Typography: High-contrast graphite (`#1F2022`) and warm charcoal with tabular figures.
- Accents: Muted sage green (`#4A6B5B`) for primary actions; muted terracotta (`#B4533C`) for alerts.
- Dark Theme: Complementary warm charcoal (`#18191A`).
- Strict Refusal List: Zero neon glows, zero gradient text, zero floating orbs, zero fake terminal windows.

### Consequences
- **Pros:**
  - Creates a distinct, timeless, premium utility brand identity.
  - Maximizes legibility and information density in large data tables and treemaps.
  - Satisfies WCAG AA contrast standards across all components.
- **Cons:**
  - Rejects off-the-shelf flashy component kits; requires intentional token discipline.

---

## ADR-007: BLAKE3 Cryptographic Hashing with Multi-Stage Sparse Pipeline

### Context
Duplicate detection requires hashing files. SHA-256 and MD5 are too slow for multi-gigabyte files; CRC32 is too collision-prone. Scanning entire multi-gigabyte files when they have completely different contents wastes disk I/O.

### Decision
Implement a multi-stage duplicate detection pipeline using **BLAKE3**:
1. Group files by exact byte size.
2. For candidates with matching sizes, compute a sparse BLAKE3 hash of the first 4KB and last 4KB.
3. For candidates matching sparse hashes, stream and compute the full BLAKE3 hash.
4. Verify NTFS hardlinks via MFT record IDs.

### Consequences
- **Pros:**
  - BLAKE3 is cryptographically secure and achieves >1.5 GB/s streaming throughput per core (up to 4x faster than SHA-256).
  - Sparse 4KB checks eliminate 95% of non-duplicates without reading the full file.
- **Cons:**
  - Adds `blake3` crate dependency to native Rust core.

---

## ADR-008: pnpm Package Manager and Node 22 for Frontend Tooling

### Context
Frontend package managers vary in disk footprint and performance. `npm` uses a flat `node_modules` structure that duplicates packages across projects. `yarn` has fragmented versions (v1 vs berry).

### Decision
Standardize on **pnpm** (installed: `10.13.1`) and **Node.js** (`v22.19.0`).

### Consequences
- **Pros:**
  - Hardlink-based content-addressable storage saves gigabytes of disk space during development.
  - Strict dependency resolution prevents phantom dependency bugs.
  - Exceptional installation speed.
- **Cons:**
  - Requires all developers to have pnpm installed (pre-installed in current environment).

