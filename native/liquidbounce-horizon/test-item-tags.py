#!/usr/bin/env python3
"""Read-only ItemTags integration, native composition, Rust syntax and Chinese checks."""
from pathlib import Path
import argparse,ast,json,runpy,shutil,subprocess,sys,tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]


def prepare_shared():
    installed='pub mod item_tags;'in(ROOT/'native/liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
    if not installed:return runpy.run_path(str(HERE/'integrate-item-tags.py'))['prepare']().changed
    paths=[ROOT/'native/liquidbounce-behaviors/src/lib.rs',*[HERE/f'engine/src/{n}'for n in ['lib.rs','behaviors.rs','config.rs','actor.rs','runtime.rs']],HERE/'make-behavior-patch.py']
    return {p:p.read_text(encoding='utf8')for p in paths}


def compose_native():
    # Stop before output publication. The actual complete generator runs in memory.
    source=(HERE/'make-behavior-patch.py').read_text(encoding='utf8')
    source=source[:source.index('\noutput = []')]
    if "'item-tags-patch.py'"not in source:
        anchor="'chest-stealer-patch.py',";assert source.count(anchor)==1
        source=source.replace(anchor,anchor+" 'item-tags-patch.py',")
    context={'__file__':str(HERE/'make-behavior-patch.py'),'__name__':'item_tags_static'}
    exec(compile(source,context['__file__'],'exec'),context)
    assert context['names'].count('client_item_tags')==1
    fragment=json.loads(context['changes']['tools/native-hook-fragments/liquidbounce_behavior.json'][1])
    hooks=[h for h in fragment['hooks']if h['name']=='client_item_tags']
    assert len(hooks)==1 and hooks[0]['target']=="horizon_client::game::native_hooks::Game<'_>::native_client_item_tags"
    native={p:after for p,(_,after)in context['changes'].items()}
    wrapper='pub(super) fn native_client_item_tags('
    definitions=[p for p,s in native.items()if wrapper in s]
    assert definitions==['crates/horizon-client/src/game/native_hooks.rs'],definitions
    assert native[definitions[0]].count(wrapper)==1
    return native


def validate_native(native):
    hooks=native['crates/horizon-client/src/game/native_hooks.rs']
    assert 'input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_item_tags(self,mod_api::mods(),input,output);}}'in hooks
    assert hooks.index('self.client_radar_tick();')<hooks.index('self.client_item_tags_tick();')
    assert hooks.index('*self.client_item_tags.borrow_mut()=Default::default();')<hooks.index('mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);')
    game=native['crates/horizon-client/src/game.rs']
    assert game.index('self.client_item_tags_alpha.set(alpha);')<game.index('let mut hud = horizon_client::spans::time("hud", || self.hud(size));')
    assert 'mod item_tags;'in game and game.count('client_item_tags:std::cell::RefCell<item_tags::State>')==1
    assert 'mod item_stack_list;'in game and game.count('client_item_stack_lists_flush(&mut batch);')==1
    assert game.index('self.client_chest_stealer_tag(&mut batch);')<game.index('self.client_item_stack_lists_flush(&mut batch);')
    assert game.index('self.client_item_stack_lists.borrow_mut().0.clear();')<game.index('self.client_item_tags_alpha.set(alpha);')
    assert '4=>Float32x2,5=>Uint32'in native['crates/horizon-client-render/src/hud.rs']
    assert '|v| v.text == 1'in native['crates/horizon-client-render/src/hud/pack_text.rs']
    shader=native['crates/horizon-client-render/src/hud.wgsl']
    assert '@location(5) shape:u32'in shader and '@location(4) @interpolate(flat) shape:u32'in shader
    assert shader.count('fn liquidbounce_rounded(')==1 and 'in.shape&0x80000000u'in shader
    owned=native['crates/horizon-client/src/game/item_tags.rs']
    assert owned.index('"action":"running"')<owned.index('self.dropped.iter().collect')
    assert owned.index('drop(native);')<owned.index('for list in lists{')
    common=native['crates/horizon-client/src/game/item_stack_list.rs']
    assert 'gradient.mul_add(-(v as f32),v as f32)'in common
    assert 'horizon_assets::item_model::DisplayContext::Gui'in common
    assert 'self.draw_item_decorations(batch,stack,x,y,None);'in common
    assert 'self.client_item_stack_list(batch,list,settings["preventOverlap"]==true);'in owned
    assert 'pub(super) type Renderer=Box<dyn Fn'in common
    assert 'std::mem::take(&mut self.client_item_stack_lists.borrow_mut().0)'in common
    chest=native['crates/horizon-client/src/game/chest_stealer.rs']
    assert 'self.client_item_stack_list(batch,super::item_stack_list::List{'in chest
    assert '},false);'in chest and 'fn chest_stealer_panel('not in chest
    assert native['crates/horizon-client/src/game/item_tags_layout.rs']==(ROOT/'native/liquidbounce-behaviors/src/item_tags_layout.rs').read_text(encoding='utf8')


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--native',action='store_true');args=parser.parse_args()
    for name in ['integrate-item-tags.py','item-tags-patch.py','update-item-tags-locales.py']:
        ast.parse((HERE/name).read_text(encoding='utf8'))
    shared=prepare_shared();files={'shared-'+str(p.relative_to(ROOT)).replace('/','_').replace('\\','_'):s for p,s in shared.items()if p.suffix=='.rs'}
    owned=[ROOT/'native/liquidbounce-behaviors/src/item_tags.rs',ROOT/'native/liquidbounce-behaviors/src/item_tags_layout.rs',HERE/'engine/src/item_tags_runtime.rs',HERE/'engine/src/config/item_tags.rs',HERE/'item-tags-game.rs',HERE/'item-stack-list-game.rs',HERE/'item-tags-hud.rs']
    for p in owned:files['owned-'+str(p.relative_to(ROOT)).replace('/','_').replace('\\','_')]=p.read_text(encoding='utf8')
    if args.native:
        native=compose_native();validate_native(native)
        for p in ['crates/horizon-client/src/game.rs','crates/horizon-client/src/game/native_hooks.rs','crates/horizon-client/src/game/chest_stealer.rs','crates/horizon-client-render/src/hud.rs','crates/horizon-client-render/src/hud/pack_text.rs']:
            files['native-'+p.replace('/','_')]=native[p]
    with tempfile.TemporaryDirectory(prefix='liquidbounce-item-tags-static-')as folder:
        temporary=Path(folder);paths=[]
        for n,s in files.items():p=temporary/(n if n.endswith('.rs')else n+'.rs');p.write_text(s,encoding='utf8');paths.append(str(p))
        r=subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',*paths],capture_output=True,text=True)
        if r.returncode:raise RuntimeError(r.stderr)
        locale=temporary/'locales';locale.mkdir()
        for code in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{code}.json',locale/f'display_{code}.json')
        if str(HERE)not in sys.path:sys.path.insert(0,str(HERE))
        update_context=runpy.run_path(str(HERE/'update-item-tags-locales.py'));update=update_context['update'];update(locale)
        first={p.name:p.read_bytes()for p in locale.iterdir()};update(locale)
        assert first=={p.name:p.read_bytes()for p in locale.iterdir()}
        for raw in first.values():
            data=json.loads(raw);assert data['moduleNames']['ItemTags']
            for suffix in update_context['TEXTS']:assert data['descriptions']['liquidbounce.module.itemTags.'+suffix]
            for label in ['Filter','Items','Background','Rect','Texture','Color','OutlineColor','Margin','Radius','Gradient','Glow','Scale','RenderOffset','RowLength','PreventOverlap','ClusterEntities','MergeMode','None','ByItem','ByComponents','Shulker','MergeStacks','ShowTitle']:
                assert data['labels'].get(label),label
    print(f'ItemTags static checks passed: {len(shared)} shared files; {len(files)} complete Rust sources parsed temporarily; native hook target/definition and HUD composition; complete Chinese descriptions/idempotent updater; no compilation or shared/staging writes')


if __name__=='__main__':main()
