"""BlinkManager global visual bridge, independent of Blink's packet/dummy hook."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/blink_esp.rs',lambda _:runpy.run_path(str(HERE/'blink-esp-game.py'))['source']())
 render=runpy.run_path(str(HERE/'blink-esp-render.py'))
 for name in ['blink_esp','blink_line']:edit(f'crates/horizon-client-render/src/gfx/{name}.rs',lambda _:render['source']())
 edit('crates/horizon-client-render/src/gfx/blink_esp.wgsl',lambda _:render['shader']())
 def game(t):
  t=once(t,'mod crosshair;','mod crosshair;\nmod blink_esp;')
  t=once(t,'        horizon_client::gfx::wings::begin_frame("null");','        horizon_client::gfx::wings::begin_frame("null");\n        horizon_client::gfx::blink_esp::begin_frame("null");\n        horizon_client::gfx::blink_line::begin_frame("null");')
  return once(t,"        // In first person only a shader pack's shadow pass draws the player.","        self.client_blink_frame(camera,alpha,&mut out);\n        // In first person only a shader pack's shadow pass draws the player.")
 edit('crates/horizon-client/src/game.rs',game)
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,anchor,(HERE/'blink-esp-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def renderer(t):
  t=once(t,'pub mod wings;','pub mod wings;\npub mod blink_esp;\npub mod blink_line;')
  t=once(t,'    wings:wings::WingsRenderer,','    wings:wings::WingsRenderer,\n    blink_esp:blink_esp::BlinkRenderer,\n    blink_line:blink_line::BlinkRenderer,')
  anchor='        let wings=wings::WingsRenderer::new(&device,frame_format,linear_light);'
  t=once(t,anchor,anchor+'\n        let blink_esp=blink_esp::BlinkRenderer::new(&device,frame_format,linear_light);\n        let blink_line=blink_line::BlinkRenderer::new(&device,frame_format,linear_light);')
  t=once(t,'            wings,','            wings,\n            blink_esp,\n            blink_line,')
  anchor='        let hand_in_world = '
  t=once(t,anchor,'        self.blink_esp.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        self.blink_line.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        let hand_in_world = !self.blink_esp.pending() && !self.blink_line.pending() && ')
  anchor='        self.breadcrumbs.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,anchor,anchor+'\n        self.blink_esp.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);\n        self.blink_line.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_blink_esp'],'helper_names':['client_blink_visual']}

