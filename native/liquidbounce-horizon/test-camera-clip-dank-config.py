#!/usr/bin/env python3
"""Snapshot and compile the real CameraClip/Dank config adapters and behavior sources."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/camera-clip-dank-config-01')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True, exist_ok=True)
    source = (HERE / 'engine/src/config.rs').read_text(encoding='utf-8')
    for module in ['camera_clip', 'dank_bobbing']:
        assert f'mod {module};' in source, f'Parent must register {module} first'
    (out / 'config.rs').write_text(source, encoding='utf-8')
    shutil.copytree(HERE / 'engine/src/config', out / 'config', dirs_exist_ok=True)
    behaviors = ROOT / 'native/liquidbounce-behaviors'
    shutil.copytree(behaviors / 'src', out / 'behaviors/src', dirs_exist_ok=True)
    shutil.copyfile(behaviors / 'Cargo.toml', out / 'behaviors/Cargo.toml')
    (out / 'Cargo.toml').write_text('''[package]
name="camera-clip-dank-config-tests"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="config.rs"
[dependencies]
liquidbounce-behaviors={path="behaviors"}
serde_json={version="1",features=["arbitrary_precision"]}
''', encoding='utf-8')
    snapshot = {str(path.relative_to(out)): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in out.rglob('*.rs')}
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(HERE / 'config-tests/target'))
    commands = []
    for name, argv in [
        ('test', ['cargo', '+stable', 'test', '--offline', '--manifest-path', str(out / 'Cargo.toml')]),
        ('clippy', ['cargo', '+stable', 'clippy', '--offline', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings']),
    ]:
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True,
                                encoding='utf-8', errors='replace')
        (out / (name + '.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        commands.append({'argv': argv, 'exitCode': result.returncode})
        print(result.stdout + result.stderr)
        if result.returncode:
            break
    report = {'passed': all(command['exitCode'] == 0 for command in commands),
              'scope': 'Unmodified snapshot of complete production config adapters and behavior sources; no registration shim. Six CameraClip/Dank tests plus existing config regressions. Actor, Host and rendering are separate integration checks.',
              'commands': commands, 'snapshotSha256': snapshot}
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
