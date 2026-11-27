#!/usr/bin/env python3
"""Standalone SmartEat policy and actual shared Gson/config adapter tests.

No staging or upstream writes. Native/full-client differential is not run.
"""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
OUT=ROOT/'build/smart-eat-tests'

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block']
    shared=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
    config='use serde_json::{Value,Map,Number,json};use std::collections::BTreeMap;\n#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{Interop,File}\nenum ModuleKind{SmartEat}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.smartEat"}}\n'
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(','fn value_of','fn parse_bool(','fn parse_float(','fn parse_int(','fn decimal_int(','\nfn leaf(']:
        if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
        if anchor=="impl Tree<'_>":config+='#[allow(dead_code)]\n'
        config+=block(shared,anchor)+'\n'
    config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;#[derive(Debug,PartialEq)]enum SettingChange{SmartEat(smart::SmartEatSetting)}#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}\n'
    config+='#[path="'+(HERE/'engine/src/config/smart_eat.rs').as_posix()+'"]mod smart;\n'+(HERE/'smart-eat-config-tests.rs').read_text(encoding='utf-8')
    (OUT/'config-fixture.rs').write_text(config,encoding='utf-8')
    source=(HERE/'smart-eat-fixture.rs').read_text(encoding='utf-8').replace('__SMART_PATH__',(ROOT/'native/liquidbounce-behaviors/src/smart_eat.rs').as_posix())
    source=source.replace('#[path="config-fixture.rs"]mod config;','extern crate self as horizon_client;\n#[cfg(test)]\npub use native::{items,item_data,item_render,first_person,brewing,StackView};\n#[cfg(test)]\n#[path="config-fixture.rs"]mod config;\n#[path="'+(HERE/'engine/src/smart_eat.rs').as_posix()+'"]pub mod actor;\n#[cfg(test)]\n#[path="native-fixture.rs"]mod native;\n#[cfg(test)]\n#[allow(clippy::field_reassign_with_default)]\nmod tests {use super::*;')+'\n}\n'
    native=(HERE/'smart-eat-native-fixture.rs').read_text(encoding='utf-8')
    native+='\n#[path="use-native-fixture.rs"]pub mod native_use;\n'
    source+='\n#[cfg(test)]pub use native::native_use::{mobs,projectiles};\n#[cfg(test)]pub mod blocks {pub use horizon_items::blocks::*;}\n'
    use_native=(HERE/'smart-eat-use-native-fixture.rs').read_text(encoding='utf-8')
    for key,file in [('ITEM_RAY','smart-eat-item-ray.rs'),('USE_RESULT','smart-eat-use-result-host.rs'),('PICKABLE','smart-eat-pickable-host.rs')]:
        use_native=use_native.replace('__'+key+'__',(HERE/file).read_text(encoding='utf-8').replace('    fn client_','    pub(super) fn client_'))
    use_native=use_native.replace('__RESOURCE_ROOT__',(ROOT/'build/resources/26.3').as_posix())
    (OUT/'use-native-fixture.rs').write_text(use_native,encoding='utf-8')
    use_rules=(ROOT/'native/liquidbounce-behaviors/src/smart_eat_use_result.rs').read_text(encoding='utf-8')
    use_rules+='\npub fn owner(item:&str)->Option<UseOwner>{match item {\n'+(HERE/'smart-eat-use-owners.rs').read_text(encoding='utf-8')+'\n}}\n'
    (OUT/'use-result.rs').write_text(use_rules,encoding='utf-8')
    (OUT/'use-tests.rs').write_text((HERE/'smart-eat-use-tests.rs').read_text(encoding='utf-8'),encoding='utf-8')
    source+='\n#[path="use-result.rs"]pub mod client_smart_eat_use;\n#[cfg(test)]#[path="use-tests.rs"]mod use_tests;\n'
    upstream=ROOT/'upstream/Horizon'
    helper=runpy.run_path(str(HERE/'smart-eat-patch.py'))
    changed={}
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'SmartEat expected one anchor ({text.count(before)}): {before[:110]!r}')
        return text.replace(before,after,1)
    def edit(path,transform):
        source=upstream/path
        text=changed.get(path,source.read_text(encoding='utf-8') if source.exists() else '')
        if path.endswith('native_hooks.rs') and '/// Integer Options.fov' not in text:
            text=text.replace('    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',
                '    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        changed[path]=transform(text)
    for prerequisite in ['attack','mining','fast-place','no-block-interact','packet','slot','fishing','multi-actions']:
        runpy.run_path(str(HERE/(prerequisite+'-patch.py')))['apply'](edit,once)
    contract=helper['apply'](edit,once)
    hand=changed['crates/horizon-client/src/game/hand.rs']
    actions=changed['crates/horizon-client/src/game/item_actions.rs']
    assert hand.count('if self.client_use_key_down() && self.use_delay == 0 && !was_using')==1
    assert actions.index('self.client_cancel_block_interact(hit)')<actions.index('game.use_hand(block_hit.as_ref())')
    contract['composition']='attack;mining;fast_place;no_block_interact;packets;slots;fishing;multi_actions;smart_eat'
    return_fixture=(HERE/'smart-eat-return-fixture.rs').read_text(encoding='utf-8')
    return_blocks={
        'LOCAL_WRAPPER':block(changed['crates/horizon-client/src/game/use_hand.rs'],'    pub(super) fn use_hand(')+'\n'+block(changed['crates/horizon-client/src/game/use_hand.rs'],'    pub(super) fn client_use_item_only('),
        'REMOTE_WRAPPER':block(changed['crates/horizon-client/src/game/remote.rs'],'    fn remote_use_item('),
        'ROTATED_WRAPPER':block(changed['crates/horizon-client/src/game/remote.rs'],'    pub(super) fn remote_use_item_rotated('),
        'ROTATION':block(changed['crates/horizon-client/src/game/native_hooks.rs'],'    pub(super) fn native_with_item_rotation<'),
    }
    for key,value in return_blocks.items():return_fixture=return_fixture.replace('__'+key+'__',value)
    (OUT/'return-fixture.rs').write_text(return_fixture,encoding='utf-8')
    source+='\n#[cfg(test)]#[path="return-fixture.rs"]mod return_tests;\n'
    source+='\n#[cfg(test)]pub mod packet_bridge {fn pitch_limit_removed()->bool {false}\n'+block((HERE/'packet-bridge.rs').read_text(encoding='utf-8'),'pub fn set_x_rot(')+'\n}\n'
    for name,value in {'NATIVE':helper['HOOKS'],'RESOURCE_ROOT':(ROOT/'build/resources/26.3').as_posix(),
        'COMPONENTS_VIEW':block((upstream/'crates/horizon-assets/src/item_model.rs').read_text(encoding='utf-8'),"impl<'a> ComponentsView<'a>"),
        **{key:block((upstream/'crates/horizon-client/src/item_render.rs').read_text(encoding='utf-8'),anchor) for key,anchor in [('TAG_JSON','pub fn tag_json('),('CUSTOM_JSON','fn custom_model_data_json('),('STACK_VIEW','pub fn stack_view(')]}}.items():native=native.replace('__'+name+'__',value)
    (OUT/'native-fixture.rs').write_text(native.replace('pub(super)','pub(crate)'),encoding='utf-8')
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    api=OUT/'api-source';native_api=api/'crates/horizon-client/src/game';native_api.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('pub use crate::hud::HudBatch;\n',encoding='utf-8')
    (native_api/'native_hooks.rs').write_text("impl Game<'_>{\n"+helper['HOOKS']+'\n}',encoding='utf-8')
    catalog={'hooks':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}",'mod_api':{'contract':'listen'}}for name in helper['HOOK_NAMES']],
        'helpers':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"}for name in helper['HELPER_NAMES']]}
    (api/'native-hooks.json').write_text(json.dumps(catalog),encoding='utf-8')
    (OUT/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate_test_hooks(&root,&out).unwrap();}',encoding='utf-8')
    manifest='[package]\nname="liquidbounce-smart-eat-tests"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n'
    for crate in ['horizon-items','horizon-world','horizon-client-core','horizon-core','horizon-protocol']:manifest+=f'{crate}={{path="{(upstream/"crates"/crate).as_posix()}"}}\n'
    manifest+='[build-dependencies]\nhorizon-mod-api-gen={path="'+(upstream/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (OUT/'Cargo.toml').write_text(manifest,encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-entity-interact-tests/target')
    result=subprocess.run(['cargo','+stable','test','--manifest-path',str(OUT/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True)
    (OUT/'test.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    print(result.stdout+result.stderr,end='')
    (OUT/'result.json').write_text(json.dumps({'exitCode':result.returncode,'policySha256':hashlib.sha256((ROOT/'native/liquidbounce-behaviors/src/smart_eat.rs').read_bytes()).hexdigest(),'transformedPaths':list(changed),'contract':contract,'scope':'source_derived_policy;actual_shared_config_adapter;actual_native_snapshot_key_draw_functions;actual_preuse_dispatch_components_ray_entity_classifier;actual_transformed_Item_use_RETURN_wrappers;actual_generator;draw_and_action_effects_stubbed;fixture_world_shapes;full_client_differential_not_run'},indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode)

if __name__=='__main__':main()
