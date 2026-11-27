"""Actual decoded original-packet observation, including chunk decode/forget fast paths."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/new_chunks.rs',lambda _:(HERE/'new-chunks-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda t:once(t,'mod native_visuals;','mod native_visuals;\nmod new_chunks;'))
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n';t=once(t,anchor,(HERE/'new-chunks-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  return once(t,'    pub(super) fn native_client_packet(&mut self, packet: &ClientboundPlayPacket) {','    pub(super) fn native_client_packet(&mut self, packet: &ClientboundPlayPacket) {\n        self.client_new_chunks_packet(packet);')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_new_chunks_geometry(&mut primitives);'))
 return {'hook_names':['client_new_chunks'],'helper_names':[]}
