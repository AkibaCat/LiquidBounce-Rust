from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod packet_logger;','pub mod packet_logger;\npub mod anti_staff;','model')
 p.replace(E+'lib.rs','mod packet_logger_runtime;','mod packet_logger_runtime;\n#[cfg(feature="behavior-hooks")]\nmod anti_staff_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub packet_logger:','    pub anti_staff:std::sync::Arc<crate::anti_staff_runtime::AntiStaffService>,\n    pub packet_logger:','field')
 p.replace(b,'            "PacketLogger",','            "PacketLogger",\n            "AntiStaff",','registration')
 p.replace(b,'            packet_logger:Default::default(),','            packet_logger:Default::default(),\n            anti_staff:Default::default(),','default')
 p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.anti_staff.configure(self.query_ids.get("AntiStaff").is_some_and(|id|registry.enabled_without_world(*id)),self.query_running(registry,"AntiStaff"));','lifecycle')
 c=E+'config.rs'
 p.replace(c,'pub use packet_logger::{PacketLoggerSetting,serialize_packet_logger};','pub use packet_logger::{PacketLoggerSetting,serialize_packet_logger};\n#[path="config/anti_staff.rs"]\nmod anti_staff;\npub use anti_staff::{AntiStaffSetting,serialize_anti_staff};','config')
 p.replace(c,'    PacketLogger=>("PacketLogger","Misc","packetLogger"),','    PacketLogger=>("PacketLogger","Misc","packetLogger"),\n    AntiStaff=>("AntiStaff","Misc","antiStaff"),','kind')
 p.replace(c,'    PacketLogger(PacketLoggerSetting),','    PacketLogger(PacketLoggerSetting),\n    AntiStaff(AntiStaffSetting),','enum',2)
 p.replace(c,'            | ModuleKind::PacketLogger','            | ModuleKind::PacketLogger\n            | ModuleKind::AntiStaff','supported')
 p.replace(c,'        ModuleKind::PacketLogger=>packet_logger::parse_packet_logger(&mut patch,&root),','        ModuleKind::PacketLogger=>packet_logger::parse_packet_logger(&mut patch,&root),\n        ModuleKind::AntiStaff=>anti_staff::parse_anti_staff(&mut patch,&root),','parser')
 p.replace(c,'        SettingChange::PacketLogger(value)=>','        SettingChange::AntiStaff(value)=>return ApplyResult::Runtime(RuntimeRequest::AntiStaff(*value)),\n        SettingChange::PacketLogger(value)=>','request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::PacketLogger=>config::serialize_packet_logger(&b.packet_logger.settings(),&common,format,&describe),','            ModuleKind::PacketLogger=>config::serialize_packet_logger(&b.packet_logger.settings(),&common,format,&describe),\n            ModuleKind::AntiStaff=>config::serialize_anti_staff(b.anti_staff.show(),&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::PacketLogger(setting))=>','                ApplyResult::Runtime(RuntimeRequest::AntiStaff(setting))=>{self.state.behaviors.borrow().anti_staff.set_show(setting.0);}\n                ApplyResult::Runtime(RuntimeRequest::PacketLogger(setting))=>','apply')
 r=E+'runtime.rs'
 p.replace(r,'self.blink_pending(host);self.no_slow_frame(host);','self.anti_staff_frame(host);self.blink_pending(host);self.no_slow_frame(host);','frame')
 p.replace(r,'        self.better_tab(host, event);','        self.better_tab(host, event);\n        self.anti_staff_tab(event);','tab suffix')
 p.replace(r,'            pipeline.add_listener(0, modules.flag_check.clone());','            pipeline.add_listener(0, modules.flag_check.clone());\n            pipeline.add_listener(0,modules.anti_staff.clone());','packets')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'anti-staff-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare()
 if args.apply:p.write()
 print('AntiStaff',len(p.steps),'edits across',len(p.text),'files','applied'if args.apply else'validated')
