#!/usr/bin/env python3
"""NoWeb WebBlock.entityInside and real PlaceWater world/bucket seams."""
from pathlib import Path
import json
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 for target,source in [
  ('crates/horizon-client/src/no_web_bridge.rs',HERE/'no-web-bridge.rs'),
  ('crates/horizon-client/src/no_web_model.rs',HERE.parent/'liquidbounce-behaviors/src/no_web.rs'),
  ('crates/horizon-client/src/game/no_web.rs',HERE/'no-web-native.rs'),
  ('crates/horizon-client/src/game/no_web_wire.rs',HERE/'engine/src/no_web_wire.rs'),
  ('crates/horizon-client/src/game/no_web_ray.rs',HERE/'no-web-ray.rs')]:
  edit(target,lambda _,p=source:p.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod high_jump_bridge;\n','pub mod high_jump_bridge;\npub mod no_web_bridge;\npub mod no_web_model;\n'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod reverse_step;\n','mod reverse_step;\nmod no_web;\nmod no_web_ray;\n'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'no-web-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 def entry(s):
  s=once(s,'    horizon_threads::configure_allocator();','    horizon_client::no_web_bridge::install(dispatch_no_web_collision);\n    horizon_threads::configure_allocator();')
  return s+'\nfn dispatch_no_web_collision(pos:[i32;3],moving:bool,ground:bool,tick:i32,result:&mut String){crate::mod_api::dispatch_client_no_web_collision(crate::mod_api::mods(),pos,moving,ground,tick,result);}\n'
 edit('crates/horizon-client/src/entry.rs',entry)
 edit('tools/native-hook-fragments/liquidbounce_no_web.json',lambda _:json.dumps({'hooks':[dict(name='client_no_web_collision',target='horizon_client_lib::no_web_bridge::native_client_no_web_collision',side='client',cancellable=False,mod_api={'contract':'listen'})],'helpers':[]},indent=2)+'\n')
 def motion(s):
  s=once(s,'"minecraft:cobweb" => self.web_inside(),','"minecraft:cobweb" => self.no_web_inside(p)?,')
  return once(s,'    fn web_inside(&mut self) {',(HERE/'no-web-motion.rs').read_text(encoding='utf8')+'\n    fn web_inside(&mut self) {')
 edit('crates/horizon-client/src/motion.rs',motion)
 edit('crates/horizon-world/src/level/mod.rs',lambda s:once(s,"impl Behaviours {\n    pub(crate) fn sim<'a, L: LevelAccess + ?Sized>(",'''impl Behaviours {
    /// NoWeb's original UP/CENTER support test, including live block entities.
    pub fn no_web_center_support<L:LevelAccess+?Sized>(&self,level:&mut L,ticks:&mut Ticks,pos:BlockPos)->bool{self.sim(level,ticks).can_support_center(pos,horizon_core::geometry::Direction::Up)}
    pub(crate) fn sim<'a, L: LevelAccess + ?Sized>('''))
 return {'hook_names':[],'helper_names':['client_no_web_plan','client_no_web_execute']}
