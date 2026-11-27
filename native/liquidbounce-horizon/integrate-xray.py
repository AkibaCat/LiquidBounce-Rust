from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod notifier;','pub mod notifier;\npub mod xray;','model')
 p.replace(E+'lib.rs','mod notifier_runtime;','mod notifier_runtime;\n#[cfg(feature="behavior-hooks")]\nmod xray_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub notifier:','    pub xray:crate::xray_runtime::XRayRuntime,\n    pub notifier:','state')
 p.replace(b,'            "Notifier",','            "Notifier",\n            "XRay",','registration')
 p.replace(b,'            notifier:Default::default(),','            notifier:Default::default(),\n            xray:Default::default(),','default')
 c=E+'config.rs'
 p.replace(c,'pub use notifier::{NotifierSetting,serialize_notifier};','pub use notifier::{NotifierSetting,serialize_notifier};\n#[path="config/xray.rs"]\nmod xray;\npub use xray::{XRaySetting,serialize_xray};','codec')
 p.replace(c,'    Notifier=>("Notifier","Misc","notifier"),','    Notifier=>("Notifier","Misc","notifier"),\n    XRay=>("XRay","Render","xRay"),','kind')
 p.replace(c,'    Notifier(NotifierSetting),','    Notifier(NotifierSetting),\n    XRay(XRaySetting),','enum',2)
 p.replace(c,'            | ModuleKind::Notifier','            | ModuleKind::Notifier\n            | ModuleKind::XRay','supported')
 p.replace(c,'        ModuleKind::Notifier=>notifier::parse_notifier(&mut patch,&root),','        ModuleKind::Notifier=>notifier::parse_notifier(&mut patch,&root),\n        ModuleKind::XRay=>xray::parse_xray(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::Notifier(value)=>','        SettingChange::XRay(value)=>return ApplyResult::Runtime(RuntimeRequest::XRay(value.clone())),\n        SettingChange::Notifier(value)=>','request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::Notifier=>config::serialize_notifier(&b.notifier.model.settings,&common,format,&describe),','            ModuleKind::Notifier=>config::serialize_notifier(&b.notifier.model.settings,&common,format,&describe),\n            ModuleKind::XRay=>config::serialize_xray(&b.xray.settings,&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::Notifier(setting))=>','                ApplyResult::Runtime(RuntimeRequest::XRay(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().xray.settings){eprintln!("[LiquidBounce] XRay config: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::Notifier(setting))=>','apply')
 r=E+'runtime.rs'
 p.replace(r,'self.anti_staff_frame(host);','self.xray_frame(host);self.anti_staff_frame(host);','frame')
 p.replace(r,'            event.gamma = gamma;\n        }\n        drop(modules);','            event.gamma = gamma;\n        }else if modules.query_running(&state.registry,"XRay") && modules.xray.settings.full_bright {event.gamma=f32::MAX;}\n        drop(modules);','FullBright gamma priority')
 commands=E+'client_commands_runtime.rs'
 p.replace(commands,'const COMMANDS: [&str; 10] = [','const COMMANDS: [&str; 11] = [','command count')
 p.replace(commands,'    "value",\n];','    "value",\n    "xray",\n];','command list')
 p.replace(commands,'"toggle" | "t" | "bind" | "binds" | "hide" | "value" | "autodisable"','"toggle" | "t" | "bind" | "binds" | "hide" | "value" | "autodisable" | "xray"','save')
 p.replace(commands,'        match root {\n            "toggle" | "t" => {','        match root {\n            "xray"=>return self.xray_command(host,r),\n            "toggle" | "t" => {','dispatch')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'xray-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare()
 if args.apply:p.write()
 print('XRay',len(p.steps),'edits across',len(p.text),'files','applied'if args.apply else'validated')
