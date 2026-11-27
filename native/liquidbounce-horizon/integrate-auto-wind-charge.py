#!/usr/bin/env python3
"""Validate all AutoWindCharge seams before publishing shared engine wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod fast_exp;','pub mod fast_exp;\npub mod auto_wind_charge;')
 p.edit('lib.rs','mod fast_exp_runtime;','mod fast_exp_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_wind_charge_runtime;')
 p.edit('behaviors.rs','    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,','    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,\n    pub auto_wind_charge:crate::auto_wind_charge_runtime::AutoWindChargeRuntime,')
 p.edit('behaviors.rs','            "FastExp",','            "FastExp",\n            "AutoWindCharge",')
 p.edit('behaviors.rs','            fast_exp:Default::default(),','            fast_exp:Default::default(),\n            auto_wind_charge:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.disabled(),','            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoWindCharge")==Some(&id)=>self.auto_wind_charge.clear(),\n            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoWindCharge")==Some(&id)=>self.auto_wind_charge.clear(),\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.disabled(),')
 p.edit('behaviors.rs','            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),','            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),\n            crate::auto_wind_charge_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoWindCharge"),')
 p.edit('config.rs','pub use fast_exp::{FastExpSetting,serialize_fast_exp};','pub use fast_exp::{FastExpSetting,serialize_fast_exp};\n#[path="config/auto_wind_charge.rs"]\nmod auto_wind_charge;\npub use auto_wind_charge::{AutoWindChargeSetting,serialize_auto_wind_charge};')
 p.edit('config.rs','    FastExp=>("FastExp","Player","fastExp"),','    FastExp=>("FastExp","Player","fastExp"),\n    AutoWindCharge=>("AutoWindCharge","Player","autoWindCharge"),')
 p.edit('config.rs','    FastExp(FastExpSetting),','    FastExp(FastExpSetting),\n    AutoWindCharge(AutoWindChargeSetting),',2)
 p.edit('config.rs','            | ModuleKind::FastExp','            | ModuleKind::FastExp\n            | ModuleKind::AutoWindCharge')
 p.edit('config.rs','        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),','        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),\n        ModuleKind::AutoWindCharge=>auto_wind_charge::parse_auto_wind_charge(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::FastExp(value)=>return ApplyResult::Runtime(RuntimeRequest::FastExp(value.clone())),','        SettingChange::FastExp(value)=>return ApplyResult::Runtime(RuntimeRequest::FastExp(value.clone())),\n        SettingChange::AutoWindCharge(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoWindCharge(value.clone())),')
 p.edit('actor.rs','            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),','            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),\n            ModuleKind::AutoWindCharge=>config::serialize_auto_wind_charge(&b.auto_wind_charge,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::AutoWindCharge(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_wind_charge){eprintln!("[LiquidBounce] AutoWindCharge config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.inventory_cleaner.clear();','        modules.inventory_cleaner.clear();\n        modules.auto_wind_charge.clear();')
 p.edit('runtime.rs','        self.fast_exp_tick(host);','        self.fast_exp_tick(host);\n        self.auto_wind_charge_tick(host);')
 p.edit('runtime.rs','            let fast_exp_running=modules.query_running(&state.registry,"FastExp");','            let fast_exp_running=modules.query_running(&state.registry,"FastExp");\n            let auto_wind_charge_running=modules.query_running(&state.registry,"AutoWindCharge");')
 p.edit('runtime.rs','                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,','                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,\n                        crate::auto_wind_charge_runtime::ROTATION_PROVIDER=>auto_wind_charge_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::fast_exp_runtime::FastExpRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::auto_wind_charge_runtime::AutoWindChargeRuntime::processor_mode(processor){\n                            (&mut modules.auto_wind_charge.rotations,mode)\n                        } else if let Some(mode)=crate::fast_exp_runtime::FastExpRuntime::processor_mode(processor){')
 p.edit('runtime.rs','        let no_block_running = modules.query_running(&state.registry, "NoBlockInteract");','        let auto_wind_charge_running=modules.query_running(&state.registry,"AutoWindCharge");\n        modules.auto_wind_charge.behavior.movement_input(auto_wind_charge_running,&mut event.jump);\n        let no_block_running = modules.query_running(&state.registry, "NoBlockInteract");')
 p.edit('runtime.rs','        if auto_dodge_jump{event.jump=true;}','        if auto_dodge_jump{event.jump=true;}\n        self.state().behaviors.borrow_mut().auto_wind_charge.record_input(event);')
 p.edit('../../make-behavior-patch.py',"'fast-exp-patch.py',","'fast-exp-patch.py', 'auto-wind-charge-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoWindCharge '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
