#!/usr/bin/env python3
"""Unchanged final Game helper and real production container click/release state."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);parser.add_argument('--native',required=True);args=parser.parse_args();out=(ROOT/args.out).resolve();native=(ROOT/args.native).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    host=(HERE/'item-scroller-host.rs').read_text(encoding='utf8');stage=native/'horizon/crates/horizon-client/src'
    assert host in (stage/'game/native_hooks.rs').read_text(encoding='utf8')
    game=(stage/'game.rs').read_text(encoding='utf8');cancel=game.index('if pressed && matches!(self.screen, Screen::Inventory | Screen::Creative)\n                        && self.client_item_scroller_cancel_click()')
    assert cancel<game.index('.container_mouse_key(button, self.creative)',cancel)
    for path in ['game.rs','game/stations.rs','game/trading.rs','game/beacon.rs','game/creative_screen.rs']:
        text=(stage/path).read_text(encoding='utf8');assert text.count('self.client_better_inventory_end(batch);\n        self.client_item_scroller_render(batch);')==1,path
    (out/'lib.rs').write_text((HERE/'item-scroller-consumption-fixture.rs').read_text(encoding='utf8').replace('/*HOST*/',host),encoding='utf8')
    (out/'host.rs').write_text(host,encoding='utf8');shutil.copyfile(stage/'container_screen.rs',out/'container_screen.rs');shutil.copyfile(ROOT/'native/liquidbounce-behaviors/src/item_scroller.rs',out/'item_scroller.rs')
    (out/'Cargo.toml').write_text('[package]\nname="item-scroller-consumption-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf8')
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Unchanged final Game helper inside explicit recording Game/menu/network fixtures plus actual complete production container_screen.rs, actual ItemScroller decisions, six focused helper tests and existing production container tests. Verifies original callback button=1, pre-click stack copy, physical input, carried/hover order, cancellation without reset, quick-craft-preserving skip release, creative display/trash and all46 packets/cache/reset order. Source assertions verify all five render seams and mouse HEAD before hotbar mapping. Full native build validates concrete Game types; no actual game/GPU claim.','nativeSnapshot':str(native.relative_to(ROOT)),'commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.rs')}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
