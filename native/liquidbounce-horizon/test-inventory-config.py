#!/usr/bin/env python3
"""Compile all real config codecs with the Inventory global group added."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    shutil.copytree(ROOT/'native/liquidbounce-behaviors',out/'behaviors',dirs_exist_ok=True,ignore=shutil.ignore_patterns('target'))
    lib=out/'behaviors/src/lib.rs';source=lib.read_text(encoding='utf-8')
    for name in ['inventory_manager','vomit']:
        if f'pub mod {name};' not in source:source+=f'\npub mod {name};\n'
    lib.write_text(source,encoding='utf-8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8');shim='mod inventory_manager;' not in config
    if shim:config+='\n#[path="config/inventory_manager.rs"]\nmod inventory_manager;\npub use inventory_manager::{GlobalInventorySetting,GlobalInventoryPatch,parse_global_inventory,serialize_global_inventory};\n'
    (out/'lib.rs').write_text(config,encoding='utf-8');shutil.copytree(HERE/'engine/src/config',out/'config',dirs_exist_ok=True)
    (out/'Cargo.toml').write_text('[package]\nname="inventory-config-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/vomit-config-target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:run=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'commands':commands,'registrationShim':shim,'sourceSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*.rs')},'scope':'Whole production config + behaviors with explicit Inventory module-export shim. Real Inventory group UI metadata/file codec, raw Gson numeric parsing, partial-invalid leaf handling and actual shared-manager response timers. Actor/UI registration remain separate.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
