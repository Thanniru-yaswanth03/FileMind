# FileMind — Windows Distribution, Installation & Verification Guide

**Release Version:** 0.2.0  
**Target Architecture:** Windows x64 (x86_64)  
**Supported OS:** Windows 10 (1809+ / 64-bit), Windows 11 (64-bit)

---

## 1. Choosing Your Release Artifact

FileMind offers three official distribution formats on [GitHub Releases](https://github.com/vibe-filemind/filemind/releases):

| Format | Artifact Filename Pattern | Best For | Elevated Rights Needed? |
| :--- | :--- | :--- | :--- |
| **Standard Setup** | `FileMind_<version>_x64-setup.exe` | General desktop users seeking Start Menu shortcuts and automatic updates. | **No** (Installs to user profile) |
| **Enterprise MSI** | `FileMind_<version>_x64_en-US.msi` | IT administrators, Group Policy deployments, Intune, or scripted silent rollouts. | **Yes** (Per-machine / Enterprise) |
| **Portable Zip** | `FileMind-v<version>-windows-x64-portable.zip` | Developers, USB flash drive storage, and users wanting zero registry or app data footprint. | **No** (Zero install) |

---

## 2. System Requirements & Prerequisites

- **Operating System:** Windows 10 (64-bit, Version 1809 Build 17763 or newer) or Windows 11 (64-bit).
- **Runtime:** Microsoft Edge WebView2 Evergreen Runtime.
  - *Note:* WebView2 is pre-installed on Windows 11 and all updated Windows 10 installations. If missing, install it from [Microsoft's Official WebView2 Download](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
- **Processor:** 64-bit Intel / AMD x86_64 processor.
- **Memory (RAM):** 4 GB minimum (8 GB recommended for indexing 500,000+ files).
- **Disk Space:** ~120 MB free space for the application; storage for the SQLite index scales proportionally (~5 MB per 100,000 indexed files).

---

## 3. Cryptographic Verification (SHA-256 Checksums)

Every release provides a detached `SHA256SUMS.txt` manifest. Before running any downloaded executable, verify its SHA-256 hash using Windows PowerShell:

```powershell
# Check the hash of your downloaded file:
Get-FileHash .\FileMind-v0.2.0-windows-x64-portable.zip -Algorithm SHA256

# Compare with the official hash listed in SHA256SUMS.txt:
# Example: 4a2b9f1d8c... FileMind-v0.2.0-windows-x64-portable.zip
```

Alternatively, extract the release bundle and run the included verification helper:
```powershell
powershell -ExecutionPolicy Bypass -File .\Verify-Checksums.ps1
```

---

## 4. Code Signing & Windows SmartScreen Notice

> [!IMPORTANT]
> **Windows Defender SmartScreen Notice:**
> Open-source community releases distributed directly via GitHub are signed with open-source workflow credentials or distributed unsigned until an Azure Trusted Signing / EV hardware token is configured.
>
> When launching a freshly downloaded binary, Windows SmartScreen may present a dialog:
> **"Windows protected your PC — Microsoft Defender SmartScreen prevented an unrecognized app from starting."**
>
> **How to Proceed Safely:**
> 1. Verify that the SHA-256 checksum matches the official hash published in `SHA256SUMS.txt`.
> 2. Click **"More info"** on the SmartScreen dialog.
> 3. Click **"Run anyway"**.
> Once community download reputation is established on Microsoft's telemetry network, this warning disappears automatically.

---

## 5. Installation Instructions

### Option A: Standard NSIS Installer (`.exe`)
1. Download `FileMind_<version>_x64-setup.exe`.
2. Double-click the installer. It installs to `%LOCALAPPDATA%\Programs\FileMind`.
3. An optional desktop icon and Start Menu entry named **"FileMind"** are created.
4. Launch FileMind from your Start Menu.

### Option B: Enterprise WiX MSI Installer (`.msi`)
1. Download `FileMind_<version>_x64_en-US.msi`.
2. Run interactive installation or execute silently via Command Prompt / PowerShell:
   ```cmd
   msiexec /i FileMind_0.2.0_x64_en-US.msi /quiet /qn /norestart
   ```

### Option C: Zero-Install Portable Edition (`.zip`)
1. Download `FileMind-v<version>-windows-x64-portable.zip`.
2. Right-click and select **"Extract All..."** to any directory (e.g. `C:\Tools\FileMind` or a USB drive).
3. Ensure `portable.flag` is present in the extracted folder.
4. Double-click `FileMind.exe`. The application starts immediately and stores all index and quarantine data inside `./data/`.

---

## 6. Clean Uninstallation & Data Retention

### Standard / MSI Installations
1. Open Windows **Settings** -> **Apps** -> **Installed Apps**.
2. Locate **FileMind** in the list and click **Uninstall**.
3. The uninstaller removes all program binaries, shortcuts, and updater files.
4. **Data Retention Prompt:** The uninstaller asks whether you wish to keep or delete your local storage index (`%LOCALAPPDATA%\FileMind\filemind.db`) and safety quarantine vault (`%LOCALAPPDATA%\FileMind\Quarantine`). If you intend to reinstall later or restore quarantined files, choose to retain your data.

### Portable Edition
Simply delete the extracted folder. Zero registry keys, system drivers, or lingering services exist on your machine.

---

## 7. Disaster Recovery & Manual Quarantine Restoration

If a file was quarantined and you need to recover it without running the FileMind UI:
1. Open your Quarantine directory:
   - Standard: `%LOCALAPPDATA%\FileMind\Quarantine\`
   - Portable: `.\data\Quarantine\`
2. Open `%LOCALAPPDATA%\FileMind\filemind.db` with any standard SQLite viewer (e.g. [DB Browser for SQLite](https://sqlitebrowser.org/)).
3. Inspect table `quarantine_records`: find the row matching your file to see `original_path` and `vault_filename`.
4. Copy the file from the Quarantine folder back to its `original_path`.

