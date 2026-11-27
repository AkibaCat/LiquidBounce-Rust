#!/usr/bin/env python3
"""Transactional complete Replenish wiring; --check leaves shared files intact."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_deposit;','pub mod auto_deposit;\npub mod replenish;')
 p.edit('lib.rs','mod auto_deposit_runtime;','mod auto_deposit_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod replenish_runtime;')
 p.edit('behaviors.rs','    pub auto_deposit:crate::auto_deposit_runtime::AutoDepositRuntime,','    pub auto_deposit:crate::auto_deposit_runtime::AutoDepositRuntime,\n    pub replenish:crate::replenish_runtime::ReplenishRuntime,')
 p.edit('behaviors.rs','            "AutoDeposit",','            "AutoDeposit",\n            "Replenish",')
 p.edit('behaviors.rs','            auto_deposit:Default::default(),','            auto_deposit:Default::default(),\n            replenish:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoDeposit")==Some(&id)=>{','''            ModuleCallback::OnEnabled(id) if self.query_ids.get("Replenish")==Some(&id)=>{self.replenish.settings.clear_tracked();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("Replenish")==Some(&id)=>{self.replenish.cancel(&mut self.inventory);}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoDeposit")==Some(&id)=>{''')
 p.edit('coroutine_ticker.rs','    AutoDeposit,','    AutoDeposit,\n    Replenish,')
 p.edit('config.rs','pub use auto_deposit::{AutoDepositSetting,serialize_auto_deposit};','pub use auto_deposit::{AutoDepositSetting,serialize_auto_deposit};\n#[path="config/replenish.rs"]\nmod replenish;\npub use replenish::{ReplenishSetting,serialize_replenish};')
 p.edit('config.rs','    AutoDeposit => ("AutoDeposit","Player","autoDeposit"),','    AutoDeposit => ("AutoDeposit","Player","autoDeposit"),\n    Replenish => ("Replenish","Player","replenish"),')
 p.edit('config.rs','"ContainerStorer"=>Some(Self::AutoDeposit),','"ContainerStorer"=>Some(Self::AutoDeposit), "Refill"=>Some(Self::Replenish),')
 p.edit('config.rs','    AutoDeposit(AutoDepositSetting),','    AutoDeposit(AutoDepositSetting),\n    Replenish(ReplenishSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoDeposit','            | ModuleKind::AutoDeposit\n            | ModuleKind::Replenish')
 p.edit('config.rs','        ModuleKind::AutoDeposit => auto_deposit::parse_auto_deposit(&mut patch,&root),','        ModuleKind::AutoDeposit => auto_deposit::parse_auto_deposit(&mut patch,&root),\n        ModuleKind::Replenish => replenish::parse_replenish(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoDeposit(value)=>{','        SettingChange::Replenish(value)=>{return ApplyResult::Runtime(RuntimeRequest::Replenish(value.clone()));}\n        SettingChange::AutoDeposit(value)=>{')
 p.edit('actor.rs','            ModuleKind::AutoDeposit => config::serialize_auto_deposit(&b.auto_deposit,&common,format,&describe),','            ModuleKind::AutoDeposit => config::serialize_auto_deposit(&b.auto_deposit,&common,format,&describe),\n            ModuleKind::Replenish => config::serialize_replenish(&b.replenish.settings,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoDeposit(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::Replenish(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().replenish.settings){eprintln!("[LiquidBounce] Replenish config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::AutoDeposit(setting))=>{''')
 p.edit('inventory_runtime.rs','        modules.auto_deposit.clear();','        modules.auto_deposit.clear();\n        modules.replenish.world_change();')
 p.edit('inventory_runtime.rs','        self.auto_deposit_plan(host,&snapshot.context);','        self.auto_deposit_plan(host,&snapshot.context);\n        self.replenish_plan(&snapshot.context);')
 p.edit('inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.replenish.owner(modules.query_running(&state.registry,"Replenish"),&context));\n                modules.inventory.observe_context(&context);')
 p.edit('inventory_runtime.rs','                    self.auto_deposit_outcome(host,outcome,&context);','                    self.auto_deposit_outcome(host,outcome,&context);\n                    self.replenish_outcome(outcome,&context);')
 p.edit('inventory_runtime.rs','    pub(crate) fn auto_tool_observe(&self, host: &dyn Host, payload: &str) {','''    pub(crate) fn auto_tool_observe(&self, host: &dyn Host, payload: &str) {
        if serde_json::from_str::<serde_json::Value>(payload).ok().is_some_and(|v|v["kind"]=="replenishScreen") {self.replenish_screen(payload);return;}''')
 p.edit('runtime.rs','                WaitOwner::AutoDeposit=>self.auto_deposit_callback(host,in_game,task.callback),','                WaitOwner::AutoDeposit=>self.auto_deposit_callback(host,in_game,task.callback),\n                WaitOwner::Replenish=>self.replenish_callback(host,in_game,task.callback),')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'replenish-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('Replenish '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
