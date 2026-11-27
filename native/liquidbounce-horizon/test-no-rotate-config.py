#!/usr/bin/env python3
"""Validate NoRotateSet config before its shared engine registration lands."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import time

ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',default='build/no-rotate-config-01')
    args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    src=HERE/'engine/src/config.rs'
    config=src.read_text(encoding='utf-8')
    if 'mod no_rotate_set;' not in config:
        config=config.replace('mod no_pose;','mod no_pose;\n#[path = "config/no_rotate_set.rs"]\nmod no_rotate_set;\npub use no_rotate_set::{NoRotateSetSetting, serialize_no_rotate_set};',1)
        config=config.replace('    NoPose => ("NoPose", "Movement", "noPose"),','    NoPose => ("NoPose", "Movement", "noPose"),\n    NoRotateSet => ("NoRotateSet", "Player", "noRotateSet"),',1)
        config=config.replace('    NoPose(NoPoseSetting),','    NoPose(NoPoseSetting),\n    NoRotateSet(NoRotateSetSetting),')
        config=config.replace('            | ModuleKind::NoPose','            | ModuleKind::NoPose\n            | ModuleKind::NoRotateSet',1)
        config=config.replace('        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),','        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),\n        ModuleKind::NoRotateSet => no_rotate_set::parse_no_rotate_set(&mut patch, &root),',1)
        config=config.replace('        SettingChange::NoPose(value) => {','        SettingChange::NoRotateSet(value) => {\n            return ApplyResult::Runtime(RuntimeRequest::NoRotateSet(*value));\n        }\n        SettingChange::NoPose(value) => {',1)
    (out/'config.rs').write_text(config,encoding='utf-8',newline='\n')
    shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
    behavior=ROOT/'native/liquidbounce-behaviors'
    shutil.copytree(behavior/'src',out/'behaviors/src',dirs_exist_ok=True)
    shutil.copyfile(behavior/'Cargo.toml',out/'behaviors/Cargo.toml')
    lib=out/'behaviors/src/lib.rs'
    text=lib.read_text(encoding='utf-8')
    if 'pub mod no_rotate_set;' not in text:lib.write_text(text+'\npub mod no_rotate_set;\n',encoding='utf-8')
    manifest='[package]\nname="no-rotate-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="config.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n'
    (out/'Cargo.toml').write_text(manifest,encoding='utf-8')
    snapshot={str(path.relative_to(out)):hashlib.sha256(path.read_bytes()).hexdigest() for path in out.rglob('*.rs')}
    env=os.environ.copy()
    env.setdefault('CARGO_TARGET_DIR',str(HERE/'config-tests/target'))
    commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        result=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(result,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if result.returncode:break
    report={'passed':all(command['exitCode']==0 for command in commands),'scope':'Snapshot of production config/helpers/behavior sources, with explicit NoRotateSet registration shim only if absent from shared config. Actual Tree/ConfigSystem adapters and tests; no actor/Host/game integration claim.','commands':commands,'snapshotSha256':snapshot,'registrationShimApplied':'mod no_rotate_set;' not in src.read_text(encoding='utf-8')}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')},indent=2))

if __name__=='__main__':main()
