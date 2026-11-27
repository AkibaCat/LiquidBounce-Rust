#!/usr/bin/env python3
"""Complete actual config snapshot with an explicit registration-only shim."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',default='build/auto-weapon-config-01');args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8');shim='mod auto_weapon;' not in config
    def once(before,after):
        nonlocal config
        if config.count(before)!=1:raise ValueError((before,config.count(before)))
        config=config.replace(before,after,1)
    if shim:
        once('mod no_pose;','mod no_pose;\n#[path="config/auto_weapon.rs"]\nmod auto_weapon;\npub use auto_weapon::{AutoWeaponSetting,serialize_auto_weapon};')
        once('    NoPose => ("NoPose", "Movement", "noPose"),','    NoPose => ("NoPose", "Movement", "noPose"),\n    AutoWeapon => ("AutoWeapon", "Combat", "autoWeapon"),')
        if config.count('    NoPose(NoPoseSetting),')!=2:raise ValueError('SettingChange/RuntimeRequest anchors')
        config=config.replace('    NoPose(NoPoseSetting),','    NoPose(NoPoseSetting),\n    AutoWeapon(AutoWeaponSetting),')
        once('            | ModuleKind::NoPose','            | ModuleKind::NoPose\n            | ModuleKind::AutoWeapon')
        once('        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),','        ModuleKind::NoPose => no_pose::parse_no_pose(&mut patch, &root),\n        ModuleKind::AutoWeapon => auto_weapon::parse_auto_weapon(&mut patch, &root),')
        once('        SettingChange::NoPose(value) => {','        SettingChange::AutoWeapon(value) => {\n            return ApplyResult::Runtime(RuntimeRequest::AutoWeapon(value.clone()));\n        }\n        SettingChange::NoPose(value) => {')
    (out/'config.rs').write_text(config,encoding='utf-8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
    behavior=ROOT/'native/liquidbounce-behaviors';shutil.copytree(behavior/'src',out/'behaviors/src',dirs_exist_ok=True);shutil.copyfile(behavior/'Cargo.toml',out/'behaviors/Cargo.toml')
    library=out/'behaviors/src/lib.rs';text=library.read_text(encoding='utf-8')
    if 'pub mod auto_weapon;' not in text:library.write_text(text+'\npub mod auto_weapon;\n',encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="auto-weapon-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="config.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf-8')
    snapshot={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')}
    env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(HERE/'config-tests/target'));commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf-8')
        commands.append({'argv':argv,'exitCode':run.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Complete production config/helpers/behavior snapshots. Explicit AutoWeapon registration shim if shared engine has not integrated it. Actual six settings, Tree, Gson leaf adapters and RuntimeRequest forwarding; actor/game execution excluded.','commands':commands,'registrationShimApplied':shim,'snapshotSha256':snapshot}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
