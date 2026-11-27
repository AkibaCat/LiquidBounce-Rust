from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'no-clip-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def motion(t):
  t=once(t,'    pub spectator: bool,\n','    pub spectator: bool,\n    pub client_no_physics: bool,\n')
  t=once(t,'    fn no_physics(&self) -> bool {\n        self.player.spectator\n','    fn no_physics(&self) -> bool {\n        self.player.spectator || self.player.client_no_physics\n')
  return once(t,'        // isAffectedByBlocks: no physics for spectators.\n        if self.player.spectator {\n','        // Entity.isAffectedByBlocks also excludes noPhysics.\n        if self.player.spectator || self.player.client_no_physics {\n')
 edit('crates/horizon-client/src/motion.rs',motion)
 edit('crates/horizon-client/src/motion/body.rs',lambda t:once(t,'            spectator: false,\n','            spectator: false,\n            client_no_physics: false,\n'))
 def mob(t):
  t=once(t,'    pub velocity: Vec3,\n    /// Entity.yRot','    pub velocity: Vec3,\n    pub client_no_physics: bool,\n    /// Entity.yRot')
  return once(t,'            previous: position,\n            velocity: Vec3::ZERO,\n            yaw,','            previous: position,\n            velocity: Vec3::ZERO,\n            client_no_physics: false,\n            yaw,')
 edit('crates/horizon-client/src/mobs.rs',mob)
 def living(t):
  t=once(t,'    fn no_physics(&self) -> bool {\n        false\n','    fn no_physics(&self) -> bool {\n        self.client_no_physics\n')
  return once(t,'    pub fn do_move(&mut self, w: &MobWorld, events: &mut Vec<MobEvent>) {\n','    pub fn do_move(&mut self, w: &MobWorld, events: &mut Vec<MobEvent>) {\n        if self.client_no_physics {self.position=self.position.add_vector(self.velocity);return;}\n')
 edit('crates/horizon-client/src/mobs/living.rs',living)
 def boats(t):
  t=once(t,'    pub velocity: Vec3,\n','    pub velocity: Vec3,\n    pub client_no_physics: bool,\n')
  t=once(t,'            velocity: Vec3::ZERO,\n','            velocity: Vec3::ZERO,\n            client_no_physics: false,\n')
  # Entity.move noPhysics skips collision and block speed multipliers while the
  # rest of AbstractBoat.tick (float/control/paddles) remains active.
  t=once(t,'        let movement = b.take_movement();\n','        let movement = if b.client_no_physics {b.velocity}else{b.take_movement()};\n')
  t=once(t,'        if let Ok(moved) = scene.resolve(b.bounds(), movement, 0.0, b.on_ground) {\n','        if b.client_no_physics {b.position=b.position.add_vector(movement);}\n        else if let Ok(moved) = scene.resolve(b.bounds(), movement, 0.0, b.on_ground) {\n')
  t=once(t,'        if !self.boat_powder_snow_cells(&b).is_empty() {','        if !b.client_no_physics && !self.boat_powder_snow_cells(&b).is_empty() {')
  return once(t,'        b.velocity = horizon_world::entity_move::scale_by_block_speed(b.velocity, factor);\n','        if !b.client_no_physics {b.velocity = horizon_world::entity_move::scale_by_block_speed(b.velocity, factor);}\n')
 edit('crates/horizon-client/src/game/boats.rs',boats)
 return {'hook_names':[],'helper_names':['client_no_clip_state','client_no_clip_flags','client_no_clip_velocity']}
