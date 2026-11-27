#!/usr/bin/env python3
"""Idempotent transactional BlockESP shared wiring; --check validates without writing."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Base=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
Layout=runpy.run_path(str(HERE/'integrate-auto-walk.py'))
ROOT,ENGINE=Layout['ROOT'],Layout['ENGINE']
class Plan(Base):
 def edit(self,name,old,new,count=1):
  path=ROOT/(ENGINE+name)
  source=self.changed.get(path,path.read_text(encoding='utf8'))
  if source.count(new)==count:return
  super().edit(name,old,new,count)
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod storage_esp;','pub mod storage_esp;\npub mod block_esp;')
 p.edit('lib.rs','mod storage_esp_runtime;','mod storage_esp_runtime;\n#[cfg(feature="behavior-hooks")]\nmod block_esp_runtime;')
 p.edit('behaviors.rs','    pub storage_esp:liquidbounce_behaviors::storage_esp::StorageEsp,','    pub storage_esp:liquidbounce_behaviors::storage_esp::StorageEsp,\n    pub block_esp:liquidbounce_behaviors::block_esp::Settings,')
 p.edit('behaviors.rs','            storage_esp:Default::default(),','            storage_esp:Default::default(),\n            block_esp:Default::default(),')
 p.edit('behaviors.rs','            "StorageESP",','            "StorageESP",\n            "BlockESP",')
 p.edit('config.rs','pub use storage_esp::{StorageEspSetting,serialize_storage_esp};','pub use storage_esp::{StorageEspSetting,serialize_storage_esp};\n#[path="config/block_esp.rs"]\nmod block_esp;\npub use block_esp::{BlockEspSetting,serialize_block_esp};')
 p.edit('config.rs','    StorageEsp => ("StorageESP", "Render", "storageESP"),','    StorageEsp => ("StorageESP", "Render", "storageESP"),\n    BlockEsp => ("BlockESP", "Render", "blockESP"),')
 p.edit('config.rs','    StorageEsp(StorageEspSetting),','    StorageEsp(StorageEspSetting),\n    BlockEsp(BlockEspSetting),',2)
 p.edit('config.rs','            | ModuleKind::StorageEsp','            | ModuleKind::StorageEsp\n            | ModuleKind::BlockEsp')
 p.edit('config.rs','        ModuleKind::StorageEsp=>storage_esp::parse_storage_esp(&mut patch,&root),','        ModuleKind::StorageEsp=>storage_esp::parse_storage_esp(&mut patch,&root),\n        ModuleKind::BlockEsp=>block_esp::parse_block_esp(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::StorageEsp(value)=>{return ApplyResult::Runtime(RuntimeRequest::StorageEsp(value.clone()));}','        SettingChange::StorageEsp(value)=>{return ApplyResult::Runtime(RuntimeRequest::StorageEsp(value.clone()));}\n        SettingChange::BlockEsp(value)=>{return ApplyResult::Runtime(RuntimeRequest::BlockEsp(value.clone()));}')
 p.edit('actor.rs','            ModuleKind::StorageEsp=>config::serialize_storage_esp(&b.storage_esp,&common,format,&describe),','            ModuleKind::StorageEsp=>config::serialize_storage_esp(&b.storage_esp,&common,format,&describe),\n            ModuleKind::BlockEsp=>config::serialize_block_esp(&b.block_esp,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::StorageEsp(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().storage_esp);}','                ApplyResult::Runtime(RuntimeRequest::StorageEsp(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().storage_esp);}\n                ApplyResult::Runtime(RuntimeRequest::BlockEsp(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().block_esp,&self.state.block_registry){eprintln!("[LiquidBounce] BlockESP config: {e}");}}')
 p.edit('runtime.rs','    #[cfg(feature="behavior-hooks")]\n    fn client_storage_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientStorageEspEvent<\'_>){self.storage_esp(host,event);}','    #[cfg(feature="behavior-hooks")]\n    fn client_storage_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientStorageEspEvent<\'_>){self.storage_esp(host,event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_block_esp(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlockEspEvent<\'_>){self.block_esp(event);}')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'block-esp-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--check',action='store_true');args=ap.parse_args();p=prepare()
 if not args.check:p.commit()
 print(f"BlockESP {'validated'if args.check else'integrated'}: {len(p.changed)} shared files")
