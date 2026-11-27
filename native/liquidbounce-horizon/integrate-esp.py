#!/usr/bin/env python3
"""Transactional complete ESP and shared TrueSight/OutlineGlow wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
Plan=runpy.run_path(str(NATIVE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod tracers;','pub mod tracers;\npub mod esp;')
 p.edit('lib.rs','mod tracers_runtime;','mod tracers_runtime;\n#[cfg(feature="behavior-hooks")]\nmod esp_runtime;')
 p.edit('behaviors.rs','    pub tracers: liquidbounce_behaviors::tracers::Tracers,','    pub tracers: liquidbounce_behaviors::tracers::Tracers,\n    pub esp:liquidbounce_behaviors::esp::Esp,')
 p.edit('behaviors.rs','            "Tracers",','            "Tracers",\n            "ESP",')
 p.edit('behaviors.rs','            tracers: Default::default(),','            tracers: Default::default(),\n            esp:Default::default(),')
 a='            ModuleCallback::OnDisabled(id) if self.query_ids.get("Tracers") == Some(&id) => { self.tracers.clear(); }'
 p.edit('behaviors.rs',a,a+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("ESP")==Some(&id)=>self.esp.clear(),')
 p.edit('config.rs','pub use tracers::{TracersSetting,serialize_tracers};','pub use tracers::{TracersSetting,serialize_tracers};\n#[path="config/esp.rs"]\nmod esp;\npub use esp::{EspSetting,serialize_esp};')
 p.edit('config.rs','    Tracers => ("Tracers", "Render", "tracers"),','    Tracers => ("Tracers", "Render", "tracers"),\n    Esp=>("ESP","Render","esp"),')
 p.edit('config.rs','    Tracers(TracersSetting),','    Tracers(TracersSetting),\n    Esp(EspSetting),',2)
 p.edit('config.rs','            | ModuleKind::Tracers','            | ModuleKind::Tracers\n            | ModuleKind::Esp')
 p.edit('config.rs','        ModuleKind::Tracers => tracers::parse_tracers(&mut patch,&root),','        ModuleKind::Tracers => tracers::parse_tracers(&mut patch,&root),\n        ModuleKind::Esp=>esp::parse_esp(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Tracers(value) => {','        SettingChange::Esp(value)=>return ApplyResult::Runtime(RuntimeRequest::Esp(value.clone())),\n        SettingChange::Tracers(value) => {')
 p.edit('actor.rs','            ModuleKind::Tracers => config::serialize_tracers(&b.tracers,&common,format,&describe),','            ModuleKind::Tracers => config::serialize_tracers(&b.tracers,&common,format,&describe),\n            ModuleKind::Esp=>config::serialize_esp(&b.esp,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Tracers(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Esp(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().esp),\n                ApplyResult::Runtime(RuntimeRequest::Tracers(setting)) => {')
 a='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.edit('runtime.rs',a,'    #[cfg(feature="behavior-hooks")]\n    fn client_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientEspEvent<\'_>){self.esp(host,event);}\n'+a)
 p.edit('runtime.rs','        self.state().behaviors.borrow_mut().tracers.clear();','        self.state().behaviors.borrow_mut().tracers.clear();\n        self.state().behaviors.borrow_mut().esp.clear();')
 p.edit('true_sight_runtime.rs','snapshot.entity.invisible,false)','snapshot.entity.invisible,modules.query_running(&state.registry,"ESP") && modules.esp.mode==3 && modules.esp.true_sight.contains(&event.entity))')
 p.edit('../../make-behavior-patch.py',"'storage-esp-patch.py',","'storage-esp-patch.py', 'esp-patch.py',")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args();plan=prepare()
 if not args.check:plan.commit()
 print('ESP '+('validated'if args.check else'integrated')+f': {len(plan.changed)} shared files')
