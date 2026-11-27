#!/usr/bin/env python3
"""Transactional Teams and shared prioritized entity color wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod targeting;','pub mod targeting;\npub mod teams;','behavior export')
 edit('native/liquidbounce-behaviors/src/targeting.rs','    pub is_friend: bool,','    pub is_friend: bool,\n    pub tag_color: Option<(i32,u32)>,','tag color field')
 edit('native/liquidbounce-behaviors/src/targeting.rs','impl TargetInfo {','''impl TargetInfo {
    /// PriorityField accepts only a strictly greater priority; equal priority keeps the first handler.
    pub fn color(&mut self,color:u32,priority:i32) {
        if priority > self.tag_color.map_or(-20,|(p,_)|p) { self.tag_color=Some((priority,color)); }
    }
    pub fn resolved_color(self)->Option<u32>{self.tag_color.map(|(_,c)|c)}''','priority color semantics')
 edit(E+'lib.rs','mod target_lock_runtime;','mod target_lock_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod teams_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub fast_use: liquidbounce_behaviors::fast_use::FastUse,','    pub fast_use: liquidbounce_behaviors::fast_use::FastUse,\n    pub teams: liquidbounce_behaviors::teams::Teams,','behavior field')
 edit(E+'behaviors.rs','            "FastUse",','            "FastUse",\n            "Teams",','registry')
 edit(E+'behaviors.rs','            fast_use: Default::default(),','            fast_use: Default::default(),\n            teams: Default::default(),','default')
 edit(E+'config.rs','pub use fast_use::{FastUseSetting, serialize_fast_use};','pub use fast_use::{FastUseSetting, serialize_fast_use};\n#[path = "config/teams.rs"]\nmod teams;\npub use teams::{TeamsSetting,serialize_teams};','config export')
 edit(E+'config.rs','    FastUse => ("FastUse", "Player", "fastUse"),','    FastUse => ("FastUse", "Player", "fastUse"),\n    Teams => ("Teams", "Misc", "teams"),','module kind')
 edit(E+'config.rs','    FastUse(FastUseSetting),','    FastUse(FastUseSetting),\n    Teams(TeamsSetting),','setting/runtime variants',2)
 edit(E+'config.rs','            | ModuleKind::FastUse','            | ModuleKind::FastUse\n            | ModuleKind::Teams','dedicated serializer')
 edit(E+'config.rs','        ModuleKind::FastUse => fast_use::parse_fast_use(&mut patch, &root),','        ModuleKind::FastUse => fast_use::parse_fast_use(&mut patch, &root),\n        ModuleKind::Teams => teams::parse_teams(&mut patch,&root),','parse dispatch')
 edit(E+'config.rs','        SettingChange::FastUse(value) => {','        SettingChange::Teams(value) => { return ApplyResult::Runtime(RuntimeRequest::Teams(value.clone())); }\n        SettingChange::FastUse(value) => {','apply dispatch')
 edit(E+'actor.rs','            ModuleKind::FastUse => config::serialize_fast_use(&b.fast_use,&common,format,&describe),','            ModuleKind::FastUse => config::serialize_fast_use(&b.fast_use,&common,format,&describe),\n            ModuleKind::Teams => config::serialize_teams(&b.teams,&common,format,&describe),','actor serializer')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::FastUse(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Teams(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().teams); }\n                ApplyResult::Runtime(RuntimeRequest::FastUse(setting)) => {','actor apply')
 edit(E+'targeting.rs','    pub abstract_client_player: bool,','    pub abstract_client_player: bool,\n    pub teams: Option<liquidbounce_behaviors::teams::Facts>,','snapshot facts')
 edit(E+'targeting.rs','        abstract_client_player: value["abstract_client_player"].as_bool().unwrap_or(false),','        abstract_client_player: value["abstract_client_player"].as_bool().unwrap_or(false),\n        teams: crate::teams_runtime::facts(&value["teams"]),','snapshot decode')
 p.section(E+'runtime.rs','    pub(crate) fn target_info(','    pub(crate) fn camera_distance_changed(',
 '''            info
        })''',
 '''            crate::teams_runtime::apply_tag(&modules.teams,modules.query_running(&self.registry,"Teams"),snapshot.entity.kind.living,snapshot.teams.as_ref(),&mut info);
            info
        })''','lazy tag cache after TargetLock')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'hitbox-patch.py',","'hitbox-patch.py', 'teams-patch.py',",'native facts patch')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);g=a.add_mutually_exclusive_group();g.add_argument('--check',action='store_true');g.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Teams edits')
 if args.apply:p.write();print('Applied Teams')
