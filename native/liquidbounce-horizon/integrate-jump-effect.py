#!/usr/bin/env python3
"""Transactional JumpEffect wiring. Draft default --check is read-only; publish separately first."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
Plan=runpy.run_path(str(NATIVE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod particles;','pub mod particles;\npub mod jump_effect;')
 p.edit('lib.rs','mod particles_runtime;','mod particles_runtime;\n#[cfg(feature="behavior-hooks")]\nmod jump_effect_runtime;')
 p.edit('behaviors.rs','    pub particles: liquidbounce_behaviors::particles::Particles,','    pub particles: liquidbounce_behaviors::particles::Particles,\n    pub jump_effect: liquidbounce_behaviors::jump_effect::JumpEffect,')
 p.edit('behaviors.rs','            "Particles",','            "Particles",\n            "JumpEffect",')
 p.edit('behaviors.rs','            particles: Default::default(),','            particles: Default::default(),\n            jump_effect:Default::default(),')
 anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("Particles") == Some(&id) => { self.particles.clear(); }'
 p.edit('behaviors.rs',anchor,anchor+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("JumpEffect")==Some(&id)=>self.jump_effect.clear(),')
 p.edit('config.rs','pub use particles::{ParticlesSetting,serialize_particles};','pub use particles::{ParticlesSetting,serialize_particles};\n#[path="config/jump_effect.rs"]\nmod jump_effect;\npub use jump_effect::{JumpEffectSetting,serialize_jump_effect};')
 p.edit('config.rs','    Particles => ("Particles", "Render", "particles"),','    Particles => ("Particles", "Render", "particles"),\n    JumpEffect=>("JumpEffect","Render","jumpEffect"),')
 p.edit('config.rs','    Particles(ParticlesSetting),','    Particles(ParticlesSetting),\n    JumpEffect(JumpEffectSetting),',2)
 p.edit('config.rs','            | ModuleKind::Particles','            | ModuleKind::Particles\n            | ModuleKind::JumpEffect')
 p.edit('config.rs','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),\n        ModuleKind::JumpEffect=>jump_effect::parse_jump_effect(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Particles(value) => {','        SettingChange::JumpEffect(value)=>return ApplyResult::Runtime(RuntimeRequest::JumpEffect(value.clone())),\n        SettingChange::Particles(value) => {')
 p.edit('actor.rs','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),\n            ModuleKind::JumpEffect=>config::serialize_jump_effect(&b.jump_effect,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::JumpEffect(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().jump_effect),\n                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {')
 p.edit('actor.rs','            ModuleKind::Wings => Some("Lines"),','            ModuleKind::JumpEffect=>Some(if self.state.behaviors.borrow().jump_effect.active()=="Simple"{"Simple"}else{"Image"}),\n            ModuleKind::Wings => Some("Lines"),')
 p.edit('runtime.rs','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {\n        self.jump_effect_tick(); // ExpiringList FIRST_PRIORITY')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.edit('runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_jump_effect_jump(&self,event:&mut horizon_client::mod_api::ClientJumpEffectJumpEvent){self.jump_effect_jump(event.position);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_jump_effect(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientJumpEffectEvent<\'_>){self.jump_effect(event);}\n'+anchor)
 p.edit('../../make-behavior-patch.py',"'target-renderer-patch.py',","'target-renderer-patch.py', 'jump-effect-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('JumpEffect '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
