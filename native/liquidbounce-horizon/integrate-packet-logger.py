from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod flag_check;','pub mod flag_check;\npub mod packet_logger;','model')
 p.replace(E+'lib.rs','mod flag_check_runtime;','mod flag_check_runtime;\n#[cfg(feature="behavior-hooks")]\nmod packet_logger_runtime;\n#[cfg(feature="behavior-hooks")]\nmod packet_logger_fields;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub flag_check:std::sync::Arc<crate::flag_check_runtime::FlagCheckService>,','    pub flag_check:std::sync::Arc<crate::flag_check_runtime::FlagCheckService>,\n    pub packet_logger:std::sync::Arc<crate::packet_logger_runtime::PacketLoggerService>,','field')
 p.replace(b,'            "FlagCheck",','            "FlagCheck",\n            "PacketLogger",','register')
 p.replace(b,'            flag_check:Default::default(),','            flag_check:Default::default(),\n            packet_logger:Default::default(),','default')
 p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.packet_logger.configure(self.query_ids.get("PacketLogger").is_some_and(|id|registry.enabled_without_world(*id)));','lifecycle')
 c=E+'config.rs'
 p.replace(c,'pub use flag_check::{FlagCheckSetting,serialize_flag_check};','pub use flag_check::{FlagCheckSetting,serialize_flag_check};\n#[path="config/packet_logger.rs"]\nmod packet_logger;\npub use packet_logger::{PacketLoggerSetting,serialize_packet_logger};','config')
 p.replace(c,'    FlagCheck=>("FlagCheck","Misc","flagCheck"),','    FlagCheck=>("FlagCheck","Misc","flagCheck"),\n    PacketLogger=>("PacketLogger","Misc","packetLogger"),','kind')
 p.replace(c,'    FlagCheck(FlagCheckSetting),','    FlagCheck(FlagCheckSetting),\n    PacketLogger(PacketLoggerSetting),','enum',2)
 p.replace(c,'            | ModuleKind::FlagCheck','            | ModuleKind::FlagCheck\n            | ModuleKind::PacketLogger','dedicated')
 p.replace(c,'        ModuleKind::FlagCheck=>flag_check::parse_flag_check(&mut patch,&root),','        ModuleKind::FlagCheck=>flag_check::parse_flag_check(&mut patch,&root),\n        ModuleKind::PacketLogger=>packet_logger::parse_packet_logger(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::FlagCheck(value)=>','        SettingChange::PacketLogger(value)=>return ApplyResult::Runtime(RuntimeRequest::PacketLogger(value.clone())),\n        SettingChange::FlagCheck(value)=>','request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::FlagCheck=>config::serialize_flag_check(&b.flag_check.settings(),&common,format,&describe),','            ModuleKind::FlagCheck=>config::serialize_flag_check(&b.flag_check.settings(),&common,format,&describe),\n            ModuleKind::PacketLogger=>config::serialize_packet_logger(&b.packet_logger.settings(),&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::FlagCheck(setting))=>','                ApplyResult::Runtime(RuntimeRequest::PacketLogger(setting))=>{self.state.behaviors.borrow().packet_logger.edit(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] PacketLogger config: {e}");}});}\n                ApplyResult::Runtime(RuntimeRequest::FlagCheck(setting))=>','apply')
 r=E+'runtime.rs'
 p.replace(r,'                rotation: Some(rotations),','                rotation: Some(rotations),\n                packet_logger:Some(modules.packet_logger.clone()),','silent sends')
 p.replace(r,'            pipeline.add_listener(0, modules.flag_check.clone());','            pipeline.add_listener(0, modules.flag_check.clone());\n            pipeline.add_listener(-1000,modules.packet_logger.clone());','packets')
 p.replace(r,'        self.flag_check_tick(host);','        self.flag_check_tick(host);\n        self.packet_logger_tick(host);','chat drain')
 packets=E+'packets.rs'
 p.replace(packets,'    fn incoming(&self, _event: &mut PacketEvent<\'_, Clientbound>) {}','    fn incoming(&self, _event: &mut PacketEvent<\'_, Clientbound>) {}\n    fn incoming_final(&self,_context:PacketContext,_packet:&Clientbound,_cancelled:bool) {}\n    fn outgoing_final(&self,_context:PacketContext,_packet:&Serverbound,_cancelled:bool) {}','final observers')
 p.replace(packets,'        if !event.is_cancelled(){for handler in handlers{handler.listener.incoming_accepted(event.context,event.packet);}}','        for handler in &handlers{handler.listener.incoming_final(event.context,event.packet,event.is_cancelled());}\n        if !event.is_cancelled(){for handler in handlers{handler.listener.incoming_accepted(event.context,event.packet);}}','incoming final cancelled state')
 p.replace(packets,'                if !event.is_cancelled() {\n                    for handler in &handlers {','                for handler in &handlers{handler.listener.outgoing_final(event.context,event.packet,event.is_cancelled());}\n                if !event.is_cancelled() {\n                    for handler in &handlers {','outgoing final cancelled state')
 browser=E+'browser.rs'
 p.replace(browser,'    hooks.state().config_root = config.config_directory.clone();','    hooks.state().behaviors.borrow().packet_logger.root(config.config_directory.clone());\n    hooks.state().config_root = config.config_directory.clone();','real config directory')
 p.replace(browser,'        if request.method==Method::GET && request.path=="/api/v1/client/registry/menu" {','''        if request.method==Method::GET && matches!(request.path.as_str(),"/api/v1/client/registry/c2s_packet"|"/api/v1/client/registry/s2c_packet"){
            let ids=if request.path.ends_with("c2s_packet"){crate::packet_logger_fields::C2S_IDS}else{crate::packet_logger_fields::S2C_IDS};
            let entries:serde_json::Map<_,_>=ids.iter().map(|id|(id.to_string(),json!({"name":id,"icon":null}))).collect();
            return ApiResponse::json(Value::Object(entries));
        }
        if request.method==Method::GET && request.path=="/api/v1/client/registry/menu" {''','packet registry API')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'packet-logger-patch.py', 'step-patch.py'):",'native chat')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare()
 if args.apply:p.write()
 print('PacketLogger',len(p.steps),'edits /',len(p.text),'files', 'applied'if args.apply else'validated')
