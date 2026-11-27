#!/usr/bin/env python3
"""All NoSlow use/sneak/block/fluid/attribute and network event seams."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for target,source in [('crates/horizon-world/src/no_slow_policy.rs','no-slow-policy.rs'),('crates/horizon-client/src/no_slow_bridge.rs','no-slow-bridge.rs'),('crates/horizon-client/src/game/no_slow.rs','no-slow-native.rs')]:edit(target,lambda _,s=source:(HERE/s).read_text(encoding='utf8'))
 edit('crates/horizon-world/src/lib.rs',lambda s:once(s,'pub mod block_walk;\n','pub mod block_walk;\npub mod no_slow_policy;\n'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod high_jump_bridge;\n','pub mod high_jump_bridge;\npub mod no_slow_bridge;\n'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod reverse_step;\n','mod reverse_step;\nmod no_slow;\n'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'no-slow-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 def remote(s):
  s=once(s,'    pub(super) fn next_sequence(&mut self) -> i32 {','    pub(super) fn next_sequence(&mut self) -> i32 {')
  s=once(s,'    fn remote_send_state(&mut self, input: u8) {','''    fn remote_send_state(&mut self, input: u8) {
        self.native_client_no_slow_network(true);
        self.no_slow_vanilla_send_state(input);
        self.native_client_no_slow_network(false);
    }
    fn no_slow_vanilla_send_state(&mut self, input: u8) {''')
  s=once(s,'    pub(super) fn native_send_with_anti_hunger(&mut self, packet: S) {','    pub(super) fn native_send_with_anti_hunger(&mut self, packet: S) {\n        horizon_client::no_slow_bridge::set_context(self.no_slow_context());')
  anchor='    fn jump_from_ground(&self) {'
  return once(s,anchor,'''    fn no_slow_block_air(&self,pos:[i32;3])->Option<bool>{
        let world=self.world.borrow();let state=world.scene.block(pos[0],pos[1],pos[2]);
        Some(world.assets.registry.state(state).is_none_or(|v|matches!(v.name.as_str(),"minecraft:air"|"minecraft:cave_air"|"minecraft:void_air")))
    }
'''+anchor)
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/anti_hunger_bridge.rs',lambda s:once(s,'    fn jump_from_ground(&self);','    fn no_slow_block_air(&self,_pos:[i32;3])->Option<bool>{None}\n    fn jump_from_ground(&self);'))
 def input(s):
  anchor='''    let mut v = scale(input, 0.98);
    if let Some(multiplier) = item_use {
        v = scale(v, multiplier);
    }
    if let Some(multiplier) = sneaking {
        v = scale(v, multiplier);
    }
    square_movement(v)
}'''
  replacement='''    modify_input_axes(input,item_use.map(|v|[v,v]),sneaking)
}
/// LocalPlayer.modifyInput with independent sideways and forward factors.
pub fn modify_input_axes(input:[f32;2],item_use:Option<[f32;2]>,sneaking:Option<f32>)->[f32;2]{
    if input[0]*input[0]+input[1]*input[1]==0.0{return input;}
    let mut v=scale(input,0.98);
    if let Some([sideways,forward])=item_use{v=[v[0]*sideways,v[1]*forward];}
    if let Some(multiplier)=sneaking{v=scale(v,multiplier);}
    square_movement(v)
}'''
  return once(s,anchor,replacement)
 edit('crates/horizon-client-input/src/input.rs',input)
 def motion(s):
  gate='self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed()'
  # Generic world coefficients and flow queries must never escape a local owner.
  # Every public movement entry installs a fresh scope, including nested false
  # scopes for the LAN server, movement history and the prediction simulator.
  for anchor in [
   '    pub fn travel_in_air(&mut self, input: Vec3) -> Result<()> {\n',
   '    pub fn movement_tick(&mut self, input: [f32; 3]) -> Result<()> {\n',
   '    pub fn player_movement_tick(&mut self, direction: [f32; 2], jumping: bool) -> Result<()> {\n',
   '    pub fn simulation_tick(&mut self, direction: [f32; 2], jumping: bool) -> Result<()> {\n',
   '    pub fn update_soul_speed_block(&mut self) -> Result<()> {\n',
   '    pub fn apply_effects_from_blocks(&mut self) -> Result<()> {\n',
   '    pub fn passenger_block_effects_tick(&mut self) -> Result<()> {\n',
   '    pub fn jump_from_ground(&mut self) -> Result<()> {\n',
   '    pub fn shulker_box_push(&mut self, movement: Vec3) -> Result<()> {\n',
   '    pub fn move_self(&mut self, movement: Vec3) -> Result<()> {\n',
   '    pub fn update_auto_jump(&mut self, xa: f32, za: f32) -> Result<()> {\n',
  ]:
   s=once(s,anchor,anchor+f'        let _no_slow_owner=horizon_world::no_slow_policy::owner_scope({gate});\n')
  anchor='        // The base tick (fluids, swimming) precedes LocalPlayer.aiStep.\n'
  s=once(s,anchor,f'        let _no_slow_owner=horizon_world::no_slow_policy::owner_scope({gate});\n'+anchor)
  for anchor in [
   '    pub fn server_travel(&mut self) -> Result<()> {\n',
   '    pub fn server_stuck_in_blocks(&mut self, from: Vec3) -> Result<()> {\n',
   '    pub fn server_block_damage(&mut self, from: Vec3) -> Result<()> {\n',
  ]:
   s=once(s,anchor,anchor+'        let _no_slow_owner=horizon_world::no_slow_policy::owner_scope(false);\n')
  old='        let modified = crate::input::modify_input(direction, self.player.item_use_speed, sneak);'
  new='        let no_slow_owner=self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed();\n        let modified = crate::no_slow_bridge::modify_input(direction,self.player.item_use_speed,sneak,keys.shift,no_slow_owner);'
  s=once(s,old,new)
  s=once(s,'            slow_item: self.item_use_slows_sprint,','            slow_item: self.item_use_slows_sprint && !(self.client_high_jump_owner&&self.client_side&&self.native_client_queries_allowed()&&horizon_world::no_slow_policy::get()[0]==1.),')
  s=once(s,'            if slime && !damage_only && !self.player.shift_key_down {','            if slime && !damage_only && !self.player.shift_key_down && horizon_world::no_slow_policy::factor(3).is_none() {')
  anchor='    fn powder_snow_inside(&mut self, p: BlockPos) -> Result<()> {\n'
  s=once(s,anchor,anchor+'''        if let Some(factor)=horizon_world::no_slow_policy::factor(5){
            self.player.velocity.x*=f64::from(factor);self.player.velocity.z*=f64::from(factor);return Ok(());
        }
''')
  # The source overwrites Math.max's result, so entity bounciness is cancelled too.
  anchor='    fn block_bounce_restitution(&mut self, state: BlockStateId) -> Result<f32> {\n'
  s=once(s,anchor,'''    fn no_slow_slime(&mut self,state:BlockStateId)->Result<bool>{Ok(horizon_world::no_slow_policy::factor(3).is_some()&&self.assets.registry.state(state).is_some_and(|s|s.name=="minecraft:slime_block"))}
'''+anchor)
  return s
 edit('crates/horizon-client/src/motion.rs',motion)
 def restitution(s):
  anchor='    fn block_bounce_restitution(&mut self, block: BlockStateId) -> Result<f32, Self::Error>;'
  s=once(s,anchor,anchor+'\n    fn no_slow_slime(&mut self,_block:BlockStateId)->Result<bool,Self::Error>{Ok(false)}')
  return once(s,'                bounce = java_max(bounce, f64::from(block));','                bounce = if world.no_slow_slime(input.block)? {0.0}else{java_max(bounce, f64::from(block))};')
 edit('crates/horizon-world/src/movement_restitution.rs',restitution)
 def properties(s):
  s=once(s,'        Ok(BoundMovementProperties { states })','''        let no_slow_kinds=(0..self.registry.len()).map(|id|match self.registry.state(BlockStateId(id as u32)).unwrap().name.as_str(){"minecraft:soul_sand"=>2,"minecraft:slime_block"=>3,"minecraft:honey_block"=>4,_=>0}).collect();
        Ok(BoundMovementProperties { states,no_slow_kinds })''')
  s=once(s,'    states: Vec<MovementBlock>,','    states: Vec<MovementBlock>,\n    no_slow_kinds:Vec<usize>,')
  anchor='''        self.states
            .get(state.0 as usize)
            .copied()
            .ok_or_else(|| invalid("movement state outside registry"))'''
  repl='''        let mut block=self.states.get(state.0 as usize).copied().ok_or_else(||invalid("movement state outside registry"))?;
        let kind=self.no_slow_kinds[state.0 as usize];
        if kind!=0&&let Some(factor)=crate::no_slow_policy::factor(kind){block.coefficients.speed_factor=factor;if kind==3{block.coefficients.friction=0.6;}}
        Ok(block)'''
  return once(s,anchor,repl)
 edit('crates/horizon-world/src/movement_properties.rs',properties)
 def fluids(s):
  guard='self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed()'
  for anchor in ['    pub fn fluid_base_tick(&mut self) -> Result<()> {\n','    pub fn update_fluids(&mut self) -> Result<bool> {\n']:
   s=once(s,anchor,anchor+f'        let _no_slow_owner=horizon_world::no_slow_policy::owner_scope({guard});\n')
  anchor="impl FluidMembership for MotionWorld<'_, '_, '_> {\n    fn fluid_in_tag(&self, kind: FluidKind, tag: FluidTagId) -> bool {\n"
  s=once(s,anchor,anchor+f'        if {guard}&&!horizon_world::no_slow_policy::fluid_collision(){{return false;}}\n')
  # Query suppression for the player's fluid tag checks as well as EntityFluidInteraction.
  for anchor,ret in [('    pub fn fluid_height(&self, tag: FluidTagId) -> f64 {\n','0.0'),('    pub fn is_eye_in_fluid(&self, tag: FluidTagId) -> bool {\n','false'),('    pub fn is_in_lava(&self) -> bool {\n','false')]:
   gate='self.client_high_jump_owner&&self.client_side&&self.native_client_queries_allowed()'
   s=once(s,anchor,anchor+f'        let _no_slow_owner=horizon_world::no_slow_policy::owner_scope({gate});\n        if !horizon_world::no_slow_policy::fluid_collision(){{return {ret};}}\n')
  anchor='    fn add_delta_movement(&mut self, delta: Vec3) -> Result<()> {\n'
  return once(s,anchor,anchor+f'        if {guard}&&!horizon_world::no_slow_policy::fluid_push(){{return Ok(());}}\n')
 edit('crates/horizon-client/src/motion/fluids.rs',fluids)
 edit('crates/horizon-world/src/fluid_flow.rs',lambda s:once(s,') -> Result<Vec3, W::Error> {\n',') -> Result<Vec3, W::Error> {\n    if !crate::no_slow_policy::fluid_push(){return Ok(Vec3::ZERO);}\n'))
 return {'hook_names':['client_no_slow_network'],'helper_names':['client_no_slow_context','client_no_slow_send','client_no_slow_slowness']}
