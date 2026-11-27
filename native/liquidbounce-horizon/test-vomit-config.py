#!/usr/bin/env python3
"""Complete production config codec; Vomit registration shim is test-only."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    spec=importlib.util.spec_from_file_location('vomit_config_base',HERE/'test-hand-derp.py');base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base);base.behaviors(out)
    lib=out/'behaviors/src/lib.rs';source=lib.read_text(encoding='utf-8')
    if 'pub mod vomit;' not in source:lib.write_text(source+'\npub mod vomit;\n',encoding='utf-8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8');shim='mod vomit;' not in config
    if shim:
        def once(before,after):
            nonlocal config
            if config.count(before)!=1:raise ValueError((before,config.count(before)))
            config=config.replace(before,after,1)
        once('mod skin_derp;','mod skin_derp;\n#[path="config/vomit.rs"]\nmod vomit;\npub use vomit::{VomitSetting,serialize_vomit};')
        once('    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),','    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),\n    Vomit => ("Vomit", "Fun", "vomit"),')
        if config.count('    SkinDerp(SkinDerpSetting),')!=2:raise ValueError('SettingChange/RuntimeRequest anchors')
        config=config.replace('    SkinDerp(SkinDerpSetting),','    SkinDerp(SkinDerpSetting),\n    Vomit(VomitSetting),')
        once('        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),','        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),\n        ModuleKind::Vomit => vomit::parse_vomit(&mut patch, &root),')
        once('        SettingChange::SkinDerp(value) => {','        SettingChange::Vomit(value) => { return ApplyResult::Runtime(RuntimeRequest::Vomit(value.clone())); }\n        SettingChange::SkinDerp(value) => {')
    (out/'lib.rs').write_text(config,encoding='utf-8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
    (out/'Cargo.toml').write_text('[package]\nname="vomit-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/vomit-config-target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':result.returncode,'durationSeconds':round(time.monotonic()-start,3)})
        if result.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'registrationShim':shim,'commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')},'scope':'Whole production configuration and behaviors, explicit test-only Vomit ModuleKind/enum forwarding registration if not yet present. Constraints old tick-to-ms migration, raw ranges, metadata, all enum subsets and partial-invalid mutable EnumSet imports. No game execution.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
