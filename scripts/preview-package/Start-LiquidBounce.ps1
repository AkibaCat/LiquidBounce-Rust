#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string] $Resources,
    [ValidatePattern('^[A-Za-z][A-Za-z0-9_-]{0,63}$')][string] $World,
    [ValidatePattern('^[A-Za-z0-9_]{1,16}$')][string] $PlayerName = 'PortTest',
    [ValidateRange(320, 7680)][int] $Width = 1280,
    [ValidateRange(240, 4320)][int] $Height = 720,
    [ValidateRange(2, 32)][int] $RenderDistance = 4,
    [switch] $Offscreen,
    [switch] $NoAudio
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$taskRoot = [IO.Path]::GetFullPath($PSScriptRoot)
$taskRuntime = Join-Path $taskRoot 'runtime'
$taskGame = Join-Path $taskRuntime 'liquidbounce-native-game.exe'
$taskTheme = Join-Path $taskRoot 'theme'
$taskData = Join-Path $taskRoot 'user-data'

function Assert-LocalPath {
    param([string] $Path)
    $taskFull = [IO.Path]::GetFullPath($Path)
    if (!$taskFull.Equals($taskRoot, [StringComparison]::OrdinalIgnoreCase) -and
        !$taskFull.StartsWith($taskRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Package data path escaped the package: $Path"
    }
    $taskCursor = $taskFull
    while ($taskCursor) {
        if (Test-Path -LiteralPath $taskCursor) {
            if (((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Package paths must not pass through a reparse point: $taskCursor"
            }
        }
        $taskParent = [IO.Directory]::GetParent($taskCursor)
        $taskCursor = if ($null -eq $taskParent) { $null } else { $taskParent.FullName }
    }
}
Assert-LocalPath $taskRoot
foreach ($taskRequired in @($taskGame, (Join-Path $taskRuntime 'liquidbounce-cef-helper.exe'),
    (Join-Path $taskRuntime 'libcef.dll'), (Join-Path $taskRuntime 'icudtl.dat'),
    (Join-Path $taskRuntime 'resources.pak'), (Join-Path $taskRuntime 'locales/en-US.pak'),
    (Join-Path $taskTheme 'index.html'))) {
    Assert-LocalPath $taskRequired
    if (!(Test-Path -LiteralPath $taskRequired -PathType Leaf)) { throw "Package file missing: $taskRequired" }
}
if (!(Test-Path -LiteralPath $Resources -PathType Container)) { throw "Prepared resources directory missing: $Resources" }
$taskResources = (Resolve-Path -LiteralPath $Resources).Path
foreach ($taskFolder in @('', 'home', 'launcher', 'game', 'player', 'data', 'config', 'trash', 'clipboard', 'desktop', 'browser-cache', 'liquidbounce')) {
    $taskPath = if ($taskFolder) { Join-Path $taskData $taskFolder } else { $taskData }
    Assert-LocalPath $taskPath
    New-Item -ItemType Directory -Path $taskPath -Force | Out-Null
}
$taskLockPath = Join-Path $taskData 'instance.lock'
Assert-LocalPath $taskLockPath
try { $taskLock = [IO.File]::Open($taskLockPath, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None) }
catch { throw "This preview is already running or its user-data directory cannot be locked: $($_.Exception.Message)" }
$taskEnvironment = [ordered]@{
    HORIZON_HOME = (Join-Path $taskData 'home')
    HORIZON_LAUNCHER_HOME = (Join-Path $taskData 'launcher')
    XDG_DATA_HOME = (Join-Path $taskData 'data')
    XDG_CONFIG_HOME = (Join-Path $taskData 'config')
    HORIZON_LAUNCHER_TRASH = (Join-Path $taskData 'trash')
    HORIZON_LAUNCHER_CLIPBOARD = (Join-Path $taskData 'clipboard')
    HORIZON_LAUNCHER_DESKTOP = (Join-Path $taskData 'desktop')
    HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
    LIQUIDBOUNCE_THEME_ROOT = $taskTheme
    LIQUIDBOUNCE_CEF_RESOURCES = $taskRuntime
    LIQUIDBOUNCE_CONFIG_DIR = (Join-Path $taskData 'liquidbounce')
    LIQUIDBOUNCE_CACHE_DIR = (Join-Path $taskData 'browser-cache')
}
if ($NoAudio) { $taskEnvironment['HORIZON_NO_AUDIO'] = '1' }
$taskPrevious = @{}
$taskExitCode = 1
try {
    foreach ($taskVariable in $taskEnvironment.Keys) {
        $taskPrevious[$taskVariable] = [Environment]::GetEnvironmentVariable($taskVariable, 'Process')
        [Environment]::SetEnvironmentVariable($taskVariable, $taskEnvironment[$taskVariable], 'Process')
    }
    $taskArguments = @('--resources', $taskResources,
        '--home', (Join-Path $taskData 'home'), '--game-dir', (Join-Path $taskData 'game'),
        '--player-dir', (Join-Path $taskData 'player'), '--control-socket', (Join-Path $taskData 'horizon-control.json'),
        '--name', $PlayerName,
        '--width', [string]$Width, '--height', [string]$Height, '--render-distance', [string]$RenderDistance)
    if ($PSBoundParameters.ContainsKey('World')) { $taskArguments += @('--world', $World, '--seed', '42', '--creative') }
    if ($Offscreen) { $taskArguments += '--offscreen' }
    Push-Location -LiteralPath (Join-Path $taskData 'game')
    try { & $taskGame @taskArguments; $taskExitCode = $LASTEXITCODE }
    finally { Pop-Location }
} finally {
    foreach ($taskVariable in $taskPrevious.Keys) {
        [Environment]::SetEnvironmentVariable($taskVariable, $taskPrevious[$taskVariable], 'Process')
    }
    $taskLock.Dispose()
}
exit $taskExitCode
