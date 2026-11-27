#!/usr/bin/env python3
"""Transactional NoWeb engine integration; --check does not change shared sources."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod high_jump;','pub mod high_jump;\npub mod no_web;')
 p.edit('lib.rs','mod high_jump_runtime;','mod high_jump_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod no_web_runtime;')
 p.edit('behaviors.rs','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,\n    pub no_web:crate::no_web_runtime::NoWebRuntime,')
 p.edit('behaviors.rs','            "HighJump",','            "HighJump",\n            "NoWeb",')
 p.edit('behaviors.rs','            high_jump: Default::default(),','            high_jump: Default::default(),\n            no_web:Default::default(),')
 p.edit('behaviors.rs','            crate::auto_walk_runtime::ROTATION_PROVIDER => self.query_running(registry, "AutoWalk"),','            crate::no_web_runtime::ROTATION_PROVIDER => self.query_running(registry,"NoWeb"),\n            crate::auto_walk_runtime::ROTATION_PROVIDER => self.query_running(registry, "AutoWalk"),')
 p.edit('coroutine_ticker.rs','    HighJump,','    HighJump,\n    NoWeb,')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {','''            ModuleCallback::OnEnabled(id) if self.query_ids.get("NoWeb")==Some(&id)=>{let id=self.no_web.start();self.coroutine_ticker.register(crate::coroutine_ticker::WaitOwner::NoWeb,id);}
            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("NoWeb")==Some(&id)=>{self.no_web.cancel();self.pending_slot_resets.push(crate::no_web_runtime::REQUESTER);}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {''')
 p.edit('config.rs','pub use high_jump::{HighJumpSetting, serialize_high_jump};','pub use high_jump::{HighJumpSetting, serialize_high_jump};\n#[path="config/no_web.rs"]\nmod no_web;\npub use no_web::{NoWebSetting,serialize_no_web};')
 p.edit('config.rs','    HighJump => ("HighJump", "Movement", "highJump"),','    HighJump => ("HighJump", "Movement", "highJump"),\n    NoWeb => ("NoWeb","Movement","noWeb"),')
 p.edit('config.rs','    HighJump(HighJumpSetting),','    HighJump(HighJumpSetting),\n    NoWeb(NoWebSetting),',2)
 p.edit('config.rs','            | ModuleKind::HighJump','            | ModuleKind::HighJump\n            | ModuleKind::NoWeb')
 p.edit('config.rs','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),\n        ModuleKind::NoWeb => no_web::parse_no_web(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::HighJump(value) => {','        SettingChange::NoWeb(value) => { return ApplyResult::Runtime(RuntimeRequest::NoWeb(value.clone())); }\n        SettingChange::HighJump(value) => {')
 p.edit('actor.rs','            ModuleKind::HighJump => {','            ModuleKind::NoWeb => config::serialize_no_web(&b.no_web,&common,format,&describe),\n            ModuleKind::HighJump => {')
 p.edit('actor.rs','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),\n            ModuleKind::NoWeb => Some(self.state.behaviors.borrow().no_web.behavior.settings.mode.tag()),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::NoWeb(setting)) => {setting.apply(&mut self.state.behaviors.borrow_mut().no_web);}\n                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {')
 p.edit('runtime.rs','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),','                WaitOwner::NoWeb => self.no_web_callback(host,in_game,task.callback),\n                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),')
 p.edit('runtime.rs','    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {','''    fn client_no_web_collision(&self,event:&mut horizon_client::mod_api::ClientNoWebCollisionEvent){self.no_web_collision(event.pos,event.moving,event.ground,event.tick,event.result);}
    #[cfg(feature = "behavior-hooks")]
    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {''')
 p.edit('runtime.rs','        let reset = {\n            let mut state = self.state();','        self.no_web_prepare(host);\n        let reset = {\n            let mut state = self.state();')
 p.edit('runtime.rs','        if let Some(yaw) = reset {\n            host.client_rotation_reset_yaw(yaw);\n        }','        if let Some(yaw) = reset {\n            host.client_rotation_reset_yaw(yaw);\n        }\n        self.no_web_tick(host);')
 p.edit('runtime.rs','            let parkour_running = modules.query_running(&state.registry, "Parkour");','            let parkour_running = modules.query_running(&state.registry, "Parkour");\n            let no_web_running = modules.query_running(&state.registry,"NoWeb");')
 p.edit('runtime.rs','                        crate::auto_walk_runtime::ROTATION_PROVIDER => auto_walk_running,','                        crate::no_web_runtime::ROTATION_PROVIDER => no_web_running,\n                        crate::auto_walk_runtime::ROTATION_PROVIDER => auto_walk_running,')
 p.edit('runtime.rs','                        } else if let Some(mode) = crate::auto_walk_runtime::AutoWalkSettings::processor_mode(processor) {','                        } else if let Some(mode) = crate::no_web_runtime::NoWebRuntime::processor_mode(processor) {\n                            (&mut modules.no_web.rotations,mode)\n                        } else if let Some(mode) = crate::auto_walk_runtime::AutoWalkSettings::processor_mode(processor) {')
 p.edit('runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {","    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {\n        self.state().behaviors.borrow_mut().no_web.reset();\n        host.client_reset_silent_slot(crate::no_web_runtime::REQUESTER);")
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'no-web-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();p=prepare()
 if not args.check:p.commit()
 print('NoWeb '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
