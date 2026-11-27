#!/usr/bin/env python3
"""Actual pinned MC stream codec versus actual generated Horizon network stacks."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    out.mkdir(parents=True, exist_ok=False)
    cache = Path(os.environ['USERPROFILE']) / '.gradle/caches'
    mc = cache / 'fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar'
    libraries = [str(mc)]
    for group in ['com.mojang', 'com.google.guava', 'com.google.code.gson', 'it.unimi.dsi',
                  'org.slf4j', 'org.joml', 'io.netty', 'org.apache.logging.log4j', 'org.apache.commons', 'org.lwjgl']:
        libraries.extend(str(p) for p in (cache / 'modules-2/files-2.1' / group).rglob('*.jar'))
    jdk = ROOT.parent / 'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
    stage = ROOT / 'build/staging/horizon'
    classes = out / 'java-classes'
    classes.mkdir()
    oracle = HERE / 'oracles/CooldownComponentFacts.java'
    probe = out / 'probe'
    (probe / 'src').mkdir(parents=True)
    shutil.copyfile(HERE / 'cooldown-component-probe.rs', probe / 'src/main.rs')
    manifest = '''[package]
name="cooldown-component-runtime-probe"
version="0.0.0"
edition="2024"
[workspace]
[dependencies]
'''
    for name, folder in [('horizon-client-lib', 'horizon-client-lib'), ('horizon-protocol', 'horizon-protocol'), ('horizon-world', 'horizon-world')]:
        manifest += f'{name}={{path={json.dumps(str(stage / "crates" / folder))}}}\n'
    manifest += 'serde_json="1.0.151"\n'
    # Path dependencies do not inherit their parent's workspace patches or
    # profiles. Use the same patched renderer crates and line-table profile as
    # the production client, including its Windows disk-cache API.
    manifest += '\n[patch.crates-io]\n'
    for name in ['wgpu-hal', 'naga', 'libmimalloc-sys']:
        manifest += f'{name}={{path={json.dumps(str(stage / "third_party" / name))}}}\n'
    manifest += '\n[profile.dev]\ndebug="line-tables-only"\n'
    for name in ['horizon-mod-api-gen', 'syn', 'proc-macro2']:
        manifest += f'\n[profile.dev.package.{name}]\nopt-level=3\n'
    (probe / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    shutil.copyfile(stage / 'Cargo.lock', probe / 'Cargo.lock')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(stage / 'target')
    commands = [
        ('java-compile', [str(jdk / 'javac.exe'), '-classpath', os.pathsep.join(libraries), '-d', str(classes), str(oracle)]),
        ('java-oracle', [str(jdk / 'java.exe'), '-classpath', os.pathsep.join([str(classes), *libraries]), 'CooldownComponentFacts', str(out / 'oracle.json')]),
        ('rust-probe', ['cargo', '+stable', 'run', '--manifest-path', str(probe / 'Cargo.toml'), '--', str(out / 'oracle.json')]),
    ]
    results = []
    for name, command in commands:
        started = time.monotonic()
        with (out / (name + '.stdout.log')).open('w', encoding='utf-8') as stdout, (out / (name + '.stderr.log')).open('w', encoding='utf-8') as stderr:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
        results.append({'name': name, 'exitCode': result.returncode, 'seconds': round(time.monotonic() - started, 3)})
        if result.returncode:
            break
    paths = [oracle, HERE / 'cooldown-component-probe.rs', HERE / 'cooldown-components-patch.py',
             stage / 'crates/horizon-client/src/item_sync.rs', stage / 'crates/horizon-client/src/lan/guest.rs', stage / 'crates/horizon-items/src/cooldowns.rs']
    report = {'passed': len(results) == len(commands) and all(r['exitCode'] == 0 for r in results),
              'scope': 'Actual MC26.3 UseCooldown stream bytes and ticks; saved component through actual generated full Horizon lan::guest::stack_to_network, Stack wire roundtrip and item_sync::from_network, cooldown group and map; removal and default fallback. Does not validate GUI pixels or whole-client parity.',
              'commands': results, 'minecraftSha256': hashlib.sha256(mc.read_bytes()).hexdigest(),
              'sourceSha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}}
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
