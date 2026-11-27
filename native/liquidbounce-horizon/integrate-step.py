#!/usr/bin/env python3
"""Transactional Step all-mode engine integration; --check changes no sources."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod high_jump;','pub mod high_jump;\npub mod step;')
 p.edit('lib.rs','mod high_jump_runtime;','mod high_jump_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod step_runtime;')
 p.edit('behaviors.rs','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,\n    pub step:std::sync::Arc<crate::step_runtime::StepService>,')
 p.edit('behaviors.rs','            "HighJump",','            "HighJump",\n            "Step",')
 p.edit('behaviors.rs','        let timer = std::sync::Arc::new(crate::timer_runtime::TimerService::default());','        let timer = std::sync::Arc::new(crate::timer_runtime::TimerService::default());\n        let step=std::sync::Arc::new(crate::step_runtime::StepService::new(timer.clone()));')
 p.edit('behaviors.rs','            high_jump: Default::default(),','            high_jump: Default::default(),\n            step,')
 p.edit('behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.step.set_running(self.query_running(registry,"Step"),self.query_running(registry,"Speed"));')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {','            ModuleCallback::OnDisabled(id) if self.query_ids.get("Step") == Some(&id) => { self.step.disabled(); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("HighJump") == Some(&id) => {')
 p.edit('coroutine_ticker.rs','    HighJump,','    HighJump,\n    Step,')
 p.edit('config.rs','pub use high_jump::{HighJumpSetting, serialize_high_jump};','pub use high_jump::{HighJumpSetting, serialize_high_jump};\n#[path="config/step.rs"]\nmod step;\npub use step::{StepSetting,serialize_step};')
 p.edit('config.rs','    HighJump => ("HighJump", "Movement", "highJump"),','    HighJump => ("HighJump", "Movement", "highJump"),\n    Step => ("Step","Movement","step"),')
 p.edit('config.rs','    HighJump(HighJumpSetting),','    HighJump(HighJumpSetting),\n    Step(StepSetting),',2)
 p.edit('config.rs','            | ModuleKind::HighJump','            | ModuleKind::HighJump\n            | ModuleKind::Step')
 p.edit('config.rs','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),\n        ModuleKind::Step => step::parse_step(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::HighJump(value) => {','        SettingChange::Step(value) => { return ApplyResult::Runtime(RuntimeRequest::Step(*value)); }\n        SettingChange::HighJump(value) => {')
 p.edit('actor.rs','            ModuleKind::HighJump => {','            ModuleKind::Step => config::serialize_step(&b.step.settings(),&common,format,&describe),\n            ModuleKind::HighJump => {')
 p.edit('actor.rs','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),\n            ModuleKind::Step => Some(self.state.behaviors.borrow().step.settings().mode.tag()),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Step(setting)) => {self.state.behaviors.borrow().step.update(|s|setting.apply(s));}\n                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {')
 p.edit('runtime.rs','            pipeline.add_listener(0, modules.timer.clone());','            pipeline.add_listener(0, modules.timer.clone());\n            pipeline.add_listener(-910,modules.step.clone());')
 p.edit('runtime.rs','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),','                WaitOwner::HighJump => self.high_jump_callback(host, in_game, task.callback),\n                WaitOwner::Step => self.step_callback(host,in_game,task.callback),')
 p.edit('runtime.rs','    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {','''    fn client_step_height(&self,event:&mut horizon_client::mod_api::ClientStepHeightEvent){self.state().behaviors.borrow().step.height(&mut event.height);}
    #[cfg(feature = "behavior-hooks")]
    fn client_step_success(&self,event:&mut horizon_client::mod_api::ClientStepSuccessEvent){*event.result=self.state().behaviors.borrow().step.success(event.adjusted_y,event.player_y);}
    #[cfg(feature = "behavior-hooks")]
    fn client_step_input(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientStepInputEvent){self.step_input([event.can_one,event.can_one_quarter,event.can_one_half],&mut event.jump,&mut event.effects);}
    #[cfg(feature = "behavior-hooks")]
    fn client_high_jump(&self, event: &mut horizon_client::mod_api::ClientHighJumpEvent) {''')
 p.edit('runtime.rs','        self.high_jump_tick(host);','        self.high_jump_tick(host);\n        self.step_tick(host);')
 p.edit('runtime.rs','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {\n        if event.dimension.is_none(){self.state().behaviors.borrow().step.clear();}')
 # Step must see every earlier adapter's MotionWorld struct literals.
 p.edit('../../make-behavior-patch.py',"'semantic-pointer-patch.py'):","'semantic-pointer-patch.py', 'step-patch.py'):")
 # Restrict the new argument to Step; earlier patch entry points stay intact.
 path=next(path for path in p.changed if path.resolve()==(HERE/'make-behavior-patch.py').resolve())
 old='    additions = module.apply(edit, replace_once)\n'
 source=p.changed[path]
 index=source.rfind(old)
 if index<0:raise ValueError('extension apply anchor missing')
 p.changed[path]=source[:index]+source[index:].replace(old,"    additions = module.apply(edit, replace_once, changes) if extension == 'step-patch.py' else module.apply(edit, replace_once)\n",1)
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();plan=prepare()
 if not args.check:plan.commit()
 print('Step '+('validated'if args.check else'integrated')+f': {len(plan.changed)} shared files')
