#!/usr/bin/env python3
"""Reproducible framed packet/SilentHotbar regression evidence."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='build/slot-modules-02')
    args = parser.parse_args()
    out = (ROOT / args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    if (out / 'report.json').exists():
        raise SystemExit('Refusing to overwrite existing verification evidence')
    commands = []

    def run(name, argv):
        started = time.monotonic()
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
        for stream in ['stdout', 'stderr']:
            (out / f'{name}.{stream}.log').write_text(getattr(result, stream), encoding='utf-8')
        commands.append({'argv':argv, 'exitCode':result.returncode, 'durationSeconds':round(time.monotonic()-started,3), 'stdout':str(out / f'{name}.stdout.log'), 'stderr':str(out / f'{name}.stderr.log')})
        return result.returncode == 0

    import sys
    for name in ['packet', 'slot']:
        if not run(name+'-fixture',[sys.executable,str(HERE / f'prepare-{name}-fixture.py')]):
            raise SystemExit('Fixture generation failed')
    paths = ['slot-patch.py','slot-bridge.rs','engine/src/slots.rs','prepare-slot-fixture.py','packet-tests/slot-fixture-tests.rs','packet-tests/tests/generated_slots.rs','packet-tests/tests/slot_service.rs','test-slot-patch.py']
    hashes = {str((HERE / path).relative_to(ROOT)):hashlib.sha256((HERE / path).read_bytes()).hexdigest() for path in paths}
    old = json.loads((ROOT / 'build/slot-modules-01/report.json').read_text(encoding='utf-8'))
    upstream = old['upstreamSourceSha256']
    manifest = str(HERE / 'packet-tests/Cargo.toml')
    if run('test',['cargo','+stable','test','--locked','--manifest-path',manifest]):
        run('clippy',['cargo','+stable','clippy','--locked','--manifest-path',manifest,'--all-targets','--','-D','warnings'])
    report = {
        'passed':all(command['exitCode']==0 for command in commands),
        'scope':'Actual transformed Session.send/receive_play, Game carried cache/NoSlotSet/replay/identity guards, SilentHotbar service regressions. Mutating main-hand paths are separately verified by hotbar-access-01.',
        'commands':commands,
        'executedTests':sum(map(int,re.findall(r'test result: ok\. (\d+) passed', (out / 'test.stdout.log').read_text(encoding='utf-8')))),
        'sourceSha256':hashes,
        'sourceUnchangedDuringVerification':all(hashlib.sha256((ROOT / path).read_bytes()).hexdigest()==digest for path,digest in hashes.items()),
        'upstreamSourceSha256':upstream,
        'upstreamUnchanged':all(hashlib.sha256((ROOT / path).read_bytes()).hexdigest()==digest for path,digest in upstream.items()),
        'fullClientCompilationPassed':False,
        'fullGameIntegrationPassed':False,
    }
    report['passed'] &= report['sourceUnchangedDuringVerification'] and report['upstreamUnchanged']
    (out / 'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out / 'report.json')},indent=2))


if __name__=='__main__':
    main()
