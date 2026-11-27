#!/usr/bin/env python3
"""Transactional EasyPearl engine wiring; never writes staging or canonical locales."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_pearl;','pub mod auto_pearl;\npub mod easy_pearl;')
 p.edit('lib.rs','mod auto_pearl_runtime;','mod auto_pearl_runtime;\n#[cfg(feature="behavior-hooks")]\nmod easy_pearl_runtime;')
 p.edit('behaviors.rs','    pub auto_pearl:crate::auto_pearl_runtime::AutoPearlRuntime,','    pub auto_pearl:crate::auto_pearl_runtime::AutoPearlRuntime,\n    pub easy_pearl:crate::easy_pearl_runtime::EasyPearlRuntime,')
 p.edit('behaviors.rs','            "AutoPearl",','            "AutoPearl",\n            "EasyPearl",')
 p.edit('behaviors.rs','            auto_pearl:Default::default(),','            auto_pearl:Default::default(),\n            easy_pearl:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoPearl")==Some(&id)=>self.auto_pearl.clear(),','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("EasyPearl")==Some(&id)=>self.easy_pearl.behavior.disabled(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoPearl")==Some(&id)=>self.auto_pearl.clear(),')
 p.edit('behaviors.rs','            crate::auto_pearl_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoPearl"),','            crate::auto_pearl_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoPearl"),\n            crate::easy_pearl_runtime::ROTATION_PROVIDER=>self.query_running(registry,"EasyPearl"),')
 p.edit('config.rs','pub use auto_pearl::{AutoPearlSetting,serialize_auto_pearl};','pub use auto_pearl::{AutoPearlSetting,serialize_auto_pearl};\n#[path="config/easy_pearl.rs"]\nmod easy_pearl;\npub use easy_pearl::{EasyPearlSetting,serialize_easy_pearl};')
 p.edit('config.rs','    AutoPearl=>("AutoPearl","Combat","autoPearl"),','    AutoPearl=>("AutoPearl","Combat","autoPearl"),\n    EasyPearl=>("EasyPearl","Misc","easyPearl"),')
 p.edit('config.rs','"PearlFollower"|"PearlTarget"=>Some(Self::AutoPearl),','"PearlFollower"|"PearlTarget"=>Some(Self::AutoPearl), "PearlHelper"|"PearlAssist"|"PearlTP"=>Some(Self::EasyPearl),')
 p.edit('config.rs','    AutoPearl(AutoPearlSetting),','    AutoPearl(AutoPearlSetting),\n    EasyPearl(EasyPearlSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoPearl','            | ModuleKind::AutoPearl\n            | ModuleKind::EasyPearl')
 p.edit('config.rs','        ModuleKind::AutoPearl=>auto_pearl::parse_auto_pearl(&mut patch,&root),','        ModuleKind::AutoPearl=>auto_pearl::parse_auto_pearl(&mut patch,&root),\n        ModuleKind::EasyPearl=>easy_pearl::parse_easy_pearl(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoPearl(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoPearl(value.clone())),','        SettingChange::AutoPearl(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoPearl(value.clone())),\n        SettingChange::EasyPearl(value)=>return ApplyResult::Runtime(RuntimeRequest::EasyPearl(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoPearl=>config::serialize_auto_pearl(&b.auto_pearl,&common,format,&describe),','            ModuleKind::AutoPearl=>config::serialize_auto_pearl(&b.auto_pearl,&common,format,&describe),\n            ModuleKind::EasyPearl=>config::serialize_easy_pearl(&b.easy_pearl,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoPearl(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::EasyPearl(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().easy_pearl){eprintln!("[LiquidBounce] EasyPearl config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoPearl(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.auto_pearl.clear();','        modules.auto_pearl.clear();\n        modules.easy_pearl.behavior.disabled();')
 p.edit('runtime.rs','        self.auto_pearl_tick(host);','        self.auto_pearl_tick(host);\n        self.easy_pearl_tick(host);')
 p.edit('runtime.rs','            let auto_pearl_running=modules.query_running(&state.registry,"AutoPearl");','            let auto_pearl_running=modules.query_running(&state.registry,"AutoPearl");\n            let easy_pearl_running=modules.query_running(&state.registry,"EasyPearl");')
 p.edit('runtime.rs','                        crate::auto_pearl_runtime::ROTATION_PROVIDER=>auto_pearl_running,','                        crate::auto_pearl_runtime::ROTATION_PROVIDER=>auto_pearl_running,\n                        crate::easy_pearl_runtime::ROTATION_PROVIDER=>easy_pearl_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_pearl_runtime::AutoPearlRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::easy_pearl_runtime::EasyPearlRuntime::processor_mode(processor){\n                            (&mut modules.easy_pearl.rotations,mode)\n                        } else if let Some(mode)=crate::auto_pearl_runtime::AutoPearlRuntime::processor_mode(processor){')
 p.edit('runtime.rs','        self.auto_pearl_prepare(host);','        self.auto_pearl_prepare(host);\n        self.easy_pearl_prepare(host);')
 anchor='    fn client_target_renderer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTargetRendererEvent<\'_>){self.target_renderer(host,event);}'
 p.edit('runtime.rs',anchor,anchor+'\n    #[cfg(feature="behavior-hooks")]\n    fn client_easy_pearl(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientEasyPearlEvent<\'_>){self.easy_pearl(host,event);}')
 p.edit('../../make-behavior-patch.py',"'auto-pearl-patch.py',","'auto-pearl-patch.py', 'easy-pearl-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('EasyPearl '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
