#!/usr/bin/env python3
"""Actual mod API generator regression: rustfmt's multi-field dispatch commas."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import time
import tomllib

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    if out.exists():
        raise SystemExit('Use a fresh evidence directory')
    out.mkdir(parents=True)
    relative = 'crates/horizon-mod-api-gen/src/lib.rs'
    original = (ROOT / 'upstream/Horizon' / relative).read_text(encoding='utf8')
    changed = {}

    def once(text, before, after):
        assert text.count(before) == 1, before
        return text.replace(before, after, 1)

    def edit(path, transform):
        if path == relative:
            changed[path] = transform(original)

    spec = importlib.util.spec_from_file_location('text_field_patch', HERE / 'text-field-protect-patch.py')
    patch = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(patch)
    patch.apply(edit, once)
    fixed = changed[relative]
    baseline = fixed.replace('''    let formatted = rustfmt(render)?;
    let formatted = std::str::from_utf8(&formatted).map_err(|e| e.to_string())?;
    if tokens(&committed)? == tokens(formatted)? {''', '''    if tokens(&committed)? == tokens(render)? {''')
    assert baseline != fixed
    commands = []
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(ROOT / 'build/text-field-render-generation-target')
    workspace = tomllib.loads((ROOT / 'upstream/Horizon/Cargo.toml').read_text(encoding='utf8'))['workspace']['package']
    for name, source in [('baseline', baseline), ('fixed', fixed)]:
        crate = out / name
        shutil.copytree(ROOT / 'upstream/Horizon/crates/horizon-mod-api-gen/src', crate / 'src')
        shutil.copyfile(ROOT / 'upstream/Horizon/crates/horizon-mod-api-gen/Cargo.toml', crate / 'Cargo.toml')
        manifest = (crate / 'Cargo.toml').read_text(encoding='utf8')
        for field in ['edition', 'rust-version', 'license']:
            manifest = manifest.replace(f'{field}.workspace = true', f'{field} = {json.dumps(workspace[field])}')
        manifest = manifest.replace('[lints]\nworkspace = true', '[workspace]')
        (crate / 'Cargo.toml').write_text(manifest, encoding='utf8')
        (crate / 'src/lib.rs').write_text(source, encoding='utf8')
        tasks = [('regression', ['test', '--lib', 'liquidbounce_render_generation_tests'])]
        if name == 'fixed':
            tasks.append(('stale', ['test', '--lib', 'stale_render_hooks_fail_the_build']))
            tasks.append(('clippy', ['clippy', '--all-targets', '--', '-D', 'warnings']))
        for label, argv in tasks:
            arguments = ['cargo', '+stable', *argv]
            started = time.monotonic()
            result = subprocess.run(arguments, cwd=crate, env=env, capture_output=True)
            for stream in ['stdout', 'stderr']:
                (out / f'{name}-{label}.{stream}.log').write_bytes(getattr(result, stream))
            commands.append(dict(variant=name, task=label, argv=arguments,
                                 exitCode=result.returncode, seconds=round(time.monotonic() - started, 3)))
    baseline_log = (out / 'baseline-regression.stdout.log').read_text(encoding='utf8')
    reproduced = commands[0]['exitCode'] != 0 and 'is out of date with the render hooks' in baseline_log
    report = dict(passed=reproduced and all(row['exitCode'] == 0 for row in commands[1:]),
                  baselineFalseStaleErrorReproduced=reproduced, commands=commands,
                  generatorBeforeSha256=hashlib.sha256(original.encode()).hexdigest(),
                  generatorAfterSha256=hashlib.sha256(fixed.encode()).hexdigest(),
                  scope='Real pinned horizon-mod-api-gen with owned patch, actual rustfmt and syn. '
                        'Original checker falsely rejects formatted multi-argument dispatch; fixed checker accepts '
                        'that exact generated code and continues rejecting changed signatures. Client integration remains separate.')
    (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps(dict(passed=report['passed'], report=str(out / 'report.json'))))
    raise SystemExit(0 if report['passed'] else 1)


if __name__ == '__main__':
    main()
