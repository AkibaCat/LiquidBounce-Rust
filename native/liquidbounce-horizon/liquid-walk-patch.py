#!/usr/bin/env python3
"""LiquidWalk shape, live packet player, jump cancellation and real fluid input."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for target,source in [
  ('crates/horizon-client/src/liquid_walk_native.rs',HERE/'liquid-walk-native.rs'),
  ('crates/horizon-world/src/liquid_walk.rs',HERE/'liquid-walk-policy.rs'),
  ('crates/horizon-world/src/liquid_walk_model.rs',HERE.parent/'liquidbounce-behaviors/src/liquid_walk.rs')]:edit(target,lambda _,p=source:p.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod high_jump_bridge;\n','pub mod high_jump_bridge;\npub mod liquid_walk_native;\n'))
 edit('crates/horizon-world/src/lib.rs',lambda s:once(s,'pub mod block_walk;\n','pub mod block_walk;\npub mod liquid_walk;\npub mod liquid_walk_model;\n'))
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,(HERE/'liquid-walk-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  anchor='        horizon_client::reverse_step_bridge::set_keys([*forward,*backward,*left,*right,*jump,*shift,*sprint]);\n'
  s=once(s,anchor,anchor+'        self.liquid_walk_input(*shift,jump);\n')
  return once(s,'        super::reverse_step::clear();\n','        super::reverse_step::clear();\n        horizon_world::liquid_walk::clear();\n')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def motion(s):
  anchor='    fn context(&self) -> CollisionContext<\'static> {\n'
  s=once(s,anchor,anchor+'        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){crate::liquid_walk_native::shape_player(self.scene,self.player,self.collision_binding);}\n')
  anchor='        self.player.position = p;\n'
  s=once(s,anchor,anchor+'        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){crate::liquid_walk_native::shape_player(self.scene,self.player,self.collision_binding);}\n')
  anchor='        entity_move::move_entity(self, MoverType::SelfMovement, movement)?;'
  s=once(s,anchor,anchor+'\n        if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){crate::liquid_walk_native::context(self.scene,self.player,self.collision_binding);}')
  anchor='            crate::spider_bridge::jump(self.player.collision.horizontal,&mut power);'
  return once(s,anchor,anchor+'\n            if crate::liquid_walk_native::cancel_jump(self.scene,self.player,self.collision_binding){return Ok(());}')
 edit('crates/horizon-client/src/motion.rs',motion)
 def bridge(s):
  anchor='    fn jump_from_ground(&self);'
  return once(s,anchor,'    /// LiquidWalk reads the same actual borrowed body at packet dispatch.\n    fn liquid_walk_context(&self)->Option<([bool;4],bool)>{None}\n'+anchor)
 edit('crates/horizon-client/src/anti_hunger_bridge.rs',bridge)
 def remote(s):
  anchor='    fn jump_from_ground(&self) {'
  return once(s,anchor,'    fn liquid_walk_context(&self)->Option<([bool;4],bool)>{let world=self.world.borrow();Some((horizon_client::liquid_walk_native::context(world.scene,world.player,world.collision_binding),world.player.fluids.was_touching_water))}\n'+anchor)
 edit('crates/horizon-client/src/game/remote.rs',remote)
 # Both resident SceneCollisions implementations and registry-only scans
 # participate in BlockCollisions, while direct getCollisionShape stays raw.
 for target,prefix in [('crates/horizon-client-render/src/collisions.rs','horizon_world'),('crates/horizon-world/src/registry_collision_scan.rs','crate')]:
  def shapes(s,prefix=prefix):
   old=f'{prefix}::block_walk::active()'
   if old not in s:raise ValueError('LiquidWalk active shape seam missing')
   s=s.replace(old,f'({old} || {prefix}::liquid_walk::active())')
   old=f'Ok({prefix}::block_walk::shape(&state.name, shape))'
   if old not in s:raise ValueError('LiquidWalk shape return seam missing')
   return s.replace(old,f'Ok({prefix}::liquid_walk::shape(matches!(state.name.as_str(),"minecraft:water"|"minecraft:lava"),{prefix}::block_walk::shape(&state.name, shape)))')
  edit(target,shapes)
 return {'hook_names':[],'helper_names':['client_liquid_walk_context']}
