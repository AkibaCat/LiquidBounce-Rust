"""Actual resident-world HoleManager/Tracker, native player post-tick and plain ESP."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/hole_esp.rs',lambda _:(HERE/'hole-esp-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/hole_tracker.rs',lambda _:(HERE.parent/'liquidbounce-behaviors/src/hole_tracker.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/hole_esp.rs',lambda _:runpy.run_path(str(HERE/'hole-esp-render.py'))['source']())
 def game(t):
  t=once(t,'mod native_visuals;','mod native_visuals;\nmod hole_esp;\nmod hole_tracker;')
  t=once(t,'    kept_block_entities: std::cell::RefCell<native_block_entity_draw::KeptGeometry>,','    client_hole_esp_cache: std::cell::RefCell<hole_esp::Cache>,\n    kept_block_entities: std::cell::RefCell<native_block_entity_draw::KeptGeometry>,')
  t=once(t,'            kept_block_entities: Default::default(),','            client_hole_esp_cache: Default::default(),\n            kept_block_entities: Default::default(),')
  return once(t,'        horizon_client::gfx::wings::begin_frame("null");','        horizon_client::gfx::wings::begin_frame("null");\n        horizon_client::gfx::hole_esp::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,anchor,(HERE/'hole-esp-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  t=once(t,'    pub(super) fn native_client_player_tick_post(&mut self) {','    pub(super) fn native_client_player_tick_post(&mut self) {\n        self.client_hole_esp_tick();')
  return once(t,'        self.client_wings_reset();','        self.client_wings_reset();\n        self.client_hole_esp_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_hole_esp_frame();'))
 def renderer(t):
  t=once(t,'pub mod wings;','pub mod wings;\npub mod hole_esp;')
  t=once(t,'    wings:wings::WingsRenderer,','    wings:wings::WingsRenderer,\n    hole_esp:hole_esp::HoleEspRenderer,')
  anchor='        let wings=wings::WingsRenderer::new(&device,frame_format,linear_light);'
  t=once(t,anchor,anchor+'\n        let hole_esp=hole_esp::HoleEspRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            wings,','            wings,\n            hole_esp,')
  anchor='        let hand_in_world = !self.wings.pending()'
  # Other independent ESP instances can already have expanded this condition.
  prepare='        self.hole_esp.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n'
  if anchor in t:t=once(t,anchor,prepare+'        let hand_in_world = !self.hole_esp.pending() && !self.wings.pending()')
  else:t=once(t,'        let hand_in_world = ',prepare+'        let hand_in_world = !self.hole_esp.pending() && ')
  anchor='        self.breadcrumbs.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,anchor,anchor+'\n        self.hole_esp.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_hole_esp'],'helper_names':[]}
