"""SwordBlock renderer expressions, after the complete KillAura pose seam."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/sword_block.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/sword_block.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/lib.rs',lambda s:once(s,'pub mod auto_dodge;','pub mod auto_dodge;\npub mod sword_block;'))
 def game(s):
  return once(s,'                        offhand: g.offhand.is_some(),','                        offhand: g.offhand.is_some() && !(g.client_sword_block_flags() & 4 != 0 && g.client_sword_block_hide_offhand()),')
 edit('crates/horizon-client/src/game.rs',game)
 a='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,a,(HERE/'sword-block-host.rs').read_text(encoding='utf8')+'\n'+a))
 def frame(s):
  s=once(s,'        let kill_aura_block_pose = self.client_kill_aura_block_pose();','        let kill_aura_block_pose = self.client_sword_block_pose();')
  return once(s,'            if !shown[i] {','            if hand == Hand::Off && self.client_sword_block_hide_stack(item.as_ref()) { continue; }\n            if !shown[i] {')
 edit('crates/horizon-client/src/game/frame_geometry.rs',frame)
 def perspective(s):
  s=once(s,'        let client_main_hand_block_pose = self.client_kill_aura_block_pose();','        let client_main_hand_block_pose = self.client_sword_block_flags() & 8 != 0 && self.client_sword_block_pose();')
  return once(s,'        let off = self.offhand.as_ref().map(|s| s.item.clone());','        let off = if self.client_sword_block_flags() & 8 != 0 && self.client_sword_block_hide_offhand() { None } else { self.offhand.as_ref().map(|s| s.item.clone()) };')
 edit('crates/horizon-client/src/game/perspective.rs',perspective)
 return {'hook_names':['client_sword_block_state'],'helper_names':[]}
