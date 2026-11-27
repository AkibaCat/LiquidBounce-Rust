#!/usr/bin/env python3
"""Independent FightBot Navigator; apply after AutoWalk and the KillAura goal adapter."""
from pathlib import Path
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    edit('crates/horizon-client/src/game/navigation_owners.rs',lambda _:(HERE/'navigation-owners.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_walk_runtime;','mod auto_walk_runtime;\nmod navigation_owners;'))
    def runtime(s):
        start=s.index('thread_local! {');end=s.index('fn key_input(',start)
        s=s[:start]+(HERE/'fightbot-navigation-state.rs').read_text(encoding='utf8')+'\n'+s[end:]
        s=once(s,'        let result = world::with_scope(self, || {\n','        let _owner = OwnerScope::enter(Owner::AutoWalk);\n        let result = world::with_scope(self, || {\n')
        return s+'\n'+(HERE/'fightbot-navigation-runtime.rs').read_text(encoding='utf8')
    edit('crates/horizon-client/src/game/auto_walk_runtime.rs',runtime)
    edit('crates/horizon-client/src/game/auto_walk_world.rs',lambda s:once(s,
        'pub(super) fn current_epoch(game:&Game<\'_>)->u64{',
        'pub(super) fn scope_is_active()->bool{ACTIVE_GAME.with(|slot|slot.borrow().is_some())}\npub(super) fn current_epoch(game:&Game<\'_>)->u64{'))
    def host(s):
        s=once(s,'            let old = g.selected;\n            g.selected = slot as usize;','            let old = g.selected;\n            super::auto_walk_runtime::claim_slot(slot as usize,old);\n            g.selected = slot as usize;')
        s=once(s,'''        if let Some(slot) = self.own_slot.take() {
            world::write(|g| {
                g.selected = slot;
                g.remote_send_carried();
            });
        }
''','''        if self.own_slot.take().is_some() {
            if let Some(slot)=super::auto_walk_runtime::release_slot() {
                world::write(|g| {g.selected=slot;g.remote_send_carried();});
            }
        }
''')
        s=once(s,'        if super::auto_walk_runtime::keeps_destroying() {','        if super::auto_walk_runtime::releasing_destroying() {')
        s=once(s,'        horizon_client::navigation_physics::set_safe_walk(false);','        super::auto_walk_runtime::navigation_safe_walk(false);')
        s=once(s,'        horizon_client::navigation_physics::set_safe_walk(safe);','        super::auto_walk_runtime::navigation_safe_walk(safe);')
        s=once(s,'''    fn stop_destroy(&mut self) {
        super::auto_walk_runtime::keep_destroying(false);
        world::write(|g| {
            g.stop_remote_mining();
        });
    }
''','''    fn stop_destroy(&mut self) {
        let stop=super::auto_walk_runtime::releasing_destroying();
        super::auto_walk_runtime::keep_destroying(false);
        if stop {world::write(|g|g.stop_remote_mining());}
    }
''')
        return s
    edit('crates/horizon-client/src/game/auto_walk_host.rs',host)
    def hooks(s):
        s=once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n','''    pub(super) fn native_client_kill_aura_navigation_config(&mut self,value:&str){self.kill_aura_navigation_config(value);}
    pub(super) fn native_client_kill_aura_navigation_result(&self)->Option<String>{self.kill_aura_navigation_result()}
    /// Player FOV scale, after vanilla effects and before zoom and easing.
''')
        s=once(s,'        if let Some((keys,_)) = navigation { [*forward,*backward,*left,*right,*jump,*shift,*sprint] = keys; }\n','''        if let Some((keys,_)) = navigation { [*forward,*backward,*left,*right,*jump,*shift,*sprint] = keys; }
        let fight_navigation = self.native_client_kill_aura_navigation_input([*forward,*backward,*left,*right,*jump,*shift,*sprint]);
        if let Some((keys,_)) = fight_navigation { [*forward,*backward,*left,*right,*jump,*shift,*sprint] = keys; }
''')
        s=once(s,'        if let Some((keys,true)) = navigation { [*forward,*backward,*left,*right,*jump,*shift] = [keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]]; }\n','''        if let Some((keys,true)) = navigation { [*forward,*backward,*left,*right,*jump,*shift] = [keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]]; }
        if let Some((_,Some(keys))) = fight_navigation { [*forward,*backward,*left,*right,*jump,*shift] = [keys[0],keys[1],keys[2],keys[3],keys[4],keys[5]]; }
''')
        return s
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    return {'hook_names':[],'helper_names':['client_kill_aura_navigation_config','client_kill_aura_navigation_result']}
