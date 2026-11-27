#requires -Version 7.0
[CmdletBinding()]
param(
    [ValidatePattern('^[A-Za-z][A-Za-z0-9_-]{0,63}$')]
    [string] $Name = 'liquidbounce-horizon-preview',
    [string[]] $ImplementedModules = @(),
    [string] $ValidationReport
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$taskProject = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskArtifacts = Join-Path $taskProject 'artifacts'
$taskDestination = [IO.Path]::GetFullPath((Join-Path $taskArtifacts $Name))
$taskRuntime = Join-Path $taskProject 'build/staging/horizon/target/debug'
$taskTheme = Join-Path $taskProject 'build/themes/chinese/dist'
$taskCef = Join-Path $taskProject 'build/cef-runtime/152.0.6/cef_windows_x86_64'
$taskTemplates = Join-Path $PSScriptRoot 'preview-package'
$taskBuildReportPath = Join-Path $taskProject 'build/integrated-build.json'
$taskCefVersion = '152.0.6+g708dc14+chromium-152.0.7977.83'

function Assert-Within {
    param([string] $Path, [string] $Root)
    $taskFull = [IO.Path]::GetFullPath($Path)
    $taskPrefix = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    if (!$taskFull.StartsWith($taskPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Path must remain inside ${Root}: $Path"
    }
}

function Assert-NoReparse {
    param([string] $Path)
    $taskCursor = [IO.Path]::GetFullPath($Path)
    while ($taskCursor) {
        if (Test-Path -LiteralPath $taskCursor) {
            $taskItem = Get-Item -LiteralPath $taskCursor -Force
            if (($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Reparse points are not accepted for package paths: $taskCursor"
            }
        }
        $taskParent = [IO.Directory]::GetParent($taskCursor)
        $taskCursor = if ($null -eq $taskParent) { $null } else { $taskParent.FullName }
    }
}

function Get-TreeFiles {
    param([string] $Root)
    Assert-NoReparse $Root
    if (!(Test-Path -LiteralPath $Root -PathType Container)) { throw "Directory missing: $Root" }
    foreach ($taskEntry in Get-ChildItem -LiteralPath $Root -Force) {
        if (($taskEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Reparse point in package input: $($taskEntry.FullName)"
        }
        if ($taskEntry.PSIsContainer) { Get-TreeFiles $taskEntry.FullName }
        else { $taskEntry }
    }
}

function Get-Digest {
    param([string] $Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-SourceFiles {
    param([string] $Root)
    Assert-NoReparse $Root
    $taskExcluded = @('target', 'dist', 'node_modules', 'vendor', 'build-tools', '__pycache__', '.git', '.gradle', 'build', 'user-data')
    foreach ($taskEntry in Get-ChildItem -LiteralPath $Root -Force) {
        if (($taskEntry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Reparse point in source input: $($taskEntry.FullName)"
        }
        if ($taskEntry.PSIsContainer) {
            if ($taskEntry.Name -notin $taskExcluded) { Get-SourceFiles $taskEntry.FullName }
        } else { $taskEntry }
    }
}

function Get-TextDigest {
    param([string] $Text)
    $taskHasher = [Security.Cryptography.SHA256]::Create()
    try { $taskBytes = $taskHasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text)) }
    finally { $taskHasher.Dispose() }
    ([BitConverter]::ToString($taskBytes)).Replace('-', '').ToLowerInvariant()
}

function Assert-ModuleNames {
    param([string[]] $Names)
    if ($Names.Count -eq 0) { throw 'Provide ImplementedModules or a successful ValidationReport with finalStatus.modules.' }
    foreach ($taskModule in $Names) {
        if ($taskModule -notmatch '^[A-Za-z][A-Za-z0-9]{0,63}$') { throw "Invalid module name: $taskModule" }
    }
    if (@($Names | Sort-Object -Unique).Count -ne $Names.Count) { throw 'Module names must be unique.' }
}

Assert-Within $taskDestination $taskArtifacts
Assert-NoReparse $taskDestination
if (Test-Path -LiteralPath $taskDestination) {
    throw "Artifact already exists; choose a new -Name. Existing files are preserved: $taskDestination"
}
Assert-NoReparse $taskBuildReportPath
$taskBuild = Get-Content -LiteralPath $taskBuildReportPath -Raw | ConvertFrom-Json
if ($taskBuild.operation -ne 'build' -or $taskBuild.passed -ne $true -or $taskBuild.exitCode -ne 0) {
    throw 'A successful build/integrated-build.json is required; a cargo check report is insufficient.'
}
$taskGame = Join-Path $taskRuntime 'liquidbounce-native-game.exe'
if (![IO.Path]::GetFullPath([string]$taskBuild.executable).Equals($taskGame, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'The build report does not identify this staging game executable.'
}
if ($taskBuild.staging.originalThemeLinked -ne $true -or
    $taskBuild.staging.horizon -ne '9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0') {
    throw 'The report must identify the pinned Horizon build with the original theme linked.'
}
Assert-NoReparse $taskGame
if ((Get-Digest $taskGame) -ne $taskBuild.executableSha256.ToLowerInvariant()) {
    throw 'The game executable changed after the successful integrated build.'
}
$taskLock = Join-Path $taskProject 'build/staging/horizon/Cargo.lock'
Assert-NoReparse $taskLock
if ((Get-Digest $taskLock) -ne $taskBuild.lockSha256.ToLowerInvariant()) {
    throw 'Cargo.lock changed after the successful integrated build.'
}
$taskSourceMap = [ordered]@{}
$taskSourceKeys = [string[]]@($taskBuild.staging.nativeSourceFiles.PSObject.Properties.Name)
if ($taskSourceKeys.Count -eq 0 -or $taskBuild.staging.nativeSourceSha256 -notmatch '^[0-9a-f]{64}$') {
    throw 'The integrated build must record nativeSourceFiles and nativeSourceSha256.'
}
[Array]::Sort($taskSourceKeys, [StringComparer]::Ordinal)
foreach ($taskRelative in $taskSourceKeys) {
    if ($taskRelative -notmatch '^(native|patches|scripts)/' -or
        $taskRelative -match '(^/|\\|:|(^|/)\.{1,2}(/|$))') {
        throw "Unsafe native source manifest path: $taskRelative"
    }
    $taskSource = Join-Path $taskProject $taskRelative
    Assert-Within $taskSource $taskProject
    Assert-NoReparse $taskSource
    $taskExpected = [string]$taskBuild.staging.nativeSourceFiles.PSObject.Properties[$taskRelative].Value
    if ($taskExpected -notmatch '^[0-9a-f]{64}$' -or (Get-Digest $taskSource) -ne $taskExpected) {
        throw "Native source changed after the successful integrated build: $taskRelative"
    }
    $taskSourceMap[$taskRelative] = $taskExpected
}
if ((Get-TextDigest ($taskSourceMap | ConvertTo-Json -Compress -Depth 3)) -ne $taskBuild.staging.nativeSourceSha256) {
    throw 'The native source manifest aggregate does not match nativeSourceSha256.'
}

$taskValidation = $null
if ($ValidationReport) {
    $taskValidationPath = (Resolve-Path -LiteralPath $ValidationReport).Path
    Assert-NoReparse $taskValidationPath
    $taskValidation = Get-Content -LiteralPath $taskValidationPath -Raw | ConvertFrom-Json
    if ($taskValidation.passed -ne $true -or $taskValidation.verificationPassed -ne $true) {
        throw 'ValidationReport must record passed=true and verificationPassed=true.'
    }
    # Scoped reports must prove the full integration run. A UI diagnostic is
    # useful for debugging but does not qualify as the game acceptance report.
    $taskValidationScopeProperty = $taskValidation.PSObject.Properties['validationScope']
    $taskFullIntegrationProperty = $taskValidation.PSObject.Properties['fullIntegrationPassed']
    if ($null -ne $taskValidationScopeProperty -or $null -ne $taskFullIntegrationProperty) {
        if ($null -eq $taskValidationScopeProperty -or $taskValidationScopeProperty.Value -cne 'full_integration' -or
            $null -eq $taskFullIntegrationProperty -or $taskFullIntegrationProperty.Value -isnot [bool] -or
            $taskFullIntegrationProperty.Value -ne $true) {
            throw 'ValidationReport is not a passed full game integration report. Packaging requires validationScope=full_integration and fullIntegrationPassed=true; UI diagnostic results cannot be packaged as full acceptance.'
        }
    }
    # A screenshot review marker requires an explicit completed visual review;
    # older baselines without this marker remain supported.
    $taskPendingVisualReviewProperty = $taskValidation.PSObject.Properties['screenshotPendingVisualReview']
    if ($null -ne $taskPendingVisualReviewProperty) {
        $taskVisualReviewProperty = $taskValidation.PSObject.Properties['uiVisualReview']
        $taskVisualPassedProperty = if ($null -ne $taskVisualReviewProperty -and $null -ne $taskVisualReviewProperty.Value) {
            $taskVisualReviewProperty.Value.PSObject.Properties['passed']
        } else { $null }
        $taskVisualStatusProperty = if ($null -ne $taskPendingVisualReviewProperty.Value) {
            $taskPendingVisualReviewProperty.Value.PSObject.Properties['status']
        } else { $null }
        if ($null -eq $taskVisualPassedProperty -or $taskVisualPassedProperty.Value -isnot [bool] -or
            $taskVisualPassedProperty.Value -ne $true -or $null -eq $taskVisualStatusProperty -or
            $taskVisualStatusProperty.Value -cne 'reviewed_pass') {
            throw 'ValidationReport has a screenshot pending visual review. Packaging requires uiVisualReview.passed=true and screenshotPendingVisualReview.status=reviewed_pass.'
        }
    }
    if ($taskValidation.structuredControl -ne $true -or $taskValidation.screenRecognitionUsed -ne $false) {
        throw 'The validation must use structured game control without screen recognition.'
    }
    $taskValidationChecks = @($taskValidation.checks.PSObject.Properties)
    if ($taskValidationChecks.Count -eq 0) { throw 'The validation report must contain individual checks.' }
    foreach ($taskCheck in $taskValidationChecks) {
        if ($taskCheck.Value.passed -ne $true) { throw "Validation check did not pass: $($taskCheck.Name)" }
    }
    if ($taskValidation.runtimeProvenance.integratedBuild.passed -ne $true -or
        $taskValidation.runtimeProvenance.integratedBuild.operation -ne 'build' -or
        $taskValidation.runtimeProvenance.integratedBuild.exitCode -ne 0) {
        throw 'The validation must identify a successful integrated game build.'
    }
    if ($taskValidation.runtimeProvenance.integratedBuild.executableSha256 -ne $taskBuild.executableSha256) {
        throw 'The validation report refers to a different game executable.'
    }
    if ($taskValidation.runtimeProvenance.executableOnDisk.sha256 -ne $taskBuild.executableSha256) {
        throw 'The actual executable hash recorded during validation differs from this build.'
    }
    if ($taskValidation.runtimeProvenance.integratedBuild.staging.nativeSourceSha256 -ne $taskBuild.staging.nativeSourceSha256) {
        throw 'The validation report refers to different compiled native sources.'
    }
    $taskObservedModules = @($taskValidation.finalStatus.modules | ForEach-Object { [string]$_.name })
    Assert-ModuleNames $taskObservedModules
    if ($ImplementedModules.Count -gt 0 -and
        @(Compare-Object ($ImplementedModules | Sort-Object) ($taskObservedModules | Sort-Object)).Count -ne 0) {
        throw 'ImplementedModules does not match the validated actual module registry.'
    }
    $ImplementedModules = $taskObservedModules
}
Assert-ModuleNames $ImplementedModules

$taskHeader = Join-Path $taskCef 'include/cef_version.h'
Assert-NoReparse $taskHeader
if ((Get-Content -LiteralPath $taskHeader -Raw) -notmatch ('#define CEF_VERSION "' + [regex]::Escape($taskCefVersion) + '"')) {
    throw 'The cached CEF headers do not identify the pinned runtime.'
}
$taskCefArchive = Get-Content -LiteralPath (Join-Path $taskCef 'archive.json') -Raw | ConvertFrom-Json
if ($taskCefArchive.name -ne ('cef_binary_' + $taskCefVersion + '_windows64_minimal.tar.bz2')) {
    throw 'The cached CEF archive metadata does not match the pinned runtime.'
}

$taskPlan = [Collections.Generic.List[object]]::new()
function Add-PackageFile {
    param([string] $Source, [string] $Relative)
    Assert-NoReparse $Source
    if (!(Test-Path -LiteralPath $Source -PathType Leaf)) { throw "Required package file missing: $Source" }
    if ([IO.Path]::GetExtension($Source) -ieq '.jar') { throw "JAR files are excluded: $Source" }
    Assert-Within (Join-Path $taskDestination $Relative) $taskDestination
    $taskPlan.Add([pscustomobject]@{ source = $Source; relative = $Relative; sha256 = (Get-Digest $Source) })
}
Add-PackageFile $taskGame 'runtime/liquidbounce-native-game.exe'
Add-PackageFile (Join-Path $taskRuntime 'liquidbounce-cef-helper.exe') 'runtime/liquidbounce-cef-helper.exe'
$taskCefFiles = @(
    'libcef.dll', 'chrome_elf.dll', 'libEGL.dll', 'libGLESv2.dll',
    'd3dcompiler_47.dll', 'dxcompiler.dll', 'dxil.dll', 'vk_swiftshader.dll', 'vulkan-1.dll',
    'chrome_100_percent.pak', 'chrome_200_percent.pak', 'resources.pak', 'icudtl.dat',
    'v8_context_snapshot.bin', 'vk_swiftshader_icd.json'
)
foreach ($taskRelative in $taskCefFiles) {
    $taskInput = Join-Path $taskRuntime $taskRelative
    $taskReference = Join-Path $taskCef $taskRelative
    Assert-NoReparse $taskReference
    Add-PackageFile $taskInput ('runtime/' + $taskRelative)
    if ($taskPlan[$taskPlan.Count - 1].sha256 -ne (Get-Digest $taskReference)) {
        throw "Staged CEF runtime differs from its pinned distribution: $taskRelative"
    }
}
$taskLocales = @(Get-TreeFiles (Join-Path $taskCef 'locales'))
if (@($taskLocales | Where-Object Name -EQ 'en-US.pak').Count -ne 1) { throw 'Pinned CEF en-US locale is missing.' }
foreach ($taskLocale in $taskLocales) {
    $taskRelative = [IO.Path]::GetRelativePath($taskCef, $taskLocale.FullName)
    $taskInput = Join-Path $taskRuntime $taskRelative
    Add-PackageFile $taskInput ('runtime/' + $taskRelative)
    if ($taskPlan[$taskPlan.Count - 1].sha256 -ne (Get-Digest $taskLocale.FullName)) {
        throw "Staged CEF locale differs from its pinned distribution: $taskRelative"
    }
}
$taskThemeReportPath = Join-Path $taskProject 'build/themes/chinese/localization-build.json'
if (!(Test-Path -LiteralPath $taskThemeReportPath -PathType Leaf)) { throw 'Build the localized theme first.' }
$taskThemeReport = Get-Content -LiteralPath $taskThemeReportPath -Raw | ConvertFrom-Json
if (!$taskThemeReport.buildCompleted -or $taskThemeReport.themeKind -ne 'derived_chinese_localization') { throw 'A completed localized theme build is required.' }
foreach ($taskEntry in $taskThemeReport.outputFiles.PSObject.Properties) {
    if ((Get-Digest (Join-Path $taskTheme $taskEntry.Name)) -ne $taskEntry.Value) { throw "Localized theme output changed: $($taskEntry.Name)" }
}
Add-PackageFile $taskThemeReportPath 'metadata/localization-build.json'
foreach ($taskFile in Get-TreeFiles $taskTheme) {
    Add-PackageFile $taskFile.FullName ('theme/' + [IO.Path]::GetRelativePath($taskTheme, $taskFile.FullName))
}
Add-PackageFile (Join-Path $taskProject 'LICENSE') 'LICENSE'
Add-PackageFile (Join-Path $taskProject 'upstream/LiquidBounce/LICENSE') 'licenses/LiquidBounce-GPL-3.0.txt'
Add-PackageFile (Join-Path $taskProject 'upstream/Horizon/LICENSE') 'licenses/Horizon-MIT.txt'
Add-PackageFile (Join-Path $taskCef 'CREDITS.html') 'licenses/Chromium-CREDITS.html'
Add-PackageFile (Join-Path $taskTemplates 'CEF-BSD.txt') 'licenses/CEF-BSD.txt'
Add-PackageFile (Join-Path $taskTemplates 'Start-LiquidBounce.ps1') 'Start-LiquidBounce.ps1'
Add-PackageFile (Join-Path $taskTemplates 'SOURCES.md') 'SOURCES.md'
Add-PackageFile (Join-Path $taskProject 'native/liquidbounce-webview/source-versions.json') 'metadata/cef-source-versions.json'

# Ship the complete local port sources, locks, tests, fixtures and build scripts.
# Binary caches, screenshots, browser diagnostics, generated bundles and game
# resources are deliberately outside this source allowlist.
$taskSourceExtensions = @('.rs', '.toml', '.lock', '.wgsl', '.json', '.py', '.ps1', '.mjs', '.md', '.patch', '.txt', '.ts', '.svelte', '.css')
foreach ($taskSource in Get-SourceFiles (Join-Path $taskProject 'native')) {
    $taskRelative = [IO.Path]::GetRelativePath($taskProject, $taskSource.FullName).Replace('\', '/')
    $taskFixture = $taskRelative -match '^native/liquidbounce-webview/fixtures/[^/]+\.html$'
    $taskLocalization = $taskRelative -match '^native/liquidbounce-interop/resources/localization/[^/]+\.json$'
    $taskMetadata = $taskSource.Name -in @('LICENSE', 'NOTICE', '.gitignore', 'horizon-native.json', 'source-versions.json')
    if ($taskSource.Extension -in $taskSourceExtensions -or $taskFixture -or $taskLocalization -or $taskMetadata) {
        Add-PackageFile $taskSource.FullName ('source/' + $taskRelative)
    }
}
foreach ($taskFolder in @('patches', 'scripts')) {
    foreach ($taskSource in Get-SourceFiles (Join-Path $taskProject $taskFolder)) {
        if ($taskSource.Extension -in $taskSourceExtensions -or $taskSource.Name -eq '.gitignore') {
            Add-PackageFile $taskSource.FullName ('source/' + [IO.Path]::GetRelativePath($taskProject, $taskSource.FullName).Replace('\', '/'))
        }
    }
}
foreach ($taskFile in @('LICENSE', 'README.md', '.gitignore')) {
    Add-PackageFile (Join-Path $taskProject $taskFile) ('source/' + $taskFile)
}
foreach ($taskFile in @('source-integrity.json', 'horizon-source-integrity.json', 'upstream-commit.json', 'upstream-tree.json',
    'PORTING.md', 'module-parity.json', 'interop-bootstrap.md', 'next-render-modules.md', 'render-module-validation.md')) {
    Add-PackageFile (Join-Path $taskProject ('research/' + $taskFile)) ('source/research/' + $taskFile)
}
Add-PackageFile $taskLock 'source/rebuild-locks/horizon-Cargo.lock'
Add-PackageFile (Join-Path $taskProject 'build/staging/horizon/Cargo.toml') 'source/rebuild-locks/horizon-Cargo.toml'
Add-PackageFile (Join-Path $taskTemplates 'source/Restore-Source-Baselines.py') 'source/Restore-Source-Baselines.py'
Add-PackageFile (Join-Path $taskTemplates 'source/Rebuild-LiquidBounce.ps1') 'source/Rebuild-LiquidBounce.ps1'
Add-PackageFile (Join-Path $taskTemplates 'source/README-REBUILD.md') 'source/README-REBUILD.md'
$taskSourceBaselines = [ordered]@{
    formatVersion = 1
    baselines = @(
        [ordered]@{
            name = 'Horizon'; commit = '9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0'
            archiveUrl = 'https://codeload.github.com/Jijjy/mc-horizon/zip/9f42064'
            archiveSha256 = '4a9798eaf7b5dce1f711307efe638f4f9eed8eafc3d58cb0415ef63a38f1c95e'
            archive = 'research/Horizon-source.zip'; archiveRoot = 'mc-horizon-9f42064'; destination = 'upstream/Horizon'
        },
        [ordered]@{
            name = 'LiquidBounce'; commit = 'ae0eb396c07f22383789b28d5912d714c6ca0a86'
            archiveUrl = 'https://codeload.github.com/CCBlueX/LiquidBounce/zip/ae0eb396c07f22383789b28d5912d714c6ca0a86'
            archiveSha256 = '98aa5c9007b80b78bae8edf85ca80d87fa65bebea41cc662ab8c2221016eb0cb'
            archive = 'research/LiquidBounce-source.zip'; archiveRoot = 'LiquidBounce-ae0eb396c07f22383789b28d5912d714c6ca0a86'
            destination = 'upstream/LiquidBounce'
        },
        [ordered]@{
            name = 'Horizon-Mixins'; commit = '5a64af938ec86b00d53d16a408aabc0038fb3e57'
            archiveUrl = 'https://codeload.github.com/WahhaWood/horizon-mixins/zip/5a64af938ec86b00d53d16a408aabc0038fb3e57'
            archiveSha256 = '0db3d966f742fa5eeb94df3456df4cf9820abb58106cede0617f5c6ec92ca8a3'
            archive = 'research/Horizon-Mixins-source.zip'; archiveRoot = 'horizon-mixins-5a64af938ec86b00d53d16a408aabc0038fb3e57'
            destination = 'upstream/Horizon-Mixins'
        }
    )
}
foreach ($taskBaseline in $taskSourceBaselines.baselines) {
    Add-PackageFile (Join-Path $taskProject $taskBaseline.archive) ('source/' + $taskBaseline.archive)
    if ($taskPlan[$taskPlan.Count - 1].sha256 -ne $taskBaseline.archiveSha256) {
        throw "Pinned source archive hash mismatch: $($taskBaseline.name)"
    }
}
# The integrated staging manifest is authoritative for every source file that
# contributed to the native build. Some of those files live under vendored
# trees, which the general source walk intentionally skips; add those exact
# manifest entries explicitly so the package can be rebuilt from its sources.
foreach ($taskRelative in $taskSourceKeys) {
    $taskSource = Join-Path $taskProject $taskRelative
    Assert-Within $taskSource $taskProject
    Assert-NoReparse $taskSource
    $taskExistingSource = @($taskPlan | Where-Object relative -EQ ('source/' + $taskRelative))
    if ($taskExistingSource.Count -eq 0) {
        Add-PackageFile $taskSource ('source/' + $taskRelative)
    }
}
foreach ($taskRelative in $taskSourceKeys) {
    $taskCopiedSource = @($taskPlan | Where-Object relative -EQ ('source/' + $taskRelative))
    if ($taskCopiedSource.Count -ne 1 -or $taskCopiedSource[0].sha256 -ne $taskSourceMap[$taskRelative]) {
        throw "The source package must contain the exact compiled native source: $taskRelative"
    }
}

# All inputs are validated before creating the destination. A failed copy is left
# for inspection; this script never recursively removes an existing artifact.
if (Test-Path -LiteralPath $taskDestination) { throw 'Artifact appeared during validation; choose a new Name.' }
New-Item -ItemType Directory -Path $taskDestination | Out-Null
foreach ($taskItem in $taskPlan) {
    Assert-NoReparse $taskDestination
    $taskOutput = Join-Path $taskDestination $taskItem.relative
    New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($taskOutput)) -Force | Out-Null
    Assert-NoReparse $taskOutput
    Copy-Item -LiteralPath $taskItem.source -Destination $taskOutput
    if ((Get-Digest $taskOutput) -ne $taskItem.sha256) { throw "Copy verification failed: $($taskItem.relative)" }
}
function Write-PackageText {
    param([string] $Relative, [string] $Text)
    $taskOutput = Join-Path $taskDestination $Relative
    Assert-Within $taskOutput $taskDestination
    Assert-NoReparse $taskOutput
    if (Test-Path -LiteralPath $taskOutput) { throw "Generated package file already exists: $Relative" }
    [IO.File]::WriteAllText($taskOutput, $Text, [Text.UTF8Encoding]::new($false))
}
$taskModuleText = ($ImplementedModules | ForEach-Object { '- ' + $_ }) -join "`n"
$taskValidationText = if ($null -ne $taskValidation) {
    '已附通过的集成验证摘要；摘要与此可执行文件的 SHA-256 一致。带作用域标记的报告必须通过完整游戏整合验收；UI 诊断结果不能替代该验收。含截图待审标记的报告，打包前必须记录视觉复核通过；无上述新字段的旧基线保持兼容。具体覆盖范围见 metadata/validation-summary.json。'
} else {
    '模块名单来自显式打包参数。本包没有附带已通过的游戏集成验证报告，不应据此宣称模块行为已通过游戏验收。'
}
$taskReadme = @"
# LiquidBounce / Horizon 26.3 开发预览

此包当前只声明以下 $($ImplementedModules.Count) 个已接入模块：

$taskModuleText

这是未完成的移植预览，尚未实现原版 236 个模块的完整功能与行为一致性。使用原版 LiquidBounce 主题；没有用重新绘制的界面替换主题。$taskValidationText

## 启动

需要 Windows x64、PowerShell 7，以及自行合法准备的 Minecraft 26.3 Horizon resources 目录。必须显式提供 resources 路径：

    pwsh -NoProfile -File .\Start-LiquidBounce.ps1 -Resources 'D:\Your\Prepared\26.3'

启动器直接使用包内 runtime/ 和 theme/，不依赖原源码工程路径。默认进入游戏主菜单，由玩家选择单人世界或多人游戏。仅显式指定 -World 时才直接创建或打开该创造模式世界。右 Shift 打开或关闭 ClickGUI。可选 -World、-PlayerName、-Width、-Height、-RenderDistance、-Offscreen 和 -NoAudio。

所有 Horizon home、世界、玩家设置、配置、日志、控制 socket 和浏览器缓存位于本包 user-data/。同一包只允许一个运行实例。资源目录只作为显式输入交给游戏；本包不包含或复制 Mojang JAR、已准备的游戏资源、既有世界、账户、会话或任何用户配置。

可执行文件与运行资源的 SHA-256 记录在 package-manifest.json；来源、许可证和已知范围见 SOURCES.md、LICENSE、licenses/ 与 metadata/。

source/ 附有当前移植源码、测试、Cargo 清单与锁文件、补丁和构建脚本，并保存了 Horizon、LiquidBounce 与 Horizon-Mixins 三份固定源码归档。完整提交、SHA-256 和备用下载地址见 source/source-baselines.json；按 source/README-REBUILD.md 可在此包内恢复基准并重新构建，不依赖原工程目录。
"@
Write-PackageText 'README.zh-CN.md' ($taskReadme + "`n")
$taskBuildSummary = [ordered]@{
    operation = $taskBuild.operation; passed = $taskBuild.passed; exitCode = $taskBuild.exitCode
    startedUtc = $taskBuild.startedUtc; finishedUtc = $taskBuild.finishedUtc
    executableSha256 = $taskBuild.executableSha256; lockSha256 = $taskBuild.lockSha256
    horizon = $taskBuild.staging.horizon; originalThemeLinked = $taskBuild.staging.originalThemeLinked
    gpuUiPatchSha256 = $taskBuild.staging.gpuUiPatchSha256
    behaviorHookPatchSha256 = $taskBuild.staging.behaviorHookPatchSha256
    nativeSourceSha256 = $taskBuild.staging.nativeSourceSha256
    nativeSourceFiles = $taskSourceMap
    implementedModules = $ImplementedModules
}
Write-PackageText 'metadata/build-summary.json' (($taskBuildSummary | ConvertTo-Json -Depth 5) + "`n")
$taskNativeManifest = [ordered]@{ nativeSourceSha256 = $taskBuild.staging.nativeSourceSha256; nativeSourceFiles = $taskSourceMap }
Write-PackageText 'source/native-source-manifest.json' (($taskNativeManifest | ConvertTo-Json -Depth 4) + "`n")
Write-PackageText 'source/source-baselines.json' (($taskSourceBaselines | ConvertTo-Json -Depth 5) + "`n")
if ($null -ne $taskValidation) {
    $taskChecks = [ordered]@{}
    foreach ($taskCheck in $taskValidation.checks.PSObject.Properties) { $taskChecks[$taskCheck.Name] = $taskCheck.Value.passed }
    $taskValidationSummary = [ordered]@{
        passed = $true; executableSha256 = $taskBuild.executableSha256; nativeSourceSha256 = $taskBuild.staging.nativeSourceSha256
        implementedModules = $ImplementedModules; checks = $taskChecks
    }
    Write-PackageText 'metadata/validation-summary.json' (($taskValidationSummary | ConvertTo-Json -Depth 5) + "`n")
}
$taskManifestFiles = @(foreach ($taskFile in Get-TreeFiles $taskDestination) {
    [ordered]@{
        path = [IO.Path]::GetRelativePath($taskDestination, $taskFile.FullName).Replace('\', '/')
        bytes = $taskFile.Length; sha256 = (Get-Digest $taskFile.FullName)
    }
})
$taskManifest = [ordered]@{
    formatVersion = 1; createdUtc = [DateTimeOffset]::UtcNow.ToString('o')
    product = 'LiquidBounce / Horizon 26.3 development preview'
    completePort = $false; implementedModules = $ImplementedModules
    cefVersion = $taskCefVersion; nativeSourceSha256 = $taskBuild.staging.nativeSourceSha256; files = $taskManifestFiles
}
Write-PackageText 'package-manifest.json' (($taskManifest | ConvertTo-Json -Depth 6) + "`n")
[pscustomobject]@{ package = $taskDestination; modules = $ImplementedModules; files = $taskManifestFiles.Count; launched = $false } | ConvertTo-Json
