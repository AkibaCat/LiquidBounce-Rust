#!/usr/bin/env python3
"""Real Parkour and SafeWalk input phases, sharing AutoWalk's scoped adapter."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    for name,source in [('edge_assistance',HERE/'edge-assistance.rs'),('edge_geometry',HERE/'edge-geometry.rs'),('edge_random',HERE.parent/'liquidbounce-behaviors/src/random.rs')]:
        content=source.read_text(encoding='utf8')
        edit('crates/horizon-client/src/game/'+name+'.rs',lambda _,v=content:v)
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_walk_runtime;\n','mod auto_walk_runtime;\nmod edge_assistance;\nmod edge_geometry;\nmod edge_random;\n'))
    edit('crates/horizon-client/src/navigation_physics.rs',lambda s:once(once(s,
        ' static SAFE_WALK:Cell<bool>=const{Cell::new(false)};',' static SAFE_WALK:Cell<bool>=const{Cell::new(false)};\n static MODULE_SAFE_WALK:Cell<bool>=const{Cell::new(false)};'),
        'pub fn set_safe_walk(value:bool){SAFE_WALK.set(value);}','pub fn set_safe_walk(value:bool){SAFE_WALK.set(value);}\npub fn set_module_safe_walk(value:bool){MODULE_SAFE_WALK.set(value);}').replace('||SAFE_WALK.get())','||SAFE_WALK.get()||MODULE_SAFE_WALK.get())'))
    def hooks(s):
        s=once(s,'    pub(super) fn native_client_auto_walk_config(','''    pub(super) fn native_client_edge_assistance_config(&mut self,value:&str){self.edge_assistance_config(value);}
    pub(super) fn native_client_edge_assistance_result(&self)->Option<String>{self.edge_assistance_result()}
    pub(super) fn native_client_auto_walk_config(''')
        target='        mod_api::dispatch_client_directional_input(self, mod_api::mods(), forward, backward, left, right, jump, shift, sprint);\n'
        s=once(s,target,'        [*forward,*backward,*left,*right,*jump,*shift,*sprint]=self.edge_assistance_input([*forward,*backward,*left,*right,*jump,*shift,*sprint],false);\n'+target)
        target='        if let Some((keys,true)) = navigation { [*forward,*backward,*left,*right,*jump,*shift] = [keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]]; }\n'
        s=once(s,target,target+'        [*forward,*backward,*left,*right,*jump,*shift,*sprint]=self.edge_assistance_input([*forward,*backward,*left,*right,*jump,*shift,*sprint],true);\n')
        s=once(s,'        super::auto_walk_runtime::level_changed();\n','        super::auto_walk_runtime::level_changed();\n        super::edge_assistance::clear();\n')
        return s
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    return {'hook_names':[],'helper_names':['client_edge_assistance_config','client_edge_assistance_result']}
