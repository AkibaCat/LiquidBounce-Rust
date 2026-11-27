"""Actual world-render, tick and independent variable-width ShaderLayer bridge."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/tracers.rs',lambda _:(HERE/'tracers-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/tracers.rs',lambda _:runpy.run_path(str(HERE/'tracers-render.py'))['source']())
 edit('crates/horizon-client/src/game.rs',lambda t:once(once(t,'mod breadcrumbs;','mod breadcrumbs;\nmod tracers;'),'        horizon_client::gfx::breadcrumbs::begin_frame("null");','        horizon_client::gfx::breadcrumbs::begin_frame("null");\n        horizon_client::gfx::tracers::begin_frame("null");'))
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n';t=once(t,anchor,(HERE/'tracers-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  t=once(t,'        self.client_breadcrumbs_tick();','        self.client_tracers_tick();\n        self.client_breadcrumbs_tick();')
  return once(t,'        self.client_breadcrumbs_reset();','        self.client_tracers_reset();\n        self.client_breadcrumbs_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_breadcrumbs_frame(alpha);','        self.client_breadcrumbs_frame(alpha);\n        self.client_tracers_frame(alpha,camera,view);'))
 def renderer(t):
  t=once(t,'pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod tracers;')
  t=once(t,'    breadcrumbs: breadcrumbs::BreadcrumbRenderer,','    breadcrumbs: breadcrumbs::BreadcrumbRenderer,\n    tracers: tracers::TracerRenderer,')
  t=once(t,'        let breadcrumbs=breadcrumbs::BreadcrumbRenderer::new(&device,frame_format,linear_light);','        let tracers=tracers::TracerRenderer::new(&device,frame_format,linear_light);\n        let breadcrumbs=breadcrumbs::BreadcrumbRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            breadcrumbs,','            breadcrumbs,\n            tracers,')
  t=once(t,'        let hand_in_world = !self.breadcrumbs.pending()', '        self.tracers.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        let hand_in_world = !self.tracers.pending() && !self.breadcrumbs.pending()')
  anchor='        self.breadcrumbs.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,anchor,anchor+'\n        self.tracers.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_tracers'],'helper_names':[]}
