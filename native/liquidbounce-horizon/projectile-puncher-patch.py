"""ProjectilePuncher native targeting, actual world rays and attack service."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/projectile_puncher_math.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/projectile_puncher.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda s:once(s,'pub mod no_web_model;','pub mod no_web_model;\npub mod projectile_puncher_math;'))
 edit('crates/horizon-client/src/game/projectile_puncher.rs',lambda _:(HERE/'projectile-puncher-native.rs').read_text(encoding='utf8'))
 def game(s):
  s=once(s,'mod offhand_native;','mod offhand_native;\nmod projectile_puncher;')
  s=once(s,'    client_chest_stealer_hide_swing: bool,','    client_chest_stealer_hide_swing: bool,\n    client_projectile_puncher_hide_implicit: bool,')
  return once(s,'            client_chest_stealer_hide_swing: false,','            client_chest_stealer_hide_swing: false,\n            client_projectile_puncher_hide_implicit: false,')
 edit('crates/horizon-client/src/game.rs',game)
 edit('crates/horizon-client/src/game/hand.rs',lambda s:once(s,'    pub(super) fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {\n','    pub(super) fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {\n        if self.client_projectile_puncher_hide_implicit{return;}\n'))
 edit('crates/horizon-client/src/game/remote.rs',lambda s:once(s,'    fn remote_spear_attack(&mut self, kinetic: horizon_client::spear::Kinetic) -> bool {',(HERE/'projectile-puncher-piercing.rs').read_text(encoding='utf8')+'\n    fn remote_spear_attack(&mut self, kinetic: horizon_client::spear::Kinetic) -> bool {'))
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'projectile-puncher-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 return {'hook_names':[],'helper_names':['client_projectile_puncher_plan','client_projectile_puncher_ready','client_projectile_puncher_attack']}
