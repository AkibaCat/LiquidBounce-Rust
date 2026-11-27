from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod anti_staff;','pub mod anti_staff;\npub mod notifier;\npub mod notifier_format;','model')
 p.replace(E+'lib.rs','mod anti_staff_runtime;','mod anti_staff_runtime;\n#[cfg(feature="behavior-hooks")]\nmod notifier_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub anti_staff:','    pub notifier:crate::notifier_runtime::NotifierRuntime,\n    pub anti_staff:','field')
 p.replace(b,'            "AntiStaff",','            "AntiStaff",\n            "Notifier",','registration')
 p.replace(b,'            anti_staff:Default::default(),','            anti_staff:Default::default(),\n            notifier:Default::default(),','default')
 c=E+'config.rs'
 p.replace(c,'pub use anti_staff::{AntiStaffSetting,serialize_anti_staff};','pub use anti_staff::{AntiStaffSetting,serialize_anti_staff};\n#[path="config/notifier.rs"]\nmod notifier;\npub use notifier::{NotifierSetting,serialize_notifier};','config')
 p.replace(c,'    AntiStaff=>("AntiStaff","Misc","antiStaff"),','    AntiStaff=>("AntiStaff","Misc","antiStaff"),\n    Notifier=>("Notifier","Misc","notifier"),','kind')
 p.replace(c,'    AntiStaff(AntiStaffSetting),','    AntiStaff(AntiStaffSetting),\n    Notifier(NotifierSetting),','enum',2)
 p.replace(c,'            | ModuleKind::AntiStaff','            | ModuleKind::AntiStaff\n            | ModuleKind::Notifier','supported')
 p.replace(c,'        ModuleKind::AntiStaff=>anti_staff::parse_anti_staff(&mut patch,&root),','        ModuleKind::AntiStaff=>anti_staff::parse_anti_staff(&mut patch,&root),\n        ModuleKind::Notifier=>notifier::parse_notifier(&mut patch,&root),','parser')
 p.replace(c,'        SettingChange::AntiStaff(value)=>','        SettingChange::Notifier(value)=>return ApplyResult::Runtime(RuntimeRequest::Notifier(value.clone())),\n        SettingChange::AntiStaff(value)=>','request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::AntiStaff=>config::serialize_anti_staff(b.anti_staff.show(),&common,format,&describe),','            ModuleKind::AntiStaff=>config::serialize_anti_staff(b.anti_staff.show(),&common,format,&describe),\n            ModuleKind::Notifier=>config::serialize_notifier(&b.notifier.model.settings,&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::AntiStaff(setting))=>','                ApplyResult::Runtime(RuntimeRequest::Notifier(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().notifier.model.settings){eprintln!("[LiquidBounce] Notifier configuration: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AntiStaff(setting))=>','apply')
 r=E+'runtime.rs'
 p.replace(r,'        self.packet_logger_tick(host);','        self.packet_logger_tick(host);\n        self.notifier_tick(host);','tick')
 p.replace(r,'    fn client_logoff_spot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientLogoffSpotEvent<\'_>){self.logoff_spot(host,event);}','    fn client_logoff_spot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientLogoffSpotEvent<\'_>){self.logoff_spot(host,event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_notifier(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientNotifierEvent<\'_>){self.notifier_event(host,event);}','events')
 p.replace(r,'        self.state().behaviors.borrow().logoff_spot.clear();','        self.state().behaviors.borrow().logoff_spot.clear();\n        self.state().behaviors.borrow_mut().notifier.model.world_change();','world')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'notifier-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare()
 if args.apply:p.write()
 print('Notifier',len(p.steps),'edits across',len(p.text),'files','applied'if args.apply else'validated')
