param(
    [string] $Game = 'D:\Develop\MCL\.references\horizon\nightly\horizon-windows-x64\horizon-client.exe',
    [string] $Controller = 'D:\Develop\MCL\.references\horizon\nightly\horizon-windows-x64\horizon-ctl.exe',
    [switch] $Offscreen
)
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$testRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot 'build\game-test'))
if (-not $testRoot.StartsWith($projectRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Test directory must remain inside this project'
}
$resources = Join-Path $projectRoot 'build\resources\26.3'
if (-not (Test-Path -LiteralPath $Game -PathType Leaf)) { throw "Game binary missing: $Game" }
if (-not (Test-Path -LiteralPath $resources -PathType Container)) { throw 'Prepare isolated resources first' }
foreach ($folder in @('home', 'launcher', 'game', 'data', 'config', 'trash', 'clipboard', 'desktop')) {
    New-Item -ItemType Directory -Path (Join-Path $testRoot $folder) -Force | Out-Null
}
$env:HORIZON_HOME = Join-Path $testRoot 'home'
$env:HORIZON_LAUNCHER_HOME = Join-Path $testRoot 'launcher'
$env:XDG_DATA_HOME = Join-Path $testRoot 'data'
$env:XDG_CONFIG_HOME = Join-Path $testRoot 'config'
$env:HORIZON_LAUNCHER_TRASH = Join-Path $testRoot 'trash'
$env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $testRoot 'clipboard'
$env:HORIZON_LAUNCHER_DESKTOP = Join-Path $testRoot 'desktop'
$env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
$env:HORIZON_NO_AUDIO = '1'
$socket = Join-Path $testRoot 'horizon-control.json'
$gameDir = Join-Path $testRoot 'game'
$launchArguments = @('--socket', $socket, '--timeout', '30', 'launch', '--game', $Game,
    '--world', 'LiquidBouncePortTest', '--seed', '42', '--creative', '--resources', $resources,
    '--game-dir', $gameDir, '--name', 'PortTest', '--width', '960', '--height', '540',
    '--render-distance', '4', '--test-terrain', '--timeout-seconds', '300')
if ($Offscreen) { $launchArguments += '--offscreen' }
Push-Location -LiteralPath $gameDir
try { & $Controller @launchArguments }
finally { Pop-Location }
exit $LASTEXITCODE
