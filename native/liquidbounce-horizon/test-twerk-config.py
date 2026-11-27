#!/usr/bin/env python3
"""Verify Twerk config with a snapshot and an explicit registration shim."""
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
    parser.add_argument('--out', default='build/twerk-config-01')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True, exist_ok=True)
    config_source = HERE / 'engine/src/config.rs'
    config = config_source.read_text(encoding='utf-8')
    shim = 'mod twerk;' not in config

    def once(before, after):
        nonlocal config
        if config.count(before) != 1:
            raise ValueError(f'Expected one registration anchor: {before}')
        config = config.replace(before, after, 1)

    if shim:
        once('mod no_pose;', 'mod no_pose;\n#[path = "config/twerk.rs"]\nmod twerk;\npub use twerk::{TwerkSetting, serialize_twerk};')
        once('    NoPose => ("NoPose", "Movement", "noPose"),', '    NoPose => ("NoPose", "Movement", "noPose"),\n    Twerk => ("Twerk", "Fun", "twerk"),')
        if config.count('    NoPose(NoPoseSetting),') != 2:
            raise ValueError('Expected SettingChange and RuntimeRequest registration anchors')
        config = config.replace('    NoPose(NoPoseSetting),', '    NoPose(NoPoseSetting),\n    Twerk(TwerkSetting),')
        once('            | ModuleKind::NoPose', '            | ModuleKind::NoPose\n            | ModuleKind::Twerk')
        once('        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),', '        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),\n        ModuleKind::Twerk => twerk::parse_twerk(&mut patch, &root),')
        once('        SettingChange::NoPose(value) => {', '        SettingChange::Twerk(value) => {\n            return ApplyResult::Runtime(RuntimeRequest::Twerk(*value));\n        }\n        SettingChange::NoPose(value) => {')
    (out / 'config.rs').write_text(config, encoding='utf-8', newline='\n')
    shutil.copytree(HERE / 'engine/src/config', out / 'config', dirs_exist_ok=True)
    behavior = ROOT / 'native/liquidbounce-behaviors'
    shutil.copytree(behavior / 'src', out / 'behaviors/src', dirs_exist_ok=True)
    shutil.copyfile(behavior / 'Cargo.toml', out / 'behaviors/Cargo.toml')
    behavior_lib = out / 'behaviors/src/lib.rs'
    library = behavior_lib.read_text(encoding='utf-8')
    if 'pub mod twerk;' not in library:
        behavior_lib.write_text(library + '\npub mod twerk;\n', encoding='utf-8')
    manifest = '[package]\nname="twerk-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="config.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n'
    (out / 'Cargo.toml').write_text(manifest, encoding='utf-8')
    snapshot = {str(path.relative_to(out)): hashlib.sha256(path.read_bytes()).hexdigest() for path in out.rglob('*.rs')}
    env = os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR', str(HERE / 'config-tests/target'))
    commands = []
    for name, argv in [('test', ['cargo', '+stable', 'test', '--manifest-path', str(out / 'Cargo.toml')]), ('clippy', ['cargo', '+stable', 'clippy', '--locked', '--manifest-path', str(out / 'Cargo.toml'), '--all-targets', '--', '-D', 'warnings'])]:
        start = time.monotonic()
        run = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        for stream in ['stdout', 'stderr']:
            (out / f'{name}.{stream}.log').write_text(getattr(run, stream), encoding='utf-8')
        commands.append({'argv': argv, 'exitCode': run.returncode, 'durationSeconds': round(time.monotonic() - start, 3)})
        if run.returncode:
            break
    report = {
        'passed': all(command['exitCode'] == 0 for command in commands),
        'scope': 'Snapshot of complete production config helpers and behavior sources; explicit Twerk registration shim only if not yet present in shared engine. Actual Tree, Gson-compatible primitive adapters and RuntimeRequest forwarding. Actor/registry/game execution is outside this fixture.',
        'commands': commands,
        'registrationShimApplied': shim,
        'snapshotSha256': snapshot,
    }
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'passed': report['passed'], 'report': str(out / 'report.json')}, indent=2))


if __name__ == '__main__':
    main()
