param(
    [string] $Name = ('native-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff')),
    [string] $Controller = 'D:\Develop\MCL\.references\horizon\nightly\horizon-windows-x64\horizon-ctl.exe',
    [int] $TimeoutSeconds = 900,
    [switch] $Offscreen
)
$ErrorActionPreference = 'Stop'
if ($Name -notmatch '^[A-Za-z][A-Za-z0-9_-]{0,63}$') { throw 'Name must be a simple test folder name' }
if ($TimeoutSeconds -lt 30) { throw 'TimeoutSeconds must be at least 30' }
$taskProject = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskRoot = [IO.Path]::GetFullPath((Join-Path $taskProject ('build/game-runs/' + $Name)))
if (!$taskRoot.StartsWith($taskProject + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Test folder must be inside this project'
}
if (Test-Path -LiteralPath $taskRoot) { throw 'Choose a fresh Name to keep each test isolated' }
$taskRuntime = Join-Path $taskProject 'build/staging/horizon/target/debug'
$taskGame = Join-Path $taskRuntime 'liquidbounce-native-game.exe'
$taskResources = Join-Path $taskProject 'build/resources/26.3'
$taskTheme = Join-Path $taskProject 'build/themes/chinese/dist'
$taskThemeReport = Join-Path $taskProject 'build/themes/chinese/localization-build.json'
if (!(Test-Path -LiteralPath $taskThemeReport -PathType Leaf)) { throw 'Build the localized theme with python scripts/build-localized-theme.py --build first' }
$taskThemeMetadata = Get-Content -LiteralPath $taskThemeReport -Raw | ConvertFrom-Json
if (!$taskThemeMetadata.buildCompleted) { throw 'The localized theme build did not complete' }
foreach ($taskEntry in $taskThemeMetadata.outputFiles.PSObject.Properties) {
    $taskOutputFile = Join-Path $taskTheme $taskEntry.Name
    if (!(Test-Path -LiteralPath $taskOutputFile -PathType Leaf) -or
        (Get-FileHash -LiteralPath $taskOutputFile -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskEntry.Value) {
        throw "Localized theme output changed: $($taskEntry.Name)"
    }
}
foreach ($taskRequired in @($taskGame, (Join-Path $taskRuntime 'liquidbounce-cef-helper.exe'),
    (Join-Path $taskRuntime 'libcef.dll'), (Join-Path $taskRuntime 'icudtl.dat'),
    (Join-Path $taskRuntime 'resources.pak'), (Join-Path $taskRuntime 'chrome_100_percent.pak'),
    (Join-Path $taskRuntime 'chrome_200_percent.pak'), (Join-Path $taskRuntime 'locales/en-US.pak'),
    (Join-Path $taskTheme 'index.html'), $Controller)) {
    if (!(Test-Path -LiteralPath $taskRequired -PathType Leaf)) { throw "Required file missing: $taskRequired" }
}
if (!(Test-Path -LiteralPath $taskResources -PathType Container)) { throw 'Prepare isolated game resources first' }
foreach ($taskFolder in @('home', 'launcher', 'game', 'data', 'config', 'trash', 'clipboard', 'desktop', 'browser-cache', 'liquidbounce')) {
    New-Item -ItemType Directory -Path (Join-Path $taskRoot $taskFolder) | Out-Null
}
$env:HORIZON_HOME = Join-Path $taskRoot 'home'
$env:HORIZON_LAUNCHER_HOME = Join-Path $taskRoot 'launcher'
$env:XDG_DATA_HOME = Join-Path $taskRoot 'data'
$env:XDG_CONFIG_HOME = Join-Path $taskRoot 'config'
$env:HORIZON_LAUNCHER_TRASH = Join-Path $taskRoot 'trash'
$env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $taskRoot 'clipboard'
$env:HORIZON_LAUNCHER_DESKTOP = Join-Path $taskRoot 'desktop'
$env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
$env:HORIZON_NO_AUDIO = '1'
$env:LIQUIDBOUNCE_THEME_ROOT = $taskTheme
$env:LIQUIDBOUNCE_CEF_RESOURCES = $taskRuntime
$env:LIQUIDBOUNCE_CONFIG_DIR = Join-Path $taskRoot 'liquidbounce'
$env:LIQUIDBOUNCE_CACHE_DIR = Join-Path $taskRoot 'browser-cache'
$taskSocket = Join-Path $taskRoot 'horizon-control.json'
$taskGameDir = Join-Path $taskRoot 'game'
$taskArguments = @('--socket', $taskSocket, '--timeout', '60', 'launch', '--wait', '--game', $taskGame,
    '--world', 'LiquidBouncePortTest', '--seed', '42', '--creative', '--resources', $taskResources,
    '--game-dir', $taskGameDir, '--name', 'PortTest', '--width', '960', '--height', '540',
    '--render-distance', '4', '--test-terrain', '--timeout-seconds', [string]$TimeoutSeconds)
if ($Offscreen) { $taskArguments += '--offscreen' }
Push-Location -LiteralPath $taskGameDir
try {
    $taskLaunchOutput = & $Controller @taskArguments
    if ($LASTEXITCODE -ne 0) { throw "Game failed to launch; inspect $taskRoot" }
    $taskLaunchState = $taskLaunchOutput | ConvertFrom-Json
    $taskLaunchResult = [pscustomobject]@{
        root = $taskRoot
        socket = $taskSocket
        interop = (Join-Path $env:LIQUIDBOUNCE_CONFIG_DIR 'interop.json')
        game = $taskGame
        processId = $taskLaunchState.launch.pid
        requestedWorld = 'LiquidBouncePortTest'
        screen = $taskLaunchState.screen
        launchedUtc = [DateTimeOffset]::UtcNow.ToString('o')
        themeKind = $taskThemeMetadata.themeKind
        themeOutputSha256 = $taskThemeMetadata.outputSha256
    }
    $taskLaunchResult | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'launch-result.json') -Encoding utf8
    $taskLaunchResult | ConvertTo-Json
} finally { Pop-Location }
