"""Actual player snapshots and full six-mode Hats draw pipe."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/hats.rs',lambda _:(HERE/'hats-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/hats.rs',lambda _:runpy.run_path(str(HERE/'hats-render.py'))['source']())
 edit('crates/horizon-client-render/src/gfx/hats-image.wgsl',lambda _:(HERE/'hats-image.wgsl').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod tracers;','mod tracers;\nmod hats;')
  t=once(t,'    rope_rot_old: (f32, f32),','    client_hats_rotations:std::cell::RefCell<hats::Rotations>,\n    rope_rot_old: (f32, f32),')
  t=once(t,'            rope_rot_old: (0.0, 10.0),','            client_hats_rotations:Default::default(),\n            rope_rot_old: (0.0, 10.0),')
  return once(t,'        horizon_client::gfx::tracers::begin_frame("null");','        horizon_client::gfx::tracers::begin_frame("null");\n        horizon_client::gfx::hats::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n';t=once(t,anchor,(HERE/'hats-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  t=once(t,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_hats_tick();')
  return once(t,'        self.client_tracers_reset();','        self.client_hats_reset();\n        self.client_tracers_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_tracers_frame(alpha,camera,view);','        self.client_tracers_frame(alpha,camera,view);\n        self.client_hats_frame(alpha);'))
 def renderer(t):
  t=once(t,'pub mod tracers;','pub mod tracers;\npub mod hats;')
  t=once(t,'    tracers: tracers::TracerRenderer,','    tracers: tracers::TracerRenderer,\n    hats:hats::HatsRenderer,')
  t=once(t,'        let tracers=tracers::TracerRenderer::new(&device,frame_format,linear_light);','        let hats=hats::HatsRenderer::new(&device,frame_format,linear_light);\n        let tracers=tracers::TracerRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            tracers,','            tracers,\n            hats,')
  t=once(t,'        let hand_in_world = !self.tracers.pending()', '        self.hats.prepare(&self.device,&self.queue,jittered,frame.camera,breadcrumbs_size,frame.title);\n        let hand_in_world = !self.hats.pending() && !self.tracers.pending()')
  anchor='        self.tracers.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,anchor,anchor+'\n        self.hats.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_hats'],'helper_names':[]}
