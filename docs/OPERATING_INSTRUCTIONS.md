# FileMind — Permanent Engineering Operating Instructions

These instructions define the mandatory engineering disciplines, quality benchmarks, and reporting protocols for the development and maintenance of FileMind.

---

## 1. Core Operating Principle

FileMind is a long-lived Windows desktop product. Engineering decisions must optimize for:
- **Correctness**
- **Maintainability**
- **Security**
- **Usability**
- **Verified Behavior**

Progress is measured by verified system behavior and software reliability, not by the raw volume of features generated per session.

---

## 2. Implementation Discipline

- **Read before writing:** Always read project documentation and architectural specifications before making changes.
- **Inspect before replacing:** Inspect existing implementations thoroughly before modifying or replacing them.
- **Preserve working code:** Never refactor or rewrite working subsystems without explicit justification.
- **Incremental verification:** Work in small, independently verifiable increments.
- **Modular boundaries:** Maintain clean, decoupled module boundaries between scanning, indexing, safety assessment, planning, quarantine, automation, and UI presentation.
- **Continuous runnability:** Keep the application compiling, runnable, and testable after each completed task or phase.
- **No phantom features:** Do not silently introduce unrelated features or expand scope without documentation.
- **No deceptive mocks:** Never substitute real functionality with mock data to make a screen appear finished.
- **Evidence over claims:** Never claim completion without executing concrete tests and inspecting output.

---

## 3. Design Discipline

- **Use the Impeccable standard:** Design work must adhere to the Impeccable design system (`docs/DESIGN_SYSTEM.md` and `DESIGN.md`).
- **Preserve desktop utility aesthetic:** Maintain FileMind's distinctive aesthetic:
  - Warm neutrals and paper-like surfaces (`#FBF9F5`, `#F3EFEA`, `#E8E2D9`).
  - High-contrast graphite typography (`#1F2022`, `#2C2D30`).
  - Restrained, purposeful accents: muted sage green (`#4A6B5B`) for positive/constructive indicators; muted terracotta (`#B4533C`) for warnings and destruction.
  - Tabular numerals (`font-variant-numeric: tabular-nums`) on all data tables, sizes, and timestamps.
- **Avoid design anti-patterns:** Reject generic AI-dashboard tropes (neon glows, purple gradients, excessive glassmorphism, floating chat bubbles, fake terminal outputs).
- **Usability benchmarks:** Review all UI changes for accessibility, keyboard navigability, semantic hierarchy, and functional clarity.

---

## 4. Security Discipline

- **Value user data:** Treat all user files, documents, repositories, and operating-system paths as precious and irreplaceable.
- **Conservative defaults:** Never assume a file is safe to delete because it is old, large, duplicated, or located in a folder with a familiar name.
- **Unified safety gateway:** All destructive actions (deletion, quarantine, purge) must pass through the deterministic Rust safety engine (`crates/filemind-core/src/safety/`).
- **No bypasses permitted:** The AI copilot, local automation engine, frontend UI, and CLI shortcuts must never be granted bypass permissions around the safety guardian.
- **Least privilege:** Avoid requiring administrator elevation for routine desktop scanning, search, and quarantine operations.

---

## 5. Testing Discipline

After each meaningful change:
1. Run relevant automated tests across the affected modules.
2. Inspect actual test logs and terminal output.
3. Fix verified regressions immediately before proceeding.
4. Add regression tests for every discovered bug or vulnerability.
5. Update documentation whenever behavioral contracts change.
6. Clearly document untested cases and environmental limitations.
7. **Never weaken or disable test assertions merely to achieve a passing build.**

---

## 6. Repository Discipline

- **Clean version history:** Maintain concise, documented commits with descriptive messages.
- **Zero data leakage:** Never commit API keys, personal credentials, private absolute paths, signing certificates, or generated local SQLite databases (`filemind.db`).
- **Release gate protocol:** Never push tags, publish releases, or modify remote GitHub repositories without explicit user authorization.

---

## 7. Mandatory Status Reporting Protocol

At the end of each engineering phase or major task, provide the following 9-point report:

1. **What was implemented:** Concrete technical additions and functional changes.
2. **Which files or modules changed:** Exact file paths and structural modifications.
3. **Which tests actually ran:** Test suites and specific commands executed.
4. **Which tests passed or failed:** Verified results and assertion metrics.
5. **Whether the application builds:** Build status of both frontend and native backend binaries.
6. **What remains incomplete:** Scope boundaries, pending integrations, and deferred tasks.
7. **Known security and reliability limitations:** Edge cases, unhandled environments, and failure modes.
8. **Decisions requiring approval:** Architectural questions, trade-offs, or external actions.
9. **Recommended next phase:** Logical, prioritized next engineering step.

