"""AirPlace at actual useItem HEAD and WorldRender FULL_BOX."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/air_place_target.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/air_place_target.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/air_place.rs',lambda _:(HERE/'air-place-native.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/lib.rs',lambda s:once(s,'pub mod auto_dodge;','pub mod auto_dodge;\npub mod air_place_target;'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod auto_dodge;','mod auto_dodge;\nmod air_place;'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n','''    pub(super) fn native_client_air_place_configuration(&self,configuration:&mut String){
        if self.native_window(){mod_api::dispatch_client_air_place_configuration(self,mod_api::mods(),configuration);}
    }
    pub(super) fn client_air_place_item_class(item:&str,class:&str)->bool{client_weapon_item_facts::item_is(item,class)}
    /// Player FOV scale, after vanilla effects and before zoom and easing.
'''))
 edit('crates/horizon-client/src/game/use_hand.rs',lambda s:once(s,'        if !self.client_smart_eat_player() {return self.client_use_hand_committed(None);}',
    '        if !self.client_smart_eat_player() {return self.client_use_hand_committed(None);}\n        if self.client_air_place_item(self.hand){self.native_client_item_interacted(true);return Ok(false);}'))
 def remote(s):
  s=once(s,'    pub(super) fn remote_use_item(&mut self, hand: Hand) -> bool {\n','    pub(super) fn remote_use_item(&mut self, hand: Hand) -> bool {\n        if self.client_air_place_item(hand){self.native_client_item_interacted(true);return false;}\n')
  return once(s,'    pub(super) fn remote_use_item_rotated(&mut self, hand: Hand, yaw: f32, pitch: f32) -> bool {\n','    pub(super) fn remote_use_item_rotated(&mut self, hand: Hand, yaw: f32, pitch: f32) -> bool {\n        if self.client_air_place_item(hand){self.native_client_item_interacted(true);return false;}\n')
 edit('crates/horizon-client/src/game/remote.rs',remote)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda s:once(s,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_air_place_geometry(&mut primitives);'))
 return {'hook_names':['client_air_place_configuration'],'helper_names':[]}
