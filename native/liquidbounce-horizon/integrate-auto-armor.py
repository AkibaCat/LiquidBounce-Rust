#!/usr/bin/env python3
"""Validate every AutoArmor seam before publishing shared engine wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod inventory_cleaner;','pub mod inventory_cleaner;\npub mod auto_armor;')
 p.edit('lib.rs','mod inventory_cleaner_runtime;','mod inventory_cleaner_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_armor_runtime;')
 p.edit('behaviors.rs','    pub item_policy:crate::inventory_cleaner_runtime::ItemPolicy,','    pub item_policy:crate::inventory_cleaner_runtime::ItemPolicy,\n    pub auto_armor:crate::auto_armor_runtime::AutoArmorRuntime,')
 p.edit('behaviors.rs','            "InventoryCleaner",','            "InventoryCleaner",\n            "AutoArmor",')
 p.edit('behaviors.rs','            item_policy:Default::default(),','            item_policy:crate::inventory_cleaner_runtime::ItemPolicy {armor:liquidbounce_behaviors::auto_armor::AutoArmor::default().policy(),..Default::default()},\n            auto_armor:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryCleaner")==Some(&id)=>{','''            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoArmor")==Some(&id)=>{self.inventory.manager.cancel_pending(crate::auto_armor_runtime::OWNER);if let Some(id)=self.auto_armor.pending{self.inventory.manager.end_plan(id);}self.auto_armor.clear();}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryCleaner")==Some(&id)=>{''')
 p.edit('coroutine_ticker.rs','    Replenish,','    Replenish,\n    AutoArmor,')
 p.edit('config.rs','pub use inventory_cleaner::{InventoryCleanerSetting,serialize_inventory_cleaner};','pub use inventory_cleaner::{InventoryCleanerSetting,serialize_inventory_cleaner};\n#[path="config/auto_armor.rs"]\nmod auto_armor;\npub use auto_armor::{AutoArmorSetting,serialize_auto_armor};')
 p.edit('config.rs','    InventoryCleaner => ("InventoryCleaner","Player","inventoryCleaner"),','    InventoryCleaner => ("InventoryCleaner","Player","inventoryCleaner"),\n    AutoArmor => ("AutoArmor","Combat","autoArmor"),')
 p.edit('config.rs','    InventoryCleaner(InventoryCleanerSetting),','    InventoryCleaner(InventoryCleanerSetting),\n    AutoArmor(AutoArmorSetting),',2)
 p.edit('config.rs','            | ModuleKind::InventoryCleaner','            | ModuleKind::InventoryCleaner\n            | ModuleKind::AutoArmor')
 p.edit('config.rs','        ModuleKind::InventoryCleaner => inventory_cleaner::parse_inventory_cleaner(&mut patch,&root),','        ModuleKind::InventoryCleaner => inventory_cleaner::parse_inventory_cleaner(&mut patch,&root),\n        ModuleKind::AutoArmor => auto_armor::parse_auto_armor(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::InventoryCleaner(value)=>{','        SettingChange::AutoArmor(value)=>{return ApplyResult::Runtime(RuntimeRequest::AutoArmor(value.clone()));}\n        SettingChange::InventoryCleaner(value)=>{')
 p.edit('actor.rs','            ModuleKind::InventoryCleaner => config::serialize_inventory_cleaner(&b.inventory_cleaner.settings,&common,format,&describe),','            ModuleKind::InventoryCleaner => config::serialize_inventory_cleaner(&b.inventory_cleaner.settings,&common,format,&describe),\n            ModuleKind::AutoArmor => config::serialize_auto_armor(&b.auto_armor.settings,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::AutoArmor(setting))=>{
                    let mut b=self.state.behaviors.borrow_mut();
                    if let Err(e)=setting.apply(&mut b.auto_armor.settings){eprintln!("[LiquidBounce] AutoArmor config: {e}");}
                    b.item_policy.armor=b.auto_armor.settings.policy();
                }
                ApplyResult::Runtime(RuntimeRequest::InventoryCleaner(setting))=>{''')
 p.edit('inventory_runtime.rs','        modules.inventory_cleaner.clear();','        modules.inventory_cleaner.clear();\n        modules.auto_armor.clear();')
 p.edit('inventory_runtime.rs','        self.inventory_cleaner_plan(host,&snapshot.context);','        self.auto_armor_plan(host,&snapshot.context);\n        self.inventory_cleaner_plan(host,&snapshot.context);')
 p.edit('inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.auto_armor.owner(modules.query_running(&state.registry,"AutoArmor")));\n                modules.inventory.observe_context(&context);')
 p.edit('inventory_runtime.rs','                    self.replenish_outcome(outcome,&context);','                    self.replenish_outcome(outcome,&context);\n                    self.auto_armor_outcome(host,outcome,&context);')
 p.edit('runtime.rs','                WaitOwner::Replenish=>self.replenish_callback(host,in_game,task.callback),','                WaitOwner::Replenish=>self.replenish_callback(host,in_game,task.callback),\n                WaitOwner::AutoArmor=>self.auto_armor_callback(host,in_game,task.callback),')
 p.edit('../../make-behavior-patch.py',"'chest-stealer-patch.py',","'chest-stealer-patch.py', 'auto-armor-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoArmor '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
