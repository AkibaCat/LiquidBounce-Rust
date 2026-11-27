#!/usr/bin/env python3
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod replenish;','pub mod replenish;\npub mod inventory_cleaner;')
 p.edit('lib.rs','mod replenish_runtime;','mod replenish_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod inventory_cleaner_runtime;')
 p.edit('behaviors.rs','    pub replenish:crate::replenish_runtime::ReplenishRuntime,','    pub replenish:crate::replenish_runtime::ReplenishRuntime,\n    pub inventory_cleaner:crate::inventory_cleaner_runtime::InventoryCleanerRuntime,\n    pub item_policy:crate::inventory_cleaner_runtime::ItemPolicy,')
 p.edit('behaviors.rs','            "Replenish",','            "Replenish",\n            "InventoryCleaner",')
 p.edit('behaviors.rs','            replenish:Default::default(),','            replenish:Default::default(),\n            inventory_cleaner:Default::default(),\n            item_policy:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnEnabled(id) if self.query_ids.get("Replenish")==Some(&id)=>{','''            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryCleaner")==Some(&id)=>{self.inventory.manager.cancel_pending(crate::inventory_cleaner_runtime::OWNER);if let Some(id)=self.inventory_cleaner.pending{self.inventory.manager.end_plan(id);}self.inventory_cleaner.clear();}
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Replenish")==Some(&id)=>{''')
 p.edit('config.rs','pub use replenish::{ReplenishSetting,serialize_replenish};','pub use replenish::{ReplenishSetting,serialize_replenish};\n#[path="config/inventory_cleaner.rs"]\nmod inventory_cleaner;\npub use inventory_cleaner::{InventoryCleanerSetting,serialize_inventory_cleaner};')
 p.edit('config.rs','    Replenish => ("Replenish","Player","replenish"),','    Replenish => ("Replenish","Player","replenish"),\n    InventoryCleaner => ("InventoryCleaner","Player","inventoryCleaner"),')
 p.edit('config.rs','"Refill"=>Some(Self::Replenish),','"Refill"=>Some(Self::Replenish), "InventoryManager"=>Some(Self::InventoryCleaner),')
 p.edit('config.rs','    Replenish(ReplenishSetting),','    Replenish(ReplenishSetting),\n    InventoryCleaner(InventoryCleanerSetting),',2)
 p.edit('config.rs','            | ModuleKind::Replenish','            | ModuleKind::Replenish\n            | ModuleKind::InventoryCleaner')
 p.edit('config.rs','        ModuleKind::Replenish => replenish::parse_replenish(&mut patch,&root),','        ModuleKind::Replenish => replenish::parse_replenish(&mut patch,&root),\n        ModuleKind::InventoryCleaner => inventory_cleaner::parse_inventory_cleaner(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Replenish(value)=>{','        SettingChange::InventoryCleaner(value)=>{return ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(value.clone()));}\n        SettingChange::Replenish(value)=>{')
 p.edit('actor.rs','            ModuleKind::Replenish => config::serialize_replenish(&b.replenish.settings,&common,format,&describe),','            ModuleKind::Replenish => config::serialize_replenish(&b.replenish.settings,&common,format,&describe),\n            ModuleKind::InventoryCleaner => config::serialize_inventory_cleaner(&b.inventory_cleaner.settings,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Replenish(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(setting))=>{
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().inventory_cleaner.settings,&self.state.item_registry){eprintln!("[LiquidBounce] InventoryCleaner config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::Replenish(setting))=>{''')
 p.edit('inventory_runtime.rs','        modules.replenish.world_change();','        modules.replenish.world_change();\n        modules.inventory_cleaner.clear();')
 p.edit('inventory_runtime.rs','        self.replenish_plan(&snapshot.context);','        self.replenish_plan(&snapshot.context);\n        self.inventory_cleaner_plan(host,&snapshot.context);')
 p.edit('inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.inventory_cleaner.owner(modules.query_running(&state.registry,"InventoryCleaner")));\n                modules.inventory.observe_context(&context);')
 p.edit('inventory_runtime.rs','                    self.replenish_outcome(outcome,&context);','''                    self.replenish_outcome(outcome,&context);
                    {let state=self.state();let mut modules=state.behaviors.borrow_mut();let modules=&mut*modules;modules.inventory_cleaner.outcome(&mut modules.inventory,outcome,&context);}''')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'inventory-cleaner-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('InventoryCleaner '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
