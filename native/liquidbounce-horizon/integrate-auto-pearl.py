#!/usr/bin/env python3
"""Publish AutoPearl wiring only after validating every shared anchor transactionally."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_buff;','pub mod auto_buff;\npub mod auto_pearl;')
 p.edit('lib.rs','mod auto_buff_runtime;','mod auto_buff_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_pearl_runtime;')
 p.edit('behaviors.rs','    pub auto_buff:crate::auto_buff_runtime::AutoBuffRuntime,','    pub auto_buff:crate::auto_buff_runtime::AutoBuffRuntime,\n    pub auto_pearl:crate::auto_pearl_runtime::AutoPearlRuntime,')
 p.edit('behaviors.rs','            "AutoBuff",','            "AutoBuff",\n            "AutoPearl",')
 p.edit('behaviors.rs','            auto_buff:Default::default(),','            auto_buff:Default::default(),\n            auto_pearl:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBuff")==Some(&id)=>{','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoPearl")==Some(&id)=>self.auto_pearl.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBuff")==Some(&id)=>{')
 p.edit('behaviors.rs','            crate::auto_buff_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBuff"),','            crate::auto_buff_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBuff"),\n            crate::auto_pearl_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoPearl"),')
 p.edit('config.rs','pub use auto_buff::{AutoBuffSetting,serialize_auto_buff};','pub use auto_buff::{AutoBuffSetting,serialize_auto_buff};\n#[path="config/auto_pearl.rs"]\nmod auto_pearl;\npub use auto_pearl::{AutoPearlSetting,serialize_auto_pearl};')
 p.edit('config.rs','    AutoBuff=>("AutoBuff","Player","autoBuff"),','    AutoBuff=>("AutoBuff","Player","autoBuff"),\n    AutoPearl=>("AutoPearl","Combat","autoPearl"),')
 p.edit('config.rs','"AutoPot"|"AutoGapple"|"AutoSoup"=>Some(Self::AutoBuff),','"AutoPot"|"AutoGapple"|"AutoSoup"=>Some(Self::AutoBuff), "PearlFollower"|"PearlTarget"=>Some(Self::AutoPearl),')
 p.edit('config.rs','    AutoBuff(AutoBuffSetting),','    AutoBuff(AutoBuffSetting),\n    AutoPearl(AutoPearlSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoBuff','            | ModuleKind::AutoBuff\n            | ModuleKind::AutoPearl')
 p.edit('config.rs','        ModuleKind::AutoBuff=>auto_buff::parse_auto_buff(&mut patch,&root),','        ModuleKind::AutoBuff=>auto_buff::parse_auto_buff(&mut patch,&root),\n        ModuleKind::AutoPearl=>auto_pearl::parse_auto_pearl(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoBuff(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuff(value.clone())),','        SettingChange::AutoBuff(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuff(value.clone())),\n        SettingChange::AutoPearl(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoPearl(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoBuff=>config::serialize_auto_buff(&b.auto_buff,&common,format,&describe),','            ModuleKind::AutoBuff=>config::serialize_auto_buff(&b.auto_buff,&common,format,&describe),\n            ModuleKind::AutoPearl=>config::serialize_auto_pearl(&b.auto_pearl,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoBuff(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::AutoPearl(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_pearl){eprintln!("[LiquidBounce] AutoPearl config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoBuff(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.auto_buff.clear();','        modules.auto_buff.clear();\n        modules.auto_pearl.clear();')
 p.edit('runtime.rs','        self.auto_buff_tick(host);','        self.auto_pearl_tick(host);\n        self.auto_buff_tick(host);')

 p.edit('runtime.rs','            let auto_buff_running=modules.query_running(&state.registry,"AutoBuff");','            let auto_buff_running=modules.query_running(&state.registry,"AutoBuff");\n            let auto_pearl_running=modules.query_running(&state.registry,"AutoPearl");')
 p.edit('runtime.rs','                        crate::auto_buff_runtime::ROTATION_PROVIDER=>auto_buff_running,','                        crate::auto_buff_runtime::ROTATION_PROVIDER=>auto_buff_running,\n                        crate::auto_pearl_runtime::ROTATION_PROVIDER=>auto_pearl_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_buff_runtime::AutoBuffRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::auto_pearl_runtime::AutoPearlRuntime::processor_mode(processor){\n                            (&mut modules.auto_pearl.rotations,mode)\n                        } else if let Some(mode)=crate::auto_buff_runtime::AutoBuffRuntime::processor_mode(processor){')
 p.edit('coroutine_ticker.rs','    AutoBuff,','    AutoBuff,\n    AutoPearl,')
 p.edit('runtime.rs','                WaitOwner::AutoBuff=>self.auto_buff_callback(host,in_game,task.callback),','                WaitOwner::AutoBuff=>self.auto_buff_callback(host,in_game,task.callback),\n                WaitOwner::AutoPearl=>self.auto_pearl_callback(host,in_game,task.callback),')
 p.edit('runtime.rs','        self.auto_deposit_prepare(host);','        self.auto_pearl_prepare(host);\n        self.auto_deposit_prepare(host);')
 p.edit('../../make-behavior-patch.py',"'auto-bow-patch.py',","'auto-bow-patch.py', 'auto-pearl-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoPearl '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
