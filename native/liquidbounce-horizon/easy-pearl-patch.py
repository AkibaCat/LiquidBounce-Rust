"""EasyPearl at actual useItem HEAD, real pick and world/HUD frame boundaries."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/easy_pearl.rs',lambda _:(HERE/'easy-pearl-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_pearl;','mod auto_pearl;\nmod easy_pearl;'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'            self.client_crosshair_overlay(&mut batch);','            self.client_crosshair_overlay(&mut batch);\n            self.client_easy_pearl_overlay(&mut batch);'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'easy-pearl-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'''            self.native_with_item_rotation(yaw, pitch, |g| {
                g.with_hand(hand, |g| g.client_use_item_only(true).unwrap_or(false) && !g.use_failed)
            })''','''            self.client_easy_pearl_local_use(hand,yaw,pitch)'''))
 edit('crates/horizon-client/src/game/use_hand.rs',lambda s:once(s,'        if !self.client_smart_eat_player() {return self.client_use_hand_committed(None);}','        if !self.client_smart_eat_player() {return self.client_use_hand_committed(None);}\n        if self.client_easy_pearl_interact(){self.native_client_item_interacted(true);return Ok(false);}'))
 def remote(s):
  for signature in ['    pub(super) fn remote_use_item(&mut self, hand: Hand) -> bool {\n','    pub(super) fn remote_use_item_rotated(&mut self, hand: Hand, yaw: f32, pitch: f32) -> bool {\n']:
   s=once(s,signature,signature+'        if self.client_easy_pearl_interact(){self.native_client_item_interacted(true);return false;}\n')
  return s
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda s:once(s,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_easy_pearl_geometry(&mut primitives,alpha);'))
 return {'hook_names':['client_easy_pearl'],'helper_names':['client_easy_pearl_query']}
