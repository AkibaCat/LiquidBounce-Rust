"""Publish FakeLag using the existing single shared BlinkManager transaction."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_shoot;','pub mod auto_shoot;\npub mod fake_lag;')
 p.edit('lib.rs','mod auto_shoot_runtime;','mod auto_shoot_runtime;\n#[cfg(feature="behavior-hooks")]\nmod fake_lag_runtime;')
 p.edit('behaviors.rs','    pub blink:std::sync::Arc<crate::blink_runtime::BlinkService>,','    pub blink:std::sync::Arc<crate::blink_runtime::BlinkService>,\n    pub fake_lag:std::sync::Arc<crate::fake_lag_runtime::FakeLagService>,')
 p.edit('behaviors.rs','            "AutoShoot",','            "AutoShoot",\n            "FakeLag",')
 p.edit('behaviors.rs','            blink,','            blink,\n            fake_lag,')
 p.edit('behaviors.rs','        let blink=std::sync::Arc::new(crate::blink_runtime::BlinkService::default());','        let fake_lag=std::sync::Arc::new(crate::fake_lag_runtime::FakeLagService::default());\n        let blink=std::sync::Arc::new(crate::blink_runtime::BlinkService::default());')
 p.edit('behaviors.rs','no_slow.clone(),anti_void.clone()));','no_slow.clone(),anti_void.clone(),fake_lag.clone()));')
 p.edit('behaviors.rs','        self.blink.set_running(self.query_running(registry,"Blink"));','        self.blink.set_running(self.query_running(registry,"Blink"));\n        self.fake_lag.set_running(self.query_running(registry,"FakeLag"));')
 p.edit('config.rs','pub use auto_shoot::{AutoShootSetting,serialize_auto_shoot};','pub use auto_shoot::{AutoShootSetting,serialize_auto_shoot};\n#[path="config/fake_lag.rs"]\nmod fake_lag;\npub use fake_lag::{FakeLagSetting,serialize_fake_lag};')
 p.edit('config.rs','    AutoShoot=>("AutoShoot","Combat","autoShoot"),','    AutoShoot=>("AutoShoot","Combat","autoShoot"),\n    FakeLag=>("FakeLag","Combat","fakeLag"),')
 p.edit('config.rs','    AutoShoot(AutoShootSetting),','    AutoShoot(AutoShootSetting),\n    FakeLag(FakeLagSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoShoot','            | ModuleKind::AutoShoot\n            | ModuleKind::FakeLag')
 p.edit('config.rs','        ModuleKind::AutoShoot=>auto_shoot::parse_auto_shoot(&mut patch,&root),','        ModuleKind::AutoShoot=>auto_shoot::parse_auto_shoot(&mut patch,&root),\n        ModuleKind::FakeLag=>fake_lag::parse_fake_lag(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoShoot(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShoot(value.clone())),','        SettingChange::AutoShoot(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShoot(value.clone())),\n        SettingChange::FakeLag(value)=>return ApplyResult::Runtime(RuntimeRequest::FakeLag(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoShoot=>config::serialize_auto_shoot(&b.auto_shoot,&common,format,&describe),','            ModuleKind::AutoShoot=>config::serialize_auto_shoot(&b.auto_shoot,&common,format,&describe),\n            ModuleKind::FakeLag=>config::serialize_fake_lag(&b.fake_lag,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoShoot(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::FakeLag(setting))=>{self.state.behaviors.borrow().fake_lag.update(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] FakeLag setting: {e}");}});}\n                ApplyResult::Runtime(RuntimeRequest::AutoShoot(setting))=>{')
 p.edit('actor.rs','            ModuleKind::AutoShoot=>Some(','            ModuleKind::FakeLag=>Some(if self.state.behaviors.borrow().fake_lag.settings().constant{"Constant"}else{"Dynamic"}),\n            ModuleKind::AutoShoot=>Some(')
 p.edit('runtime.rs','        self.blink_tick(host);','        self.blink_tick(host);\n        self.fake_lag_tick(host);')
 p.edit('runtime.rs','self.freeze_frame(host);self.criticals_frame(host);','self.freeze_frame(host);self.fake_lag_refresh(host,false);self.criticals_frame(host);')
 a='    fn client_criticals_receive_probe(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientCriticalsReceiveProbeEvent)'
 p.edit('runtime.rs',a,'    fn client_fake_lag_capture(&self,host:&dyn Host,_event:&mut horizon_client::mod_api::ClientFakeLagCaptureEvent){self.fake_lag_refresh(host,false);}\n    #[cfg(feature="behavior-hooks")]\n'+a)
 c='criticals_runtime.rs'
 p.edit(c,'    anti_void:Arc<crate::anti_void_runtime::AntiVoidService>,','    anti_void:Arc<crate::anti_void_runtime::AntiVoidService>,\n    fake_lag:Arc<crate::fake_lag_runtime::FakeLagService>,',2)
 p.edit(c,'            anti_void,','            anti_void,\n            fake_lag,')
 p.edit(c,'moving:Option<bool>, container:bool) -> BlinkAction','moving:Option<bool>, container:bool, fake_packet:liquidbounce_behaviors::fake_lag::PacketKind) -> BlinkAction')
 p.edit(c,'        let inventory=if outgoing','        let fake_lag=self.fake_lag.queues(outgoing,fake_packet,s.blink.queue.front().map(|p|p.millis));\n        let inventory=if outgoing')
 p.edit(c,'if self.blink_module.queues(outgoing)||','if fake_lag||self.blink_module.queues(outgoing)||')
 p.edit(c,'self.action(&mut s, outgoing, false,None,false)','self.action(&mut s, outgoing, false,None,false,Default::default())')
 p.edit(c,'horizon_client::inventory_move::container_packet(event.packet))','horizon_client::inventory_move::container_packet(event.packet),crate::fake_lag_runtime::packet(event.packet))')
 p.edit(c,'self.0.action(&mut s, false, false,None,false)','self.0.action(&mut s, false, false,None,false,Default::default())')
 p.edit('../../make-behavior-patch.py',"'auto-shoot-patch.py',","'auto-shoot-patch.py', 'fake-lag-patch.py',")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');a=parser.parse_args();p=prepare()
 if not a.check:p.commit()
 print('FakeLag '+('validated'if a.check else'integrated')+f': {len(p.changed)} shared files')
