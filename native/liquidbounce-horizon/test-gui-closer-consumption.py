#!/usr/bin/env python3
"""Unchanged Game helper and transformed production screen boundaries, real Text."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def block(text,start):
    first=text.index('{',start);end=first+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end],end
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--native',required=True);a=p.parse_args();out=(ROOT/a.out).resolve();native=(ROOT/a.native).resolve()
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);stage=native/'horizon/crates/horizon-client/src'
    host=(HERE/'gui-closer-host.rs').read_text(encoding='utf8');assert host in (stage/'game/native_hooks.rs').read_text(encoding='utf8')
    storage=(stage/'game/storage.rs').read_text(encoding='utf8');opening,_=block(storage,storage.index('{ let client_gui_closer_next'))
    options=(stage/'game/options.rs').read_text(encoding='utf8');start=options.index('let client_gui_closer_next = if let Some(screen)');guard=options.index('if self.client_gui_closer_accept_screen',start);_,end=block(options,guard);dynamic=options[start:end]
    remote=(stage/'game/remote.rs').read_text(encoding='utf8');start=remote.index('let client_gui_closer_next = if self.creative');guard=remote.index('if self.client_gui_closer_accept_screen',start);_,end=block(remote,guard);player=remote[start:end]
    hooks=(stage/'game/native_hooks.rs').read_text(encoding='utf8');start=hooks.index('let client_gui_closer_next = if self.creative');guard=hooks.index('if self.client_gui_closer_accept_screen',start);_,end=block(hooks,guard);native_player=hooks[start:end]
    station_text=(stage/'game/remote_stations.rs').read_text(encoding='utf8');station,_=block(station_text,station_text.index('{ let client_gui_closer_next'))
    merchant_text=(stage/'game/trading.rs').read_text(encoding='utf8');merchant,_=block(merchant_text,merchant_text.index('{ let client_gui_closer_next'))
    start=remote.index('if !self.client_gui_closer_accept_screen(Screen::Inventory)');end=remote.index('        if let Some(key) = self.container {',start);server_boundary=remote[start:end]
    for path,count in [('game.rs',2),('game/boats.rs',1),('game/mount_menu.rs',1),('game/minecarts.rs',1),('game/remote_stations.rs',1),('game/storage.rs',1),('game/trading.rs',1),('game/play_keys.rs',1),('game/remote.rs',2),('game/game_mode.rs',1),('game/options.rs',1),('game/native_hooks.rs',1)]:
        actual=(stage/path).read_text(encoding='utf8').count('self.client_gui_closer_accept_screen(');assert actual==count,(path,actual,count)
    assert remote.index('if !self.client_gui_closer_accept_screen(Screen::Inventory)')<remote.index('self.open_native_screen(key, target, p.container, &menu, extra);')
    remote_open,_=block(remote,remote.index('fn remote_open_screen('))
    assert 'reset_screen' not in remote_open[:remote_open.index('if menu == "minecraft:lectern"')]
    assert 'self.recipe_ui.reset_screen();\n            self.remote_open_lectern' in remote_open
    assert 'self.recipe_ui.reset_screen();\n                // Unsupported menu:' in remote_open
    for temporary,next_fn in [('fn client_inventory_mapping(', 'pub(super) fn native_client_inventory_snapshot('),('fn native_client_inventory_click(', 'pub(super) fn native_client_inventory_creative(')]:
        context=hooks[hooks.index(temporary):hooks.index(next_fn,hooks.index(temporary))]
        assert 'client_gui_closer_accept_screen' not in context
    snapshot=out/'behaviors';shutil.copytree(ROOT/'native/liquidbounce-behaviors',snapshot,dirs_exist_ok=True,ignore=shutil.ignore_patterns('target'))
    lib=snapshot/'src/lib.rs';body=lib.read_text(encoding='utf8');lib.write_text(body+('' if 'pub mod gui_closer;' in body else '\npub mod gui_closer;\n'),encoding='utf8')
    manifest=snapshot/'Cargo.toml';manifest_text=manifest.read_text(encoding='utf8')
    if "java_regex" not in manifest_text:manifest.write_text(manifest_text+'\njava_regex={path="vendor/java_regex"}\n',encoding='utf8')
    fixture=(HERE/'gui-closer-consumption-fixture.rs').read_text(encoding='utf8').replace('/*HOST*/',host).replace('/*STORAGE_OPEN*/',opening).replace('/*DYNAMIC_RETURN*/',dynamic).replace('/*PLAYER_INVENTORY*/',player).replace('/*NATIVE_PLAYER_INVENTORY*/',native_player).replace('/*STATION_OPEN*/',station).replace('/*MERCHANT_OPEN*/',merchant).replace('/*SERVER_BOUNDARY*/',server_boundary)
    (out/'lib.rs').write_text(fixture,encoding='utf8');(out/'host.rs').write_text(host,encoding='utf8')
    (out/'screen-boundaries.json').write_text(json.dumps({'storage':opening,'dynamic':dynamic,'player':player,'native_player':native_player,'station':station,'merchant':merchant,'server_boundary':server_boundary},indent=2)+'\n',encoding='utf8')
    core=(ROOT/'upstream/Horizon/crates/horizon-client-core').as_posix()
    (out/'Cargo.toml').write_text('[package]\nname="gui-closer-consumption-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nliquidbounce-behaviors={path="behaviors"}\nhorizon-client-core={path="'+core+'"}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--locked','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();run=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
        for stream in ['stdout','stderr']:(out/f'{name}.{stream}.log').write_text(getattr(run,stream),encoding='utf8')
        commands.append({'argv':argv,'exitCode':run.returncode,'seconds':round(time.monotonic()-start,3)})
        if run.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Unchanged Game helper, exact transformed production storage/player/native-keymapping/station/merchant/dynamic return bodies and server acceptance prefix, actual GUICloser and actual Horizon Text. Explicit recording Game/title providers. Tests cancel before old-screen lifecycle/keymapping/init/recipe reset/capture/use release, evaluate dynamic RHS once, retain no cancellation flag, and verify native message spans/copy/hover. Source assertions cover all14 entry points, preserve lectern/unsupported reset behavior and exclude two temporary inventory contexts. Full concrete Game compilation and actual game execution remain separate.','nativeSnapshot':str(native.relative_to(ROOT)),'commands':commands,'snapshotSha256':{str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*') if p.is_file() and p.suffix in ['.rs','.bin','.toml','.json'] and p.name!='report.json'}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
