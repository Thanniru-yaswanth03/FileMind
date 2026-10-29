# FileMind Checksum Verification Script
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $scriptDir) { $scriptDir = "." }
$checksumsFile = Join-Path $scriptDir "SHA256SUMS.txt"
if (-not (Test-Path $checksumsFile)) {
    Write-Host "[ERROR] SHA256SUMS.txt not found in $scriptDir" -ForegroundColor Red
    exit 1
}
$checksums = Get-Content $checksumsFile
foreach ($line in $checksums) {
    if ($line -match "^([a-f0-9]{64})\s+(.+)$") {
        $expected = $matches[1]
        $filename = $matches[2]
        $filePath = Join-Path $scriptDir $filename
        if (Test-Path $filePath) {
            $actual = (Get-FileHash $filePath -Algorithm SHA256).Hash.ToLower()
            if ($actual -eq $expected) {
                Write-Host "[VERIFIED OK] $filename matches $expected" -ForegroundColor Green
            } else {
                Write-Host "[MISMATCH ERROR] $filename CHECKSUM FAILED!" -ForegroundColor Red
                Write-Host "  Expected: $expected"
                Write-Host "  Actual:   $actual"
            }
        } else {
            Write-Host "[NOT FOUND] $filePath" -ForegroundColor Yellow
        }
    }
}
