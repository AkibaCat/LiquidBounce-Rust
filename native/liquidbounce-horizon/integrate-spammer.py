#!/usr/bin/env python3
"""Transactional Spammer wiring; default validates, --apply commits."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod teams;','pub mod teams;\npub mod spammer;','behavior export')
 edit(E+'lib.rs','mod teams_runtime;','mod teams_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod spammer_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub teams: liquidbounce_behaviors::teams::Teams,','    pub teams: liquidbounce_behaviors::teams::Teams,\n    pub spammer: crate::spammer_runtime::SpammerRuntime,','field')
 edit(E+'behaviors.rs','            "Teams",','            "Teams",\n            "Spammer",','registry')
 edit(E+'behaviors.rs','            teams: Default::default(),','            teams: Default::default(),\n            spammer: Default::default(),','default')
 edit(E+'behaviors.rs','"Timer" | "ElytraSwap" | "Plugins" | "OldRegen"','"Timer" | "ElytraSwap" | "Plugins" | "OldRegen" | "Spammer"','disable on quit')
 edit(E+'behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoWalk") == Some(&id) => {','            ModuleCallback::CancelSequences(id) | ModuleCallback::OnDisabled(id) if self.query_ids.get("Spammer") == Some(&id) => { self.spammer.cancel(); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoWalk") == Some(&id) => {','worker cancellation')
 edit(E+'config.rs','pub use teams::{TeamsSetting,serialize_teams};','pub use teams::{TeamsSetting,serialize_teams};\n#[path = "config/spammer.rs"]\nmod spammer;\npub use spammer::{SpammerSetting,serialize_spammer};','config export')
 edit(E+'config.rs','    Teams => ("Teams", "Misc", "teams"),','    Teams => ("Teams", "Misc", "teams"),\n    Spammer => ("Spammer", "Misc", "spammer"),','module kind')
 edit(E+'config.rs','    Teams(TeamsSetting),','    Teams(TeamsSetting),\n    Spammer(SpammerSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::Teams','            | ModuleKind::Teams\n            | ModuleKind::Spammer','dedicated serializer')
 edit(E+'config.rs','        ModuleKind::Teams => teams::parse_teams(&mut patch,&root),','        ModuleKind::Teams => teams::parse_teams(&mut patch,&root),\n        ModuleKind::Spammer => spammer::parse_spammer(&mut patch,&root),','parse dispatch')
 edit(E+'config.rs','        SettingChange::Teams(value) => {','        SettingChange::Spammer(value) => { return ApplyResult::Runtime(RuntimeRequest::Spammer(value.clone())); }\n        SettingChange::Teams(value) => {','apply dispatch')
 edit(E+'actor.rs','            ModuleKind::Teams => config::serialize_teams(&b.teams,&common,format,&describe),','            ModuleKind::Teams => config::serialize_teams(&b.teams,&common,format,&describe),\n            ModuleKind::Spammer => config::serialize_spammer(&b.spammer.settings,&common,format,&describe),','serializer')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Teams(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Spammer(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().spammer.settings); }\n                ApplyResult::Runtime(RuntimeRequest::Teams(setting)) => {','actor apply')
 edit(E+'runtime.rs','        self.plugins_tick(host);','        self.plugins_tick(host);\n        self.spammer_tick(host);','actual phase')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'teams-patch.py',","'teams-patch.py', 'spammer-patch.py',",'native helper generation')
 edit(E+'config.rs','"BlockOverlay" => Some(Self::BlockOutline),','"FastEat" => Some(Self::FastUse), "BlockOverlay" => Some(Self::BlockOutline),','FastEat alias')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Spammer edits')
 if args.apply:p.write();print('Applied Spammer')
