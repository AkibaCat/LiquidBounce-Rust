#!/usr/bin/env python3
"""Transactional complete AutoDeposit wiring; --check leaves shared files intact."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod chest_cleaner;','pub mod chest_cleaner;\npub mod auto_deposit;')
 p.edit('lib.rs','mod chest_cleaner_runtime;','mod chest_cleaner_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod auto_deposit_runtime;')
 p.edit('behaviors.rs','    pub chest_cleaner:crate::chest_cleaner_runtime::ChestCleanerRuntime,','    pub chest_cleaner:crate::chest_cleaner_runtime::ChestCleanerRuntime,\n    pub auto_deposit:crate::auto_deposit_runtime::AutoDepositRuntime,')
 p.edit('behaviors.rs','            "ChestCleaner",','            "ChestCleaner",\n            "AutoDeposit",')
 p.edit('behaviors.rs','            chest_cleaner:Default::default(),','            chest_cleaner:Default::default(),\n            auto_deposit:Default::default(),')
 p.edit('behaviors.rs','            crate::no_web_runtime::ROTATION_PROVIDER => self.query_running(registry,"NoWeb"),','            crate::auto_deposit_runtime::ROTATION_PROVIDER => self.query_running(registry,"AutoDeposit"),\n            crate::no_web_runtime::ROTATION_PROVIDER => self.query_running(registry,"NoWeb"),')
 p.edit('behaviors.rs','            ModuleCallback::CancelSequences(id) if self.query_ids.get("ChestCleaner") == Some(&id) => {','''            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoDeposit")==Some(&id)=>{
                self.inventory.manager.cancel_pending(crate::auto_deposit_runtime::OWNER);self.auto_deposit.clear();self.pending_slot_resets.push(crate::auto_deposit_runtime::REQUESTER);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("ChestCleaner") == Some(&id) => {''')
 p.edit('coroutine_ticker.rs','    HighJump,','    HighJump,\n    AutoDeposit,')
 p.edit('config.rs','pub use chest_cleaner::{ChestCleanerSetting,serialize_chest_cleaner};','pub use chest_cleaner::{ChestCleanerSetting,serialize_chest_cleaner};\n#[path="config/auto_deposit.rs"]\nmod auto_deposit;\npub use auto_deposit::{AutoDepositSetting,serialize_auto_deposit};')
 p.edit('config.rs','    ChestCleaner => ("ChestCleaner","Player","chestCleaner"),','    ChestCleaner => ("ChestCleaner","Player","chestCleaner"),\n    AutoDeposit => ("AutoDeposit","Player","autoDeposit"),')
 p.edit('config.rs','"ContainerCleaner" => Some(Self::ChestCleaner),','"ContainerCleaner" => Some(Self::ChestCleaner), "AutoStore"|"ContainerStorer"=>Some(Self::AutoDeposit),')
 p.edit('config.rs','    ChestCleaner(ChestCleanerSetting),','    ChestCleaner(ChestCleanerSetting),\n    AutoDeposit(AutoDepositSetting),',2)
 p.edit('config.rs','            | ModuleKind::ChestCleaner','            | ModuleKind::ChestCleaner\n            | ModuleKind::AutoDeposit')
 p.edit('config.rs','        ModuleKind::ChestCleaner => chest_cleaner::parse_chest_cleaner(&mut patch,&root),','        ModuleKind::ChestCleaner => chest_cleaner::parse_chest_cleaner(&mut patch,&root),\n        ModuleKind::AutoDeposit => auto_deposit::parse_auto_deposit(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::ChestCleaner(value) => {','        SettingChange::AutoDeposit(value)=>{return ApplyResult::Runtime(RuntimeRequest::AutoDeposit(value.clone()));}\n        SettingChange::ChestCleaner(value) => {')
 p.edit('actor.rs','            ModuleKind::ChestCleaner => config::serialize_chest_cleaner(&b.chest_cleaner.settings,&common,format,&describe),','            ModuleKind::ChestCleaner => config::serialize_chest_cleaner(&b.chest_cleaner.settings,&common,format,&describe),\n            ModuleKind::AutoDeposit => config::serialize_auto_deposit(&b.auto_deposit,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::ChestCleaner(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::AutoDeposit(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_deposit,&self.state.item_registry,&self.state.block_registry){eprintln!("[LiquidBounce] AutoDeposit config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::ChestCleaner(setting)) => {''')
 p.edit('inventory_runtime.rs','        modules.chest_cleaner.clear();','        modules.chest_cleaner.clear();\n        modules.auto_deposit.clear();')
 p.edit('inventory_runtime.rs','        self.chest_cleaner_plan(host,&snapshot.context);','        self.chest_cleaner_plan(host,&snapshot.context);\n        self.auto_deposit_plan(host,&snapshot.context);')
 p.edit('inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.auto_deposit.owner(modules.query_running(&state.registry,"AutoDeposit")));\n                modules.inventory.observe_context(&context);')
 p.edit('inventory_runtime.rs','                    if disable {\n                        self.elytra_swap_disable(host);','                    self.auto_deposit_outcome(host,outcome,&context);\n                    if disable {\n                        self.elytra_swap_disable(host);')
 p.edit('runtime.rs','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),','                WaitOwner::AutoDeposit=>self.auto_deposit_callback(host,in_game,task.callback),\n                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),')
 p.edit('runtime.rs','        self.no_web_prepare(host);','        self.auto_deposit_prepare(host);\n        self.no_web_prepare(host);')
 p.edit('runtime.rs','        self.no_web_tick(host);','        self.no_web_tick(host);\n        self.auto_deposit_punch_tick(host);')
 p.edit('runtime.rs','            let no_web_running = modules.query_running(&state.registry,"NoWeb");','            let no_web_running = modules.query_running(&state.registry,"NoWeb");\n            let auto_deposit_running=modules.query_running(&state.registry,"AutoDeposit");')
 p.edit('runtime.rs','                        crate::no_web_runtime::ROTATION_PROVIDER => no_web_running,','                        crate::auto_deposit_runtime::ROTATION_PROVIDER=>auto_deposit_running,\n                        crate::no_web_runtime::ROTATION_PROVIDER => no_web_running,')
 p.edit('runtime.rs','                        } else if let Some(mode) = crate::no_web_runtime::NoWebRuntime::processor_mode(processor) {','                        } else if let Some(mode)=crate::auto_deposit_runtime::AutoDepositRuntime::processor_mode(processor){\n                            (&mut modules.auto_deposit.rotations,mode)\n                        } else if let Some(mode) = crate::no_web_runtime::NoWebRuntime::processor_mode(processor) {')
 p.edit('../../make-behavior-patch.py',"'chest-cleaner-patch.py',","'chest-cleaner-patch.py', 'auto-deposit-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoDeposit '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
