"""Wings actual player/body bridge and independent depth-tested shader renderer."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/wings.rs',lambda _:(HERE/'wings-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/wings.rs',lambda _:runpy.run_path(str(HERE/'wings-render.py'))['source']())
 edit('crates/horizon-client-render/src/gfx/wings.wgsl',lambda _:(HERE/'wings.wgsl').read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod hats;','mod hats;\nmod wings;');t=once(t,'    rope_rot_old: (f32, f32),','    client_wings_bodies:std::cell::RefCell<wings::Bodies>,\n    rope_rot_old: (f32, f32),');t=once(t,'            rope_rot_old: (0.0, 10.0),','            client_wings_bodies:Default::default(),\n            rope_rot_old: (0.0, 10.0),');return once(t,'        horizon_client::gfx::hats::begin_frame("null");','        horizon_client::gfx::hats::begin_frame("null");\n        horizon_client::gfx::wings::begin_frame("null");')
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(t):
  anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n';t=once(t,anchor,(HERE/'wings-host.rs').read_text(encoding='utf8')+'\n'+anchor);t=once(t,'    pub(super) fn native_client_tick_post(&mut self) {','    pub(super) fn native_client_tick_post(&mut self) {\n        self.client_wings_tick();');return once(t,'        self.client_hats_reset();','        self.client_wings_reset();\n        self.client_hats_reset();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/lan_actions.rs',lambda t:once(t,'impl GuestPlay {','impl GuestPlay {\n    pub(super) fn client_wings_swinging(&self)->bool{self.swing.swinging()}'))
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_hats_frame(alpha);','        self.client_hats_frame(alpha);\n        self.client_wings_frame(alpha);'))
 def renderer(t):
  t=once(t,'pub mod hats;','pub mod hats;\npub mod wings;');t=once(t,'    hats:hats::HatsRenderer,','    hats:hats::HatsRenderer,\n    wings:wings::WingsRenderer,');t=once(t,'        let hats=hats::HatsRenderer::new(&device,frame_format,linear_light);','        let wings=wings::WingsRenderer::new(&device,frame_format,linear_light);\n        let hats=hats::HatsRenderer::new(&device,frame_format,linear_light);');t=once(t,'            hats,','            hats,\n            wings,');t=once(t,'        let hand_in_world = !self.hats.pending()', '        self.wings.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        let hand_in_world = !self.wings.pending() && !self.hats.pending()');anchor='        self.hats.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);';return once(t,anchor,anchor+'\n        self.wings.draw(&self.device,&mut encoder,world_view,world_depth,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_wings'],'helper_names':[]}
