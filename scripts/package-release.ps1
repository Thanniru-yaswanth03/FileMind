# FileMind Release Engineering & Packaging Script
# Generates release artifacts for Windows x64: Portable Zip, checksums, and verification manifest.
param (
    [string]$Version = "0.2.0",
    [switch]$PortableOnly = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
Set-Location $RootDir

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   FILEMIND RELEASE PACKAGING PIPELINE (v$Version)          " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# 1. Environment and Toolchain Check
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:USERPROFILE\.w64devkit\w64devkit\bin;$env:PATH"

Write-Host "[1/5] Verifying toolchain & environment..." -ForegroundColor Yellow
$rustcVer = & rustc --version
$nodeVer = & node --version
Write-Host "      Rust: $rustcVer"
Write-Host "      Node: $nodeVer"

# 2. Build Frontend Production Assets
Write-Host "[2/5] Building frontend production bundle (Vite + React 19)..." -ForegroundColor Yellow
& pnpm run build
if ($LASTEXITCODE -ne 0) {
    throw "Frontend production build failed with exit code $LASTEXITCODE"
}

# 3. Build Native Rust Engine
Write-Host "[3/5] Compiling native release engine (filemind-core)..." -ForegroundColor Yellow
& cargo build --release --manifest-path crates/filemind-core/Cargo.toml
if ($LASTEXITCODE -ne 0) {
    throw "Cargo release build failed with exit code $LASTEXITCODE"
}

# 4. Assemble Release Staging Directory
$ReleaseDir = Join-Path $RootDir "releases\v$Version"
if (Test-Path $ReleaseDir) {
    Remove-Item -Recurse -Force $ReleaseDir
}
New-Item -ItemType Directory -Path $ReleaseDir -Force | Out-Null

$PortableStaging = Join-Path $RootDir "target\staging\FileMind-portable-x64"
if (Test-Path $PortableStaging) {
    Remove-Item -Recurse -Force $PortableStaging
}
New-Item -ItemType Directory -Path $PortableStaging -Force | Out-Null

Write-Host "[4/5] Staging portable distribution bundle..." -ForegroundColor Yellow

# Copy primary binary as FileMind.exe
Copy-Item "target\release\filemind-core.exe" (Join-Path $PortableStaging "FileMind.exe") -Force

# Copy production web assets
Copy-Item -Recurse "dist" (Join-Path $PortableStaging "dist") -Force

# Create portable.flag (redirects SQLite and quarantine to ./data/)
New-Item -ItemType File -Path (Join-Path $PortableStaging "portable.flag") -Force | Out-Null

# Copy license & write quickstart readme
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" $PortableStaging -Force
}

$readmeContent = @"
========================================================================
FILEMIND v$Version — PORTABLE ZERO-INSTALL EDITION FOR WINDOWS
========================================================================

FileMind is a local-first Windows storage intelligence utility and 
conservative file safety engine.

HOW TO RUN:
1. Double-click 'FileMind.exe' to start FileMind.
2. The application will launch and automatically open http://localhost:41732
   in your default web browser (or WebView2).
3. Core features run 100% locally with zero cloud account requirements.

DATA RETENTION:
Because 'portable.flag' is present, all FileMind data is stored in:
   .\data\filemind.db    (SQLite storage index & audit log)
   .\data\Quarantine\   (Safety quarantine vault)
No files are written to %LOCALAPPDATA% or the Windows Registry.

CLEAN UNINSTALL:
Simply delete this folder. Zero registry remnants or background daemons remain.

SECURITY & VERIFICATION:
Verify the SHA-256 hash using PowerShell:
   Get-FileHash .\FileMind.exe -Algorithm SHA256

PROJECT REPOSITORY:
https://github.com/vibe-filemind/filemind
========================================================================
"@

Set-Content -Path (Join-Path $PortableStaging "README.txt") -Value $readmeContent -Encoding UTF8

# Compress Portable Bundle
$ZipOutput = Join-Path $ReleaseDir "FileMind-v$Version-windows-x64-portable.zip"
Write-Host "      Compressing portable package to: $ZipOutput"
Compress-Archive -Path "$PortableStaging\*" -DestinationPath $ZipOutput -Force

# 5. Generate Cryptographic Checksums (SHA256SUMS.txt)
Write-Host "[5/5] Generating cryptographic SHA-256 checksums..." -ForegroundColor Yellow
$hashList = @()

Get-ChildItem $ReleaseDir -Filter "*.zip" | ForEach-Object {
    $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower()
    $entry = "$hash  $($_.Name)"
    $hashList += $entry
    Write-Host "      $($_.Name): $hash" -ForegroundColor Green
}

$checksumPath = Join-Path $ReleaseDir "SHA256SUMS.txt"
$hashList | Set-Content -Path $checksumPath -Encoding UTF8

# Write PowerShell verification helper script
$verifyScript = @"
# FileMind Checksum Verification Script
`$scriptDir = Split-Path -Parent `$MyInvocation.MyCommand.Path
if (-not `$scriptDir) { `$scriptDir = "." }
`$checksumsFile = Join-Path `$scriptDir "SHA256SUMS.txt"
if (-not (Test-Path `$checksumsFile)) {
    Write-Host "[ERROR] SHA256SUMS.txt not found in `$scriptDir" -ForegroundColor Red
    exit 1
}
`$checksums = Get-Content `$checksumsFile
foreach (`$line in `$checksums) {
    if (`$line -match "^([a-f0-9]{64})\s+(.+)$") {
        `$expected = `$matches[1]
        `$filename = `$matches[2]
        `$filePath = Join-Path `$scriptDir `$filename
        if (Test-Path `$filePath) {
            `$actual = (Get-FileHash `$filePath -Algorithm SHA256).Hash.ToLower()
            if (`$actual -eq `$expected) {
                Write-Host "[VERIFIED OK] `$filename matches `$expected" -ForegroundColor Green
            } else {
                Write-Host "[MISMATCH ERROR] `$filename CHECKSUM FAILED!" -ForegroundColor Red
                Write-Host "  Expected: `$expected"
                Write-Host "  Actual:   `$actual"
            }
        } else {
            Write-Host "[NOT FOUND] `$filePath" -ForegroundColor Yellow
        }
    }
}
"@

Set-Content -Path (Join-Path $ReleaseDir "Verify-Checksums.ps1") -Value $verifyScript -Encoding UTF8

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   PACKAGING COMPLETE: Artifacts saved to releases\v$Version  " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
