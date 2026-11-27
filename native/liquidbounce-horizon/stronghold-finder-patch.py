"""StrongholdFinder consumed Item.use facts, queued spawn, resident sections and real HUD."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/stronghold_finder.rs',lambda _:(HERE/'stronghold-finder-game.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod protection_zones;','mod protection_zones;\nmod stronghold_finder;')
  s=once(s,'    client_protection_zones_cache: std::cell::RefCell<protection_zones::Cache>,','    client_protection_zones_cache: std::cell::RefCell<protection_zones::Cache>,\n    client_stronghold_cache:std::cell::RefCell<stronghold_finder::Cache>,')
  s=once(s,'            client_protection_zones_cache: Default::default(),','            client_protection_zones_cache: Default::default(),\n            client_stronghold_cache:Default::default(),')
  return once(s,'            self.client_easy_pearl_overlay(&mut batch);','            self.client_easy_pearl_overlay(&mut batch);\n            self.client_stronghold_overlay(&mut batch);')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(s):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  s=once(s,anchor,(HERE/'stronghold-finder-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  s=once(s,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_stronghold_tick();')
  s=once(s,'    pub(super) fn native_client_packet(&mut self, packet: &ClientboundPlayPacket) {','    pub(super) fn native_client_packet(&mut self, packet: &ClientboundPlayPacket) {\n        self.client_stronghold_packet(packet);')
  anchor='    pub(super) fn native_client_world_change(&self, dimension: Option<&str>) {\n        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() { return; }'
  return once(s,anchor,anchor+'\n        *self.client_stronghold_cache.borrow_mut()=Default::default();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda s:once(s,'        self.client_protection_zones_geometry(&mut primitives);','        self.client_protection_zones_geometry(&mut primitives);\n        self.client_stronghold_geometry(&mut primitives,alpha);'))
 def local(s):
  return once(s,'            if let Some(pass)=pass {self.native_client_item_interacted(pass);}', '            self.client_stronghold_used(&*stack.item=="minecraft:ender_eye",pass==Some(false));\n            if let Some(pass)=pass {self.native_client_item_interacted(pass);}')
 edit('crates/horizon-client/src/game/use_hand.rs',local)
 def remote(s):
  for signature in ['    pub(super) fn remote_use_item(&mut self, hand: Hand) -> bool {','    pub(super) fn remote_use_item_rotated(&mut self, hand: Hand, yaw: f32, pitch: f32) -> bool {']:
   start=s.index(signature);end=s.index('\n    }',start)+6;body=s[start:end]
   anchor='        if let Some(pass)=pass {self.native_client_item_interacted(pass);}'
   body=once(body,'        let pass=','        let stronghold_eye=self.with_hand(hand,|g|g.held().is_some_and(|s|&*s.item=="minecraft:ender_eye"));\n        let pass=')
   body=once(body,anchor,'        self.client_stronghold_used(stronghold_eye,pass==Some(false));\n'+anchor)
   s=s[:start]+body+s[end:]
  return s
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/end.rs',lambda s:once(s,'        let r = self.world.random.next_float();\n        let pitch = 0.33 + r * (0.5 - 0.33);','        self.client_stronghold_local_spawn(id,start.to_array());\n        let r = self.world.random.next_float();\n        let pitch = 0.33 + r * (0.5 - 0.33);'))
 return {'hook_names':['client_stronghold_finder'],'helper_names':[]}
