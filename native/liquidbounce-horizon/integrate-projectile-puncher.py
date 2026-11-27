#!/usr/bin/env python3
"""Validate ProjectilePuncher shared wiring transactionally; root alone applies."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
E='native/liquidbounce-horizon/engine/src/'
def build_plan():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod offhand;','pub mod offhand;\npub mod combat_rotations;\npub mod projectile_puncher;','models')
 p.replace(E+'lib.rs','mod offhand_runtime;','mod offhand_runtime;\n#[cfg(feature="behavior-hooks")]\nmod projectile_puncher_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub offhand:crate::offhand_runtime::OffhandRuntime,','    pub offhand:crate::offhand_runtime::OffhandRuntime,\n    pub projectile_puncher:crate::projectile_puncher_runtime::ProjectilePuncherRuntime,','state')
 p.replace(b,'            "Offhand",','            "Offhand",\n            "ProjectilePuncher",','module registry')
 p.replace(b,'            offhand:Default::default(),','            offhand:Default::default(),\n            projectile_puncher:Default::default(),','default')
 p.replace(b,'            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>','            ModuleCallback::OnDisabled(id) if self.query_ids.get("ProjectilePuncher")==Some(&id)=>self.projectile_puncher.target=None,\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("FastExp")==Some(&id)=>','disable clears only target')
 p.replace(b,'            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),','            crate::projectile_puncher_runtime::ROTATION_PROVIDER=>self.query_running(registry,"ProjectilePuncher"),\n            crate::fast_exp_runtime::ROTATION_PROVIDER=>self.query_running(registry,"FastExp"),','provider lifecycle')
 c=E+'config.rs'
 p.replace(c,'pub use offhand::{OffhandSetting,serialize_offhand};','pub use offhand::{OffhandSetting,serialize_offhand};\n#[path="config/combat_rotations.rs"]\nmod combat_rotations;\n#[path="config/projectile_puncher.rs"]\nmod projectile_puncher;\npub use projectile_puncher::{ProjectilePuncherSetting,serialize_projectile_puncher};','config codecs')
 p.replace(c,'    Offhand => ("Offhand","Player","offhand"),','    Offhand => ("Offhand","Player","offhand"),\n    ProjectilePuncher=>("ProjectilePuncher","World","projectilePuncher"),','kind')
 p.replace(c,'    Offhand(OffhandSetting),','    Offhand(OffhandSetting),\n    ProjectilePuncher(ProjectilePuncherSetting),','setting requests',count=2)
 p.replace(c,'            | ModuleKind::Offhand','            | ModuleKind::Offhand\n            | ModuleKind::ProjectilePuncher','dedicated config')
 p.replace(c,'        ModuleKind::Offhand=>offhand::parse_offhand(&mut patch,&root),','        ModuleKind::Offhand=>offhand::parse_offhand(&mut patch,&root),\n        ModuleKind::ProjectilePuncher=>projectile_puncher::parse_projectile_puncher(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::Offhand(value)=>return ApplyResult::Runtime(RuntimeRequest::Offhand(value.clone())),','        SettingChange::Offhand(value)=>return ApplyResult::Runtime(RuntimeRequest::Offhand(value.clone())),\n        SettingChange::ProjectilePuncher(value)=>return ApplyResult::Runtime(RuntimeRequest::ProjectilePuncher(value.clone())),','runtime request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::Offhand=>config::serialize_offhand(&b.offhand.state.settings,&common,format,&describe),','            ModuleKind::Offhand=>config::serialize_offhand(&b.offhand.state.settings,&common,format,&describe),\n            ModuleKind::ProjectilePuncher=>config::serialize_projectile_puncher(&b.projectile_puncher,&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::Offhand(setting))=>','                ApplyResult::Runtime(RuntimeRequest::ProjectilePuncher(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().projectile_puncher){eprintln!("[LiquidBounce] ProjectilePuncher config: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::Offhand(setting))=>','live settings')
 p.replace(a,'ModuleKind::Offhand=>vec!["AutoTotem"],','ModuleKind::Offhand=>vec!["AutoTotem"], ModuleKind::ProjectilePuncher=>vec!["AntiFireball"],','source alias')
 r=E+'runtime.rs'
 p.replace(r,'        self.crosshair_first_tick(host); // FIRST_PRIORITY GameTick','        self.crosshair_first_tick(host); // FIRST_PRIORITY GameTick\n        self.projectile_puncher_first_tick();','click plan and Fail FIRST_PRIORITY')
 p.replace(r,'        self.no_web_prepare(host);','        self.no_web_prepare(host);\n        self.projectile_puncher_prepare(host);','RotationUpdate before manager')
 p.replace(r,'            let fast_exp_running=modules.query_running(&state.registry,"FastExp");','            let fast_exp_running=modules.query_running(&state.registry,"FastExp");\n            let projectile_puncher_running=modules.query_running(&state.registry,"ProjectilePuncher");','provider state')
 p.replace(r,"            // FIRST_PRIORITY GameTick consumes the previous tick's request.",'''            let projectile_previous_target=rotations.previous_rotation_target();
            let projectile_server=rotations.server_rotation(modules.criticals.is_lagging(),modules.query_running(&state.registry,"Freeze"));
            // FIRST_PRIORITY GameTick consumes the previous tick's request.''','Interpolation and Fail context')
 p.replace(r,'                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,','                        crate::fast_exp_runtime::ROTATION_PROVIDER=>fast_exp_running,\n                        crate::projectile_puncher_runtime::ROTATION_PROVIDER=>projectile_puncher_running,','provider dispatch')
 p.replace(r,'                        if processor == 0 {','''                        if let Some(id)=processor.checked_sub(crate::projectile_puncher_runtime::PROCESSOR_BASE).filter(|id|liquidbounce_behaviors::combat_rotations::Settings::decode(*id).is_some()){
                            modules.projectile_puncher.ai_notice_pending=liquidbounce_behaviors::combat_rotations::Settings::decode(id).is_some_and(|v|v.0==liquidbounce_behaviors::combat_rotations::SmoothMode::Ai);
                            return modules.projectile_puncher.rotations.process(id,projectile_puncher_running,from,to,last,Rotation::new(last_yaw,last_pitch),projectile_previous_target,projectile_server,&mut liquidbounce_behaviors::random::random_unit_f32).map_err(liquidbounce_behaviors::rotations::RotationError::Processor);
                        }
                        if processor == 0 {''','captured combat processor chain')
 p.replace(r,'        self.no_web_tick(host);','        self.projectile_puncher_ai_notification(host);\n        self.projectile_puncher_tick(host);\n        self.no_web_tick(host);','GameTick attack after manager')
 p.replace('native/liquidbounce-behaviors/src/rotations.rs','    pub fn reset_world(&mut self) {','    pub fn previous_rotation_target(&self)->Option<Rotation>{self.previous_target.map(|p|p.target.rotation)}\n    pub fn reset_world(&mut self) {','shared previous target getter')
 click='native/liquidbounce-behaviors/src/crosshair_clicker.rs'
 p.replace(click,'    fn key_click(&mut self, id: Id);','    fn key_click(&mut self, id: Id);\n    /// Notify running clickers outside this crosshair family after success.\n    fn click_success(&mut self,_id:Id) {}','external clicker coordination seam')
 p.replace(click,'        clicks = clicks.wrapping_add(1);','        host.click_success(id);\n        clicks = clicks.wrapping_add(1);','both way success coordination')
 click=E+'crosshair_clicker_runtime.rs'
 p.replace(click,'    fn crosshair_running(&self) -> [bool; 3] {','    pub(crate) fn crosshair_running(&self) -> [bool; 3] {','shared running states')
 p.replace(click,'    fn crosshair_cooldown(&self, host: &dyn Host) -> Option<(Cooldown, serde_json::Value)> {','    pub(crate) fn crosshair_cooldown(&self, host: &dyn Host) -> Option<(Cooldown, serde_json::Value)> {','shared live cooldown and miss clock')
 p.replace(click,'        clicker::player_press(','        self.projectile_puncher_player_press(attack,open,pressed);\n        clicker::player_press(','physical same mapping presses')
 p.replace(click,'    fn key_click(&mut self, id: Id) {','    fn click_success(&mut self,id:Id){self.hooks.projectile_puncher_other_click(id);}\n    fn key_click(&mut self, id: Id) {','crosshair success notification')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'projectile-puncher-patch.py', 'step-patch.py'):",'native patch after shared services')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} ProjectilePuncher changes across {len(p.text)} shared files')
 if args.apply:p.write();print('Integrated ProjectilePuncher')
