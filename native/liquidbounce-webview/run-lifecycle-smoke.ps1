param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '../../build/cef-lifecycle-01'),
    [ValidateRange(0, 60)][int]$SettleSeconds = 0
)
$ErrorActionPreference = 'Stop'
# Run an already built native fixture. Build/staging remains the caller's responsibility.
$taskExecutablePath = [IO.Path]::GetFullPath($Executable)
if (!(Test-Path -LiteralPath $taskExecutablePath -PathType Leaf)) {
    throw "Native smoke executable is missing: $taskExecutablePath"
}
$taskExecutableDirectory = Split-Path -Parent $taskExecutablePath
foreach ($taskResource in @('liquidbounce-cef-helper.exe', 'libcef.dll', 'icudtl.dat')) {
    $taskResourcePath = Join-Path $taskExecutableDirectory $taskResource
    if (!(Test-Path -LiteralPath $taskResourcePath -PathType Leaf)) {
        throw "Required native CEF runtime file is missing: $taskResourcePath"
    }
}
$taskOutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $taskOutputDirectory -Force | Out-Null
$taskImagePath = Join-Path $taskOutputDirectory 'lifecycle.png'
$taskReportPath = Join-Path $taskOutputDirectory 'report.json'
& $taskExecutablePath '--lifecycle' $taskImagePath $SettleSeconds $taskReportPath
if ($LASTEXITCODE -ne 0) {
    throw "Native CEF lifecycle fixture failed with exit code $LASTEXITCODE; report: $taskReportPath"
}
Write-Output "Native CEF lifecycle fixture passed. PNG: $taskImagePath"
Write-Output "Evidence: $taskReportPath"
