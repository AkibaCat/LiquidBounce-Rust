"""ModuleBlockESP loaded-section tracker with native voxel shape and independent glow pass."""
from pathlib import Path
import runpy
HERE=Path(__file__).resolve().parent
def apply(edit,once):
 renderer=runpy.run_path(str(HERE/'storage-esp-render.py'))['source']().replace('StorageRenderer','BlockEspRenderer').replace('StorageESP','BlockESP')
 edit('crates/horizon-client/src/game/block_esp.rs',lambda _:(HERE/'block-esp-game.rs').read_text(encoding='utf8'))
 edit('crates/horizon-client-render/src/gfx/block_esp.rs',lambda _:renderer)
 def game(s):
  s=once(s,'mod storage_esp;','mod storage_esp;\nmod block_esp;')
  s=once(s,'    client_nametags:std::cell::RefCell<nametags::State>,','    client_nametags:std::cell::RefCell<nametags::State>,\n    client_block_esp:std::cell::RefCell<block_esp::State>,')
  s=once(s,'            client_nametags:Default::default(),','            client_nametags:Default::default(),\n            client_block_esp:Default::default(),')
  anchor='        let mut entities = std::mem::take(&mut self.entity_vertices);'
  return once(s,anchor,'        if !title{self.client_block_esp_frame(camera);}else{horizon_client::gfx::block_esp::begin_frame("null");}\n'+anchor)
 edit('crates/horizon-client/src/game.rs',game)
 def hooks(s):
  anchor='    pub(super) fn native_client_storage_esp('
  s=once(s,anchor,'    pub(super) fn native_client_block_esp(&self,input:&str,geometry:&mut String){if self.native_window(){mod_api::dispatch_client_block_esp(self,mod_api::mods(),input,geometry);}}\n\n'+anchor)
  return once(s,'        *self.client_nametags.borrow_mut()=Default::default();','        *self.client_nametags.borrow_mut()=Default::default();\n        *self.client_block_esp.borrow_mut()=Default::default();')
 edit('crates/horizon-client/src/game/native_hooks.rs',hooks)
 def gfx(s):
  s=once(s,'pub mod storage_plain;','pub mod storage_plain;\npub mod block_esp;')
  s=once(s,'    storage_plain:storage_plain::StoragePlainRenderer,','    storage_plain:storage_plain::StoragePlainRenderer,\n    block_esp:block_esp::BlockEspRenderer,')
  anchor='        let storage_plain=storage_plain::StoragePlainRenderer::new(&device,frame_format,linear_light);'
  s=once(s,anchor,anchor+'\n        let block_esp=block_esp::BlockEspRenderer::new(&device,frame_format,linear_light,&entity_layout);')
  s=once(s,'            storage_esp,storage_plain,','            storage_esp,storage_plain,block_esp,')
  anchor='        self.storage_plain.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);'
  s=once(s,anchor,anchor+'\n        self.block_esp.prepare(&self.device,jittered,frame.camera,breadcrumbs_size,frame.title);')
  s=once(s,'        let hand_in_world = ','        let hand_in_world = !self.block_esp.pending() && ')
  anchor='        self.storage_plain.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees);'
  return once(s,anchor,anchor+'\n        self.block_esp.draw(&self.device,&mut encoder,world_view,frame.shader_game_time,frame.camera,views.primary().view,frame.fov_degrees,&self.entity_group);')
 edit('crates/horizon-client-render/src/gfx.rs',gfx)
 return {'hook_names':['client_block_esp'],'helper_names':[]}
