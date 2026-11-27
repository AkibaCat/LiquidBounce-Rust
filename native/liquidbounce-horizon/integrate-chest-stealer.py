#!/usr/bin/env python3
"""Validate all ChestStealer anchors before publishing shared engine wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
BasePlan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
class Plan(BasePlan):
 def commit(self):
  for path,original in self.original.items():
   if path.read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during integration')
  written=[]
  try:
   for path,source in self.changed.items():path.write_text(source,encoding='utf8',newline='\n');written.append(path)
  except BaseException:
   for path in written:path.write_text(self.original[path],encoding='utf8',newline='\n')
   raise
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod inventory_cleaner;','pub mod inventory_cleaner;\npub mod chest_stealer;')
 p.edit('lib.rs','mod inventory_cleaner_runtime;','mod inventory_cleaner_runtime;\n#[cfg(feature="behavior-hooks")]\nmod chest_stealer_runtime;')
 p.edit('behaviors.rs','    pub item_policy:crate::inventory_cleaner_runtime::ItemPolicy,','    pub item_policy:crate::inventory_cleaner_runtime::ItemPolicy,\n    pub chest_stealer:crate::chest_stealer_runtime::ChestStealerRuntime,')
 p.edit('behaviors.rs','            "InventoryCleaner",','            "InventoryCleaner",\n            "ChestStealer",')
 p.edit('behaviors.rs','            item_policy:Default::default(),','            item_policy:Default::default(),\n            chest_stealer:Default::default(),')
 p.edit('behaviors.rs','            crate::auto_deposit_runtime::ROTATION_PROVIDER => self.query_running(registry,"AutoDeposit"),','            crate::auto_deposit_runtime::ROTATION_PROVIDER => self.query_running(registry,"AutoDeposit"),\n            crate::chest_stealer_runtime::ROTATION_PROVIDER => self.query_running(registry,"ChestStealer"),')
 p.edit('behaviors.rs','            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryCleaner")==Some(&id)=>{','''            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("ChestStealer")==Some(&id)=>{self.inventory.manager.cancel_pending(crate::chest_stealer_runtime::OWNER);if let Some(id)=self.chest_stealer.pending{self.inventory.manager.end_plan(id);}self.chest_stealer.clear();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryCleaner")==Some(&id)=>{''')
 p.edit('config.rs','pub use inventory_cleaner::{InventoryCleanerSetting,serialize_inventory_cleaner};','pub use inventory_cleaner::{InventoryCleanerSetting,serialize_inventory_cleaner};\n#[path="config/chest_stealer.rs"]\nmod chest_stealer;\npub use chest_stealer::{ChestStealerSetting,serialize_chest_stealer};')
 p.edit('config.rs','    InventoryCleaner => ("InventoryCleaner","Player","inventoryCleaner"),','    InventoryCleaner => ("InventoryCleaner","Player","inventoryCleaner"),\n    ChestStealer => ("ChestStealer","Player","chestStealer"),')
 p.edit('config.rs','"InventoryManager"=>Some(Self::InventoryCleaner),','"InventoryManager"=>Some(Self::InventoryCleaner), "ContainerStealer"=>Some(Self::ChestStealer),')
 p.edit('config.rs','    InventoryCleaner(InventoryCleanerSetting),','    InventoryCleaner(InventoryCleanerSetting),\n    ChestStealer(ChestStealerSetting),',2)
 p.edit('config.rs','            | ModuleKind::InventoryCleaner','            | ModuleKind::InventoryCleaner\n            | ModuleKind::ChestStealer')
 p.edit('config.rs','        ModuleKind::InventoryCleaner => inventory_cleaner::parse_inventory_cleaner(&mut patch,&root),','        ModuleKind::InventoryCleaner => inventory_cleaner::parse_inventory_cleaner(&mut patch,&root),\n        ModuleKind::ChestStealer => chest_stealer::parse_chest_stealer(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::InventoryCleaner(value)=>{','        SettingChange::ChestStealer(value)=>{return ApplyResult::Runtime(RuntimeRequest::ChestStealer(value.clone()));}\n        SettingChange::InventoryCleaner(value)=>{')
 p.edit('actor.rs','            ModuleKind::InventoryCleaner => config::serialize_inventory_cleaner(&b.inventory_cleaner.settings,&common,format,&describe),','            ModuleKind::InventoryCleaner => config::serialize_inventory_cleaner(&b.inventory_cleaner.settings,&common,format,&describe),\n            ModuleKind::ChestStealer => config::serialize_chest_stealer(&b.chest_stealer,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::ChestStealer(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().chest_stealer,&self.state.item_registry,&self.state.block_registry){eprintln!("[LiquidBounce] ChestStealer config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(setting))=>{''')
 p.edit('browser.rs','                state.block_registry = block_registry;','''                state.block_registry = block_registry;
                state.behaviors.borrow_mut().chest_stealer.initialize_storage(&state.block_registry);''')
 p.edit('inventory_runtime.rs','        modules.inventory_cleaner.clear();','        modules.inventory_cleaner.clear();\n        modules.chest_stealer.clear();')
 p.edit('inventory_runtime.rs','                Observation::UseOn(pos) => {','                Observation::UseOn(pos) => {\n                    self.chest_stealer_manual(pos);')
 p.edit('inventory_runtime.rs','        self.inventory_cleaner_plan(host,&snapshot.context);','        self.inventory_cleaner_plan(host,&snapshot.context);\n        self.chest_stealer_plan(host,&snapshot.context);')
 p.edit('inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.chest_stealer.owner(modules.query_running(&state.registry,"ChestStealer")));\n                modules.inventory.observe_context(&context);')
 p.edit('inventory_runtime.rs','                    self.replenish_outcome(outcome,&context);','                    self.replenish_outcome(outcome,&context);\n                    self.chest_stealer_outcome(host,outcome,&context);')
 p.edit('runtime.rs','        self.better_inventory_render(event.query, event.draw);','        if self.chest_stealer_render(_host,event.query,event.draw){return;}\n        self.better_inventory_render(event.query, event.draw);')
 p.edit('runtime.rs','        self.auto_deposit_prepare(host);','        self.auto_deposit_prepare(host);\n        self.chest_aura_prepare(host);')
 p.edit('runtime.rs','        self.auto_deposit_punch_tick(host);','        self.auto_deposit_punch_tick(host);\n        self.chest_aura_tick(host);')
 p.edit('runtime.rs','            let auto_deposit_running=modules.query_running(&state.registry,"AutoDeposit");','            let auto_deposit_running=modules.query_running(&state.registry,"AutoDeposit");\n            let chest_stealer_running=modules.query_running(&state.registry,"ChestStealer");')
 p.edit('runtime.rs','                        crate::auto_deposit_runtime::ROTATION_PROVIDER=>auto_deposit_running,','                        crate::auto_deposit_runtime::ROTATION_PROVIDER=>auto_deposit_running,\n                        crate::chest_stealer_runtime::ROTATION_PROVIDER=>chest_stealer_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_deposit_runtime::AutoDepositRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::chest_stealer_runtime::ChestStealerRuntime::processor_mode(processor){\n                            (&mut modules.chest_stealer.rotations,mode)\n                        } else if let Some(mode)=crate::auto_deposit_runtime::AutoDepositRuntime::processor_mode(processor){')
 p.edit('../../make-behavior-patch.py',"'inventory-cleaner-patch.py',","'inventory-cleaner-patch.py', 'chest-stealer-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('ChestStealer '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
