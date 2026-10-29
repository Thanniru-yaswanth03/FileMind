# FileMind — Privacy Model & Local Data Architecture

**Version:** 0.2.0  
**Status:** Canonical Privacy Policy & Data Architecture  
**Core Principle:** 100% Local-First. Zero Mandatory Cloud Accounts. Zero Remote Telemetry.

---

## 1. Zero Remote Telemetry Guarantee

FileMind does **not** collect, transmit, or monetize personal data or telemetry:
- **No Analytics:** There are no analytics tracking SDKs (no Google Analytics, no PostHog, no Mixpanel).
- **No Crash Senders:** No unconsented automated error uploaders (no Sentry, no Bugsnag).
- **No Cloud Account:** All scanning, indexing, duplicate finding, safety classification, and cleanup features run completely offline.
- **No Background Phone-Home:** The application does not ping remote servers when scanning directories, reading files, or generating cleanup plans.

---

## 2. Local Filesystem Footprint

FileMind stores all persistent data on your local computer. It respects Windows operating system standards and provides an isolated portable mode:

### 2.1 Standard Installation Paths (Default)

| Asset | Local Windows Filesystem Path | Purpose |
| :--- | :--- | :--- |
| **SQLite Index Database** | `%LOCALAPPDATA%\FileMind\filemind.db` | Contains the file metadata index, scan sessions, FTS5 document index, and safety audit logs. |
| **SQLite Journal / WAL** | `%LOCALAPPDATA%\FileMind\filemind.db-wal` | Write-Ahead Log for high-performance concurrent database transactions. |
| **Safety Quarantine Vault** | `%LOCALAPPDATA%\FileMind\Quarantine\` | Stores files moved during safety-guarded cleanup operations. Preserves original metadata and permissions. |
| **User Settings & Automations** | Stored inside `filemind.db` | Contains automation rules, custom scan exclusions, theme preferences, and cooldown limits. |

### 2.2 Portable Mode (`portable.flag`)

When running from the portable distribution or if a file named `portable.flag` exists beside `FileMind.exe`:
- **SQLite Database:** `./data/filemind.db`
- **Quarantine Vault:** `./data/Quarantine/`
- **Footprint:** 100% self-contained inside the application folder. Zero files are written to `%LOCALAPPDATA%`, and zero keys are written to the Windows Registry.

---

## 3. How to Manage and Delete Your Data

### 3.1 How to Delete the Local Index
To completely wipe FileMind's database and index:
1. Open FileMind and navigate to **Settings** -> **Storage Index**.
2. Click **"Clear File Index"** or **"Prune Stale Sessions"**.
3. Alternatively, close FileMind and delete the folder `%LOCALAPPDATA%\FileMind` (or `./data` in portable mode).
4. On next launch, a clean database will be initialized automatically.

### 3.2 How to Exclude Sensitive Folders
You can exclude any folder or drive path from being scanned or indexed:
- **Built-in System Exclusions:** Core system paths (`$Recycle.Bin`, `System Volume Information`, `pagefile.sys`, `swapfile.sys`, `hiberfil.sys`, DOS device names `CON`, `NUL`, `AUX`) are automatically and permanently excluded by default.
- **Custom Exclusions:** In the top toolbar scan selector or in **Settings**, specify exclusion directories or path patterns (e.g. `C:\Users\<user>\PrivateDocs`). The scanner skips these paths entirely during traversal.

---

## 4. Optional AI Integrations & Network Behavior

FileMind includes an optional advisory assistant (JARVIS Desktop Copilot):

### 4.1 Local Inference (Ollama)
- FileMind connects to local Ollama instances running on your computer at `http://127.0.0.1:11434`.
- Queries, file paths, and summaries remain on your local machine and are processed by local GPU/CPU models.
- **Zero data is sent to external AI clouds.**

### 4.2 Offline Fallback Mode
- If Ollama is not installed or unreachable, FileMind automatically activates its built-in offline heuristic dispatcher.
- The advisory assistant answers core commands ("Find duplicates", "Summarize scan", "Explain safety rules") locally without network connectivity.

### 4.3 How to Disable AI Features
- In **Settings** -> **Copilot & AI Integrations**, toggle **"Enable Copilot"** to `OFF`.
- When disabled, no requests are made to local inference endpoints or Ollama ports.

### 4.4 Voice Synthesis & Recognition
- Voice input and output rely exclusively on the browser/WebView2 native Web Speech API (`SpeechRecognition` and `SpeechSynthesis`).
- FileMind does not transmit audio recordings to third-party cloud speech services. Voice features can be muted or toggled off with one click in the Copilot interface.

---

## 5. Diagnostic Log Export & Redaction

When exporting diagnostic logs or audit trails:
- **Redaction Protocol:** FileMind strips user profile names, environment credentials, private API keys, and sensitive tokens (`.env`, `id_rsa`, `.pem`) before writing diagnostic exports.
- **No Silent Upload:** Diagnostic bundles are saved to a `.txt` or `.json` file chosen by the user. They are never sent to remote endpoints automatically.

