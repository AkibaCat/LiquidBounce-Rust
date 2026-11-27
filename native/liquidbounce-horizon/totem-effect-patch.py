"""Actual EntityEvent35 snapshot, ordered Soul geometry and independent ShaderLayer."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'breadcrumbs-render.rs').is_file()else HERE.parents[1]

def apply(edit,once):
 renderer=runpy.run_path(str(HERE/'totem-effect-render.py'))
 edit('crates/horizon-client/src/game/totem_effect.rs',lambda _:(HERE/'totem-effect-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/totem_effect.rs',lambda _:renderer['source']())
 edit('crates/horizon-client-render/src/gfx/totem-effect.wgsl',lambda _:renderer['shader']())
 for file in sorted((NATIVE/'breadcrumbs-shaders').iterdir()):edit('crates/horizon-client-render/src/gfx/totem-effect-shaders/'+file.name,lambda _,file=file:file.read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod target_renderer;','mod target_renderer;\nmod totem_effect;')
  return once(t,'        horizon_client::gfx::breadcrumbs::begin_frame("null");','        horizon_client::gfx::breadcrumbs::begin_frame("null");\n        horizon_client::gfx::totem_effect::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,'    pub(super) fn native_client_totem_effect(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_totem_effect(self,mod_api::mods(),input,geometry);}}\n\n'+anchor))
 edit('crates/horizon-client/src/game/remote.rs',lambda t:once(t,'        self.native_client_packet(&packet);','        self.native_client_packet(&packet);\n        self.client_totem_effect_packet(&packet);'))
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_target_renderer_frame(alpha);','        self.client_target_renderer_frame(alpha);\n        self.client_totem_effect_frame(alpha);'))
 def gfx(t):
  t=once(t,'pub mod target_renderer;','pub mod target_renderer;\npub mod totem_effect;')
  t=once(t,'    target_renderer:target_renderer::TargetRenderer,','    target_renderer:target_renderer::TargetRenderer,\n    totem_effect:totem_effect::TotemEffectRenderer,')
  t=once(t,'        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);','        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);\n        let totem_effect=totem_effect::TotemEffectRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            target_renderer,','            target_renderer,\n            totem_effect,')
  anchor='        self.target_renderer.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);'
  t=once(t,anchor,anchor+'\n        self.totem_effect.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);')
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.totem_effect.pending() && ')
  anchor='        self.target_renderer.draw(&self.device,&mut encoder,world_view,world_depth);'
  return once(t,anchor,anchor+'\n        self.totem_effect.draw(&self.device,&mut encoder,world_view,world_depth,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_totem_effect'],'helper_names':[]}
