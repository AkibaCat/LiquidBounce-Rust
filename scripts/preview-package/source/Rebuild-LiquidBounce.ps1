#requires -Version 7.0
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$taskSourceRoot = [IO.Path]::GetFullPath($PSScriptRoot)
Push-Location -LiteralPath $taskSourceRoot
try {
    python ./Restore-Source-Baselines.py
    if ($LASTEXITCODE -ne 0) { throw 'Pinned source restoration failed.' }
    python ./scripts/build-localized-theme.py --build
    if ($LASTEXITCODE -ne 0) { throw 'Localized theme build failed.' }
    python ./scripts/stage-horizon-build.py --webview
    if ($LASTEXITCODE -ne 0) { throw 'Isolated source staging failed.' }
    $taskRecordedLock = Join-Path $taskSourceRoot 'rebuild-locks/horizon-Cargo.lock'
    $taskStagedLock = Join-Path $taskSourceRoot 'build/staging/horizon/Cargo.lock'
    Copy-Item -LiteralPath $taskRecordedLock -Destination $taskStagedLock -Force
    & (Join-Path $taskSourceRoot 'scripts/Build-LiquidBounce.ps1')
    if ((Get-FileHash -LiteralPath $taskRecordedLock -Algorithm SHA256).Hash -ne
        (Get-FileHash -LiteralPath $taskStagedLock -Algorithm SHA256).Hash) {
        throw 'The rebuild changed the recorded Cargo.lock; inspect the new build before using it.'
    }
    Write-Output (Join-Path $taskSourceRoot 'build/staging/horizon/target/debug/liquidbounce-native-game.exe')
} finally { Pop-Location }
