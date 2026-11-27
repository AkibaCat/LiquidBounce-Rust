"""Full StorageESP real storage snapshots, native model capture and independent styles."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 renderer=runpy.run_path(str(HERE/'storage-esp-render.py'))
 edit('crates/horizon-client/src/game/storage_esp.rs',lambda _:(HERE/'storage-esp-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/storage_esp.rs',lambda _:renderer['source']())
 edit('crates/horizon-client-render/src/gfx/storage_plain.rs',lambda _:renderer['plain_source']())
 for name in ['storage-esp-model.wgsl','storage-esp-sobel.frag']:edit('crates/horizon-client-render/src/gfx/'+name,lambda _,name=name:(HERE/name).read_text(encoding='utf8'))
 def game(t):
  t=once(t,'mod native_visuals;','mod native_visuals;\nmod storage_esp;')
  anchor='        let mut entities = std::mem::take(&mut self.entity_vertices);'
  return once(t,anchor,'        if !title {self.client_storage_esp_frame(camera,alpha);}else{horizon_client::gfx::storage_esp::begin_frame("null");horizon_client::gfx::storage_plain::begin_frame("null");}\n'+anchor)
 edit('crates/horizon-client/src/game.rs',game)
 # The native API generator reads only native_hooks.rs, not sibling Game impls.
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda t:once(t,'    /// Player FOV scale, after vanilla effects and before zoom and easing.\n','    pub(super) fn native_client_storage_esp(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_storage_esp(self,mod_api::mods(),input,geometry);}}\n\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'))
 def skin(t):
  # All selected living models must emit actual CPU vertices, including equipment layers.
  t=once(t,'r.skins_entities() && !horizon_client::gfx::chams::active()','r.skins_entities() && !horizon_client::gfx::chams::active() && !horizon_client::gfx::storage_esp::active()')
  return once(t,'!horizon_client::gfx::chams::active() && self.renderer','!horizon_client::gfx::chams::active() && !horizon_client::gfx::storage_esp::active() && self.renderer')
 edit('crates/horizon-client/src/game/creatures.rs',skin)
 def mobs(t):
  anchor='            let chams=self.client_chams_entity(mob.id as i32);'
  t=once(t,anchor,anchor+'\n            let storage=horizon_client::gfx::storage_esp::entity_color(mob.id as i32).is_some();')
  t=once(t,'            if !chams && culling.hides_entity() {','            if !chams && !storage && culling.hides_entity() {')
  # Invisibility must still submit the original outline silhouette while keeping the ordinary model hidden.
  t=once(t,'            if mob.invisible && true_sight.is_none() && outline.is_none() && !layers_only {','            if mob.invisible && true_sight.is_none() && outline.is_none() && !layers_only && !storage {')
  t=once(t,'            if modded {','            if modded {\n                horizon_client::gfx::storage_esp::capture_entity(mob.id as i32,&out[entity_start..]);\n                if storage && mob.invisible && true_sight.is_none() && outline.is_none(){out.truncate(entity_start);continue;}')
  anchor='            if let Some(rgb) = outline {\n                self.mark_outline(&out[entity_start..], rgb);'
  t=once(t,anchor,'            horizon_client::gfx::storage_esp::capture_entity(mob.id as i32,&out[entity_start..]);\n            if storage && mob.invisible && true_sight.is_none() && outline.is_none(){out.truncate(entity_start);continue;}\n'+anchor)
  t=once(t,'                if !chams && outline.is_none() {','                if !chams && !storage && outline.is_none() {')
  t=once(t,'            if !chams && outline.is_none() && culling.checks_vertices() && beam.is_none() {','            if !chams && !storage && outline.is_none() && culling.checks_vertices() && beam.is_none() {')
  return t
 edit('crates/horizon-client/src/game/creatures/mob_geometry.rs',mobs)
 def model_render(t):
  t=once(t,'if super::trader::is_invisible(m) && !crate::true_sight::active() {','if super::trader::is_invisible(m) && !crate::true_sight::active() && crate::gfx::storage_esp::entity_color(m.id as i32).is_none() {')
  return once(t,'if super::trader::is_invisible(m) && !crate::visual_environment::allows(crate::visual_environment::Choice::InvisibleEntities) { return; }','if super::trader::is_invisible(m) && crate::gfx::storage_esp::entity_color(m.id as i32).is_none() && !crate::visual_environment::allows(crate::visual_environment::Choice::InvisibleEntities) { return; }')
 edit('crates/horizon-client/src/mobs/render.rs',model_render)
 def vehicle(t,var,method):
  start=t.index('    pub(super) fn '+method+'(');end=t.index('\n    }',start)+6;body=t[start:end]
  anchor=f'            let culling = self.entity_cull_decision({var}, alpha);'if var=='b'else'            let culling = self.entity_cull_decision_in_bounds('
  body=once(body,anchor,f'            let storage=horizon_client::gfx::storage_esp::entity_color({var}.id as i32).is_some();\n'+anchor)
  body=once(body,'            if culling.hides_entity() {','            if !storage && culling.hides_entity() {')
  anchor=f'            self.finish_entity_occlusion(culling, {var}, out, entity_start);'
  body=once(body,anchor,f'            horizon_client::gfx::storage_esp::capture_entity({var}.id as i32,&out[entity_start..]);\n            if !storage {{self.finish_entity_occlusion(culling, {var}, out, entity_start);}}')
  return t[:start]+body+t[end:]
 edit('crates/horizon-client/src/game/boats.rs',lambda t:vehicle(t,'b','boat_geometry'))
 edit('crates/horizon-client/src/game/minecarts.rs',lambda t:vehicle(t,'c','cart_geometry'))
 def gfx(t):
  t=once(t,'pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod storage_esp;\npub mod storage_plain;')
  t=once(t,'    breadcrumbs: breadcrumbs::BreadcrumbRenderer,','    breadcrumbs: breadcrumbs::BreadcrumbRenderer,\n    storage_esp:storage_esp::StorageRenderer,\n    storage_plain:storage_plain::StoragePlainRenderer,')
  anchor='        let panorama = panorama::Panorama::new(&device, frame_format, panorama_faces);'
  t=once(t,anchor,'        let storage_esp=storage_esp::StorageRenderer::new(&device,frame_format,linear_light,&entity_layout);\n        let storage_plain=storage_plain::StoragePlainRenderer::new(&device,frame_format,linear_light);\n'+anchor)
  t=once(t,'            breadcrumbs,','            breadcrumbs,\n            storage_esp,storage_plain,')
  anchor='        self.breadcrumbs.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);'
  t=once(t,anchor,anchor+'\n        self.storage_esp.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);\n        self.storage_plain.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);')
  t=once(t,'        let hand_in_world = ','        let hand_in_world = !self.storage_esp.pending() && !self.storage_plain.pending() && ')
  anchor='        self.breadcrumbs.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(t,anchor,anchor+'\n        self.storage_esp.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees,&self.entity_group);\n        self.storage_plain.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_storage_esp'],'helper_names':[]}
