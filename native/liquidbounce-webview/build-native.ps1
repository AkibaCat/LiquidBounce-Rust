param(
    [switch]$Typecheck,
    [switch]$Smoke,
    [switch]$PrepareOnly,
    [string]$Url = '--fixture',
    [string]$Output = (Join-Path $PSScriptRoot 'cef-osr-smoke.png'),
    [ValidateRange(0, 60)][int]$SettleSeconds = 2,
    [string]$Report = (Join-Path $PSScriptRoot 'native-smoke-verification.json')
)
$ErrorActionPreference = 'Stop'
$taskProjectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskStage = Join-Path $taskProjectRoot 'build/staging/horizon'
if (!(Test-Path (Join-Path $taskStage 'crates/horizon-hooks/src/gpu_ui.rs'))) {
    throw 'First prepare the isolated Horizon staging copy with scripts/stage-horizon-build.py --gpu-ui.'
}
$env:CARGO_TARGET_DIR = Join-Path $taskStage 'target'
Push-Location $PSScriptRoot
try {
    if ($Typecheck) {
        cargo check --features cef-typecheck,cef-smoke,horizon-gpu --all-targets
        if ($LASTEXITCODE -ne 0) { throw 'CEF Rust API type checking failed.' }
        return
    }
    $taskTools = Join-Path $PSScriptRoot 'build-tools'
    New-Item -ItemType Directory -Force $taskTools | Out-Null
    $taskCmakeArchive = Join-Path $taskTools 'cmake.whl'
    $taskNinjaArchive = Join-Path $taskTools 'ninja.zip'
    if (!(Test-Path $taskCmakeArchive)) {
        Invoke-WebRequest 'https://files.pythonhosted.org/packages/18/58/909d6d99acb4e0886d0f660cf4e0fb26f586590e370b2e4ce7a10d06b145/cmake-3.31.6-py3-none-win_amd64.whl' -OutFile $taskCmakeArchive
    }
    if (!(Test-Path $taskNinjaArchive)) {
        Invoke-WebRequest 'https://github.com/ninja-build/ninja/releases/download/v1.13.2/ninja-win.zip' -OutFile $taskNinjaArchive
    }
    foreach ($taskCheck in @(
        @{ Path = $taskCmakeArchive; Hash = 'bbaed969cef3c427f4f17591feb28db4ae595e3a4bbd45cb35522cee14df6a32' },
        @{ Path = $taskNinjaArchive; Hash = '07fc8261b42b20e71d1720b39068c2e14ffcee6396b76fb7a795fb460b78dc65' }
    )) {
        if ((Get-FileHash $taskCheck.Path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskCheck.Hash) {
            throw "Build tool SHA-256 mismatch: $($taskCheck.Path)"
        }
    }
    $taskCmakeBin = Join-Path $taskTools 'cmake/cmake/data/bin'
    $taskNinjaBin = Join-Path $taskTools 'ninja'
    if (!(Test-Path (Join-Path $taskCmakeBin 'cmake.exe'))) {
        [IO.Compression.ZipFile]::ExtractToDirectory($taskCmakeArchive, (Join-Path $taskTools 'cmake'), $true)
    }
    if (!(Test-Path (Join-Path $taskNinjaBin 'ninja.exe'))) {
        [IO.Compression.ZipFile]::ExtractToDirectory($taskNinjaArchive, $taskNinjaBin, $true)
    }
    if (!$env:VSCMD_VER) {
        $taskProgramFilesX86 = ${env:ProgramFiles(x86)}
        if (!$taskProgramFilesX86) {
            $taskProgramFilesX86 = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFilesX86)
        }
        $taskVsWhere = Join-Path $taskProgramFilesX86 'Microsoft Visual Studio/Installer/vswhere.exe'
        $taskVsRoot = & $taskVsWhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if (!$taskVsRoot) {
            $taskVsCandidate = & $taskVsWhere -latest -products '*' -property installationPath
            if ($taskVsCandidate -and (Test-Path (Join-Path $taskVsCandidate 'VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe'))) {
                $taskVsRoot = $taskVsCandidate
            }
        }
        if (!$taskVsRoot) {
            $taskProgramFiles = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles)
            $taskVsCandidates = Get-ChildItem -Path (Join-Path $taskProgramFiles 'Microsoft Visual Studio/*/*') -Directory -ErrorAction SilentlyContinue
            foreach ($taskCandidate in $taskVsCandidates) {
                if (Test-Path (Join-Path $taskCandidate.FullName 'VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe')) {
                    $taskVsRoot = $taskCandidate.FullName
                    break
                }
            }
        }
        if (!$taskVsRoot) { throw 'Visual Studio x64 C++ build tools are required by CEF.' }
        & (Join-Path $taskVsRoot 'Common7/Tools/Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
    }
    $env:PATH = $taskCmakeBin + ';' + $taskNinjaBin + ';' + $env:PATH
    $env:CEF_PATH = Join-Path $taskProjectRoot 'build/cef-runtime'
    if ($PrepareOnly) { return }
    # Deliberately excludes cef-typecheck/dox: this downloads and links real CEF 152.0.6.
    cargo build --features cef-smoke,horizon-gpu --bins
    if ($LASTEXITCODE -ne 0) { throw 'Native CEF helper/smoke build failed.' }
    if ($Smoke) {
        & (Join-Path $env:CARGO_TARGET_DIR 'debug/liquidbounce-cef-smoke.exe') $Url $Output $SettleSeconds $Report
        if ($LASTEXITCODE -ne 0) { throw 'Native CEF OnPaint smoke failed.' }
    }
} finally {
    Pop-Location
}
