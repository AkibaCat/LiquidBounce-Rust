#!/usr/bin/env python3
"""Validate every AutoBuff shared seam before a transactional publish."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod fast_exp;','pub mod fast_exp;\npub mod auto_buff;')
 p.edit('lib.rs','mod fast_exp_runtime;','mod fast_exp_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_buff_runtime;')
 p.edit('behaviors.rs','    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,','    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,\n    pub auto_buff:crate::auto_buff_runtime::AutoBuffRuntime,')
 p.edit('behaviors.rs','            "FastExp",','            "FastExp",\n            "AutoBuff",')
 p.edit('behaviors.rs','            fast_exp:Default::default(),','            fast_exp:Default::default(),\n            auto_buff:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.disabled(),','''            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBuff")==Some(&id)=>{
                for owner in [crate::auto_buff_runtime::REFILL_OWNER,crate::auto_buff_runtime::STACK_OWNER]{self.inventory.manager.cancel_pending(owner);}
                for plan in [self.auto_buff.refill_pending,self.auto_buff.stack_pending].into_iter().flatten(){self.inventory.manager.end_plan(plan);}
                self.pending_slot_resets.push(crate::auto_buff_runtime::REQUESTER);self.auto_buff.clear();
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>self.fast_exp.behavior.disabled(),''')
 p.edit('behaviors.rs','            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),','            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),\n            crate::auto_buff_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBuff"),')
 p.edit('config.rs','pub use fast_exp::{FastExpSetting,serialize_fast_exp};','pub use fast_exp::{FastExpSetting,serialize_fast_exp};\n#[path="config/auto_buff.rs"]\nmod auto_buff;\npub use auto_buff::{AutoBuffSetting,serialize_auto_buff};')
 p.edit('config.rs','    FastExp=>("FastExp","Player","fastExp"),','    FastExp=>("FastExp","Player","fastExp"),\n    AutoBuff=>("AutoBuff","Player","autoBuff"),')
 p.edit('config.rs','"ContainerStealer"=>Some(Self::ChestStealer),','"ContainerStealer"=>Some(Self::ChestStealer), "AutoPot"|"AutoGapple"|"AutoSoup"=>Some(Self::AutoBuff),')
 p.edit('config.rs','    FastExp(FastExpSetting),','    FastExp(FastExpSetting),\n    AutoBuff(AutoBuffSetting),',2)
 p.edit('config.rs','            | ModuleKind::FastExp','            | ModuleKind::FastExp\n            | ModuleKind::AutoBuff')
 p.edit('config.rs','        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),','        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),\n        ModuleKind::AutoBuff=>auto_buff::parse_auto_buff(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::FastExp(value)=>return ApplyResult::Runtime(RuntimeRequest::FastExp(value.clone())),','        SettingChange::FastExp(value)=>return ApplyResult::Runtime(RuntimeRequest::FastExp(value.clone())),\n        SettingChange::AutoBuff(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBuff(value.clone())),')
 p.edit('actor.rs','            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),','            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),\n            ModuleKind::AutoBuff=>config::serialize_auto_buff(&b.auto_buff,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::AutoBuff(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_buff){eprintln!("[LiquidBounce] AutoBuff config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.inventory_cleaner.clear();','        modules.inventory_cleaner.clear();\n        modules.auto_buff.clear();')
 p.edit('inventory_runtime.rs','        self.inventory_cleaner_plan(host,&snapshot.context);','        self.inventory_cleaner_plan(host,&snapshot.context);\n        self.auto_buff_plan(host,&snapshot.context);')
 p.edit('inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.extend(modules.auto_buff.owners(modules.query_running(&state.registry,"AutoBuff")));\n                modules.inventory.observe_context(&context);')
 p.edit('inventory_runtime.rs','                    self.replenish_outcome(outcome,&context);','                    self.replenish_outcome(outcome,&context);\n                    self.auto_buff_outcome(host,outcome,&context);')
 p.edit('coroutine_ticker.rs','    FastExp,','    FastExp,\n    AutoBuff,')
 p.edit('runtime.rs','                WaitOwner::FastExp=>self.fast_exp_callback(host,in_game,task.callback),','                WaitOwner::FastExp=>self.fast_exp_callback(host,in_game,task.callback),\n                WaitOwner::AutoBuff=>self.auto_buff_callback(host,in_game,task.callback),')
 p.edit('runtime.rs','        self.crosshair_first_tick(host); // FIRST_PRIORITY GameTick','        self.crosshair_first_tick(host); // FIRST_PRIORITY GameTick\n        self.auto_buff_first_tick();')
 p.edit('runtime.rs','        self.fast_exp_tick(host);','        self.fast_exp_tick(host);\n        self.auto_buff_tick(host);')
 p.edit('runtime.rs','        self.smart_eat_use_key(&mut event.pressed);','        self.smart_eat_use_key(&mut event.pressed);\n        self.auto_buff_use_key(&mut event.pressed);')
 p.edit('runtime.rs','        if auto_dodge_jump{event.jump=true;}','        if auto_dodge_jump{event.jump=true;}\n        self.state().behaviors.borrow_mut().auto_buff.record_input(event);')
 p.edit('runtime.rs','            let fast_exp_running=modules.query_running(&state.registry,"FastExp");','            let fast_exp_running=modules.query_running(&state.registry,"FastExp");\n            let auto_buff_running=modules.query_running(&state.registry,"AutoBuff");')
 p.edit('runtime.rs','                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,','                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,\n                        crate::auto_buff_runtime::ROTATION_PROVIDER=>auto_buff_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::fast_exp_runtime::FastExpRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::auto_buff_runtime::AutoBuffRuntime::processor_mode(processor){\n                            (&mut modules.auto_buff.rotations,mode)\n                        } else if let Some(mode)=crate::fast_exp_runtime::FastExpRuntime::processor_mode(processor){')
 p.edit('runtime.rs','        self.particles_attack(host,event.target as i32);','        let auto_buff_owns_slot=host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);\n        self.particles_attack(host,event.target as i32);')
 p.edit('runtime.rs','&weapon.context(false, state.registry.enabled_without_world(modules.query_ids["MaceKill"]))','&weapon.context(auto_buff_owns_slot, state.registry.enabled_without_world(modules.query_ids["MaceKill"]))')
 p.edit('runtime.rs','        use liquidbounce_behaviors::auto_weapon::Snapshot;','        let auto_buff_owns_slot=host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);\n        use liquidbounce_behaviors::auto_weapon::Snapshot;')
 p.edit('runtime.rs','&snapshot.context(false, state.registry.enabled_without_world(modules.query_ids["MaceKill"]))','&snapshot.context(auto_buff_owns_slot, state.registry.enabled_without_world(modules.query_ids["MaceKill"]))')
 p.edit('crosshair_clicker_runtime.rs','        let target = self.host.client_crosshair_clicker_target(target);','        let auto_buff_owns_slot=self.host.client_silent_slot_modified_by(crate::auto_buff_runtime::REQUESTER);\n        let target = self.host.client_crosshair_clicker_target(target);')
 p.edit('crosshair_clicker_runtime.rs','                    &weapon.context(\n                        false,','                    &weapon.context(\n                        auto_buff_owns_slot,')
 p.edit('../../make-behavior-patch.py',"'fast-exp-patch.py',","'fast-exp-patch.py', 'auto-buff-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoBuff '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
