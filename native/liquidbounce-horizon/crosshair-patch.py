"""Pinned Crosshair: original HUD overlay layer and default-crosshair cancellation."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/crosshair.rs',lambda _:(HERE/'crosshair-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/crosshair_raster.rs',lambda _:(HERE/'crosshair-raster.rs').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod wings;','mod wings;\nmod crosshair;')
  t=once(t,'            if self.effective_camera_type().first_person()\n                && (!spectator || self.spectator_crosshair())','            if !self.client_crosshair_hide() && self.effective_camera_type().first_person()\n                && (!spectator || self.spectator_crosshair())')
  return once(t,'            self.client_damage_particles_overlay(&mut batch);','            self.client_damage_particles_overlay(&mut batch);\n            self.client_crosshair_overlay(&mut batch);')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  t=once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'crosshair-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
  t=once(t,'fn client_crosshair_entity(&self) -> Option<i32>','pub(super) fn client_crosshair_entity(&self) -> Option<i32>')
  t=once(t,'    pub(super) fn native_client_tick_post(&mut self) {','    pub(super) fn native_client_tick_post(&mut self) {\n        self.client_crosshair_tick();')
  return once(t,'        self.client_wings_reset();','        self.client_crosshair_reset();\n        self.client_wings_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 return {'hook_names':['client_crosshair'],'helper_names':[]}
