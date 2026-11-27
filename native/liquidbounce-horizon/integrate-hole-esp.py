"""Transactional HoleESP wiring; default validates only, root applies serially."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod crosshair;','pub mod crosshair;\npub mod hole_tracker;\npub mod hole_esp;','behavior exports')
 p.replace(E+'lib.rs','mod crosshair_runtime;','mod crosshair_runtime;\n#[cfg(feature="behavior-hooks")]\nmod hole_esp_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub crosshair:','    pub hole_esp: liquidbounce_behaviors::hole_esp::HoleEsp,\n    pub crosshair:','settings')
 p.replace(E+'behaviors.rs','            crosshair: Default::default(),','            hole_esp: Default::default(),\n            crosshair: Default::default(),','default')
 p.replace(E+'behaviors.rs','            "Crosshair",','            "Crosshair",\n            "HoleESP",','registration')
 p.replace(E+'config.rs','#[path = "config/crosshair.rs"]','#[path = "config/hole_esp.rs"]\nmod hole_esp;\npub use hole_esp::{HoleEspSetting,serialize_hole_esp};\n#[path = "config/crosshair.rs"]','config export')
 p.replace(E+'config.rs','    Crosshair => ("Crosshair", "Render", "crosshair"),','    Crosshair => ("Crosshair", "Render", "crosshair"),\n    HoleEsp => ("HoleESP", "Render", "holeESP"),','kind')
 p.replace(E+'config.rs','    Crosshair(CrosshairSetting),','    Crosshair(CrosshairSetting),\n    HoleEsp(HoleEspSetting),','request and setting variants',2)
 p.replace(E+'config.rs','            | ModuleKind::Crosshair','            | ModuleKind::Crosshair\n            | ModuleKind::HoleEsp','supported config')
 p.replace(E+'config.rs','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),\n        ModuleKind::HoleEsp => hole_esp::parse_hole_esp(&mut patch,&root),','parser')
 p.replace(E+'config.rs','        SettingChange::Crosshair(value) => {','        SettingChange::HoleEsp(value) => { return ApplyResult::Runtime(RuntimeRequest::HoleEsp(value.clone())); }\n        SettingChange::Crosshair(value) => {','apply dispatch')
 p.replace(E+'actor.rs','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),\n            ModuleKind::HoleEsp => config::serialize_hole_esp(&b.hole_esp,&common,format,&describe),','serializer')
 p.replace(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::HoleEsp(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().hole_esp); }\n                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','actor setter')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.replace(E+'runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_hole_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientHoleEspEvent<\'_>){self.hole_esp(host,event);}\n'+anchor,'hook')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'crosshair-patch.py'","'crosshair-patch.py', 'hole-esp-patch.py'",'native patch')
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--apply',action='store_true');a=parser.parse_args();p=prepare();print(f'Validated {len(p.steps)} HoleESP changes across {len(p.text)} shared files')
 if a.apply:p.write();print('Integrated HoleESP')
