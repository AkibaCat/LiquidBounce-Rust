#!/usr/bin/env python3
"""Transactional FastExp engine/native/config integration."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan'];E='native/liquidbounce-horizon/engine/src/'
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod fast_use;','pub mod fast_use;\npub mod fast_exp;','model')
 p.replace(E+'lib.rs','mod fast_use_runtime;','mod fast_use_runtime;\n#[cfg(feature="behavior-hooks")]\nmod fast_exp_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub fast_use: liquidbounce_behaviors::fast_use::FastUse,','    pub fast_use: liquidbounce_behaviors::fast_use::FastUse,\n    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,','state')
 p.replace(E+'behaviors.rs','            "FastUse",','            "FastUse",\n            "FastExp",','registry')
 p.replace(E+'behaviors.rs','            fast_use: Default::default(),','            fast_use: Default::default(),\n            fast_exp:Default::default(),','default')
 p.replace(E+'behaviors.rs','"QuickPerspectiveSwap" | "FreeLook" | "Anchor"','"QuickPerspectiveSwap" | "FreeLook" | "Anchor" | "FastExp"','hold/disableOnQuit')
 anchor='            ModuleCallback::OnEnabled(id) if self.query_ids.get("AntiCheatDetect") == Some(&id) => {'
 p.replace(E+'behaviors.rs',anchor,'''            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.disabled(),
            ModuleCallback::CancelSequences(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.cancel(),
'''+anchor,'lifecycle')
 anchor='            crate::no_web_runtime::ROTATION_PROVIDER => self.query_running(registry,"NoWeb"),'
 p.replace(E+'behaviors.rs',anchor,'            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),\n'+anchor,'rotation provider')
 p.replace(E+'config.rs','pub use fast_use::{FastUseSetting, serialize_fast_use};','pub use fast_use::{FastUseSetting, serialize_fast_use};\n#[path="config/fast_exp.rs"]\nmod fast_exp;\npub use fast_exp::{FastExpSetting,serialize_fast_exp};','codec')
 p.replace(E+'config.rs','    FastUse => ("FastUse", "Player", "fastUse"),','    FastUse => ("FastUse", "Player", "fastUse"),\n    FastExp=>("FastExp","Player","fastExp"),','kind')
 p.replace(E+'config.rs','    FastUse(FastUseSetting),','    FastUse(FastUseSetting),\n    FastExp(FastExpSetting),','requests',count=2)
 p.replace(E+'config.rs','            | ModuleKind::FastUse','            | ModuleKind::FastUse\n            | ModuleKind::FastExp','dedicated')
 anchor='        ModuleKind::FastUse => fast_use::parse_fast_use(&mut patch, &root),'
 p.replace(E+'config.rs',anchor,anchor+'\n        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),','parse')
 p.replace(E+'config.rs','        SettingChange::FastUse(value) => {','        SettingChange::FastExp(value)=>return ApplyResult::Runtime(RuntimeRequest::FastExp(value.clone())),\n        SettingChange::FastUse(value) => {','request')
 anchor='            ModuleKind::FastUse => config::serialize_fast_use(&b.fast_use,&common,format,&describe),'
 p.replace(E+'actor.rs',anchor,'            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),\n'+anchor,'serialize')
 p.replace(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::FastUse(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().fast_exp){eprintln!("[LiquidBounce] FastExp config leaf skipped: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::FastUse(setting)) => {''','setting')
 p.replace(E+'coroutine_ticker.rs','    FastUse,','    FastUse,\n    FastExp,','FIFO owner')
 anchor='                WaitOwner::VehicleControl=>self.vehicle_control_callback(host,in_game,task.callback),'
 p.replace(E+'runtime.rs',anchor,anchor+'\n                WaitOwner::FastExp=>self.fast_exp_callback(host,in_game,task.callback),','FIFO callback')
 p.replace(E+'runtime.rs','        self.vehicle_control_tick(host);','        self.fast_exp_tick(host);\n        self.vehicle_control_tick(host);','tick')
 p.replace(E+'runtime.rs','            let no_web_running = modules.query_running(&state.registry,"NoWeb");','            let no_web_running = modules.query_running(&state.registry,"NoWeb");\n            let fast_exp_running=modules.query_running(&state.registry,"FastExp");','provider active')
 p.replace(E+'runtime.rs','                        crate::no_web_runtime::ROTATION_PROVIDER => no_web_running,','                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,\n                        crate::no_web_runtime::ROTATION_PROVIDER => no_web_running,','provider dispatch')
 anchor='                        } else if let Some(mode) = crate::no_web_runtime::NoWebRuntime::processor_mode(processor) {'
 p.replace(E+'runtime.rs',anchor,'                        } else if let Some(mode)=crate::fast_exp_runtime::FastExpRuntime::processor_mode(processor){\n                            (&mut modules.fast_exp.rotations,mode)\n'+anchor,'processor')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'fast-exp-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');a=a.parse_args();p=prepare()
 if a.apply:p.write()
 print(f'FastExp {"integrated"if a.apply else"validated"}: {len(p.steps)} changes/{len(p.text)} shared files')
