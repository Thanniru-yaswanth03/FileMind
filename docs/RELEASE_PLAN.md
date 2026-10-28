# FileMind — Release Engineering & Distribution Plan

**Document Version:** 1.0.0  
**Target Platform:** Windows 10 & Windows 11 (x64 and ARM64)  
**Status:** Approved Release Specification  

---

## 1. Release Architecture & Packaging Matrix

FileMind is distributed as a self-contained, native Windows desktop application packaged via **Tauri 2**.

```
┌─────────────────────────────────────────────────────────────┐
│ GitHub Tag Push (vX.Y.Z)                                    │
└───────────────┬─────────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────────┐
│ GitHub Actions Runner (windows-latest)                      │
│ 1. pnpm build (Vite + React 19 Frontend)                    │
│ 2. cargo tauri build --release                              │
│ 3. Sign Executables (Microsoft Authenticode / EV)           │
│ 4. Sign Update Artifacts (Tauri Ed25519 Private Key)        │
└───────────────┬─────────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────────┐
│ Distribution Artifacts                                      │
│ • FileMind_x64_en-US.msi (Enterprise / System Installer)    │
│ • FileMind_x64_setup.exe (NSIS Standard Installer)          │
│ • FileMind_portable_x64.zip (Zero-Install Portable Archive) │
│ • latest.json (Tauri Auto-Updater Manifest)                 │
└─────────────────────────────────────────────────────────────┘
```

### 1.1 Distribution Formats
1. **NSIS Installer (`.exe`):**
   - Recommended for general desktop users.
   - Installs to `%LOCALAPPDATA%\Programs\FileMind` without requiring UAC administrator rights.
   - Creates Start Menu shortcuts, optional desktop shortcut, and registers in Windows "Installed Apps".
2. **WiX MSI Installer (`.msi`):**
   - Targeted at enterprise environments, IT administrators, and scripted Group Policy deployments.
   - Supports silent installation flags (`/quiet /qn`).
3. **Portable Archive (`.zip`):**
   - Standalone directory with zero registry pollution.
   - Detects existence of a `portable.flag` file beside the binary: when present, FileMind redirects its SQLite database and config directory to `./data/` instead of `%LOCALAPPDATA%`.

---

## 2. Code Signing & Windows SmartScreen

Unsigned Windows binaries trigger severe SmartScreen warning prompts ("Windows protected your PC"). FileMind enforces strict Authenticode signing:

### 2.1 Authenticode Code Signing Architecture
- **Production Releases:** Signed using **Azure Trusted Signing** (Microsoft's modern cloud-based HSM code signing service) or an EV (Extended Validation) hardware token.
- **Timestamping:** Every signature is timestamped via DigiCert/RFC 3161 (`http://timestamp.digicert.com`) ensuring binaries remain valid even after the signing certificate expires.
- **Staging / Development Builds:** Signed with an internal self-signed development certificate (`FileMindDevCert.pfx`) to validate updater workflows in staging environments.

---

## 3. Tauri Auto-Updater Infrastructure

FileMind includes a built-in, secure, privacy-respecting auto-updater powered by Tauri 2's native updater plugin.

### 3.1 Ed25519 Cryptographic Signatures
- Every release artifact generates a detached Ed25519 signature file (`.sig`).
- The application binary embeds the public key inside `tauri.conf.json`:
  ```json
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk...",
      "endpoints": [
        "https://github.com/vibe-filemind/filemind/releases/latest/download/latest.json"
      ]
    }
  }
  ```
- Before executing any binary replacement, the background updater verifies:
  1. HTTPS TLS certificate of the download endpoint.
  2. SHA256 content hash of the downloaded installer.
  3. Ed25519 signature verified against the embedded public key.
- **User Agency:** The user can configure the update policy in Settings:
  - *Automatic Check & Notify (Default):* Informs the user when an update is ready; prompts before restarting.
  - *Manual Only:* Never polls GitHub for updates; completely silent.

---

## 4. GitHub Actions CI/CD Workflow

The release workflow is triggered automatically whenever a semantic release tag is pushed (`git push origin v1.0.0`):

```yaml
name: Release FileMind

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  build-and-release:
    permissions:
      contents: write
    runs-on: windows-latest
    steps:
      - name: Checkout Repository
        uses: actions/checkout@v4

      - name: Setup Node.js 22
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Setup pnpm
        uses: pnpm/action-setup@v3
        with:
          version: 10

      - name: Setup Rust Stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc

      - name: Install Frontend Dependencies
        run: pnpm install --frozen-lockfile

      - name: Run Test Suites
        run: |
          pnpm test
          cargo test --all

      - name: Build and Package Tauri App
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_KEY_PWD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'FileMind ${{ github.ref_name }}'
          releaseBody: 'See CHANGELOG.md for detailed release notes.'
          releaseDraft: false
          prerelease: false
```

---

## 5. Versioning & Deprecation Policy

FileMind adheres strictly to **Semantic Versioning 2.0.0** (`MAJOR.MINOR.PATCH`):
- **PATCH Releases (e.g., 1.0.1):** Bug fixes, safety rule additions, performance optimizations. Zero schema or breaking changes.
- **MINOR Releases (e.g., 1.1.0):** New features (e.g., new developer ecosystem scanner, new visualizer chart mode), backward-compatible database schema migrations.
- **MAJOR Releases (e.g., 2.0.0):** Significant architectural redesigns or breaking changes to database schemas or configuration files.

### 5.1 Rollback & Recovery Invariant
In the event of an unexpected release regression, the previous version installer remains permanently accessible on GitHub Releases, and database schema migrations are designed with backward compatibility so that rolling back the binary does not corrupt user databases or quarantine vaults.

