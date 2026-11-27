"""AntiVoid actual local replay, sendPosition event, and global empty-shape policy."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    for target,source in [
        ('crates/horizon-client/src/game/anti_void_native.rs','anti-void-native.rs'),
        ('crates/horizon-client/src/anti_void_bridge.rs','anti-void-bridge.rs'),
        ('crates/horizon-world/src/anti_void.rs','anti-void-policy.rs')]:
        edit(target,lambda _,source=source:(HERE/source).read_text(encoding='utf8'))
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod anti_bot_native;','mod anti_bot_native;\nmod anti_void_native;'))
    edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod spider_bridge;','pub mod spider_bridge;\npub mod anti_void_bridge;'))
    edit('crates/horizon-world/src/lib.rs',lambda s:once(s,'pub mod spider;','pub mod spider;\npub mod anti_void;'))
    def hooks(s):
        a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
        s=once(s,a,(HERE/'anti-void-host.rs').read_text(encoding='utf8')+'\n'+a)
        a='        mod_api::dispatch_client_player_tick_post(self, mod_api::mods());'
        s=once(s,a,a+'\n        horizon_world::anti_void::player_dead(self.player.as_ref().is_none_or(|p|p.health.dead_or_dying()));')
        a='        horizon_world::liquid_walk::clear();'
        return once(s,a,a+'\n        horizon_world::anti_void::player_dead(true);')
    edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
    def entry(s):
        a='    horizon_threads::configure_allocator();'
        s=once(s,a,'    horizon_client::anti_void_bridge::install(dispatch_anti_void_shape);\n    horizon_world::anti_void::install(horizon_client::anti_void_bridge::ghost_floor);\n'+a)
        return s+'\nfn dispatch_anti_void_shape(active:&mut bool,floor:&mut f64){crate::mod_api::dispatch_client_anti_void_shape(crate::mod_api::mods(),active,floor);}\n'
    edit('crates/horizon-client/src/entry.rs',entry)
    edit('tools/native-hook-fragments/liquidbounce_anti_void.json',lambda _:json.dumps({'hooks':[{'name':'client_anti_void_shape','target':'horizon_client_lib::anti_void_bridge::native_client_anti_void_shape','side':'client','cancellable':False,'mod_api':{'contract':'listen'}}],'helpers':[]},indent=2)+'\n')
    edit('crates/horizon-client/src/anti_hunger_bridge.rs',lambda s:once(s,'    fn jump_from_ground(&self);','    fn anti_void_dead(&self)->Option<bool>{None}\n    fn jump_from_ground(&self);'))
    edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,'    fn jump_from_ground(&self) {','    fn anti_void_dead(&self)->Option<bool>{Some(self.world.borrow().player.health.dead_or_dying())}\n    fn jump_from_ground(&self) {'))
    def motion(s):
        a="    fn context(&self) -> CollisionContext<'static> {\n"
        return once(s,a,a+'        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){horizon_world::anti_void::player_dead(self.player.health.dead_or_dying());}\n')
    edit('crates/horizon-client/src/motion.rs',motion)
    def scan(s):
        a='            let shape=crate::spider::shape(p.y,shape);'
        return once(s,a,a+'\n            let shape=crate::anti_void::shape(p.y,shape);')
    edit('crates/horizon-world/src/block_collision_scan.rs',scan)
    for target,prefix in [('crates/horizon-client-render/src/collisions.rs','horizon_world'),('crates/horizon-world/src/registry_collision_scan.rs','crate')]:
        def active(s,prefix=prefix):
            a=f'{prefix}::block_walk::active()'
            if not s.count(a):raise ValueError('AntiVoid active shape seam missing')
            return s.replace(a,f'({a} || {prefix}::anti_void::active())')
        edit(target,active)
    return {'hook_names':['client_anti_void_network'],'helper_names':['client_anti_void_query','client_anti_void_clear','client_anti_void_raise']}
