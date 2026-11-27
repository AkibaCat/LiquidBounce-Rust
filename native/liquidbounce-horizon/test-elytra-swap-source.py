#!/usr/bin/env python3
"""Unmodified ModuleElytraSwap, real Kotlin coroutines, controlled stack facts."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def inputs():
    enable = {'op': 'enable', 'value': True}
    start = {'op': 'start'}
    resume = {'op': 'resume'}
    cancel = {'op': 'cancel'}
    def stack(slot, bits):
        return {'op': 'stack', 'slot': slot, 'bits': bits}
    rows = []
    for chest, bits, source in itertools.product([1, 0, 2, 6, 8, 10], range(16), [0, 8, 9, 35, 36, 37, 39, 40]):
        rows.append([enable, stack(38, chest), stack(source, bits), start, resume])
    # Find first usable glider, skip a damaged glider, and include offhand only.
    for source in range(36):
        rows.append([enable, stack(0, 6), stack(source, 2), stack(40, 2), start, resume])
    # After partial execution and cancellation, capture direction is retained
    # even if the chest has changed. A new enabling alone clears the capture.
    for chest, replace, retry in itertools.product([1, 0, 2, 8], [1, 0, 2, 8], [2, 8, 10]):
        rows.append([enable, stack(38, chest), stack(0, 10), start, cancel,
                     stack(38, replace), stack(0, retry), start, resume,
                     {'op': 'enable', 'value': False}, enable, start, resume])
    return rows


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    out.mkdir(parents=True, exist_ok=False)
    fixture = out / 'fixture'
    shutil.copytree(HERE / 'elytra-swap-source-fixture', fixture)
    source = ROOT / 'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/misc/ModuleElytraSwap.kt'
    shutil.copyfile(source, fixture / source.name)
    rows = inputs()
    (out / 'commands.json').write_text(json.dumps(rows), encoding='utf-8')
    jdk = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    kotlin = ROOT.parent / 'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    gson = next((Path(os.environ['USERPROFILE']) / '.gradle/caches/modules-2/files-2.1/com.google.code.gson').rglob('gson-*.jar'))
    jars = [gson, *kotlin.glob('kotlin-stdlib-*.jar'), *kotlin.glob('kotlinx-coroutines-core-jvm-*.jar')]
    classes = out / 'classes'
    classes.mkdir()
    cp = os.pathsep.join(map(str, [classes, *jars]))
    for name in ['elytra_swap', 'vomit', 'random']:
        shutil.copyfile(ROOT / f'native/liquidbounce-behaviors/src/{name}.rs', out / f'{name}.rs')
    shutil.copyfile(fixture / 'oracle.rs', out / 'oracle.rs')
    (out / 'lib.rs').write_text('pub mod elytra_swap;\npub mod vomit;\npub mod random;\n#[cfg(test)]mod oracle;\n', encoding='utf-8')
    (out / 'Cargo.toml').write_text('[package]\nname="elytra-swap-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n', encoding='utf-8')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(HERE / 'packet-tests/target')
    commands = []
    for name, argv in [
        ('kotlin-compile', [str(jdk / 'java.exe'), '-cp', str(kotlin / '*'), 'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler', '-no-stdlib', '-no-reflect', '-classpath', cp, '-d', str(classes), *map(str, sorted(fixture.glob('*.kt')))]),
        ('kotlin-source', [str(jdk / 'java.exe'), '-cp', cp, 'MainKt', str(out / 'commands.json'), str(out / 'oracle.json')]),
        ('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml')]),
        ('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings']),
    ]:
        started = time.monotonic()
        with (out / (name + '.stdout.log')).open('w', encoding='utf-8') as stdout, (out / (name + '.stderr.log')).open('w', encoding='utf-8') as stderr:
            result = subprocess.run(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
        commands.append({'name': name, 'exitCode': result.returncode, 'seconds': round(time.monotonic() - started, 3)})
        if result.returncode:
            break
    report = {'passed': len(commands) == 4 and all(c['exitCode'] == 0 for c in commands), 'scenarios': len(rows),
              'commands': commands, 'unchangedOriginalKotlin': source.read_bytes() == (fixture / source.name).read_bytes(),
              'sourceSha256': hashlib.sha256(source.read_bytes()).hexdigest(),
              'scope': 'Complete unchanged ModuleElytraSwap decisions and real kotlinx coroutine perform suspension/cancellation; controlled stack facts and inventoryPlan provider. Does not validate native component/tag extraction, InventoryManager execution or game effects.'}
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
