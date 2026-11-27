#!/usr/bin/env python3
"""Transactional VehicleControl wiring; validate all anchors before apply."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
E='native/liquidbounce-horizon/engine/src/'
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod vehicle_boost;','pub mod vehicle_boost;\npub mod vehicle_control;','behavior')
 p.replace(E+'lib.rs','mod no_web_runtime;','mod no_web_runtime;\n#[cfg(feature="behavior-hooks")]\nmod vehicle_control_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub vehicle_boost: liquidbounce_behaviors::vehicle_boost::VehicleBoost,','    pub vehicle_boost: liquidbounce_behaviors::vehicle_boost::VehicleBoost,\n    pub vehicle_control:liquidbounce_behaviors::vehicle_control::VehicleControl,','state')
 p.replace(E+'behaviors.rs','            "VehicleBoost",','            "VehicleBoost",\n            "VehicleControl",','registry')
 p.replace(E+'behaviors.rs','            vehicle_boost: Default::default(),','            vehicle_boost: Default::default(),\n            vehicle_control:Default::default(),','default')
 anchor='            ModuleCallback::OnEnabled(id) if self.query_ids.get("AntiCheatDetect") == Some(&id) => {'
 p.replace(E+'behaviors.rs',anchor,'''            ModuleCallback::OnEnabled(id) if self.query_ids.get("VehicleControl")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::VehicleControlEnabled);}
            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("VehicleControl")==Some(&id)=>{self.vehicle_control.cancel();}
'''+anchor,'lifecycle')
 p.replace(E+'one_shots_runtime.rs','    AntiCheatDetectEnabled,','    AntiCheatDetectEnabled,\n    VehicleControlEnabled,','effect')
 anchor='            OneShotEffect::AntiCheatDetectEnabled => self.anti_cheat_detect_enabled(host),'
 p.replace(E+'one_shots_runtime.rs',anchor,anchor+'\n            OneShotEffect::VehicleControlEnabled=>self.vehicle_control_enabled(host),','enabled chat')
 p.replace(E+'config.rs','pub use vehicle_boost::{VehicleBoostSetting, serialize_vehicle_boost};','pub use vehicle_boost::{VehicleBoostSetting, serialize_vehicle_boost};\n#[path="config/vehicle_control.rs"]\nmod vehicle_control;\npub use vehicle_control::{VehicleControlSetting,serialize_vehicle_control};','codec')
 p.replace(E+'config.rs','    VehicleBoost => ("VehicleBoost", "Movement", "vehicleBoost"),','    VehicleBoost => ("VehicleBoost", "Movement", "vehicleBoost"),\n    VehicleControl=>("VehicleControl","Movement","vehicleControl"),','kind')
 p.replace(E+'config.rs','    VehicleBoost(VehicleBoostSetting),','    VehicleBoost(VehicleBoostSetting),\n    VehicleControl(VehicleControlSetting),','requests',count=2)
 p.replace(E+'config.rs','            | ModuleKind::VehicleBoost','            | ModuleKind::VehicleBoost\n            | ModuleKind::VehicleControl','dedicated')
 anchor='        ModuleKind::VehicleBoost => vehicle_boost::parse_vehicle_boost(&mut patch, &root),'
 p.replace(E+'config.rs',anchor,anchor+'\n        ModuleKind::VehicleControl=>vehicle_control::parse_vehicle_control(&mut patch,&root),','parse')
 anchor='        SettingChange::VehicleBoost(value) => {'
 p.replace(E+'config.rs',anchor,'        SettingChange::VehicleControl(value)=>return ApplyResult::Runtime(RuntimeRequest::VehicleControl(value.clone())),\n'+anchor,'runtime setting')
 p.replace(E+'actor.rs','            ModuleKind::VehicleBoost => {','            ModuleKind::VehicleControl=>config::serialize_vehicle_control(&b.vehicle_control.settings,&common,format,&describe),\n            ModuleKind::VehicleBoost => {','serialize')
 p.replace(E+'actor.rs',' ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"],',' ModuleKind::VehicleControl=>vec!["BoatFly"], ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"],','alias')
 anchor='                ApplyResult::Runtime(RuntimeRequest::VehicleBoost(setting)) => {'
 p.replace(E+'actor.rs',anchor,'''                ApplyResult::Runtime(RuntimeRequest::VehicleControl(setting))=>{
                    let mut m=self.state.behaviors.borrow_mut();
                    if let Err(e)=setting.apply(&mut m.vehicle_control.settings){eprintln!("[LiquidBounce] VehicleControl config leaf skipped: {e}");}
                    if !m.vehicle_control.settings.rehook{m.vehicle_control.cancel();}
                }
'''+anchor,'setting')
 p.replace(E+'coroutine_ticker.rs','    NoWeb,','    NoWeb,\n    VehicleControl,','owner')
 anchor='                WaitOwner::NoWeb => self.no_web_callback(host,in_game,task.callback),'
 p.replace(E+'runtime.rs',anchor,anchor+'\n                WaitOwner::VehicleControl=>self.vehicle_control_callback(host,in_game,task.callback),','FIFO')
 anchor='        self.spear_kill_tick(host); // FINAL_DECISION (-500)'
 p.replace(E+'runtime.rs',anchor,'        self.vehicle_control_tick(host);\n'+anchor,'GameTick priority')
 anchor='''            serde_json::json!({"shift":twerk_shift,"finalShift":event.shift}),
        );
    }'''
 p.replace(E+'runtime.rs',anchor,'''            serde_json::json!({"shift":twerk_shift,"finalShift":event.shift}),
        );
        drop(state);
        self.vehicle_control_input(host,event);
    }''','MovementInput')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'vehicle-control-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');a=a.parse_args();p=prepare()
 if a.apply:p.write()
 print(f'VehicleControl {"integrated"if a.apply else"validated"}: {len(p.steps)} changes/{len(p.text)} shared files')
