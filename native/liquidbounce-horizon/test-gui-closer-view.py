#!/usr/bin/env python3
"""Exact retained-view source with explicit recording menu/World providers."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
def body(text,name):
    start=text.index(name);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]
def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--native',required=True);a=p.parse_args();out=ROOT/a.out;native=ROOT/a.native
    if (out/'report.json').exists():raise SystemExit('Refusing overwrite')
    out.mkdir(parents=True,exist_ok=True);stage=native/'horizon/crates/horizon-client/src';view=(HERE/'gui-closer-view.rs').read_text(encoding='utf8');assert view==(stage/'game/gui_closer_view.rs').read_text(encoding='utf8')
    remote=(stage/'game/remote.rs').read_text(encoding='utf8');hooks=(stage/'game/native_hooks.rs').read_text(encoding='utf8')
    assertions=[]
    for path,name,scope in [('game.rs','fn hud_layers(','with_gui_closer_view'),('game/window_handler.rs','fn handle_window_event(','with_gui_closer_view'),('game/pad_menu.rs','fn pad_menu_frame(','with_gui_closer_view'),('game/remote.rs','fn handle_packet(','with_gui_closer_active'),('game/remote.rs','fn remote_close_container(','with_gui_closer_active'),('game.rs','fn close_container(','with_gui_closer_active')]:
        assert scope in body((stage/path).read_text(encoding='utf8'),name);assertions.append(path+':'+name)
    for name in ['click','close','use','creative']:
        assert 'with_gui_closer_active' in body(hooks,'fn native_client_inventory_'+name+'(')
    assert 'with_gui_closer_active' in body(hooks,'fn native_client_inventory_snapshot(')
    for name in ['remote_menu_ops','remote_throw','remote_quick_move_all','remote_quick_move_unpredicted']:
        assert 'client_gui_closer_stale_input' in body(remote,'fn '+name+'(')
    opening=body(remote,'fn remote_open_screen(')
    assert opening.index('self.world.block_entities.insert(key, entity)')<opening.index('if !self.client_gui_closer_accept_screen')<opening.index('self.open_native_screen(')
    recipe=body((stage/'game/recipe_book.rs').read_text(encoding='utf8'),'fn remote_ghost_recipe(')
    assert recipe.index('p.container != self.remote_menu_id()')<recipe.index('with_gui_closer_view')
    fixture=(HERE/'gui-closer-view-fixture.rs').read_text(encoding='utf8').replace('/*VIEW*/',view).replace('/*REMOTE_MENU*/',body(remote,'pub(super) fn remote_menu('))
    (out/'lib.rs').write_text(fixture,encoding='utf8');(out/'view.rs').write_text(view,encoding='utf8')
    for code in ['en_us','zh_cn','zh_tw']:shutil.copyfile(ROOT/'build/resources/26.3/assets/minecraft/lang'/f'{code}.json',out/f'{code}.json')
    core=(ROOT/'upstream/Horizon/crates/horizon-client-core').as_posix();(out/'Cargo.toml').write_text('[package]\nname="gui-closer-view-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nhorizon-client-core={path="'+core+'"}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');commands.append({'argv':argv,'exitCode':r.returncode,'seconds':round(time.monotonic()-start,3)})
        (out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf8')
        if r.returncode:break
    report={'passed':all(c['exitCode']==0 for c in commands),'scope':'Exact production ContainerView/exchange/UI and active scopes plus transformed remote_menu accessor. Explicit recording Game/World/menu providers, real Horizon Text type. Source assertions connect actual HUD/input/pad/network/close and inventory Host consumers and active construction-before-cancellation. Tests distinguish old display menu, new packet state, shared player inventory, same reused remote entity key, double chest halves, repeated cancellation, id validation, active close, accepted same-enum replacement, abandoned constructor and nested Host reentrancy. Concrete complete Game compilation and game execution remain separate.','nativeSnapshot':a.native,'consumerAssertions':assertions,'commands':commands,'sha256':{str(f.relative_to(out)):hashlib.sha256(f.read_bytes()).hexdigest() for f in out.iterdir() if f.suffix in ['.rs','.toml']}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}))
if __name__=='__main__':main()
