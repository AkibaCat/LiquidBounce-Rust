#!/usr/bin/env python3
"""Transactional TotemEffect wiring. Draft default --check is read-only; publish separately first."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
Plan=runpy.run_path(str(NATIVE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod particles;','pub mod particles;\npub mod totem_effect;')
 p.edit('lib.rs','mod particles_runtime;','mod particles_runtime;\n#[cfg(feature="behavior-hooks")]\nmod totem_effect_runtime;')
 p.edit('behaviors.rs','    pub particles: liquidbounce_behaviors::particles::Particles,','    pub particles: liquidbounce_behaviors::particles::Particles,\n    pub totem_effect: liquidbounce_behaviors::totem_effect::TotemEffect,')
 p.edit('behaviors.rs','            "Particles",','            "Particles",\n            "TotemEffect",')
 p.edit('behaviors.rs','            particles: Default::default(),','            particles: Default::default(),\n            totem_effect:Default::default(),')
 anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("Particles") == Some(&id) => { self.particles.clear(); }'
 p.edit('behaviors.rs',anchor,anchor+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("TotemEffect")==Some(&id)=>self.totem_effect.clear(),')
 p.edit('config.rs','pub use particles::{ParticlesSetting,serialize_particles};','pub use particles::{ParticlesSetting,serialize_particles};\n#[path="config/totem_effect.rs"]\nmod totem_effect;\npub use totem_effect::{TotemEffectSetting,serialize_totem_effect};')
 p.edit('config.rs','    Particles => ("Particles", "Render", "particles"),','    Particles => ("Particles", "Render", "particles"),\n    TotemEffect=>("TotemEffect","Render","totemEffect"),')
 p.edit('config.rs','    Particles(ParticlesSetting),','    Particles(ParticlesSetting),\n    TotemEffect(TotemEffectSetting),',2)
 p.edit('config.rs','            | ModuleKind::Particles','            | ModuleKind::Particles\n            | ModuleKind::TotemEffect')
 p.edit('config.rs','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),\n        ModuleKind::TotemEffect=>totem_effect::parse_totem_effect(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Particles(value) => {','        SettingChange::TotemEffect(value)=>return ApplyResult::Runtime(RuntimeRequest::TotemEffect(value.clone())),\n        SettingChange::Particles(value) => {')
 p.edit('actor.rs','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),\n            ModuleKind::TotemEffect=>config::serialize_totem_effect(&b.totem_effect,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::TotemEffect(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().totem_effect),\n                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {')
 p.edit('actor.rs','            ModuleKind::Wings => Some("Lines"),','            ModuleKind::TotemEffect=>Some(if self.state.behaviors.borrow().totem_effect.active()=="Shockwave"{"Shockwave"}else{"Soul"}),\n            ModuleKind::Wings => Some("Lines"),')
 p.edit('runtime.rs','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {\n        self.totem_effect_tick(); // ExpiringList FIRST_PRIORITY')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.edit('runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_totem_effect(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTotemEffectEvent<\'_>){self.totem_effect(host,event);}\n'+anchor)
 p.edit('../../make-behavior-patch.py',"'target-renderer-patch.py',","'target-renderer-patch.py', 'totem-effect-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('TotemEffect '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
