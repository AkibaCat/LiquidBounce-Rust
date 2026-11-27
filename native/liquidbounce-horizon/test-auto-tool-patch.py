#!/usr/bin/env python3
"""Compile true transformed mining seams, generated hooks, actor and config."""
import hashlib
import json
import os
from pathlib import Path
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
UPSTREAM=ROOT/'upstream/Horizon'
OUT=ROOT/'build/auto-tool-tests'

def main():
    block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block']
    changed={}
    def once(text,before,after):
        if text.count(before)!=1:raise ValueError(f'AutoTool expected one anchor ({text.count(before)}): {before[:110]!r}')
        return text.replace(before,after,1)
    def edit(path,transform):
        source=UPSTREAM/path
        text=changed.get(path,source.read_text(encoding='utf-8') if source.exists() else '')
        if path.endswith('native_hooks.rs') and '/// Integer Options.fov' not in text:
            text=text.replace('    /// Player FOV scale, after vanilla effects and before zoom and easing.\n','    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
        changed[path]=transform(text)
    for name in ['attack-patch.py','packet-patch.py','slot-patch.py','fast-place-patch.py','no-block-interact-patch.py','multi-actions-patch.py']:
        runpy.run_path(str(HERE/name))['apply'](edit,once)
    helper=runpy.run_path(str(HERE/'auto-tool-patch.py'));contract=helper['apply'](edit,once)
    OUT.mkdir(parents=True,exist_ok=True)
    first=changed['crates/horizon-client/src/first_person.rs'];declarations=[]
    for declaration in ['pub enum Hand','impl Hand','pub struct SwingAnimation','impl SwingAnimation','pub enum Hit','pub enum UseResult','impl UseResult','pub trait HandWorld','pub struct Destroying','impl Default for Destroying','impl Destroying','pub struct Clicks']:
        snippet=block(first,declaration)
        if declaration.startswith('pub enum') or declaration=='pub struct SwingAnimation':snippet='#[derive(Clone,Copy,Debug,PartialEq)]\n'+snippet
        elif declaration=='pub struct Destroying':snippet='#[derive(Clone,Debug,PartialEq)]\n'+snippet
        elif declaration=='pub struct Clicks':snippet='#[derive(Default)]\n'+snippet
        declarations.append(snippet)
    stations=(UPSTREAM/'crates/horizon-client/src/stations.rs').read_text(encoding='utf-8')
    native=(HERE/'auto-tool-native-fixture.rs').read_text(encoding='utf-8')
    for name,value in {'MAX_DAMAGE':block(stations,'pub fn max_damage('),'COMPONENT_INT':block(stations,'fn component_int('),'FIRST_PERSON':'\n'.join(declarations),
        'NATIVE':helper['HOOKS'],'REMOTE_CONTINUE':block(changed['crates/horizon-client/src/game/remote.rs'],'fn remote_continue_attack('),
        'COMPONENTS_VIEW':block((UPSTREAM/'crates/horizon-assets/src/item_model.rs').read_text(encoding='utf-8'),"impl<'a> ComponentsView<'a>"),
        **{key:block((UPSTREAM/'crates/horizon-client/src/item_render.rs').read_text(encoding='utf-8'),anchor) for key,anchor in [('TAG_JSON','pub fn tag_json('),('CUSTOM_JSON','fn custom_model_data_json('),('STACK_VIEW','pub fn stack_view(')]},
        'HAND_FORWARD':block(changed['crates/horizon-client/src/game/hand.rs'],'fn client_block_breaking_progress('),'RESOURCE_ROOT':(ROOT/'build/resources/26.3').as_posix()}.items():native=native.replace('__'+name+'__',value)
    native=native.replace('pub(super)','pub(crate)')
    hand=changed['crates/horizon-client/src/game/hand.rs']
    attack_strength=(UPSTREAM/'crates/horizon-client/src/attack_strength.rs').read_text(encoding='utf-8')
    strength='#[derive(Clone,Debug,Default,PartialEq)]'+block(attack_strength,'pub struct AttackStrength')+'\n#[derive(Clone,Debug,PartialEq,Eq)]'+block(attack_strength,'pub struct HeldStack')+'\n'+block(attack_strength,'pub fn delay(')+'\n'+block(attack_strength,'impl AttackStrength')
    render=(HERE/'auto-tool-render-fixture.rs').read_text(encoding='utf-8')
    for name,value in {'ATTACK_STRENGTH':strength,'SHOWS_HELD':block(hand,'fn shows_held_stack('),'MATCHES_SWAP':block(hand,'pub(super) fn matches_for_swap('),'TICK_HAND':block(hand,'pub(super) fn tick_hand(')}.items():render=render.replace('__'+name+'__',value)
    native+='\n'+render.replace('pub(super)','pub(crate)')
    # Cursor, menu, and network side effects are fixture interfaces. Every
    # native snapshot and invoker body above is the unchanged production body.
    (OUT/'native-fixture.rs').write_text(native,encoding='utf-8')
    config=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
    config_source='use serde_json::{Value,Map,Number,json};use std::collections::BTreeMap;\n#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{Interop,File}\nenum ModuleKind{AutoTool}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.autoTool"}}\n'
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(','fn value_of','fn parse_bool(','fn parse_float(','fn pair_members','fn parse_int_pair(','fn parse_int(','fn decimal_int(']:
        if anchor=='pub struct ConfigError':config_source+='#[derive(Debug)]\n'
        config_source+=block(config,anchor)+'\n'
    config_source+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;enum SettingChange{AutoTool(auto::AutoToolSetting)}#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}\n'
    config_source+='#[path="'+(HERE/'engine/src/config/auto_tool.rs').as_posix()+'"]mod auto;\n'+(HERE/'auto-tool-config-tests.rs').read_text(encoding='utf-8')
    (OUT/'config-fixture.rs').write_text(config_source,encoding='utf-8')
    source=(HERE/'auto-tool-fixture.rs').read_text(encoding='utf-8')
    for name,path in {'POLICY_PATH':ROOT/'native/liquidbounce-behaviors/src/auto_tool.rs','RENDER_PATH':ROOT/'native/liquidbounce-behaviors/src/silent_hotbar_render.rs','ACTOR_PATH':HERE/'engine/src/auto_tool.rs'}.items():source=source.replace('__'+name+'__',path.as_posix())
    source=source.replace('#[path="native-fixture.rs"]mod native;','#[path="native-fixture.rs"]mod native;\npub use native::{menu,stations,spear,item_render,StackView};\npub mod items{pub use horizon_items::items::*;}pub mod item_data{pub use horizon_items::item_data::*;}')
    bridge=(HERE/'packet-bridge.rs').read_text(encoding='utf-8')
    packets=(HERE/'engine/src/packets.rs').read_text(encoding='utf-8')
    contract='use horizon_protocol::packets::play::{Clientbound,Serverbound};\n'
    for anchor in ['pub enum TransferOrigin','pub enum Delivery','pub struct PacketContext','impl PacketContext']:
        if anchor.startswith('pub '):contract+='#[derive(Clone,Copy,Debug,PartialEq,Eq)]\n'
        contract+=block(bridge,anchor)+'\n'
    contract+=block(bridge,'pub struct PacketEvent')+'\n'+block(bridge,"impl<'a, P> PacketEvent")+'\n'+block(packets,'pub trait PacketListener')
    (OUT/'packet-contract.rs').write_text(contract,encoding='utf-8')
    source+='\npub mod mod_api{pub use horizon_protocol::packets::play::Serverbound as ServerboundPlayPacket;}\n#[path="packet-contract.rs"]pub mod packets;\n'
    source+=(HERE/'auto-tool-packet-fixture.rs').read_text(encoding='utf-8').replace('__PACKET_PATH__',(HERE/'engine/src/auto_tool_packets.rs').as_posix())
    (OUT/'lib.rs').write_text(source,encoding='utf-8')
    api=OUT/'api-source';native_api=api/'crates/horizon-client/src/game';native_api.mkdir(parents=True,exist_ok=True)
    (api/'tools/native-hook-fragments').mkdir(parents=True,exist_ok=True)
    (api/'crates/horizon-client/src/mod_api.rs').write_text('',encoding='utf-8')
    (native_api/'native_hooks.rs').write_text("impl Game<'_>{\n"+helper['HOOKS']+'\n}',encoding='utf-8')
    catalog={'hooks':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}",'mod_api':{'contract':'listen'}}for name in helper['HOOK_NAMES']],
        'helpers':[{'name':name,'target':f"horizon_client::game::native_hooks::Game<'_>::native_{name}"}for name in helper['HELPER_NAMES']]}
    (api/'native-hooks.json').write_text(json.dumps(catalog),encoding='utf-8')
    (OUT/'build.rs').write_text('fn main(){let root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");let out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());horizon_mod_api_gen::generate_test_hooks(&root,&out).unwrap();}',encoding='utf-8')
    manifest='[package]\nname="liquidbounce-auto-tool-tests"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n'
    for crate in ['horizon-items','horizon-world','horizon-protocol']:
        manifest+=f'{crate}={{path="{(UPSTREAM/"crates"/crate).as_posix()}"}}\n'
    manifest+='[build-dependencies]\nhorizon-mod-api-gen={path="'+(UPSTREAM/'crates/horizon-mod-api-gen').as_posix()+'"}\n'
    (OUT/'Cargo.toml').write_text(manifest,encoding='utf-8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-entity-interact-tests/target')
    result=subprocess.run(['cargo','+stable','test','--manifest-path',str(OUT/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True)
    (OUT/'test.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    print(result.stdout+result.stderr,end='')
    evidence={'exitCode':result.returncode,'sourceSha256':hashlib.sha256(source.encode()).hexdigest(),'fragment':contract,'transformedPaths':list(changed),
        'composedExtensions':['attack','slots','FastPlace','NoBlockInteract','MultiActions','AutoTool'],
        'scope':'actual_generator;actual_native_snapshot;actual_Horizon_Items_BlockFacts_BlockRegistry;actual_transformed_Destroying_continue_and_remote_continue;actual_tick_hand_and_AttackStrength;actual_config_and_actor;actual_accepted_packet_collector_and_protocol_types;actual_Java_InventoryMenu_slots;menu_network_cursor_effects_stubbed;full_client_differential_not_run'}
    (OUT/'result.json').write_text(json.dumps(evidence,indent=2)+'\n',encoding='utf-8')
    raise SystemExit(result.returncode)

if __name__=='__main__':main()
