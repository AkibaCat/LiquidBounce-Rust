"""Complete independent JumpEffect render and synchronous local jump HEAD bridge."""
from pathlib import Path
import json,runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'breadcrumbs-render.rs').is_file()else HERE.parents[1]
def apply(edit,once):
 renderer=runpy.run_path(str(HERE/'jump-effect-render.py'))
 edit('crates/horizon-client/src/jump_effect_bridge.rs',lambda _:(HERE/'jump-effect-bridge.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game/jump_effect.rs',lambda _:(HERE/'jump-effect-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/jump_effect.rs',lambda _:renderer['source']())
 edit('crates/horizon-client-render/src/gfx/jump-effect.wgsl',lambda _:renderer['shader']())
 edit('crates/horizon-client-render/src/gfx/jump-effect-textures.rs',lambda _:renderer['textures']())
 for file in sorted((NATIVE/'breadcrumbs-shaders').iterdir()):edit('crates/horizon-client-render/src/gfx/jump-effect-shaders/'+file.name,lambda _,file=file:file.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/client_lib.rs',lambda t:once(t,'pub mod high_jump_bridge;','pub mod high_jump_bridge;\npub mod jump_effect_bridge;'))
 def entry(t):
  t=once(t,'    horizon_client::high_jump_bridge::install(dispatch_high_jump);','    horizon_client::high_jump_bridge::install(dispatch_high_jump);\n    horizon_client::jump_effect_bridge::install(dispatch_jump_effect_jump);')
  return t+'\nfn dispatch_jump_effect_jump(position:[f64;3]){crate::mod_api::dispatch_client_jump_effect_jump(crate::mod_api::mods(),position);}\n'
 edit('crates/horizon-client/src/entry.rs',entry)
 edit('tools/native-hook-fragments/liquidbounce_jump_effect.json',lambda _:json.dumps({'hooks':[dict(name='client_jump_effect_jump',target='horizon_client_lib::jump_effect_bridge::native_client_jump_effect_jump',side='client',cancellable=False,mod_api={'contract':'listen'})],'helpers':[]},indent=2)+'\n')
 edit('crates/horizon-client/src/motion.rs',lambda t:once(t,'            crate::high_jump_bridge::jump(&mut power);','            crate::jump_effect_bridge::jump([self.player.position.x,self.player.position.y,self.player.position.z]);\n            crate::high_jump_bridge::jump(&mut power);'))
 def game(t):
  t=once(t,'mod target_renderer;','mod target_renderer;\nmod jump_effect;')
  return once(t,'        horizon_client::gfx::breadcrumbs::begin_frame("null");','        horizon_client::gfx::breadcrumbs::begin_frame("null");\n        horizon_client::gfx::jump_effect::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  t=once(t,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_jump_effect_errors();')
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  return once(t,anchor,'    pub(super) fn native_client_jump_effect(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_jump_effect(self,mod_api::mods(),input,geometry);}}\n\n'+anchor)
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_target_renderer_frame(alpha);','        self.client_target_renderer_frame(alpha);\n        self.client_jump_effect_frame(alpha);'))
 def gfx(t):
  t=once(t,'pub mod target_renderer;','pub mod target_renderer;\npub mod jump_effect;')
  t=once(t,'    target_renderer:target_renderer::TargetRenderer,','    target_renderer:target_renderer::TargetRenderer,\n    jump_effect:jump_effect::JumpEffectRenderer,')
  t=once(t,'        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);','        let target_renderer=target_renderer::TargetRenderer::new(&device,frame_format,linear_light);\n        let jump_effect=jump_effect::JumpEffectRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            target_renderer,','            target_renderer,\n            jump_effect,')
  anchor='        self.target_renderer.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);'
  t=once(t,anchor,anchor+'\n        self.jump_effect.prepare(&self.device,&self.queue,jittered,frame.camera,breadcrumbs_size,frame.title);')
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.jump_effect.pending() && ')
  anchor='        self.target_renderer.draw(&self.device,&mut encoder,world_view,world_depth);'
  return once(t,anchor,anchor+'\n        self.jump_effect.draw(&self.device,&mut encoder,world_view,world_depth,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_jump_effect'],'helper_names':[]}
