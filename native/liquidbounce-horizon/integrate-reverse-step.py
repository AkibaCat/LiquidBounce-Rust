#!/usr/bin/env python3
"""Transactional ReverseStep engine integration; --check leaves sources intact."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-auto-walk.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod high_jump;','pub mod high_jump;\npub mod reverse_step;')
 p.edit('lib.rs','mod high_jump_runtime;','mod high_jump_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod reverse_step_runtime;')
 p.edit('behaviors.rs','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,','    pub high_jump: liquidbounce_behaviors::high_jump::HighJump,\n    pub reverse_step: liquidbounce_behaviors::reverse_step::Settings,')
 p.edit('behaviors.rs','            "HighJump",','            "HighJump",\n            "ReverseStep",')
 p.edit('behaviors.rs','            high_jump: Default::default(),','            high_jump: Default::default(),\n            reverse_step: Default::default(),')
 p.edit('config.rs','pub use high_jump::{HighJumpSetting, serialize_high_jump};','pub use high_jump::{HighJumpSetting, serialize_high_jump};\n#[path = "config/reverse_step.rs"]\nmod reverse_step;\npub use reverse_step::{ReverseStepSetting,serialize_reverse_step};')
 p.edit('config.rs','    HighJump => ("HighJump", "Movement", "highJump"),','    HighJump => ("HighJump", "Movement", "highJump"),\n    ReverseStep => ("ReverseStep", "Movement", "reverseStep"),')
 p.edit('config.rs','    HighJump(HighJumpSetting),','    HighJump(HighJumpSetting),\n    ReverseStep(ReverseStepSetting),',2)
 p.edit('config.rs','            | ModuleKind::HighJump','            | ModuleKind::HighJump\n            | ModuleKind::ReverseStep')
 p.edit('config.rs','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),','        ModuleKind::HighJump => high_jump::parse_high_jump(&mut patch, &root),\n        ModuleKind::ReverseStep => reverse_step::parse_reverse_step(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::HighJump(value) => {','        SettingChange::ReverseStep(value) => { return ApplyResult::Runtime(RuntimeRequest::ReverseStep(*value)); }\n        SettingChange::HighJump(value) => {')
 p.edit('actor.rs','            ModuleKind::HighJump => {','            ModuleKind::ReverseStep => config::serialize_reverse_step(&b.reverse_step,&common,format,&describe),\n            ModuleKind::HighJump => {')
 p.edit('actor.rs','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),','            ModuleKind::HighJump => Some(self.state.behaviors.borrow().high_jump.settings.mode.tag()),\n            ModuleKind::ReverseStep => Some(self.state.behaviors.borrow().reverse_step.mode.tag()),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::ReverseStep(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().reverse_step); }\n                ApplyResult::Runtime(RuntimeRequest::HighJump(setting)) => {')
 p.edit('runtime.rs','        self.high_jump_tick(host);','        self.high_jump_tick(host);\n        self.reverse_step_tick(host);')
 p.edit('../../make-behavior-patch.py',"'movement-guard-patch.py', 'freeze-patch.py',","'movement-guard-patch.py', 'freeze-patch.py', 'reverse-step-patch.py',")
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();plan=prepare()
 if not args.check:plan.commit()
 print('ReverseStep '+('validated' if args.check else 'integrated')+f': {len(plan.changed)} shared files')
