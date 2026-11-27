#!/usr/bin/env python3
"""Publish AutoBow wiring only after validating every shared anchor transactionally."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_buff;','pub mod auto_buff;\npub mod auto_bow;\npub mod auto_bow_math;')
 p.edit('lib.rs','mod auto_buff_runtime;','mod auto_buff_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_bow_runtime;')
 p.edit('behaviors.rs','    pub auto_buff:crate::auto_buff_runtime::AutoBuffRuntime,','    pub auto_buff:crate::auto_buff_runtime::AutoBuffRuntime,\n    pub auto_bow:crate::auto_bow_runtime::AutoBowRuntime,')
 p.edit('behaviors.rs','            "AutoBuff",','            "AutoBuff",\n            "AutoBow",')
 p.edit('behaviors.rs','            auto_buff:Default::default(),','            auto_buff:Default::default(),\n            auto_bow:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBuff")==Some(&id)=>{','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBuff")==Some(&id)=>{')
 p.edit('behaviors.rs','            crate::auto_buff_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBuff"),','            crate::auto_buff_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBuff"),\n            crate::auto_bow_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBow"),')
 p.edit('config.rs','pub use auto_buff::{AutoBuffSetting,serialize_auto_buff};','pub use auto_buff::{AutoBuffSetting,serialize_auto_buff};\n#[path="config/auto_bow.rs"]\nmod auto_bow;\npub use auto_bow::{AutoBowSetting,serialize_auto_bow};')
 p.edit('config.rs','    AutoBuff=>("AutoBuff","Player","autoBuff"),','    AutoBuff=>("AutoBuff","Player","autoBuff"),\n    AutoBow=>("AutoBow","Combat","autoBow"),')
 p.edit('config.rs','"AutoPot"|"AutoGapple"|"AutoSoup"=>Some(Self::AutoBuff),','"AutoPot"|"AutoGapple"|"AutoSoup"=>Some(Self::AutoBuff), "BowAssist"|"BowAimbot"=>Some(Self::AutoBow),')
 p.edit('config.rs','    AutoBuff(AutoBuffSetting),','    AutoBuff(AutoBuffSetting),\n    AutoBow(AutoBowSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoBuff','            | ModuleKind::AutoBuff\n            | ModuleKind::AutoBow')
 p.edit('config.rs','        ModuleKind::AutoBuff=>auto_buff::parse_auto_buff(&mut patch,&root),','        ModuleKind::AutoBuff=>auto_buff::parse_auto_buff(&mut patch,&root),\n        ModuleKind::AutoBow=>auto_bow::parse_auto_bow(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoBuff(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuff(value.clone())),','        SettingChange::AutoBuff(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuff(value.clone())),\n        SettingChange::AutoBow(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBow(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoBuff=>config::serialize_auto_buff(&b.auto_buff,&common,format,&describe),','            ModuleKind::AutoBuff=>config::serialize_auto_buff(&b.auto_buff,&common,format,&describe),\n            ModuleKind::AutoBow=>config::serialize_auto_bow(&b.auto_bow,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoBuff(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_bow){eprintln!("[LiquidBounce] AutoBow config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoBuff(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.auto_buff.clear();','        modules.auto_buff.clear();\n        modules.auto_bow.clear();')
 # AutoBow is registered before AutoBuff and SmartEat in pristine ModuleManager.
 p.edit('runtime.rs','        self.auto_buff_tick(host);','        self.auto_bow_tick(host);\n        self.auto_buff_tick(host);')
 p.edit('runtime.rs','        self.smart_eat_use_key(&mut event.pressed);','        self.auto_bow_use_key(&mut event.pressed);\n        self.smart_eat_use_key(&mut event.pressed);')
 p.edit('runtime.rs','            let auto_buff_running=modules.query_running(&state.registry,"AutoBuff");','            let auto_buff_running=modules.query_running(&state.registry,"AutoBuff");\n            let auto_bow_running=modules.query_running(&state.registry,"AutoBow");')
 p.edit('runtime.rs','                        crate::auto_buff_runtime::ROTATION_PROVIDER=>auto_buff_running,','                        crate::auto_buff_runtime::ROTATION_PROVIDER=>auto_buff_running,\n                        crate::auto_bow_runtime::ROTATION_PROVIDER=>auto_bow_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_buff_runtime::AutoBuffRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::auto_bow_runtime::AutoBowRuntime::processor_mode(processor){\n                            (&mut modules.auto_bow.rotations,mode)\n                        } else if let Some(mode)=crate::auto_buff_runtime::AutoBuffRuntime::processor_mode(processor){')
 p.edit('../../make-behavior-patch.py',"'auto-buff-patch.py',","'auto-buff-patch.py', 'auto-bow-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoBow '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
