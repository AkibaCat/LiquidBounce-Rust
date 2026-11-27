#!/usr/bin/env python3
"""Execute unchanged production native helper bodies with a recording HudBatch."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);args=p.parse_args();out=(ROOT/args.out).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);(out/'behaviors/src').mkdir(parents=True,exist_ok=True)
    shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/better_inventory.rs',out/'behaviors/src/better_inventory.rs')
    shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/better_inventory_locale.rs',out/'behaviors/src/better_inventory_locale.rs')
    (out/'behaviors/src/lib.rs').write_text('pub mod better_inventory;\n',encoding='utf-8');(out/'behaviors/Cargo.toml').write_text('[package]\nname="liquidbounce-behaviors"\nversion="0.0.0"\nedition="2024"\n',encoding='utf-8')
    helper=(HERE/'better-inventory-host.rs').read_text(encoding='utf-8');fixture=(HERE/'better-inventory-consumption-fixture.rs').read_text(encoding='utf-8').replace('/*HOST*/',helper)
    (out/'lib.rs').write_text('mod better_inventory_draw;\nmod native {\nuse crate::better_inventory_draw;\n'+fixture.replace('#![allow(dead_code)]','')+'\n}\npub(crate) use native::{Creative,item_data,item_render};\n',encoding='utf-8')
    # The fixture intentionally defines uncalled service surfaces for the full
    # unchanged helper. Allow only dead code and test-only argument counts.
    text=(out/'lib.rs').read_text(encoding='utf-8');(out/'lib.rs').write_text('#![allow(dead_code,clippy::too_many_arguments)]\n'+text,encoding='utf-8')
    shutil.copyfile(HERE/'engine/src/better_inventory_draw.rs',out/'better_inventory_draw.rs')
    shutil.copyfile(HERE/'better-inventory-locale-oracle.json',out/'locale-oracle.json')
    (out/'locale-platform/src').mkdir(parents=True,exist_ok=True)
    shutil.copyfile(HERE/'better-inventory-locale-platform.rs',out/'locale-platform/src/lib.rs')
    (out/'locale-platform/Cargo.toml').write_text('[package]\nname="horizon-system"\nversion="0.0.0"\nedition="2024"\n',encoding='utf-8')
    text=(out/'lib.rs').read_text(encoding='utf-8');(out/'lib.rs').write_text('#![forbid(unsafe_code)]\n'+text,encoding='utf-8')
    (out/'Cargo.toml').write_text('[package]\nname="better-inventory-consumption"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nhorizon-system={path="locale-platform"}\nserde_json="1"\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/better-inventory-consumption-target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:run=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'commands':commands,'helperSha256':hashlib.sha256(helper.encode()).hexdigest(),'scope':'Unchanged complete production native helper bodies and draw protocol, actual behavior decisions, recording HudBatch with the production vertex scaling body. Verifies slot->menu indices incl creative/trash, colors, rects, textures, real entry duration, source pose origin and nested preview/text scaling, component holes/default/removal, skip filtering and carried gate. Graphics GPU, real component codecs, live runtime and game execution are separate native/full-game checks.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
