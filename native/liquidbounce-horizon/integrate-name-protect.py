"""Transactional NameProtect engine/Font service wiring. --check never writes."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']


def prepare():
    p=Plan()
    p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod inventory_tracker;','pub mod inventory_tracker;\npub mod name_protect;')
    p.edit('lib.rs','mod inventory_tracker_runtime;','mod inventory_tracker_runtime;\n#[cfg(feature="behavior-hooks")]\nmod name_protect_runtime;')
    p.edit('behaviors.rs','    pub inventory_tracker:','    pub name_protect:std::sync::Arc<crate::name_protect_runtime::NameProtectService>,\n    pub inventory_tracker:')
    p.edit('behaviors.rs','            inventory_tracker:Default::default(),','            name_protect:Default::default(),\n            inventory_tracker:Default::default(),')
    p.edit('behaviors.rs','            "InventoryTracker",','            "InventoryTracker",\n            "NameProtect",')
    anchor='        self.sword_block.set_running(self.query_running(registry,"SwordBlock"));'
    p.edit('behaviors.rs',anchor,anchor+'\n        self.name_protect.set_running(self.query_running(registry,"NameProtect"));')
    p.edit('config.rs','pub use inventory_tracker::{InventoryTrackerSetting,serialize_inventory_tracker};','pub use inventory_tracker::{InventoryTrackerSetting,serialize_inventory_tracker};\n#[path="config/name_protect.rs"]\nmod name_protect;\npub use name_protect::{NameProtectSetting,serialize_name_protect};')
    p.edit('config.rs','    InventoryTracker => ("InventoryTracker", "World", "inventoryTracker"),','    InventoryTracker => ("InventoryTracker", "World", "inventoryTracker"),\n    NameProtect => ("NameProtect", "Misc", "nameProtect"),')
    p.edit('config.rs','    InventoryTracker(InventoryTrackerSetting),','    InventoryTracker(InventoryTrackerSetting),\n    NameProtect(NameProtectSetting),',2)
    p.edit('config.rs','            | ModuleKind::InventoryTracker','            | ModuleKind::InventoryTracker\n            | ModuleKind::NameProtect')
    p.edit('config.rs','        ModuleKind::InventoryTracker => inventory_tracker::parse_inventory_tracker(&mut patch,&root),','        ModuleKind::InventoryTracker => inventory_tracker::parse_inventory_tracker(&mut patch,&root),\n        ModuleKind::NameProtect => name_protect::parse_name_protect(&mut patch,&root),')
    p.edit('config.rs','        SettingChange::InventoryTracker(value) => {','        SettingChange::NameProtect(value) => { return ApplyResult::Runtime(RuntimeRequest::NameProtect(value.clone())); }\n        SettingChange::InventoryTracker(value) => {')
    p.edit('actor.rs','            ModuleKind::InventoryTracker => config::serialize_inventory_tracker(b.inventory_tracker.model.save_players,&common,format,&describe),','            ModuleKind::InventoryTracker => config::serialize_inventory_tracker(b.inventory_tracker.model.save_players,&common,format,&describe),\n            ModuleKind::NameProtect => config::serialize_name_protect(&b.name_protect.settings(),&common,format,&describe),')
    p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::InventoryTracker(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::NameProtect(setting)) => { setting.apply(&self.state.behaviors.borrow().name_protect); }\n                ApplyResult::Runtime(RuntimeRequest::InventoryTracker(setting)) => {')
    p.edit('runtime.rs','        let state = State::default();','        let state = State::default();\n        #[cfg(feature="behavior-hooks")]\n        horizon_client::text::install_name_protection(state.behaviors.borrow().name_protect.clone());')
    p.edit('runtime.rs','        self.criticals_tick(host);','        self.name_protect_tick(host);\n        self.criticals_tick(host);')
    # The real-name portion of FriendManager.list is the original explicit bypass consumer.
    old='''                            for (name, alias) in entries {
                                host.chat(&format!(
                                    "{name} ({})",
                                    alias.as_deref().unwrap_or(name)
                                ));
                            }'''
    new='''                            for (index,(name, alias)) in entries.into_iter().enumerate() {
                                let default_name=format!("Friend {index}");
                                host.client_name_protect_friend(name,alias.as_deref().unwrap_or(&default_name),&s.behaviors.borrow().client_commands.prefix);
                            }'''
    p.edit('client_commands_runtime.rs',old,new)
    p.edit('../../make-behavior-patch.py',"'inventory-tracker-patch.py',","'inventory-tracker-patch.py', 'name-protect-patch.py',")
    return p


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    plan=prepare()
    if not args.check:plan.commit()
    print(f'NameProtect {"validated" if args.check else "integrated"}: {len(plan.changed)} shared files')
