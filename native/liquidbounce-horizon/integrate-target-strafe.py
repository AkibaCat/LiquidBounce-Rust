#!/usr/bin/env python3
"""Idempotent TargetStrafe shared wiring; --check never writes shared files."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-block-esp.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod aimbot;','pub mod aimbot;\npub mod target_strafe;')
 p.edit('lib.rs','mod aimbot_runtime;','mod aimbot_runtime;\n#[cfg(feature="behavior-hooks")]\nmod target_strafe_runtime;')
 p.edit('behaviors.rs','    pub aimbot:crate::aimbot_runtime::AimbotRuntime,','    pub aimbot:crate::aimbot_runtime::AimbotRuntime,\n    pub target_strafe:crate::target_strafe_runtime::TargetStrafeRuntime,')
 p.edit('behaviors.rs','            aimbot:Default::default(),','            aimbot:Default::default(),\n            target_strafe:Default::default(),')
 p.edit('behaviors.rs','            "Aimbot",','            "Aimbot",\n            "TargetStrafe",')
 p.edit('config.rs','pub use aimbot::{AimbotSetting,serialize_aimbot};','pub use aimbot::{AimbotSetting,serialize_aimbot};\n#[path="config/target_strafe.rs"]\nmod target_strafe;\npub use target_strafe::{TargetStrafeSetting,serialize_target_strafe};')
 p.edit('config.rs','    Aimbot=>("Aimbot","Combat","aimbot"),','    Aimbot=>("Aimbot","Combat","aimbot"),\n    TargetStrafe=>("TargetStrafe","Movement","targetStrafe"),')
 p.edit('config.rs','    Aimbot(AimbotSetting),','    Aimbot(AimbotSetting),\n    TargetStrafe(TargetStrafeSetting),',2)
 p.edit('config.rs','            | ModuleKind::Aimbot','            | ModuleKind::Aimbot\n            | ModuleKind::TargetStrafe')
 p.edit('config.rs','        ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),','        ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),\n        ModuleKind::TargetStrafe=>target_strafe::parse_target_strafe(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Aimbot(value)=>return ApplyResult::Runtime(RuntimeRequest::Aimbot(value.clone())),','        SettingChange::Aimbot(value)=>return ApplyResult::Runtime(RuntimeRequest::Aimbot(value.clone())),\n        SettingChange::TargetStrafe(value)=>return ApplyResult::Runtime(RuntimeRequest::TargetStrafe(value.clone())),')
 p.edit('actor.rs','            ModuleKind::Aimbot=>config::serialize_aimbot(&b.aimbot,&common,format,&describe),','            ModuleKind::Aimbot=>config::serialize_aimbot(&b.aimbot,&common,format,&describe),\n            ModuleKind::TargetStrafe=>config::serialize_target_strafe(&b.target_strafe.settings,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Aimbot(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::TargetStrafe(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().target_strafe.settings){eprintln!("[LiquidBounce] TargetStrafe config: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::Aimbot(setting))=>{')
 p.edit('runtime.rs','        self.scaffold_input_model(host,event);','        self.target_strafe_input(host,event);\n        self.scaffold_input_model(host,event);')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'target-strafe-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--check',action='store_true');a=ap.parse_args();p=prepare()
 if not a.check:p.commit()
 print(f"TargetStrafe {'validated'if a.check else'integrated'}: {len(p.changed)} shared files")
