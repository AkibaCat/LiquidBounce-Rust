#!/usr/bin/env python3
"""Complete production config and behavior snapshot; absent registration declarations only."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    spec=importlib.util.spec_from_file_location('config_base',HERE/'test-hand-derp.py');base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base);base.behaviors(out)
    manifest=out/"behaviors/Cargo.toml";manifest_text=manifest.read_text(encoding="utf8")
    if "java_regex" not in manifest_text:manifest.write_text(manifest_text+'\njava_regex={path="vendor/java_regex"}\n',encoding="utf8")
    lib=out/'behaviors/src/lib.rs';text=lib.read_text(encoding='utf8')
    if 'pub mod gui_closer;' not in text:lib.write_text(text+'\npub mod gui_closer;\n',encoding='utf8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf8');shim='mod gui_closer;' not in config
    if shim:
        def once(before,after):
            nonlocal config
            if config.count(before)!=1:raise ValueError((before,config.count(before)))
            config=config.replace(before,after,1)
        once('mod skin_derp;','mod skin_derp;\n#[path="config/gui_closer.rs"] mod gui_closer;\npub use gui_closer::{GuiCloserSetting,serialize_gui_closer};')
        once('    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),','    SkinDerp => ("SkinDerp", "Fun", "skinDerp"),\n    GuiCloser => ("GUICloser", "Misc", "guiCloser"),')
        if config.count('    SkinDerp(SkinDerpSetting),')!=2:raise ValueError('SettingChange/RuntimeRequest anchors')
        config=config.replace('    SkinDerp(SkinDerpSetting),','    SkinDerp(SkinDerpSetting),\n    GuiCloser(GuiCloserSetting),')
        once('        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),','        ModuleKind::SkinDerp => skin_derp::parse_skin_derp(&mut patch, &root),\n        ModuleKind::GuiCloser => gui_closer::parse_gui_closer(&mut patch, &root),')
        once('        SettingChange::SkinDerp(value) => {','        SettingChange::GuiCloser(value) => { return ApplyResult::Runtime(RuntimeRequest::GuiCloser(value.clone())); }\n        SettingChange::SkinDerp(value) => {')
    (out/'lib.rs').write_text(config,encoding='utf8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
    (out/'Cargo.toml').write_text('[package]\nname="gui-closer-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/better-inventory-target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf8')
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'registrationShim':shim,'scope':'Complete production config/behavior snapshot with declaration-only absent GuiCloser registration. File/Interop trees, private value inclusion, TEXT inner metadata, Java Regex string/null parsing, atomic list replacement, order/duplicates/null retention, Unicode choice parsing and independent leaf failure. No actual game claim.','commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*') if p.is_file() and p.suffix in ['.rs','.bin','.toml']}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
