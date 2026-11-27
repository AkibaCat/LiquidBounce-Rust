#!/usr/bin/env python3
"""Transactional NoSlow engine integration; --check leaves shared sources intact."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod high_jump;','pub mod high_jump;\npub mod no_slow;')
 p.edit('lib.rs','mod high_jump_runtime;','mod high_jump_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod no_slow_runtime;')
 p.edit('behaviors.rs','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,\n    pub no_slow:std::sync::Arc<crate::no_slow_runtime::NoSlowService>,')
 p.edit('behaviors.rs','            "HighJump",','            "HighJump",\n            "NoSlow",')
 p.edit('behaviors.rs','            high_jump: Default::default(),','            high_jump: Default::default(),\n            no_slow:Default::default(),')
 p.edit('behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.no_slow.set_running(self.query_running(registry,"NoSlow"));')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {','''            ModuleCallback::CancelSequences(id)|ModuleCallback::OnDisabled(id) if self.query_ids.get("NoSlow")==Some(&id)=>{self.no_slow.cancel();}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {''')
 p.edit('coroutine_ticker.rs','    HighJump,','    HighJump,\n    NoSlow,')
 p.edit('config.rs','pub use high_jump::{HighJumpSetting, serialize_high_jump};','pub use high_jump::{HighJumpSetting, serialize_high_jump};\n#[path="config/no_slow.rs"]\nmod no_slow;\npub use no_slow::{NoSlowSetting,serialize_no_slow};')
 p.edit('config.rs','    HighJump => ("HighJump", "Movement", "highJump"),','    HighJump => ("HighJump", "Movement", "highJump"),\n    NoSlow => ("NoSlow","Movement","noSlow"),')
 p.edit('config.rs','    HighJump(HighJumpSetting),','    HighJump(HighJumpSetting),\n    NoSlow(NoSlowSetting),',2)
 p.edit('config.rs','            | ModuleKind::HighJump','            | ModuleKind::HighJump\n            | ModuleKind::NoSlow')
 p.edit('config.rs','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),\n        ModuleKind::NoSlow => no_slow::parse_no_slow(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::HighJump(value) => {','        SettingChange::NoSlow(value) => { return ApplyResult::Runtime(RuntimeRequest::NoSlow(*value)); }\n        SettingChange::HighJump(value) => {')
 p.edit('actor.rs','            ModuleKind::HighJump => {','            ModuleKind::NoSlow => config::serialize_no_slow(&b.no_slow.settings(),&common,format,&describe),\n            ModuleKind::HighJump => {')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::NoSlow(setting)) => {self.state.behaviors.borrow().no_slow.update(|s|setting.apply(s));}\n                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {')
 p.edit('runtime.rs','            pipeline.add_listener(0,modules.sneak.clone());','''            pipeline.add_listener(0,modules.sneak.clone());
            pipeline.add_listener(0,modules.no_slow.clone());
            pipeline.add_listener(-500,std::sync::Arc::new(crate::no_slow_runtime::NoSlowBlink(modules.no_slow.clone())));
            pipeline.add_listener(-1000,std::sync::Arc::new(crate::no_slow_runtime::NoSlowFinal(modules.no_slow.clone())));''')
 p.edit('runtime.rs','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),','                WaitOwner::NoSlow => self.no_slow_callback(host,in_game,task.callback),\n                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),')
 p.edit('runtime.rs','    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {','''    fn client_no_slow_network(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientNoSlowNetworkEvent){self.no_slow_network(host,event.pre);}
    #[cfg(feature = "behavior-hooks")]
    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {''')
 p.edit('runtime.rs','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {\n        self.no_slow_tick(host);')
 p.edit('runtime.rs','        self.sneak_input(host,event);','        self.sneak_input(host,event);\n        self.no_slow_input(host,event);')
 p.edit('runtime.rs','self.freeze_frame(host);self.criticals_frame(host);','self.no_slow_frame(host);self.freeze_frame(host);self.criticals_frame(host);')
 p.edit('runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {","    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {\n        self.state().behaviors.borrow().no_slow.world_change();")
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'no-slow-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();p=prepare()
 if not args.check:p.commit()
 print('NoSlow '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
