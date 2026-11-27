#requires -Version 7.0
[CmdletBinding()]
param(
    [string] $LauncherExe,
    [string] $LauncherSource,
    [string] $Resources,
    [string] $Destination,
    [switch] $ResourcesOnly
)
$ErrorActionPreference = 'Stop'
$taskProject = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if (!$Resources) {
    $taskPortable = Join-Path $taskProject 'build/portable-resources/26.3'
    $Resources = if (Test-Path -LiteralPath (Join-Path $taskPortable 'portable-assets.json') -PathType Leaf) {
        $taskPortable
    } else { Join-Path $taskProject 'build/resources/26.3' }
}
if (!$Destination) { $Destination = Join-Path $taskProject '../ReleasesPack/liquidbounce-horizon-263-native-20261127' }
if (!$LauncherSource) { $LauncherSource = Join-Path $taskProject 'native/liquidbounce-launcher' }
$taskArguments = @((Join-Path $PSScriptRoot 'package-liquidbounce-launcher.py'),
    '--resources', $Resources, '--destination', $Destination, '--source', $LauncherSource)
if ($ResourcesOnly) { $taskArguments += '--resources-only' }
elseif (!$LauncherExe) { throw 'Provide -LauncherExe pointing to the compiled Rust launcher.' }
else { $taskArguments += @('--launcher', $LauncherExe) }
python @taskArguments
if ($LASTEXITCODE -ne 0) { throw 'Launcher release packaging failed.' }
