# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Tauri 2, Rust (native engine), React 19, TypeScript, Vite, Tailwind CSS with CSS Variables, SQLite with FTS5 (`rusqlite`), Optional Local LLM (Ollama)

## Users

Power users, software engineers, creators, and professionals on Windows who deal with sprawling disk usage, massive developer caches (`node_modules`, `.cargo`, build targets, virtualenvs, Docker artifacts), duplicate files, and cluttered storage, and who require absolute safety and determinism when inspecting and reclaiming space.

## Product Purpose

FileMind is a local-first, privacy-respecting Windows desktop utility that unites high-performance storage visualization, metadata indexing, duplicate detection, dependency-aware safety scoring, intelligent cleanup planning, and recoverable quarantine operations. It eliminates the anxiety of accidental file deletion while empowering users to understand and manage their storage footprint without cloud telemetry, subscriptions, or mandatory AI models.

## Positioning

Unlike conventional disk cleaners that use opaque heuristics or blanket deletion scripts, FileMind is an evidence-based, deterministic storage intelligence tool. It never treats "unknown" as "safe", insists on staging operations through a reversible quarantine engine, and treats the optional AI subsystem strictly as an advisory copilot running through typed tool interfaces with zero direct filesystem privileges.

## Operating Context

- Windows 10/11 desktop workstations and laptops.
- Deep nested directory hierarchies, NTFS junctions, symlinks, hidden system attributes, locked files, and permission boundaries.
- Massive developer directories (`node_modules`, Python venvs, Rust `target/`, Gradle/Maven caches, IDE caches, Docker desktop storage).
- Multi-drive environments (NVMe system drives, secondary SSDs, external backup HDDs, network shares).
- Offline or air-gapped environments where external network requests are restricted or prohibited.

## Capabilities and Constraints

- **Offline & Local-First:** All database queries, indexing, analysis, and rule evaluations execute strictly on-device. Zero telemetry, zero mandatory internet connection.
- **Tauri 2 & Rust Engine:** Native Rust threads handle low-level Windows APIs (NTFS traversal, file attributes, hashing, canonicalization, recycle bin / quarantine vaulting), exposing strictly typed IPC commands to the web frontend.
- **Deterministic Safety Engine:** Every deletion or cleanup target is scored against system safety registries, active processes, file locks, parent project roots, and package manifests. "Unknown safety" is rejected by default.
- **Staged Quarantine:** Destructive deletes are strictly gated behind a two-stage quarantine workflow: soft quarantine with manifest recording and zero-data-loss rollback capability before any permanent purge.
- **Isolated Advisory Copilot:** An optional JARVIS-style assistant connects to local Ollama endpoints. The model possesses zero raw filesystem access; it interacts exclusively via read-only inspection tools and structured proposal generation.
- **Performance Budget:** 60fps UI rendering; background scans execute with low I/O priority (`IDLE_PRIORITY_CLASS` / background thread priority) to prevent system freezing.

## Brand Commitments

- **Tone & Identity:** Premium desktop utility software. High craft, quiet confidence, precision, earned trust, and deep respect for the user's data.
- **Anti-Patterns:** No neon gradients, no cyberpunk AI orbs, no fake terminal windows, no noisy glassmorphism, no marketing fluff or telemetry prompts.
- **Visual Palette:** Warm ivory and paper-like neutral surfaces, graphite and warm charcoal typography, restrained muted sage green primary accent, muted terracotta for alerts/warnings.

## Evidence on Hand

- Windows 11 host environment with Node.js 22.19.0, npm 11.2.0, pnpm 10.13.1, and WebView2 Runtime installed.
- Impeccable design system engine available in the local configuration.
- Missing prerequisites documented: Rust toolchain (`rustup`/`cargo`) and Visual Studio C++ Build Tools (MSVC v143 + Windows 11 SDK).

## Product Principles

1. **Safety Over Speed:** Preserving user data is an absolute invariant. When in doubt, halt and request verification; unknown safety is never safe.
2. **Deterministic Truth:** The UI and engine reflect verified filesystem and database states, not speculative or hallucinated data.
3. **Earned Familiarity:** Interaction patterns must feel natural to Windows power users—compact tables, standard key shortcuts, clear breadcrumbs, and zero decorative friction.
4. **Resilient Under Absence:** If the optional AI subsystem is disabled, missing, or unconfigured, 100% of the core utility remains completely functional, responsive, and ergonomic.
5. **Transparency & Auditability:** Every cleanup action, quarantine move, and automated rule execution is logged into an append-only audit ledger with rollback manifests.

## Accessibility & Inclusion

- Keyboard navigability across all views (file lists, tree grids, action bars, dialogs).
- Full contrast compliance: body copy ≥ 4.5:1, large headings and active controls ≥ 3:1 against paper-neutral surfaces.
- High-visibility focus indicators with deliberate keyboard rings; support for Windows High Contrast mode and reduced motion preferences (`prefers-reduced-motion`).

