"""Publish SwordBlock after KillAura; validate the complete shared transaction first."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod fake_lag;','pub mod fake_lag;\npub mod sword_block;')
 p.edit('lib.rs','mod fake_lag_runtime;','mod fake_lag_runtime;\n#[cfg(feature="behavior-hooks")]\nmod sword_block_runtime;')
 p.edit('behaviors.rs','    pub fake_lag:std::sync::Arc<crate::fake_lag_runtime::FakeLagService>,','    pub fake_lag:std::sync::Arc<crate::fake_lag_runtime::FakeLagService>,\n    pub sword_block:std::sync::Arc<crate::sword_block_runtime::SwordBlockService>,')
 p.edit('behaviors.rs','            "FakeLag",','            "FakeLag",\n            "SwordBlock",')
 p.edit('behaviors.rs','            fake_lag,','            fake_lag,\n            sword_block:Default::default(),')
 p.edit('behaviors.rs','        self.fake_lag.set_running(self.query_running(registry,"FakeLag"));','        self.fake_lag.set_running(self.query_running(registry,"FakeLag"));\n        self.sword_block.set_running(self.query_running(registry,"SwordBlock"));')
 p.edit('runtime.rs','    _packets: std::sync::Arc<crate::packets::PacketPipeline>,','    _packets: std::sync::Arc<crate::packets::PacketPipeline>,\n    #[cfg(feature="behavior-hooks")]\n    pub(crate) sword_block:std::sync::Arc<crate::sword_block_runtime::SwordBlockService>,')
 p.edit('runtime.rs','        let state = State::default();','        let state = State::default();\n        #[cfg(feature="behavior-hooks")]\n        let sword_block=state.behaviors.borrow().sword_block.clone();')
 p.edit('runtime.rs','            _packets: packets,','            _packets: packets,\n            #[cfg(feature="behavior-hooks")]\n            sword_block,')
 a='    fn client_fake_lag_capture(&self,host:&dyn Host,_event:&mut horizon_client::mod_api::ClientFakeLagCaptureEvent)'
 p.edit('runtime.rs',a,'    fn client_sword_block_state(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientSwordBlockStateEvent){event.flags=self.sword_block.flags();}\n    #[cfg(feature="behavior-hooks")]\n'+a)
 p.edit('config.rs','pub use fake_lag::{FakeLagSetting,serialize_fake_lag};','pub use fake_lag::{FakeLagSetting,serialize_fake_lag};\n#[path="config/sword_block.rs"]\nmod sword_block;\npub use sword_block::{SwordBlockSetting,serialize_sword_block};')
 p.edit('config.rs','    FakeLag=>("FakeLag","Combat","fakeLag"),','    FakeLag=>("FakeLag","Combat","fakeLag"),\n    SwordBlock=>("SwordBlock","Combat","swordBlock"),')
 p.edit('config.rs','    FakeLag(FakeLagSetting),','    FakeLag(FakeLagSetting),\n    SwordBlock(SwordBlockSetting),',2)
 p.edit('config.rs','            | ModuleKind::FakeLag','            | ModuleKind::FakeLag\n            | ModuleKind::SwordBlock')
 p.edit('config.rs','        ModuleKind::FakeLag=>fake_lag::parse_fake_lag(&mut patch,&root),','        ModuleKind::FakeLag=>fake_lag::parse_fake_lag(&mut patch,&root),\n        ModuleKind::SwordBlock=>sword_block::parse_sword_block(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::FakeLag(value)=>return ApplyResult::Runtime(RuntimeRequest::FakeLag(value.clone())),','        SettingChange::FakeLag(value)=>return ApplyResult::Runtime(RuntimeRequest::FakeLag(value.clone())),\n        SettingChange::SwordBlock(value)=>return ApplyResult::Runtime(RuntimeRequest::SwordBlock(value.clone())),')
 p.edit('actor.rs','            ModuleKind::FakeLag=>config::serialize_fake_lag(&b.fake_lag,&common,format,&describe),','            ModuleKind::FakeLag=>config::serialize_fake_lag(&b.fake_lag,&common,format,&describe),\n            ModuleKind::SwordBlock=>config::serialize_sword_block(&b.sword_block.settings(),&common,format,&describe),')
 a='                ApplyResult::Runtime(RuntimeRequest::FakeLag(setting))=>{'
 p.edit('actor.rs',a,'                ApplyResult::Runtime(RuntimeRequest::SwordBlock(setting))=>{let service=self.state.behaviors.borrow().sword_block.clone();let mut settings=service.settings();setting.apply(&mut settings);service.configure(settings);}\n'+a)
 p.edit('../../make-behavior-patch.py',"'kill-aura-block-input-patch.py',","'kill-aura-block-input-patch.py', 'sword-block-patch.py',")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');a=parser.parse_args();p=prepare()
 if not a.check:p.commit()
 print('SwordBlock '+('validated'if a.check else'integrated')+f': {len(p.changed)} shared files')
