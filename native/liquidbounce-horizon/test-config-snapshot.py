#!/usr/bin/env python3
"""Compile the production config and behavior snapshot without gameplay or shims."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    out.relative_to((ROOT / 'build').resolve())
    if out.exists():
        raise SystemExit('Choose a fresh output directory to retain prior evidence')
    out.mkdir(parents=True)
    shutil.copyfile(HERE / 'engine/src/config.rs', out / 'config.rs')
    shutil.copytree(HERE / 'engine/src/config', out / 'config')
    behaviors = ROOT / 'native/liquidbounce-behaviors'
    shutil.copytree(behaviors / 'src', out / 'behaviors/src')
    shutil.copyfile(behaviors / 'Cargo.toml', out / 'behaviors/Cargo.toml')
    (out / 'Cargo.toml').write_text('''[package]
name="production-config-snapshot"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="config.rs"
[dependencies]
liquidbounce-behaviors={path="behaviors"}
serde_json={version="1",features=["arbitrary_precision"]}
''', encoding='utf-8')
    snapshot = {str(p.relative_to(out)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in out.rglob('*.rs')}
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(HERE / 'config-tests/target')
    commands = []
    for name, arguments in [
        ('test', ['test', '--lib']),
        ('clippy', ['clippy', '--locked', '--all-targets', '--', '-D', 'warnings']),
    ]:
        argv = ['cargo', '+stable', *arguments[:1], '--manifest-path', str(out/'Cargo.toml'), *arguments[1:]]
        started = time.monotonic()
        with (out / f'{name}.stdout.log').open('w', encoding='utf-8') as stdout, \
                (out / f'{name}.stderr.log').open('w', encoding='utf-8') as stderr:
            result = subprocess.run(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
        commands.append({'argv': argv, 'exitCode': result.returncode,
                         'durationSeconds': round(time.monotonic()-started, 3)})
        if result.returncode:
            break
    report = {'passed': all(c['exitCode'] == 0 for c in commands),
              'scope': 'Unmodified production config and behavior snapshots; no registration shim. '
                       'Does not verify the registry actor, native hooks, or gameplay.',
              'commands': commands, 'snapshotSha256': snapshot}
    (out/'report.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out/'report.json')}))
    raise SystemExit(0 if report['passed'] else 1)

if __name__ == '__main__':
    main()
