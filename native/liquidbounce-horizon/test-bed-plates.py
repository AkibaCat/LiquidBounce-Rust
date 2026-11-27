#!/usr/bin/env python3
"""Read-only BedPlates transaction/native composition/syntax/Chinese validation."""
from pathlib import Path
import argparse,ast,json,runpy,shutil,subprocess,sys,tempfile
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def prepare_shared():
    installed='pub mod bed_plates;'in(ROOT/'native/liquidbounce-behaviors/src/lib.rs').read_text(encoding='utf8')
    if not installed:return runpy.run_path(str(HERE/'integrate-bed-plates.py'))['prepare']().changed
    paths=[ROOT/'native/liquidbounce-behaviors/src/lib.rs',*[HERE/f'engine/src/{n}'for n in ['lib.rs','behaviors.rs','config.rs','actor.rs','runtime.rs']],HERE/'make-behavior-patch.py']
    return {p:p.read_text(encoding='utf8')for p in paths}
def validate_choice_labels():
    core=ROOT/'native/liquidbounce-theme/src/integration/localization-core.ts';ui=core.with_name('localization.ts');generator=ROOT/'scripts/make-theme-localization-patch.py'
    if 'export function choiceLabel('in core.read_text(encoding='utf8'):changed={p:p.read_text(encoding='utf8')for p in[core,ui,generator]}
    else:changed={p.resolve():s for p,s in runpy.run_path(str(HERE/'integrate-bed-plates-choice-labels.py'))['prepare']().changed.items()}
    assert 'snapshot.labels[key]) || label(snapshot, name, spaced)'in changed[core]
    assert 'export const translateChoice = derived('in changed[ui]
    source=changed[generator];context={'__file__':str(generator),'__name__':'bed_plates_choice_static'};exec(compile(source,str(generator),'exec'),context)
    relative='src/routes/clickgui/setting/MultiChooseSetting.svelte';original=(ROOT/'upstream/LiquidBounce/src-theme'/relative).read_text(encoding='utf8');patched=context['patch_source'](relative,original,{})
    assert '$translateChoice(cSetting, choice, $spaceSeperatedNames)'in patched and 'import {translate, translateChoice}'in patched
    assert 'cSetting.value.includes(choice)'in patched and 'handleChange(choice)'in patched
    return changed
def compose_native():
    source=(HERE/'make-behavior-patch.py').read_text(encoding='utf8');source=source[:source.index('\noutput = []')]
    if "'bed-plates-patch.py'"not in source:
        anchor="'item-tags-patch.py',";assert source.count(anchor)==1;source=source.replace(anchor,anchor+" 'bed-plates-patch.py',")
    context={'__file__':str(HERE/'make-behavior-patch.py'),'__name__':'bed_plates_static'};exec(compile(source,context['__file__'],'exec'),context)
    assert context['names'].count('client_bed_plates')==1
    fragment=json.loads(context['changes']['tools/native-hook-fragments/liquidbounce_behavior.json'][1]);hooks=[h for h in fragment['hooks']if h['name']=='client_bed_plates']
    assert len(hooks)==1 and hooks[0]['target']=="horizon_client::game::native_hooks::Game<'_>::native_client_bed_plates"
    native={p:after for p,(_,after)in context['changes'].items()};wrapper='pub(super) fn native_client_bed_plates('
    assert [p for p,s in native.items()if wrapper in s]==['crates/horizon-client/src/game/native_hooks.rs']
    assert native['crates/horizon-client/src/game/native_hooks.rs'].count(wrapper)==1
    return native
def validate_native(native):
    hooks=native['crates/horizon-client/src/game/native_hooks.rs'];game=native['crates/horizon-client/src/game.rs'];owned=native['crates/horizon-client/src/game/bed_plates.rs']
    assert hooks.index('self.client_item_tags_tick();')<hooks.index('self.client_bed_plates_tick();')
    assert hooks.index('self.client_bed_plates_world_change();')<hooks.index('mod_api::dispatch_client_world_change(self, mod_api::mods(), dimension);')
    assert hooks.index('self.client_bed_plates_position(p.position);')<hooks.index('mod_api::dispatch_client_packet(self, mod_api::mods(), packet);')
    assert hooks.count('self.client_bed_plates_key(key,pressed);')==1
    assert hooks.count('self.client_bed_plates_key(key,event.state==winit::event::ElementState::Pressed);')==1
    assert game.count('client_bed_plates:std::cell::RefCell<bed_plates::State>')==1 and 'mod bed_plates;'in game
    assert game.index('self.client_item_tags_overlay(&mut batch);')<game.index('self.client_bed_plates_overlay(&mut batch);')<game.index('self.client_item_stack_lists_flush(&mut batch);')
    common=native['crates/horizon-client/src/game/item_stack_list.rs'];assert 'pub(super) type Renderer=Box<dyn Fn'in common
    assert 'renderer:Some(renderer)'in owned and 'self.draw_item_decorations('not in owned
    assert 'self.inventory.iter().take(9)'in owned and 'has_correct_tool_for_drops(s.as_ref(),true,name)'in owned
    assert 'components.get("minecraft:dyed_color")'in owned and 'self.armor[i]'in owned
    assert 'Weak::ptr_eq(previous,&weak)'in owned and 'section.blocks.maybe_has('in owned
    assert 'facts(default).flags&horizon_world::level::data::state_flags::CONDUCTOR'in owned
    assert 'block.ends_with("_stained_glass")'in owned and 'stained_glass_pane'not in owned
    assert 'let map_id=[8usize,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29][dye]'in owned
    assert 'x+17.-width,y+9.'in owned and 'u32::MAX,true);'in owned
    assert 'rendered+=1;'in owned and owned.index('clip.w<=0.')<owned.index('rendered+=1;')
    assert native['crates/horizon-client/src/game/bed_plates_scan.rs']==(ROOT/'native/liquidbounce-behaviors/src/bed_plates_scan.rs').read_text(encoding='utf8')
    runpy.run_path(str(HERE/'test-item-tags.py'))['validate_native'](native)
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--native',action='store_true');args=parser.parse_args()
    for name in['integrate-bed-plates.py','integrate-bed-plates-choice-labels.py','bed-plates-patch.py','update-bed-plates-locales.py','test-bed-plates-focused.py']:ast.parse((HERE/name).read_text(encoding='utf8'))
    choice_shared=validate_choice_labels()
    shared=prepare_shared();files={'shared-'+str(p.relative_to(ROOT)).replace('/','_').replace('\\','_'):s for p,s in shared.items()if p.suffix=='.rs'}
    owned=[ROOT/'native/liquidbounce-behaviors/src/bed_plates.rs',ROOT/'native/liquidbounce-behaviors/src/bed_plates_scan.rs',HERE/'engine/src/bed_plates_runtime.rs',HERE/'engine/src/config/bed_plates.rs',HERE/'bed-plates-game.rs']
    for p in owned:files['owned-'+str(p.relative_to(ROOT)).replace('/','_').replace('\\','_')]=p.read_text(encoding='utf8')
    if args.native:
        native=compose_native();validate_native(native)
        for p in['crates/horizon-client/src/game.rs','crates/horizon-client/src/game/native_hooks.rs']:files['native-'+p.replace('/','_')]=native[p]
    with tempfile.TemporaryDirectory(prefix='liquidbounce-bed-plates-static-')as folder:
        temporary=Path(folder);paths=[]
        for n,s in files.items():p=temporary/(n if n.endswith('.rs')else n+'.rs');p.write_text(s,encoding='utf8');paths.append(str(p))
        result=subprocess.run(['rustfmt','--edition','2024','--config','skip_children=true',*paths],capture_output=True,text=True)
        if result.returncode:raise RuntimeError(result.stderr)
        locale=temporary/'locales';locale.mkdir()
        for code in['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{code}.json',locale/f'display_{code}.json')
        if str(HERE)not in sys.path:sys.path.insert(0,str(HERE))
        chest={p.name:json.loads(p.read_text(encoding='utf8'))['labels']['Chest']for p in locale.iterdir()}
        update_context=runpy.run_path(str(HERE/'update-bed-plates-locales.py'));update=update_context['update'];update(locale);first={p.name:p.read_bytes()for p in locale.iterdir()};update(locale);assert first=={p.name:p.read_bytes()for p in locale.iterdir()}
        for name,raw in first.items():
            data=json.loads(raw);assert data['labels']['Chest']==chest[name];assert data['labels']['liquidbounce.module.bedPlates.ignoreSelfBed.color.slots.Chest']=='胸部'
        for raw in first.values():
            data=json.loads(raw);assert data['moduleNames']['BedPlates']
            for suffix,translations in update_context['TEXTS'].items():
                text=data['descriptions']['liquidbounce.module.bedPlates.'+suffix];assert text and text!=translations[0]
                for placeholder in['{x}','{y}','{z}']:assert(placeholder in text)==(placeholder in translations[0])
            for label in update_context['LABELS']:assert data['labels'].get(label),label
    print(f'BedPlates static checks passed: {len(shared)} engine and {len(choice_shared)} scoped-label shared anchors; {len(files)} complete Rust sources parsed temporarily; actual unique native hook and ItemTags common queue coexistence; {len(update_context["TEXTS"])} Chinese descriptions/messages per locale, scoped armor Chest and updater idempotence; no compilation or shared/staging writes')
if __name__=='__main__':main()
