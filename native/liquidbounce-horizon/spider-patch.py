#!/usr/bin/env python3
"""Complete Spider native velocity, global shape and live PlayerJumpEvent seam."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent

def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'spider-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    edit('crates/horizon-client/src/spider_bridge.rs',lambda _:(HERE/'spider-bridge.rs').read_text(encoding='utf8'))
    edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod high_jump_bridge;\n','pub mod high_jump_bridge;\npub mod spider_bridge;\n'))
    def entry(t):
        t=once(t,'    horizon_threads::configure_allocator();','    horizon_client::spider_bridge::install(dispatch_spider_jump,dispatch_spider_shape_active);\n    horizon_world::spider::install(horizon_client::spider_bridge::active);\n    horizon_threads::configure_allocator();')
        return t+'\nfn dispatch_spider_jump(horizontal_collision:bool,motion:&mut f32){crate::mod_api::dispatch_client_spider_jump(crate::mod_api::mods(),horizontal_collision,motion);}\nfn dispatch_spider_shape_active(active:&mut bool){crate::mod_api::dispatch_client_spider_shape_active(crate::mod_api::mods(),active);}\n'
    edit('crates/horizon-client/src/entry.rs',entry)
    edit('tools/native-hook-fragments/liquidbounce_spider.json',lambda _:json.dumps({'hooks':[dict(name=name,target='horizon_client_lib::spider_bridge::native_'+name,side='client',cancellable=False,mod_api={'contract':'listen'}) for name in ['client_spider_jump','client_spider_shape_active']],'helpers':[]},indent=2)+'\n')
    edit('crates/horizon-world/src/spider.rs',lambda _:(HERE/'spider-policy.rs').read_text(encoding='utf8'))
    edit('crates/horizon-world/src/spider_geometry.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/spider_geometry.rs').read_text(encoding='utf8'))
    edit('crates/horizon-world/src/lib.rs',lambda t:once(t,'pub mod block_walk;\n','pub mod block_walk;\npub mod spider;\n'))
    def scan(t):
        anchor='''            let shape = self.world.modify_collision_shape(block, p, shape)
                .map_err(CollisionScanError::World)?;
'''
        return once(t,anchor,anchor+'            let shape=crate::spider::shape(p.y,shape);\n')
    edit('crates/horizon-world/src/block_collision_scan.rs',scan)
    def motion(t):
        t=once(t,'            crate::high_jump_bridge::jump(&mut power);','            crate::high_jump_bridge::jump(&mut power);\n            crate::spider_bridge::jump(self.player.collision.horizontal,&mut power);')
        context='    fn context(&self) -> CollisionContext<\'static> {\n'
        t=once(t,context,context+'        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){horizon_world::spider::player(self.player.position.y,self.player.shift_key_down,self.player.ground.on_ground);}\n')
        t=once(t,'        self.player.position = p;\n','        self.player.position = p;\n        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){horizon_world::spider::player(p.y,self.player.shift_key_down,self.player.ground.on_ground);}\n')
        return t
    edit('crates/horizon-client/src/motion.rs',motion)
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,'        mod_api::dispatch_client_player_tick_post(self, mod_api::mods());','        mod_api::dispatch_client_player_tick_post(self, mod_api::mods());\n        self.sync_native_spider_player();'))
    return {'hook_names':[],'helper_names':['client_spider_context','client_spider_velocity','client_spider_player_sync']}
