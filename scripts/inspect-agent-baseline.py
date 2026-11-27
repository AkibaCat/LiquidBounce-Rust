#!/usr/bin/env python3
"""Record the pinned Agent source/archive and its published common snapshot."""
from pathlib import Path
import hashlib
import json
import zipfile

ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'upstream/Minecraft-Agent'
RESEARCH=ROOT/'research'
COMMON=BASE/'common/src/main/kotlin'
SNAPSHOT='1.0.0-20261120.103027-24'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    archive=RESEARCH/f'minecraft-agent-common-{SNAPSHOT}-sources.jar'
    same=[];different=[];missing=[]
    with zipfile.ZipFile(archive) as source:
        for name in source.namelist():
            if not name.endswith('.kt'):continue
            path=COMMON/name
            if not path.exists():missing.append(name)
            elif path.read_bytes()!=source.read(name):different.append(name)
            else:same.append(name)
    groups={}
    for path in COMMON.rglob('*.kt'):
        name=path.relative_to(COMMON).as_posix()
        group=name.removeprefix('net/ccbluex/agent/').split('/')[0]
        entry=groups.setdefault(group,{'files':0,'lines':0})
        entry['files']+=1;entry['lines']+=len(path.read_text(encoding='utf-8').splitlines())
    result={
        'commit':'0abf69ac35157f33348310326017f4e3fea6b95f',
        'commitDate':'2026-11-20T10:26:53Z',
        'liquidBounceCommit':'ae0eb396c07f22383789b28d5912d714c6ca0a86',
        'commonSnapshot':SNAPSHOT,
        'snapshotSourceComparison':{'sameFiles':len(same),'different':different,'missing':missing},
        'commonPackages':groups,
        'sha256':{str(path.relative_to(ROOT)):sha(path) for path in [
            RESEARCH/'Minecraft-Agent-0abf69ac-source.zip',archive,
            RESEARCH/f'minecraft-agent-common-{SNAPSHOT}.jar',
            RESEARCH/'minecraft-agent-common-metadata.xml',
            RESEARCH/'minecraft-agent-baseline-commit.json']},
        'runtimeRule':'All player-facing implementation remains Rust; the original JVM artifact is development-oracle input only.',
        'coverage':'Dependency source provenance only; not an AutoWalk implementation or gameplay validation.'}
    (RESEARCH/'minecraft-agent-baseline.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(result,indent=2))
    if different or missing:raise SystemExit('Published sources do not match the pinned archive')

if __name__=='__main__':main()
