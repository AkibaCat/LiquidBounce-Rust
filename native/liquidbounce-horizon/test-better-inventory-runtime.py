#!/usr/bin/env python3
"""Complete engine against the real generated API; explicit BI registration shims."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--native',required=True);p.add_argument('--engine-source',help='Explicit frozen complete production engine baseline, before unrelated ABI changes');args=p.parse_args();out=(ROOT/args.out).resolve();native=(ROOT/args.native).resolve();engine_source=(ROOT/args.engine_source).resolve() if args.engine_source else HERE/'engine'
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True)
    baseline={'path':str(engine_source),'sources':{str(path.relative_to(engine_source)):hashlib.sha256(path.read_bytes()).hexdigest() for path in engine_source.rglob('*.rs')}}
    baseline_shims=engine_source.parent/'integration-shims.json'
    if baseline_shims.is_file():baseline['recordedIntegrationShims']=json.loads(baseline_shims.read_text(encoding='utf-8'))
    for label,source in [('engine',engine_source),('core',HERE.parent/'liquidbounce-core'),('behaviors',HERE.parent/'liquidbounce-behaviors')]:shutil.copytree(source,out/label,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','.git'))
    overlays={}
    for relative in ['src/better_inventory_draw.rs','src/better_inventory_runtime.rs','src/config/better_inventory.rs']:
        source=HERE/'engine'/relative;shutil.copy2(source,out/'engine'/relative);overlays[relative]=hashlib.sha256(source.read_bytes()).hexdigest()
    changes=[]
    def edit(relative,before,after):
        path=out/relative;text=path.read_text(encoding='utf-8')
        if text.count(before)!=1:raise ValueError((relative,before,text.count(before)))
        path.write_text(text.replace(before,after,1),encoding='utf-8');changes.append({'file':relative,'before':before,'after':after})
    manifest=out/'engine/Cargo.toml';manifest.write_text('[package]\nname="liquidbounce-horizon"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[features]\nbehavior-hooks=["dep:liquidbounce-behaviors"]\nwebview=[]\n[dependencies]\nhorizon-client={path='+json.dumps((native/'crates/horizon-client').as_posix())+',default-features=true}\nliquidbounce-core={path="../core"}\nliquidbounce-behaviors={path="../behaviors",optional=true}\nserde_json={version="1",features=["arbitrary_precision"]}\n\n[patch.crates-io]\n'+''.join(crate+'={path='+json.dumps((native/'third_party'/crate).as_posix())+'}\n' for crate in ['wgpu-hal','naga','libmimalloc-sys'])+'\n[profile.dev]\ndebug="line-tables-only"\n',encoding='utf-8');shutil.copy2(native/'Cargo.lock',out/'engine/Cargo.lock')
    lib=out/'behaviors/src/lib.rs';text=lib.read_text(encoding='utf-8')
    if 'pub mod better_inventory;' not in text:lib.write_text(text+'\npub mod better_inventory;\n',encoding='utf-8')
    lib=out/'engine/src/lib.rs';text=lib.read_text(encoding='utf-8')
    if 'mod better_inventory_draw;' not in text:lib.write_text(text+'\n#[cfg(feature="behavior-hooks")]\nmod better_inventory_draw;\n#[cfg(feature="behavior-hooks")]\nmod better_inventory_runtime;\n',encoding='utf-8')
    if 'pub better_inventory:' not in (out/'engine/src/behaviors.rs').read_text(encoding='utf-8'):
        edit('engine/src/behaviors.rs','    pub inventory: crate::inventory::InventoryActor,','    pub inventory: crate::inventory::InventoryActor,\n    pub better_inventory: liquidbounce_behaviors::better_inventory::BetterInventory,')
        edit('engine/src/behaviors.rs','            inventory: Default::default(),','            inventory: Default::default(),\n            better_inventory: Default::default(),')
    if 'fn client_better_inventory_render(' not in (out/'engine/src/runtime.rs').read_text(encoding='utf-8'):
        edit('engine/src/runtime.rs','    fn client_auto_tool_frame(','    fn client_better_inventory_render(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBetterInventoryRenderEvent<\'_>){self.better_inventory_render(event.query,event.draw);}\n\n    #[cfg(feature="behavior-hooks")]\n    fn client_auto_tool_frame(')
    (out/'integration-shims.json').write_text(json.dumps(changes,indent=2)+'\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/auto-weapon-native-target');argv=['cargo','+stable','check','--manifest-path',str(manifest),'--features','behavior-hooks','--lib'];start=time.monotonic()
    with (out/'check.stdout.log').open('w',encoding='utf-8') as stdout,(out/'check.stderr.log').open('w',encoding='utf-8') as stderr:run=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
    scope='Complete production engine baseline, shared real inventory actor/executor and lifecycle, current complete BI render/behavior sources, live setting and lastClickedSlot reads, and real generated Horizon event ABI. Only absent BI field/module/hook declarations are supplied as explicit temporary recorded shims. Actual game and canonical module/UI registration are separate.'
    if args.engine_source:scope+=' Explicit frozen engine baseline predates concurrent Timer ABI integration; no Timer stubs or business-method removals are supplied. Final canonical cumulative Timer/BI integration remains a parent check.'
    report={'passed':run.returncode==0,'commands':[{'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)}],'engineBaseline':baseline,'currentEngineOverlays':overlays,'integrationShims':changes,'sources':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('better_inventory*.rs')},'scope':scope}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
