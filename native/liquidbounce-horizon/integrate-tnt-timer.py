#!/usr/bin/env python3
"""Transactional complete TNTTimer wiring; --check is read-only."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
Plan=runpy.run_path(str(NATIVE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod particles;','pub mod particles;\npub mod indicator;\npub mod tnt_timer;')
 p.edit('lib.rs','mod particles_runtime;','mod particles_runtime;\n#[cfg(feature="behavior-hooks")]\nmod tnt_timer_runtime;')
 p.edit('behaviors.rs','    pub particles: liquidbounce_behaviors::particles::Particles,','    pub particles: liquidbounce_behaviors::particles::Particles,\n    pub tnt_timer: liquidbounce_behaviors::tnt_timer::TNTTimer,')
 p.edit('behaviors.rs','            "Particles",','            "Particles",\n            "TNTTimer",')
 p.edit('behaviors.rs','            particles: Default::default(),','            particles: Default::default(),\n            tnt_timer:Default::default(),')
 p.edit('config.rs','pub use particles::{ParticlesSetting,serialize_particles};','pub use particles::{ParticlesSetting,serialize_particles};\n#[path="config/tnt_timer.rs"]\nmod tnt_timer;\npub use tnt_timer::{TNTTimerSetting,serialize_tnt_timer};')
 p.edit('config.rs','    Particles => ("Particles", "Render", "particles"),','    Particles => ("Particles", "Render", "particles"),\n    TNTTimer=>("TNTTimer","Render","tntTimer"),')
 p.edit('config.rs','    Particles(ParticlesSetting),','    Particles(ParticlesSetting),\n    TNTTimer(TNTTimerSetting),',2)
 p.edit('config.rs','            | ModuleKind::Particles','            | ModuleKind::Particles\n            | ModuleKind::TNTTimer')
 p.edit('config.rs','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),\n        ModuleKind::TNTTimer=>tnt_timer::parse_tnt_timer(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Particles(value) => {','        SettingChange::TNTTimer(value)=>return ApplyResult::Runtime(RuntimeRequest::TNTTimer(value.clone())),\n        SettingChange::Particles(value) => {')
 p.edit('actor.rs','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),\n            ModuleKind::TNTTimer=>config::serialize_tnt_timer(&b.tnt_timer,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::TNTTimer(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().tnt_timer),\n                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.edit('runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_tnt_timer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTntTimerEvent<\'_>){self.tnt_timer(host,event);}\n'+anchor)
 p.edit('../../make-behavior-patch.py',"'target-renderer-patch.py',","'target-renderer-patch.py', 'tnt-timer-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('TNTTimer '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
