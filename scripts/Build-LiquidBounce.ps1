param(
    [switch] $Check,
    [switch] $TestEngine,
    [switch] $SkipCefBuild
)
$ErrorActionPreference = 'Stop'
if ($Check -and $TestEngine) { throw 'Choose one of Check or TestEngine' }
$taskProject = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location -LiteralPath $taskProject
try {
    python scripts/stage-horizon-build.py --webview
    if ($LASTEXITCODE -ne 0) { throw 'Horizon staging failed' }
    # Establish the pinned CEF tools and runtime before linking the game.
    if ($SkipCefBuild) {
        & (Join-Path $taskProject 'native/liquidbounce-webview/build-native.ps1') -PrepareOnly
    } else {
        & (Join-Path $taskProject 'native/liquidbounce-webview/build-native.ps1')
    }
    $env:CEF_PATH = Join-Path $taskProject 'build/cef-runtime'
    $env:CARGO_TARGET_DIR = Join-Path $taskProject 'build/staging/horizon/target'
    Push-Location -LiteralPath (Join-Path $taskProject 'build/staging/horizon')
    try {
        # The rendering crate consumes committed hook declarations directly;
        # generate those from the staged catalog before compiling its consumers.
        cargo +stable run -p xtask -- mod-api
        if ($LASTEXITCODE -ne 0) { throw 'Staged native render hook generation failed' }
        $taskStarted = [DateTimeOffset]::UtcNow
        if ($TestEngine) { cargo +stable test -p liquidbounce-horizon --lib --features behavior-hooks,webview }
        elseif ($Check) { cargo +stable check -p liquidbounce-native-game }
        else { cargo +stable build -p liquidbounce-native-game }
        $taskExitCode = $LASTEXITCODE
        $taskReport = [ordered]@{
            operation = $(if ($TestEngine) { 'test-engine' } elseif ($Check) { 'check' } else { 'build' })
            startedUtc = $taskStarted.ToString('o')
            finishedUtc = [DateTimeOffset]::UtcNow.ToString('o')
            exitCode = $taskExitCode
            passed = ($taskExitCode -eq 0)
            lockSha256 = (Get-FileHash -LiteralPath 'Cargo.lock' -Algorithm SHA256).Hash.ToLowerInvariant()
            renderHooksSha256 = (Get-FileHash -LiteralPath 'crates/horizon-client-render/src/gfx/native_render/generated.rs' -Algorithm SHA256).Hash.ToLowerInvariant()
            buildDriverSha256 = (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'Build-LiquidBounce.ps1') -Algorithm SHA256).Hash.ToLowerInvariant()
            staging = (Get-Content -LiteralPath (Join-Path $taskProject 'build/staging-report.json') -Raw | ConvertFrom-Json)
        }
        if (!$Check -and !$TestEngine -and $taskExitCode -eq 0) {
            $taskReport.executable = (Join-Path $env:CARGO_TARGET_DIR 'debug/liquidbounce-native-game.exe')
            $taskReport.executableSha256 = (Get-FileHash -LiteralPath $taskReport.executable -Algorithm SHA256).Hash.ToLowerInvariant()
        }
        $taskReport | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskProject ('build/integrated-' + $taskReport.operation + '.json')) -Encoding utf8
        if ($taskExitCode -ne 0) { throw 'Integrated LiquidBounce game build failed' }
    } finally { Pop-Location }
} finally { Pop-Location }
