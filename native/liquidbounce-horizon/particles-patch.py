"""Actual attack/tick/camera bridge and frozen image pipeline with scene depth."""
from pathlib import Path
import base64
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
FILES=['glow','star','dollar','crown','heart','lightning','line','point','rhombus','snowflake','spark']
def textures():
 source=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/particles'
 return '// SPDX-License-Identifier: GPL-3.0-or-later\n// Pinned upstream PNG bytes; base64 avoids binary patch portability issues.\nconst PNGS:[&str;11]=[\n'+''.join('"'+base64.b64encode((source/(name+'.png')).read_bytes()).decode()+'",\n'for name in FILES)+'];\n'
def apply(edit,once):
 edit('crates/horizon-client/src/game/particles.rs',lambda _:(HERE/'particles-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/particles.rs',lambda _:(HERE/'particles-render.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/particles.wgsl',lambda _:(HERE/'particles.wgsl').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/particles-textures.rs',lambda _:textures())
 def game(t):
  t=once(t,'mod native_visuals;','mod native_visuals;\nmod particles;')
  return once(t,'        horizon_client::gfx::wings::begin_frame("null");','        horizon_client::gfx::wings::begin_frame("null");\n        horizon_client::gfx::particles::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
  t=once(t,anchor,(HERE/'particles-host.rs').read_text(encoding='utf8')+'\n'+anchor)
  return once(t,'    pub(super) fn native_client_tick_pre(&mut self) {','    pub(super) fn native_client_tick_pre(&mut self) {\n        self.client_particles_tick();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_particles_frame(alpha);'))
 def renderer(t):
  t=once(t,'pub mod wings;','pub mod wings;\npub mod particles;')
  t=once(t,'    wings:wings::WingsRenderer,','    wings:wings::WingsRenderer,\n    particles:particles::ParticlesRenderer,')
  anchor='        let wings=wings::WingsRenderer::new(&device,frame_format,linear_light);'
  t=once(t,anchor,anchor+'\n        let particles=particles::ParticlesRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            wings,','            wings,\n            particles,')
  t=once(t,'        let hand_in_world = ','        self.particles.prepare(&self.device,&self.queue,jittered,frame.camera,frame.title);\n        let hand_in_world = !self.particles.pending() && ')
  anchor='        self.wings.draw(&self.device,&mut encoder,world_view,world_depth,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,anchor,anchor+'\n        self.particles.draw(&self.device,&mut encoder,world_view,world_depth);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_particles'],'helper_names':['client_particles_context','client_particles_direction']}
