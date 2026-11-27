"""Complete ModuleBreadcrumbs native render bridge and original nine ShaderLayer styles."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 edit('crates/horizon-client/src/game/breadcrumbs.rs',lambda _:(HERE/'breadcrumbs-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/breadcrumbs.rs',lambda _:(HERE/'breadcrumbs-render.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/breadcrumbs.wgsl',lambda _:(HERE/'breadcrumbs.wgsl').read_text(encoding='utf8'))
 for file in sorted((HERE/'breadcrumbs-shaders').iterdir()):edit('crates/horizon-client-render/src/gfx/breadcrumbs-shaders/'+file.name,lambda _,file=file:file.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda t:once(once(t,'mod native_visuals;','mod native_visuals;\nmod breadcrumbs;'),'        let mut chams_settings="null".to_owned();','        horizon_client::gfx::breadcrumbs::begin_frame("null");\n        let mut chams_settings="null".to_owned();'))
 def hooks(t):
  t=once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n',(HERE/'breadcrumbs-host.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n')
  t=once(t,'        mod_api::dispatch_client_tick_post(self, mod_api::mods());','        self.client_breadcrumbs_tick();\n        mod_api::dispatch_client_tick_post(self, mod_api::mods());')
  return once(t,'        super::auto_walk_runtime::level_changed();','        self.client_breadcrumbs_reset();\n        super::auto_walk_runtime::level_changed();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 edit('crates/horizon-client/src/game/world_primitives.rs',lambda t:once(t,'        self.client_block_outline_geometry(&mut primitives);','        self.client_block_outline_geometry(&mut primitives);\n        self.client_breadcrumbs_frame(alpha);'))
 def renderer(t):
  t=once(t,'pub mod world_primitives;','pub mod world_primitives;\npub mod breadcrumbs;')
  t=once(t,'    world_primitives: world_primitives::WorldPrimitives,','    world_primitives: world_primitives::WorldPrimitives,\n    breadcrumbs: breadcrumbs::BreadcrumbRenderer,')
  t=once(t,'        let panorama = panorama::Panorama::new(&device, frame_format, panorama_faces);','        let breadcrumbs=breadcrumbs::BreadcrumbRenderer::new(&device,frame_format,linear_light);\n        let panorama = panorama::Panorama::new(&device, frame_format, panorama_faces);')
  t=once(t,'            world_primitives,','            world_primitives,\n            breadcrumbs,')
  t=once(t,'        let hand_in_world = !self.chams.pending() && (frame.hand.len() / 4) > 0','        let breadcrumbs_size=self.world_post.size().filter(|_|scaled.is_some()).unwrap_or([self.config.width,self.config.height]);\n        self.breadcrumbs.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        let hand_in_world = !self.breadcrumbs.pending() && !self.chams.pending() && (frame.hand.len() / 4) > 0')
  return once(t,'        self.chams.composite(&mut encoder,world_view);','        self.chams.composite(&mut encoder,world_view);\n        self.breadcrumbs.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',renderer)
 return {'hook_names':['client_breadcrumbs'],'helper_names':[]}
