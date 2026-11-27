#!/usr/bin/env python3
"""Scaffold actual-world feature queries and synchronous jump phase hooks.

Compose after high-jump/anti-hunger/no-slow and before Step. This fragment is
published with the complete Scaffold implementation, never independently.
"""
from pathlib import Path
import json
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def apply(edit, once):
    for target, source in [
        ('crates/horizon-client/src/scaffold_motion_bridge.rs', HERE/'scaffold-motion-bridge.rs'),
        ('crates/horizon-client/src/scaffold_math.rs', HERE/'scaffold-math.rs'),
        ('crates/horizon-client/src/game/scaffold_feature_rules.rs', HERE.parent/'liquidbounce-behaviors/src/scaffold_features.rs'),
        ('crates/horizon-client/src/game/scaffold_features.rs', HERE/'scaffold-features-native.rs'),
    ]:
        edit(target, lambda _, source=source: source.read_text(encoding='utf8'))
    edit('crates/horizon-client/src/client_lib.rs', lambda text: once(text,
        'pub mod high_jump_bridge;\n', 'pub mod high_jump_bridge;\npub mod scaffold_motion_bridge;\npub mod scaffold_math;\n'))
    edit('crates/horizon-client/src/game.rs', lambda text: once(text,
        'mod block_items;\n', 'mod block_items;\nmod scaffold_features;\nmod scaffold_feature_rules;\n'))
    def entry(text):
        text=once(text,'    horizon_threads::configure_allocator();',
            '    horizon_client::scaffold_motion_bridge::install(dispatch_scaffold_jump, dispatch_scaffold_after_jump);\n    horizon_threads::configure_allocator();')
        return text+'''
fn dispatch_scaffold_jump(y:f64,motion:f32){crate::mod_api::dispatch_client_scaffold_jump(crate::mod_api::mods(),y,motion);}
fn dispatch_scaffold_after_jump(yaw:f32,forward:bool,backward:bool,left:bool,right:bool,x:&mut f64,y:&mut f64,z:&mut f64){crate::mod_api::dispatch_client_scaffold_after_jump(crate::mod_api::mods(),yaw,forward,backward,left,right,x,y,z);}
'''
    edit('crates/horizon-client/src/entry.rs',entry)
    edit('tools/native-hook-fragments/liquidbounce_scaffold_motion.json',lambda _:json.dumps({'hooks':[
        dict(name='client_scaffold_jump',target='horizon_client_lib::scaffold_motion_bridge::native_client_scaffold_jump',side='client',cancellable=False,mod_api={'contract':'listen'}),
        dict(name='client_scaffold_after_jump',target='horizon_client_lib::scaffold_motion_bridge::native_client_scaffold_after_jump',side='client',cancellable=False,mod_api={'contract':'listen'}),
    ],'helpers':[]},indent=2)+'\n')
    def motion(text):
        text=once(text,'            living_motion::jump_from_ground_with_power(self,power)?;',
            '            crate::scaffold_motion_bridge::jump(self.player.position.y,power);\n            living_motion::jump_from_ground_with_power(self,power)?;')
        return once(text,'        if !self.player.replaying {\n            native_living_jump(self.player.entity_id as u32, "minecraft:player");',
            '        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){crate::scaffold_motion_bridge::after_jump(self.player.yaw,&mut self.player.velocity);}\n        if !self.player.replaying {\n            native_living_jump(self.player.entity_id as u32, "minecraft:player");')
    edit('crates/horizon-client/src/motion.rs',motion)
    def safe_walk(text):
        text=once(text,'fn safe_walk(&mut self, config: &Value, mut keys: Input) -> Input {',
            'fn safe_walk(&mut self, config: &Value, mut keys: Input, scaffold:bool) -> Input {')
        text=once(text,'        horizon_client::navigation_physics::set_module_safe_walk(mode == "Safe");',
            '        if scaffold{horizon_client::navigation_physics::set_scaffold_safe_walk(mode=="Safe");}else{horizon_client::navigation_physics::set_module_safe_walk(mode == "Safe");}')
        # Eagle runs earlier in the composition and rewrites the normal call
        # through its local `input` binding. Accept both source shapes so the
        # generated normal SafeWalk path receives the new `scaffold` flag.
        if text.count('state.safe_walk(&config, input(raw))')==1:
            text=text.replace('state.safe_walk(&config, input(raw))','state.safe_walk(&config, input(raw),false)',1)
        elif text.count('state.safe_walk(&config, input)')==1:
            text=text.replace('state.safe_walk(&config, input)','state.safe_walk(&config, input,false)',1)
        text=once(text,'    STATE.with(|s| *s.borrow_mut() = None);',
            '    STATE.with(|s| *s.borrow_mut() = None);\n    SCAFFOLD_STATE.with(|s| *s.borrow_mut() = None);\n    horizon_client::navigation_physics::set_scaffold_safe_walk(false);\n    horizon_client::navigation_physics::set_scaffold_down(false);')
        return text+'\n'+(HERE/'scaffold-safe-walk-native.rs').read_text(encoding='utf8')
    edit('crates/horizon-client/src/game/edge_assistance.rs',safe_walk)
    def physics(text):
        text=once(text,' static MODULE_SAFE_WALK:Cell<bool>=const{Cell::new(false)};',
            ' static MODULE_SAFE_WALK:Cell<bool>=const{Cell::new(false)};\n static SCAFFOLD_SAFE_WALK:Cell<bool>=const{Cell::new(false)};\n static SCAFFOLD_DOWN:Cell<bool>=const{Cell::new(false)};')
        text=once(text,'pub fn set_module_safe_walk(value:bool){MODULE_SAFE_WALK.set(value);}',
            'pub fn set_module_safe_walk(value:bool){MODULE_SAFE_WALK.set(value);}\npub fn set_scaffold_safe_walk(value:bool){SCAFFOLD_SAFE_WALK.set(value);}\npub fn set_scaffold_down(value:bool){SCAFFOLD_DOWN.set(value);}\npub fn scaffold_down()->bool{SCAFFOLD_DOWN.get()}')
        return once(text,'||SAFE_WALK.get()||MODULE_SAFE_WALK.get())','||SAFE_WALK.get()||MODULE_SAFE_WALK.get()||SCAFFOLD_SAFE_WALK.get())&&!SCAFFOLD_DOWN.get()')
    edit('crates/horizon-client/src/navigation_physics.rs',physics)
    def final_down(text):
        anchor='        [*forward,*backward,*left,*right,*jump,*shift,*sprint]=self.edge_assistance_input([*forward,*backward,*left,*right,*jump,*shift,*sprint],true);\n'
        return once(text,anchor,anchor+'        if crate::navigation_physics::scaffold_down(){*shift=false;}\n')
    edit('crates/horizon-client/src/game/native_hooks.rs',final_down)
    host=(HERE/'scaffold-features-host.rs').read_text(encoding='utf8')
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda text:once(text,anchor,host+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_scaffold_features']}
