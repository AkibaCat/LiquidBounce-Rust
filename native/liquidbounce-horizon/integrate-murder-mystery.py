#!/usr/bin/env python3
"""Transactional source-complete MurderMystery wiring; parent applies once."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
    p=Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod teams;','pub mod teams;\npub mod murder_mystery;','behavior export')
    p.replace(E+'lib.rs','mod teams_runtime;','mod teams_runtime;\n#[cfg(feature="behavior-hooks")]\nmod murder_mystery_runtime;','runtime export')
    b=E+'behaviors.rs'
    p.replace(b,'    pub teams: liquidbounce_behaviors::teams::Teams,','    pub teams: liquidbounce_behaviors::teams::Teams,\n    pub murder_mystery:liquidbounce_behaviors::murder_mystery::State,','complete state')
    p.replace(b,'            "Teams",','            "Teams",\n            "MurderMystery",','registry')
    p.replace(b,'            teams: Default::default(),','            teams: Default::default(),\n            murder_mystery:Default::default(),','default')
    # sync_packet_modules is read-only. Its provider expression reads the live
    # source role directly so toggling the module never leaves a stale gate.
    p.replace(b,'&&!self.auto_dodge.murder_mystery_disallows);','&&!self.murder_mystery.disallows_arrow_dodge(self.query_running(registry,"MurderMystery")));','immediate AutoDodge provider lifecycle')
    anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("Macros")==Some(&id)=>'
    p.replace(b,anchor,'            ModuleCallback::OnDisabled(id) if self.query_ids.get("MurderMystery")==Some(&id)=>{self.murder_mystery.reset();self.auto_dodge.murder_mystery_disallows=false;}\n            ModuleCallback::OnEnabled(id) if self.query_ids.get("MurderMystery")==Some(&id)=>{self.auto_dodge.murder_mystery_disallows=self.murder_mystery.disallows_arrow_dodge(true);}\n'+anchor,'disable reset and reenable retained role')
    c=E+'config.rs'
    p.replace(c,'pub use teams::{TeamsSetting,serialize_teams};','pub use teams::{TeamsSetting,serialize_teams};\n#[path="config/murder_mystery.rs"]\nmod murder_mystery;\npub use murder_mystery::{MurderMysterySetting,serialize_murder_mystery};','config export')
    p.replace(c,'    Teams => ("Teams", "Misc", "teams"),','    Teams => ("Teams", "Misc", "teams"),\n    MurderMystery => ("MurderMystery", "Render", "murderMystery"),','source category')
    p.replace(c,'    Teams(TeamsSetting),','    Teams(TeamsSetting),\n    MurderMystery(MurderMysterySetting),','setting and request',count=2)
    p.replace(c,'            | ModuleKind::Teams','            | ModuleKind::Teams\n            | ModuleKind::MurderMystery','dedicated serializer')
    p.replace(c,'        ModuleKind::Teams => teams::parse_teams(&mut patch,&root),','        ModuleKind::Teams => teams::parse_teams(&mut patch,&root),\n        ModuleKind::MurderMystery=>murder_mystery::parse_murder_mystery(&mut patch,&root),','parse')
    p.replace(c,'        SettingChange::Teams(value) => {','        SettingChange::MurderMystery(value)=>return ApplyResult::Runtime(RuntimeRequest::MurderMystery(value.clone())),\n        SettingChange::Teams(value) => {','apply')
    a=E+'actor.rs'
    p.replace(a,'            ModuleKind::Teams => config::serialize_teams(&b.teams,&common,format,&describe),','            ModuleKind::Teams => config::serialize_teams(&b.teams,&common,format,&describe),\n            ModuleKind::MurderMystery=>config::serialize_murder_mystery(&b.murder_mystery,&common,format,&describe),','serialize')
    p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::Teams(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::MurderMystery(setting))=>{let mut b=self.state.behaviors.borrow_mut();setting.clone().apply(&mut b.murder_mystery);b.auto_dodge.murder_mystery_disallows=b.murder_mystery.disallows_arrow_dodge(b.query_running(&self.state.registry,"MurderMystery"));b.sync_packet_modules(&self.state.registry);}\n                ApplyResult::Runtime(RuntimeRequest::Teams(setting)) => {','live mode and dependency')
    t=E+'targeting.rs'
    p.replace(t,'    pub abstract_client_player: bool,','    pub abstract_client_player: bool,\n    pub murder_mystery_uuid:Option<String>,','real UUID snapshot')
    p.replace(t,'        abstract_client_player: value["abstract_client_player"].as_bool().unwrap_or(false),','        abstract_client_player: value["abstract_client_player"].as_bool().unwrap_or(false),\n        murder_mystery_uuid:value["murderMysteryUuid"].as_str().map(str::to_owned),','UUID decode')
    r=E+'runtime.rs'
    p.section(r,'    pub(crate) fn target_info(','    pub(crate) fn camera_distance_changed(','            let modules = self.behaviors.borrow();','            let mut modules = self.behaviors.borrow_mut();','tag effect state')
    p.section(r,'    pub(crate) fn target_info(','    pub(crate) fn camera_distance_changed(','            info\n        })','            let murder_running=modules.query_running(&self.registry,"MurderMystery");\n            crate::murder_mystery_runtime::apply_tag(&mut modules.murder_mystery,murder_running,snapshot.abstract_client_player,snapshot.player_name.as_deref(),snapshot.murder_mystery_uuid.as_deref(),&mut info);\n            info\n        })','lazy source tag35 targeting and prefix')
    p.replace(r,'    #[cfg(feature="behavior-hooks")]\n    fn client_blink_esp(','    #[cfg(feature="behavior-hooks")]\n    fn client_murder_mystery(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientMurderMysteryEvent<\'_>){self.murder_mystery(host,event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_blink_esp(','native mutable actor hook')
    p.replace(r,'        self.auto_queue_world_change();','        self.auto_queue_world_change();\n        self.murder_mystery_reset();','world reset')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'blink-patch.py',","'blink-patch.py', 'murder-mystery-patch.py',",'native extension after Blink replay packet seam')
    return p
if __name__=='__main__':
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} MurderMystery changes across {len(p.text)} shared files')
    if args.apply:p.write();print('Integrated MurderMystery')
