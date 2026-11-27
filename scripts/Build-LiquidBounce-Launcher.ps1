#requires -Version 7.0
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$taskProject = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskSource = Join-Path $taskProject 'native/liquidbounce-launcher'
$taskStage = Join-Path $taskProject 'build/staging/launcher'
New-Item -ItemType Directory -Force $taskStage | Out-Null
# Give the packaged binary its own name so a running development launcher does
# not lock Cargo's output. Sources, assets and the dependency lock stay intact.
foreach ($taskEntry in @('src', 'assets', 'Cargo.lock', 'build.rs')) {
    Copy-Item -LiteralPath (Join-Path $taskSource $taskEntry) -Destination $taskStage -Recurse -Force
}
$taskManifest = Get-Content -LiteralPath (Join-Path $taskSource 'Cargo.toml') -Raw
$taskManifest = $taskManifest.Replace('[package]', "[package]`nautobins = false")
$taskManifest += "`n[[bin]]`nname = 'LiquidBounce-Launcher'`npath = 'src/main.rs'`n"
Set-Content -LiteralPath (Join-Path $taskStage 'Cargo.toml') -Value $taskManifest -Encoding utf8
$taskOldTarget = $env:CARGO_TARGET_DIR
try {
    $env:CARGO_TARGET_DIR = Join-Path $taskSource 'target'
    cargo +stable build --release --locked --manifest-path (Join-Path $taskStage 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw 'Launcher build failed.' }
    Write-Output (Join-Path $env:CARGO_TARGET_DIR 'release/LiquidBounce-Launcher.exe')
} finally {
    $env:CARGO_TARGET_DIR = $taskOldTarget
}
