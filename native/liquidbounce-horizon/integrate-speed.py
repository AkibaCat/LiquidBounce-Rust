#!/usr/bin/env python3
"""Idempotently wire the complete ModuleSpeed model into the shared engine."""
from pathlib import Path
import argparse, runpy
HERE = Path(__file__).resolve().parent
Base = runpy.run_path(str(HERE / 'integrate-target-strafe.py'))['Plan']

def prepare():
    p = Base()
    p.edit('../../../liquidbounce-behaviors/src/lib.rs', 'pub mod strafe;', 'pub mod strafe;\npub mod speed;')
    p.edit('lib.rs', 'mod target_strafe_runtime;', 'mod target_strafe_runtime;\n#[cfg(feature="behavior-hooks")]\nmod speed_runtime;')
    p.edit('behaviors.rs', '    pub target_strafe:crate::target_strafe_runtime::TargetStrafeRuntime,', '    pub target_strafe:crate::target_strafe_runtime::TargetStrafeRuntime,\n    pub speed: crate::speed_runtime::SpeedRuntime,')
    p.edit('behaviors.rs', '            target_strafe:Default::default(),', '            target_strafe:Default::default(),\n            speed:Default::default(),')
    p.edit('behaviors.rs', '            "TargetStrafe",', '            "TargetStrafe",\n            "Speed",')
    p.edit('config.rs', 'pub use target_strafe::{TargetStrafeSetting,serialize_target_strafe};', 'pub use target_strafe::{TargetStrafeSetting,serialize_target_strafe};\n#[path="config/speed.rs"]\nmod speed;\npub use speed::{SpeedSetting,serialize_speed};')
    p.edit('config.rs', '    TargetStrafe=>("TargetStrafe","Movement","targetStrafe"),', '    TargetStrafe=>("TargetStrafe","Movement","targetStrafe"),\n    Speed=>("Speed","Movement","speed"),')
    p.edit('config.rs', '    TargetStrafe(TargetStrafeSetting),', '    TargetStrafe(TargetStrafeSetting),\n    Speed(SpeedSetting),', 2)
    p.edit('config.rs', '            | ModuleKind::TargetStrafe', '            | ModuleKind::TargetStrafe\n            | ModuleKind::Speed')
    p.edit('config.rs', '        ModuleKind::TargetStrafe=>target_strafe::parse_target_strafe(&mut patch,&root),', '        ModuleKind::TargetStrafe=>target_strafe::parse_target_strafe(&mut patch,&root),\n        ModuleKind::Speed=>speed::parse_speed(&mut patch,&root),')
    p.edit('config.rs', '        SettingChange::TargetStrafe(value)=>return ApplyResult::Runtime(RuntimeRequest::TargetStrafe(value.clone())),', '        SettingChange::TargetStrafe(value)=>return ApplyResult::Runtime(RuntimeRequest::TargetStrafe(value.clone())),\n        SettingChange::Speed(value)=>return ApplyResult::Runtime(RuntimeRequest::Speed(value.clone())),')
    p.edit('actor.rs', '            ModuleKind::TargetStrafe=>config::serialize_target_strafe(&b.target_strafe.settings,&common,format,&describe),', '            ModuleKind::TargetStrafe=>config::serialize_target_strafe(&b.target_strafe.settings,&common,format,&describe),\n            ModuleKind::Speed=>config::serialize_speed(&b.speed.settings,&common,format,&describe),')
    p.edit('actor.rs', '                ApplyResult::Runtime(RuntimeRequest::TargetStrafe(setting))=>{', '                ApplyResult::Runtime(RuntimeRequest::Speed(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().speed.settings){eprintln!("[LiquidBounce] Speed config: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::TargetStrafe(setting))=>{')
    p.edit('runtime.rs', '        self.target_strafe_input(host,event);', '        self.speed_input(host,event);\n        self.target_strafe_input(host,event);')
    p.edit('runtime.rs', '        self.scaffold_tick(host);', '        self.scaffold_tick(host);\n        self.speed_tick(host);')
    p.edit('runtime.rs', '        modules.air_jump.player_jump(running, event.on_ground);', '        modules.air_jump.player_jump(running, event.on_ground);\n        self.speed_jump(host,event);')
    p.edit('../../make-behavior-patch.py', "'target-strafe-patch.py', 'step-patch.py')", "'target-strafe-patch.py', 'speed-patch.py', 'step-patch.py')")
    return p

if __name__ == '__main__':
    ap = argparse.ArgumentParser(); ap.add_argument('--check', action='store_true'); args = ap.parse_args()
    p = prepare()
    if not args.check: p.commit()
    print(f"Speed {'validated' if args.check else 'integrated'}: {len(p.changed)} shared files")
