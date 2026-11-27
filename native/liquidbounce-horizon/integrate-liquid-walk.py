#!/usr/bin/env python3
"""Transactional LiquidWalk engine registration; --check leaves shared files intact."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod high_jump;','pub mod high_jump;\npub mod liquid_walk;')
 p.edit('lib.rs','mod high_jump_runtime;','mod high_jump_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod liquid_walk_runtime;')
 p.edit('behaviors.rs','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,\n    pub liquid_walk:std::sync::Arc<crate::liquid_walk_runtime::LiquidWalkService>,')
 p.edit('behaviors.rs','            "HighJump",','            "HighJump",\n            "LiquidWalk",')
 p.edit('behaviors.rs','        let timer = std::sync::Arc::new(crate::timer_runtime::TimerService::default());','        let timer = std::sync::Arc::new(crate::timer_runtime::TimerService::default());\n        let liquid_walk=std::sync::Arc::new(crate::liquid_walk_runtime::LiquidWalkService::new(timer.clone()));')
 p.edit('behaviors.rs','            high_jump: Default::default(),','            high_jump: Default::default(),\n            liquid_walk,')
 p.edit('behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.liquid_walk.set_running(self.query_running(registry,"LiquidWalk"));')
 p.edit('config.rs','pub use high_jump::{HighJumpSetting, serialize_high_jump};','pub use high_jump::{HighJumpSetting, serialize_high_jump};\n#[path="config/liquid_walk.rs"]\nmod liquid_walk;\npub use liquid_walk::{LiquidWalkSetting,serialize_liquid_walk};')
 p.edit('config.rs','    HighJump => ("HighJump", "Movement", "highJump"),','    HighJump => ("HighJump", "Movement", "highJump"),\n    LiquidWalk => ("LiquidWalk","Movement","liquidWalk"),')
 p.edit('config.rs','    HighJump(HighJumpSetting),','    HighJump(HighJumpSetting),\n    LiquidWalk(LiquidWalkSetting),',2)
 p.edit('config.rs','            | ModuleKind::HighJump','            | ModuleKind::HighJump\n            | ModuleKind::LiquidWalk')
 p.edit('config.rs','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),\n        ModuleKind::LiquidWalk => liquid_walk::parse_liquid_walk(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::HighJump(value) => {','        SettingChange::LiquidWalk(value) => { return ApplyResult::Runtime(RuntimeRequest::LiquidWalk(*value)); }\n        SettingChange::HighJump(value) => {')
 p.edit('actor.rs','            ModuleKind::HighJump => {','            ModuleKind::LiquidWalk => config::serialize_liquid_walk(&b.liquid_walk.settings(),&common,format,&describe),\n            ModuleKind::HighJump => {')
 p.edit('actor.rs','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),\n            ModuleKind::LiquidWalk => Some(self.state.behaviors.borrow().liquid_walk.settings().mode.tag()),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::LiquidWalk(setting)) => {self.state.behaviors.borrow().liquid_walk.update(|s|setting.apply(s));}\n                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {')
 p.edit('runtime.rs','            pipeline.add_listener(0, modules.timer.clone());','            pipeline.add_listener(0, modules.timer.clone());\n            pipeline.add_listener(0,modules.liquid_walk.clone());')
 p.edit('runtime.rs','        self.high_jump_tick(host);','        self.high_jump_tick(host);\n        self.liquid_walk_tick(host);')
 p.edit('runtime.rs','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {\n        if event.dimension.is_none(){self.state().behaviors.borrow().liquid_walk.clear();}')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'liquid-walk-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();plan=prepare()
 if not args.check:plan.commit()
 print('LiquidWalk '+('validated'if args.check else'integrated')+f': {len(plan.changed)} shared files')
