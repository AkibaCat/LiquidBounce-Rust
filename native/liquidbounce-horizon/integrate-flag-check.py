"""One-time transactional FlagCheck integration; no placeholder module."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod air_place;','pub mod air_place;\npub mod flag_check;','model')
 p.replace(E+'lib.rs','mod air_place_runtime;','mod air_place_runtime;\n#[cfg(feature="behavior-hooks")]\nmod flag_check_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub air_place:liquidbounce_behaviors::air_place::Settings,','    pub air_place:liquidbounce_behaviors::air_place::Settings,\n    pub flag_check:std::sync::Arc<crate::flag_check_runtime::FlagCheckService>,','field')
 p.replace(b,'            "AirPlace",','            "AirPlace",\n            "FlagCheck",','registry')
 p.replace(b,'            air_place:Default::default(),','            air_place:Default::default(),\n            flag_check:Default::default(),','default')
 p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.flag_check.configure(self.query_running(registry,"FlagCheck"));','lifecycle')
 c=E+'config.rs'
 p.replace(c,'pub use air_place::{AirPlaceSetting,serialize_air_place};','pub use air_place::{AirPlaceSetting,serialize_air_place};\n#[path="config/flag_check.rs"]\nmod flag_check;\npub use flag_check::{FlagCheckSetting,serialize_flag_check};','config')
 p.replace(c,'    AirPlace=>("AirPlace","World","airPlace"),','    AirPlace=>("AirPlace","World","airPlace"),\n    FlagCheck=>("FlagCheck","Misc","flagCheck"),','kind')
 p.replace(c,'    AirPlace(AirPlaceSetting),','    AirPlace(AirPlaceSetting),\n    FlagCheck(FlagCheckSetting),','enum',2)
 p.replace(c,'            | ModuleKind::AirPlace','            | ModuleKind::AirPlace\n            | ModuleKind::FlagCheck','dedicated')
 p.replace(c,'        ModuleKind::AirPlace=>air_place::parse_air_place(&mut patch,&root),','        ModuleKind::AirPlace=>air_place::parse_air_place(&mut patch,&root),\n        ModuleKind::FlagCheck=>flag_check::parse_flag_check(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::AirPlace(value)=>','        SettingChange::FlagCheck(value)=>return ApplyResult::Runtime(RuntimeRequest::FlagCheck(value.clone())),\n        SettingChange::AirPlace(value)=>','request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::AirPlace=>config::serialize_air_place(&b.air_place,&common,format,&describe),','            ModuleKind::AirPlace=>config::serialize_air_place(&b.air_place,&common,format,&describe),\n            ModuleKind::FlagCheck=>config::serialize_flag_check(&b.flag_check.settings(),&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::AirPlace(setting))=>','                ApplyResult::Runtime(RuntimeRequest::FlagCheck(setting))=>{let state=&self.state;let b=state.behaviors.borrow();b.flag_check.edit(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] FlagCheck setting: {e}");}});b.flag_check.configure(b.query_running(&state.registry,"FlagCheck"));}\n                ApplyResult::Runtime(RuntimeRequest::AirPlace(setting))=>','apply')
 p.replace(a,'ModuleKind::MoreCarry=>vec!["XCarry"],','ModuleKind::FlagCheck=>vec!["FlagDetect"], ModuleKind::MoreCarry=>vec!["XCarry"],','alias')
 r=E+'runtime.rs'
 p.replace(r,'            pipeline.add_listener(0, modules.auto_disable_packets.clone());','            pipeline.add_listener(0, modules.auto_disable_packets.clone());\n            pipeline.add_listener(0, modules.flag_check.clone());','packets')
 p.replace(r,'    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {\n        self.flag_check_tick(host);','tick')
 p.replace(E+'blink_esp_runtime.rs','        *event.geometry = json!({"body":body,"line":line,"model":model}).to_string();','        crate::flag_check_runtime::append_geometry(&modules.flag_check,p["firstPerson"].as_bool().unwrap_or(true),&mut body);\n        *event.geometry = json!({"body":body,"line":line,"model":model}).to_string();','wireframe renderer')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'flag-check-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--apply',action='store_true');a=parser.parse_args();p=prepare()
 if a.apply:p.write()
 print(f'FlagCheck {"integrated"if a.apply else "validated"}: {len(p.steps)} changes / {len(p.text)} files')
