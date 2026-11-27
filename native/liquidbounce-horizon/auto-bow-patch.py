from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for suffix in ('native','ray','forecast'):
  name='auto_bow' if suffix=='native' else f'auto_bow_{suffix}'
  edit(f'crates/horizon-client/src/game/{name}.rs',lambda _,suffix=suffix:(HERE/f'auto-bow-{suffix}.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/auto_bow_math.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/auto_bow_math.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_walk_runtime;','mod auto_walk_runtime;\nmod auto_bow;\nmod auto_bow_math;\nmod auto_bow_ray;\nmod auto_bow_forecast;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'auto-bow-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 edit('crates/horizon-client/src/game/scoreboard.rs',lambda s:once(s,'    fn scoreboard_name(&self, id: u32, uuid: [i32; 4])','    pub(super) fn scoreboard_name(&self, id: u32, uuid: [i32; 4])'))
 def remote(s):
  s=once(s,'    fn remote_tick_using(&mut self) {','    pub(super) fn remote_tick_using(&mut self) {')
  # The bow chronometer is a module-level observer, so it runs while AutoBow is disabled too.
  anchor='    pub(super) fn release_remote_use(&mut self) {\n'
  s=once(s,anchor,anchor+'        if self.native_client_slot_owner()&&let Some((slot,_))=self.using&&let Some(stack)=self.stack_in(slot){super::auto_bow::stopped(stack);}\n')
  anchor='            } else if r.use_item != item {\n'
  s=once(s,anchor,anchor+'                if let Some(old)=r.use_item.as_deref(){super::auto_bow::stopped_id(old);}\n')
  anchor='        let in_hand = slot == self.client_hotbar_slot() || slot == super::shield::OFFHAND_SLOT;\n'
  s=once(s,anchor,anchor+'        if !in_hand&&let Some(stack)=stack.as_ref(){super::auto_bow::stopped(stack);}\n')
  s=once(s,'        if self.mod_use_tick(slot, ticks, !self.client_use_key_down() || !in_hand) {\n','        if self.mod_use_tick(slot, ticks, (!super::auto_bow::updating()&&!self.client_use_key_down()) || !in_hand) {\n')
  return s
 edit('crates/horizon-client/src/game/remote.rs',remote)
 anchor='    pub(super) fn stop_using_item(&mut self, stack: &ItemStack) {\n'
 def using(s):
  s=once(s,anchor,anchor+'        if self.native_client_slot_owner(){super::auto_bow::stopped(stack);}\n')
  return once(s,'        if !self.client_use_key_down() || hand_changed || self.screen != Screen::Playing {\n','        if hand_changed || (!super::auto_bow::updating()&&(!self.client_use_key_down()||self.screen!=Screen::Playing)) {\n')
 edit('crates/horizon-client/src/game/using_item.rs',using)
 # Native armor stands did not previously retain Entity.tickCount. Both actual server and remote ticks do.
 def armor(s):
  s=once(s,'pub struct ArmorStand {\n','pub struct ArmorStand {\n    pub liquidbounce_tick_count:i32,\n')
  s=once(s,'    pub fn new(id: u32, position: Vec3, yaw: f32) -> Self {\n        Self {','    pub fn new(id: u32, position: Vec3, yaw: f32) -> Self {\n        Self {\n            liquidbounce_tick_count:0,')
  a='    pub fn interpolate(&mut self, step: Option<(Vec3, f32)>) {\n'
  return once(s,a,a+'        self.liquidbounce_tick_count=self.liquidbounce_tick_count.wrapping_add(1);\n')
 edit('crates/horizon-client/src/entities.rs',armor)
 anchor='            // Entity.baseTick clears firstTick only when the entity actually ticks.\n'
 edit('crates/horizon-client/src/game/armor_stands.rs',lambda s:once(s,anchor,'            a.liquidbounce_tick_count=a.liquidbounce_tick_count.wrapping_add(1);\n'+anchor))
 edit('crates/horizon-client/src/game/dragon.rs',lambda s:once(s,'    ambient_time: i32,\n    ticks: i32,\n','    ambient_time: i32,\n    pub(super) ticks: i32,\n'))
 return {'hook_names':[],'helper_names':['client_auto_bow_query','client_auto_bow_update_use']}
