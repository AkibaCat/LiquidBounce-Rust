"""Transactional StorageESP wiring; root applies serially after frozen handoff."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod storage_esp;','behavior')
 p.replace(E+'lib.rs','mod breadcrumbs_runtime;','mod breadcrumbs_runtime;\n#[cfg(feature="behavior-hooks")]\nmod storage_esp_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub crosshair:','    pub storage_esp:liquidbounce_behaviors::storage_esp::StorageEsp,\n    pub crosshair:','state')
 p.replace(E+'behaviors.rs','            crosshair: Default::default(),','            storage_esp:Default::default(),\n            crosshair: Default::default(),','default')
 p.replace(E+'behaviors.rs','            "Crosshair",','            "Crosshair",\n            "StorageESP",','registration')
 p.replace(E+'behaviors.rs','            view && !registry.running(self.no_bob_id),','            view && !registry.running(self.no_bob_id) && !(self.query_running(registry,"StorageESP") && self.storage_esp.active(self.query_running(registry,"ChestStealer")) && self.storage_esp.show_tracers()),','source tracer bob cancellation')
 p.replace(E+'chest_stealer_runtime.rs','impl ChestStealerRuntime {','impl ChestStealerRuntime {\n    pub(crate)fn storage_interacted(&self)->&BTreeSet<[i32;3]>{&self.interacted}','actual interacted containers')
 p.replace(E+'config.rs','#[path = "config/crosshair.rs"]','#[path="config/storage_esp.rs"]\nmod storage_esp;\npub use storage_esp::{StorageEspSetting,serialize_storage_esp};\n#[path = "config/crosshair.rs"]','config export')
 p.replace(E+'config.rs','    Crosshair => ("Crosshair", "Render", "crosshair"),','    Crosshair => ("Crosshair", "Render", "crosshair"),\n    StorageEsp => ("StorageESP", "Render", "storageESP"),','kind')
 p.replace(E+'config.rs','"PlayerTrails" => Some(Self::Breadcrumbs),','"ChestESP"=>Some(Self::StorageEsp), "PlayerTrails" => Some(Self::Breadcrumbs),','alias')
 p.replace(E+'config.rs','    Crosshair(CrosshairSetting),','    Crosshair(CrosshairSetting),\n    StorageEsp(StorageEspSetting),','request and setting',2)
 p.replace(E+'config.rs','            | ModuleKind::Crosshair','            | ModuleKind::Crosshair\n            | ModuleKind::StorageEsp','config support')
 p.replace(E+'config.rs','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),\n        ModuleKind::StorageEsp=>storage_esp::parse_storage_esp(&mut patch,&root),','parser')
 p.replace(E+'config.rs','        SettingChange::Crosshair(value) => {','        SettingChange::StorageEsp(value)=>{return ApplyResult::Runtime(RuntimeRequest::StorageEsp(value.clone()));}\n        SettingChange::Crosshair(value) => {','apply')
 p.replace(E+'actor.rs','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),\n            ModuleKind::StorageEsp=>config::serialize_storage_esp(&b.storage_esp,&common,format,&describe),','serializer')
 p.replace(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::StorageEsp(setting))=>{setting.apply(&mut self.state.behaviors.borrow_mut().storage_esp);}\n                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','setter')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.replace(E+'runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_storage_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientStorageEspEvent<\'_>){self.storage_esp(host,event);}\n'+anchor,'hook')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'logoff-spot-patch.py'","'logoff-spot-patch.py', 'storage-esp-patch.py'",'native patch')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} StorageESP changes across {len(p.text)} shared files')
 if args.apply:p.write();print('Integrated StorageESP')
