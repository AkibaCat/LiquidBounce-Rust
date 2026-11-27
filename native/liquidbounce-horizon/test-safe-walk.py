#!/usr/bin/env python3
"""Small SafeWalk policy differential fixture; no native client is built."""
import hashlib
import json
import os
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OUT = ROOT / 'build/safe-walk-tests'

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    if not (OUT/'oracle.json').is_file():
        raise SystemExit('Run collect-safe-walk-original.py first')
    sources = {
        'safe_walk':ROOT/'native/liquidbounce-behaviors/src/safe_walk.rs',
        'random':ROOT/'native/liquidbounce-behaviors/src/random.rs',
        'rotations':ROOT/'native/liquidbounce-behaviors/src/rotations.rs',
        'java_case':ROOT/'native/liquidbounce-behaviors/src/navigation/java_case.rs',
        'tests':HERE/'safe-walk-tests.rs',
    }
    lib = '#![cfg(test)]\n#![allow(dead_code)]\n'
    for name, path in sources.items():
        lib += f'#[path="{path.as_posix()}"]pub mod {name};\n'
    lib += 'pub mod navigation{pub use crate::java_case;}\n'
    (OUT/'lib.rs').write_text(lib, encoding='utf-8')
    (OUT/'Cargo.toml').write_text('[package]\nname="liquidbounce-safe-walk-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n', encoding='utf-8')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(ROOT/'build/safe-walk-tests/target')
    commands = []
    for name, argv in [
        ('test',['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml')]),
        ('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings']),
    ]:
        result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace')
        (OUT/(name+'.log')).write_text(result.stdout+result.stderr, encoding='utf-8')
        print(result.stdout+result.stderr, end='')
        commands.append(dict(name=name, exitCode=result.returncode))
        if result.returncode:
            break
    owned = [Path(__file__),HERE/'collect-safe-walk-original.py',*sources.values()]
    report = dict(
        passed=len(commands)==2 and all(c['exitCode']==0 for c in commands),
        commands=commands,
        oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),
        sourceSha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in owned},
        scope='500 untouched ModuleSafeWalk and RefreshableRangeValue scenarios, exact state/input/lazy query trace and real JDK random draws; geometry/native/full client differential not_run',
    )
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(0 if report['passed'] else 1)

if __name__ == '__main__':
    main()
