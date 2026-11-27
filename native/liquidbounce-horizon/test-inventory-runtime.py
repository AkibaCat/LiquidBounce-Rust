#!/usr/bin/env python3
"""Compile the real adapter with explicit, recorded integration shims."""
from pathlib import Path
import argparse,hashlib,json,os,re,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);parser.add_argument('--native',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve();native=(ROOT/args.native).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite verification evidence')
    out.mkdir(parents=True,exist_ok=True)
    for label,source in [('engine',HERE/'engine'),('core',HERE.parent/'liquidbounce-core'),('behaviors',HERE.parent/'liquidbounce-behaviors')]:
        shutil.copytree(source,out/label,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','.git'))
    changes=[]
    def edit(relative,before,after):
        path=out/relative;text=path.read_text(encoding='utf-8')
        if before not in text:raise ValueError((relative,before))
        path.write_text(text.replace(before,after),encoding='utf-8');changes.append({'file':relative,'before':before,'after':after})
    engine=out/'engine/Cargo.toml'
    engine.write_text('[package]\nname="liquidbounce-horizon"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[features]\nbehavior-hooks=["dep:liquidbounce-behaviors"]\nwebview=[]\n[dependencies]\nhorizon-client={path='+json.dumps(str(native/'crates/horizon-client').replace('\\','/'))+',default-features=true}\nliquidbounce-core={path="../core"}\nliquidbounce-behaviors={path="../behaviors",optional=true}\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf-8')
    # Preserve the pinned client's vendored renderer patches and dependency
    # resolution when this adapter is checked from its own temporary workspace.
    with engine.open('a',encoding='utf-8') as manifest:
        manifest.write('\n[patch.crates-io]\n')
        for crate in ['wgpu-hal','naga','libmimalloc-sys']:
            manifest.write(crate+'={path='+json.dumps((native/'third_party'/crate).as_posix())+'}\n')
        manifest.write('\n[profile.dev]\ndebug="line-tables-only"\n')
    shutil.copy2(native/'Cargo.lock',out/'engine/Cargo.lock')
    for name in ['inventory_manager','vomit']:
        path=out/'behaviors/src/lib.rs';text=path.read_text(encoding='utf-8')
        if f'pub mod {name};' not in text:path.write_text(text+f'\npub mod {name};\n',encoding='utf-8')
    edit('engine/src/lib.rs','mod auto_tool_packets;','#[path="inventory_packets.rs"]\nmod auto_tool_packets;\n#[cfg(feature="behavior-hooks")]\nmod inventory;')
    edit('engine/src/lib.rs','mod auto_tool_runtime;','#[path="inventory_runtime.rs"]\nmod auto_tool_runtime;')
    edit('engine/src/behaviors.rs','pub inventory: crate::auto_tool::InventoryActor,','pub inventory: crate::inventory::InventoryActor,\n    pub vomit: liquidbounce_behaviors::vomit::Vomit,')
    edit('engine/src/behaviors.rs','            inventory: Default::default(),','            inventory: Default::default(),\n            vomit: Default::default(),')
    edit('engine/src/coroutine_ticker.rs','    SkinDerp,','    SkinDerp,\n    InventoryManager,')
    edit('engine/src/runtime.rs','                WaitOwner::SkinDerp => self.skin_derp_callback(host, in_game, task.callback),','                WaitOwner::SkinDerp => self.skin_derp_callback(host, in_game, task.callback),\n                WaitOwner::InventoryManager => self.inventory_callback(host,in_game,task.callback),')
    edit('engine/src/runtime.rs','    fn client_auto_tool_frame(','    fn client_inventory_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientInventoryWakeEvent) {self.inventory_frame(host);}\n\n    #[cfg(feature="behavior-hooks")]\n    fn client_auto_tool_frame(')
    (out/'integration-shims.json').write_text(json.dumps(changes,indent=2)+'\n',encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/auto-weapon-native-target');commands=[]
    for name,argv in [('check',['cargo','+stable','check','--manifest-path',str(engine),'--features','behavior-hooks','--lib'])]:
        start=time.monotonic()
        with (out/f'{name}.stdout.log').open('w',encoding='utf-8') as stdout,(out/f'{name}.stderr.log').open('w',encoding='utf-8') as stderr:run=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'commands':commands,'integrationShims':changes,'sources':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [out/'engine/src/inventory.rs',out/'engine/src/inventory_runtime.rs',out/'engine/src/inventory_packets.rs']},'scope':'Full production Rust engine adapter compile against the independently generated real Horizon API. Explicit temporary field/module/wait/hook integration shims are recorded; no shared staging or canonical runtime edited. Config, UI registration and real game execution remain separate.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
