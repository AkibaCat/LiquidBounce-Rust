#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Add a native launcher and prepared resources to an existing local preview."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from datetime import datetime, timezone


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def tree_files(root: Path, excluded: set[str] | None = None):
    excluded = excluded or set()
    for directory, dirs, files in os.walk(root, followlinks=False):
        base = Path(directory)
        for name in dirs[:]:
            child = base / name
            if name in excluded:
                dirs.remove(name)
            elif child.is_symlink() or (hasattr(child, 'is_junction') and child.is_junction()):
                raise ValueError(f'Reparse point in package input: {child}')
        for name in sorted(files):
            child = base / name
            if child.is_symlink():
                raise ValueError(f'Symbolic link in package input: {child}')
            yield child


def record(path: Path, root: Path) -> dict:
    return {'path': path.relative_to(root).as_posix(), 'bytes': path.stat().st_size, 'sha256': digest(path)}


def copy_verified(source: Path, destination: Path, expected: str | None = None) -> str:
    expected = expected or digest(source)
    destination.parent.mkdir(parents=True, exist_ok=True)
    if not destination.is_file() or destination.stat().st_size != source.stat().st_size or digest(destination) != expected:
        shutil.copy2(source, destination)
    if digest(destination) != expected:
        raise ValueError(f'Copy hash differs: {destination}')
    return expected


def update_runtime_docs(root: Path, launcher_included: bool) -> None:
    start = root / 'Start-LiquidBounce.ps1'
    text = start.read_text(encoding='utf-8-sig')
    text = text.replace(
        '[Parameter(Mandatory = $true)][ValidateNotNullOrEmpty()][string] $Resources,',
        "[ValidateNotNullOrEmpty()][string] $Resources = (Join-Path $PSScriptRoot 'resources/26.3'),",
    )
    start.write_text(text, encoding='utf-8')

    readme_path = root / 'README.zh-CN.md'
    readme = readme_path.read_text(encoding='utf-8-sig')
    module_section = readme.split('## 启动', 1)[0].rstrip()
    launch_text = (
        '双击本目录的 **LiquidBounce-Launcher.exe**，再点击“启动游戏”。也可使用 ReleasesPack 根目录的 Start-LiquidBounce.cmd。'
        '启动器为 Rust + egui 原生程序，主窗口为 300 × 450，使用 LiquidLauncher 黑色玻璃界面、蓝色 #4677FF 强调色与圆角控件。'
        '点击玩家名称打开账户弹窗，选择微软正版登录或离线登录。\n\n'
        if launcher_included else ''
    )
    readme_path.write_text(module_section + '\n\n## 启动\n\n' + launch_text +
        '包内已附启动所需的 Minecraft 26.3 prepared resources，位于 resources/26.3/。'
        '默认使用此目录，无需填写原工程路径。运行环境为 Windows x64。\n\n'
        '资源目录同时包含 assets/objects/ 哈希对象、assets/indexes/34.json 索引，以及游戏直接读取的 assets/minecraft/ 展开文件。'
        '音效、标题全景、中文字体与语言、内置资源包已一并补齐；全部索引引用均校验 SHA-1 与大小。'
        '资源来源、索引版本及体积见 metadata/portable-assets.json。\n\n'
        '微软正版登录采用 LiquidLauncher 同源的 Microsoft/Xbox/Minecraft 设备授权流程，'
        '需要账户本人在微软页面授权；账户缓存由 Windows DPAPI 加密，游戏会话通过子进程标准输入传递。'
        '本次构建尚未完成人工微软联网登录验收。\n\n'
        '命令行备用启动需要 PowerShell 7：\n\n'
        '    pwsh -NoProfile -File .\\Start-LiquidBounce.ps1\n\n'
        '使用其他资源目录时仍可传入 -Resources 参数。默认进入游戏主菜单；'
        '传入 -World 世界名时才直接进入指定世界。右 Shift 打开或关闭 ClickGUI。\n\n'
        '世界、玩家设置、配置、日志和浏览器缓存保存在 user-data/。本包没有复制已有世界、账户、会话或浏览器缓存。'
        '资源是此本地发布包的游戏运行输入；资源文件与哈希见 metadata/resources-manifest.json。\n\n'
        '包内文件 SHA-256 见 package-manifest.json；游戏构建与未完成功能范围见 metadata/build-summary.json。'
        '本包仍是 210 个模块具备 native 逻辑、236 个模块已注册的开发预览，未宣称所有模块均完成或通过完整游戏验收。\n\n'
        'source/ 附有移植源码、锁文件、补丁、构建脚本和固定源码归档。'
        '启动器源码位于 source/native/liquidbounce-launcher/，重建方法见该目录 README.zh-CN.md。\n',
        encoding='utf-8')

    sources_path = root / 'SOURCES.md'
    sources = sources_path.read_text(encoding='utf-8-sig')
    marker = 'No Mojang JAR, prepared game resources, pre-existing world, account, session,'
    if marker in sources:
        sources = sources.split(marker, 1)[0].rstrip()
    if '## Local launcher and prepared resources' in sources:
        sources = sources.split('## Local launcher and prepared resources', 1)[0].rstrip()
    sources += ('\n\n## Local launcher and prepared resources\n\n'
        'This local release includes the prepared Minecraft 26.3 resources used by the game in resources/26.3/. '
        'metadata/resources-manifest.json records the file hashes and the Horizon preparation metadata. '
        'The complete local version-26.3 asset index and its verified objects are retained under '
        'resources/26.3/assets/indexes/ and assets/objects/, with every indexed file also expanded '
        'under assets/ for the native renderer and audio backend. metadata/portable-assets.json '
        'identifies the exact index hashes and notes republished translations. '
        'Existing worlds, accounts, session credentials, personal game settings and browser caches are not copied. '
        'Mutable game files are created under user-data/.\n\n'
        'LiquidBounce-Launcher.exe is a Rust/egui launcher. Its sources and Cargo.lock are included under '
        'source/native/liquidbounce-launcher/. Its black glass surfaces, rounded controls and blue #4677FF accent '
        'reference LiquidLauncher commit a2420d4e93dc2700bfedca68ecbef143dfb66c2b '
        '(https://github.com/CCBlueX/LiquidLauncher/tree/a2420d4e93dc2700bfedca68ecbef143dfb66c2b). '
        'The LiquidBounce mark and bundled theme assets come from the pinned LiquidBounce source above; '
        'the avatar is based on the user-provided launcher reference. Click the player name to choose '
        'Microsoft authentication or an offline profile. The launcher opens the bundled runtime directly '
        'and defaults to the resources folder beside it. It does not depend on the original source workspace.\n\n'
        'The bundled Inter font is licensed under the SIL Open Font License 1.1; see licenses/Inter-OFL.txt '
        'and source/native/liquidbounce-launcher/assets/OFL-Inter.txt. Chinese fallback fonts are loaded '
        'from the local Windows installation at runtime and are not redistributed with the launcher.\n')
    sources += ('\nMicrosoft/Xbox/Minecraft authentication follows the pinned LiquidLauncher implementation '
        'and uses minecraft-auth 0.1.1 (LGPL-3.0-or-later; https://github.com/CCBlueX/minecraft-auth-rs). '
        'The Microsoft title ID is 00000000402b5328, as used by the upstream dependency; it is separate '
        'from the auth.liquidbounce.net application ID. The unmodified crate source is included under '
        'source/upstream/minecraft-auth/minecraft-auth-0.1.1/, and its full license is in '
        'licenses/minecraft-auth-LGPL-3.0.txt. Cached credentials use Windows DPAPI, and the game session '
        'is passed through child-process stdin. Interactive Microsoft authorization requires the account '
        'owner and has not been manually tested online for this build.\n')
    sources_path.write_text(sources, encoding='utf-8')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--resources', required=True, type=Path)
    parser.add_argument('--destination', required=True, type=Path)
    parser.add_argument('--launcher', type=Path)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--resources-only', action='store_true')
    args = parser.parse_args()
    root = args.destination.resolve()
    resources = args.resources.resolve()
    manifest_path = root / 'package-manifest.json'
    if not manifest_path.is_file() or not (root / 'runtime/liquidbounce-native-game.exe').is_file():
        raise ValueError('Destination must be an existing game preview with package-manifest.json and runtime.')
    if not (resources / 'horizon-resources.json').is_file():
        raise ValueError('Prepared resources metadata is missing.')
    preparation = json.loads((resources / 'horizon-resources.json').read_text(encoding='utf-8-sig'))
    if preparation.get('minecraft') != '26.3':
        raise ValueError('Prepared resources do not identify Minecraft 26.3.')
    if not args.resources_only and (args.launcher is None or not args.launcher.is_file() or not args.source.is_dir()):
        raise ValueError('The compiled launcher executable and source directory are required.')
    portable_path = resources / 'portable-assets.json'
    portable = json.loads(portable_path.read_text(encoding='utf-8')) if portable_path.is_file() else None
    if portable is not None:
        if portable.get('minecraft') != '26.3' or portable.get('allReferencesVerified') is not True:
            raise ValueError('Portable resources have not passed complete asset reference verification.')
        index_path = resources / 'asset-index.json'
        if digest(index_path) != portable.get('assetIndexSha256'):
            raise ValueError('The portable asset index differs from its verified metadata.')
        alias = resources / 'assets/indexes' / (str(portable['assetIndexId']) + '.json')
        if digest(alias) != portable.get('assetIndexSha256'):
            raise ValueError('The portable asset index alias differs from its verified metadata.')

    entries = []
    skipped = []
    resource_target = root / 'resources/26.3'
    for source in tree_files(resources):
        relative = source.relative_to(resources)
        if source.name == '.horizon-prepare.lock' or (len(relative.parts) == 1 and source.suffix == '.log'):
            skipped.append(relative.as_posix())
            continue
        expected = digest(source)
        target = resource_target / relative
        copy_verified(source, target, expected)
        entries.append({'path': relative.as_posix(), 'bytes': source.stat().st_size, 'sha256': expected})
    entries.sort(key=lambda value: value['path'])
    resource_report = {
        'formatVersion': 2 if portable else 1, 'createdUtc': datetime.now(timezone.utc).isoformat(),
        'minecraft': '26.3', 'relativeRoot': 'resources/26.3', 'fileCount': len(entries),
        'bytes': sum(entry['bytes'] for entry in entries), 'files': entries,
        'excludedPreparationFiles': skipped, 'preparation': preparation,
    }
    if portable:
        resource_report['portableAssets'] = portable
        write_json(root / 'metadata/portable-assets.json', portable)
    write_json(root / 'metadata/resources-manifest.json', resource_report)

    if not args.resources_only:
        launcher_hash = copy_verified(args.launcher.resolve(), root / 'LiquidBounce-Launcher.exe')
        source_root = args.source.resolve()
        copy_verified(source_root / 'assets/OFL-Inter.txt', root / 'licenses/Inter-OFL.txt')
        auth_source = Path(__file__).resolve().parent.parent / 'upstream/minecraft-auth/minecraft-auth-0.1.1'
        if not (auth_source / 'LICENSE').is_file():
            raise ValueError('The pinned minecraft-auth 0.1.1 source and license are required.')
        copy_verified(auth_source / 'LICENSE', root / 'licenses/minecraft-auth-LGPL-3.0.txt')
        for source in tree_files(auth_source, {'target', '.git', '__pycache__'}):
            relative = source.relative_to(auth_source)
            copy_verified(source, root / 'source/upstream/minecraft-auth/minecraft-auth-0.1.1' / relative)
        source_records = []
        for source in tree_files(source_root, {'target', '.git', '__pycache__'}):
            relative = source.relative_to(source_root)
            if source.suffix.lower() in {'.pdb', '.exe'}:
                continue
            copy_verified(source, root / 'source/native/liquidbounce-launcher' / relative)
            source_records.append(record(source, source_root))
        write_json(root / 'metadata/launcher-build.json', {
            'formatVersion': 1, 'packagedUtc': datetime.now(timezone.utc).isoformat(),
            'executable': 'LiquidBounce-Launcher.exe', 'sha256': launcher_hash,
            'ui': 'Rust + egui', 'window': {'width': 300, 'height': 450},
            'defaultResources': 'resources/26.3', 'sourceFiles': source_records,
            'styleReference': {'project': 'LiquidLauncher', 'commit': 'a2420d4e93dc2700bfedca68ecbef143dfb66c2b',
                'accentColor': '#4677FF', 'surface': 'black glass'},
            'fonts': {'bundled': 'Inter (SIL OFL 1.1)', 'chineseFallback': 'system Windows fonts; not redistributed'},
            'authentication': {'referenceCommit': 'a2420d4e93dc2700bfedca68ecbef143dfb66c2b',
                'library': 'minecraft-auth 0.1.1', 'license': 'LGPL-3.0-or-later',
                'microsoftTitleId': '00000000402b5328', 'cacheProtection': 'Windows DPAPI',
                'gameSessionTransport': 'child-process stdin', 'manualMicrosoftLoginVerified': False},
        })
        for filename in ['Package-LiquidBounce-Launcher.ps1', 'package-liquidbounce-launcher.py',
                         'prepare-portable-assets.py', 'sync-portable-resources.py', 'PORTABLE-RESOURCES.zh-CN.md']:
            source = Path(__file__).parent / filename
            copy_verified(source, root / 'source/scripts' / filename)
        shortcut = root.parent / 'Start-LiquidBounce.cmd'
        shortcut.write_text('@echo off\r\nstart "" "%~dp0' + root.name + '\\LiquidBounce-Launcher.exe"\r\n', encoding='utf-8')

    update_runtime_docs(root, (root / 'LiquidBounce-Launcher.exe').is_file())
    manifest = json.loads(manifest_path.read_text(encoding='utf-8-sig'))
    manifest['updatedUtc'] = datetime.now(timezone.utc).isoformat()
    manifest['preparedResources'] = {'minecraft': '26.3', 'path': 'resources/26.3',
        'manifest': 'metadata/resources-manifest.json', 'fileCount': len(entries), 'bytes': resource_report['bytes']}
    if portable:
        manifest['preparedResources'].update(portableAssets='metadata/portable-assets.json',
            assetIndexSha1=portable['assetIndexSha1'], allReferencesVerified=True)
    if not args.resources_only:
        manifest['launcher'] = {'executable': 'LiquidBounce-Launcher.exe', 'sha256': launcher_hash,
            'ui': 'Rust + egui', 'width': 300, 'height': 450}
    manifest['files'] = sorted((record(path, root) for path in tree_files(root, {'user-data'})
        if path != manifest_path), key=lambda value: value['path'])
    write_json(manifest_path, manifest)
    print(json.dumps({'package': str(root), 'launcherIncluded': (root/'LiquidBounce-Launcher.exe').is_file(),
        'resourceFiles': len(entries), 'resourceBytes': resource_report['bytes'],
        'manifestFiles': len(manifest['files']), 'completePort': manifest.get('completePort')}, ensure_ascii=False))


if __name__ == '__main__':
    main()
