"""InventoryTracker/CommandInvsee transactional wiring. --check never writes."""
from pathlib import Path
import argparse, runpy
HERE = Path(__file__).resolve().parent
Plan = runpy.run_path(str(HERE / 'integrate-chest-stealer.py'))['Plan']


def prepare():
    p = Plan()
    p.edit('../../../liquidbounce-behaviors/src/lib.rs', 'pub mod inventory_manager;',
           'pub mod inventory_manager;\npub mod inventory_tracker;')
    p.edit('lib.rs', 'mod logoff_spot_runtime;',
           'mod logoff_spot_runtime;\n#[cfg(feature="behavior-hooks")]\nmod inventory_tracker_runtime;')
    p.edit('behaviors.rs', '    pub logoff_spot:',
           '    pub inventory_tracker:crate::inventory_tracker_runtime::InventoryTrackerRuntime,\n    pub logoff_spot:')
    p.edit('behaviors.rs', '            logoff_spot: Default::default(),',
           '            inventory_tracker:Default::default(),\n            logoff_spot: Default::default(),')
    p.edit('behaviors.rs', '            "LogoffSpot",',
           '            "LogoffSpot",\n            "InventoryTracker",')
    anchor = '            ModuleCallback::OnDisabled(id) if self.query_ids.get("LogoffSpot")==Some(&id)=>{self.logoff_spot.clear();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::LogoffSpotDisabled);}'
    p.edit('behaviors.rs', anchor, anchor + '\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("InventoryTracker")==Some(&id)=>{self.inventory_tracker.reset();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::InventoryTrackerReset);}')
    p.edit('one_shots_runtime.rs', '    LogoffSpotDisabled,',
           '    LogoffSpotDisabled,\n    InventoryTrackerReset,')
    p.edit('one_shots_runtime.rs', '            OneShotEffect::LogoffSpotDisabled=>self.logoff_spot_disabled(host),',
           '            OneShotEffect::LogoffSpotDisabled=>self.logoff_spot_disabled(host),\n            OneShotEffect::InventoryTrackerReset=>self.inventory_tracker_flush(host),')
    p.edit('config.rs', 'pub use logoff_spot::{LogoffSpotSetting,serialize_logoff_spot};',
           'pub use logoff_spot::{LogoffSpotSetting,serialize_logoff_spot};\n#[path="config/inventory_tracker.rs"]\nmod inventory_tracker;\npub use inventory_tracker::{InventoryTrackerSetting,serialize_inventory_tracker};')
    p.edit('config.rs', '    LogoffSpot => ("LogoffSpot", "Render", "logoffSpot"),',
           '    LogoffSpot => ("LogoffSpot", "Render", "logoffSpot"),\n    InventoryTracker => ("InventoryTracker", "World", "inventoryTracker"),')
    p.edit('config.rs', '    LogoffSpot(LogoffSpotSetting),',
           '    LogoffSpot(LogoffSpotSetting),\n    InventoryTracker(InventoryTrackerSetting),', 2)
    p.edit('config.rs', '            | ModuleKind::LogoffSpot',
           '            | ModuleKind::LogoffSpot\n            | ModuleKind::InventoryTracker')
    p.edit('config.rs', '        ModuleKind::LogoffSpot => logoff_spot::parse_logoff_spot(&mut patch,&root),',
           '        ModuleKind::LogoffSpot => logoff_spot::parse_logoff_spot(&mut patch,&root),\n        ModuleKind::InventoryTracker => inventory_tracker::parse_inventory_tracker(&mut patch,&root),')
    p.edit('config.rs', '        SettingChange::LogoffSpot(value) => {',
           '        SettingChange::InventoryTracker(value) => { return ApplyResult::Runtime(RuntimeRequest::InventoryTracker(*value)); }\n        SettingChange::LogoffSpot(value) => {')
    p.edit('actor.rs', '            ModuleKind::LogoffSpot => config::serialize_logoff_spot(b.logoff_spot.send_in_chat(),&common,format,&describe),',
           '            ModuleKind::LogoffSpot => config::serialize_logoff_spot(b.logoff_spot.send_in_chat(),&common,format,&describe),\n            ModuleKind::InventoryTracker => config::serialize_inventory_tracker(b.inventory_tracker.model.save_players,&common,format,&describe),')
    p.edit('actor.rs', '                ApplyResult::Runtime(RuntimeRequest::LogoffSpot(setting)) => {',
           '                ApplyResult::Runtime(RuntimeRequest::InventoryTracker(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().inventory_tracker); }\n                ApplyResult::Runtime(RuntimeRequest::LogoffSpot(setting)) => {')
    anchor = '    fn client_logoff_spot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientLogoffSpotEvent<\'_>){self.logoff_spot(host,event);}'
    p.edit('runtime.rs', anchor, anchor + '\n    #[cfg(feature="behavior-hooks")]\n    fn client_inventory_tracker(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientInventoryTrackerEvent<\'_>){self.inventory_tracker_event(event.input,event.output);}')
    anchor = '    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {'
    p.edit('runtime.rs', anchor, anchor + '\n        self.state().behaviors.borrow_mut().inventory_tracker.model.reset();')
    p.edit('runtime.rs', '        self.projectile_puncher_prepare(host);',
           '        self.inventory_tracker_flush(host);\n        self.projectile_puncher_prepare(host);')
    p.edit('client_commands_runtime.rs', 'const COMMANDS: [&str; 11] = [', 'const COMMANDS: [&str; 12] = [')
    p.edit('client_commands_runtime.rs', '    "hide",', '    "hide",\n    "invsee",')
    p.edit('client_commands_runtime.rs', '            "xray"=>return self.xray_command(host,r),',
           '            "xray"=>return self.xray_command(host,r),\n            "invsee"=>return self.inventory_tracker_command(host,r),')
    p.edit('../../make-behavior-patch.py', "'logoff-spot-patch.py',", "'logoff-spot-patch.py', 'inventory-tracker-patch.py',")
    return p


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    plan = prepare()
    if not args.check:
        plan.commit()
    print(f'InventoryTracker {"validated" if args.check else "integrated"}: {len(plan.changed)} shared files')
