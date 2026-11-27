"""Transactional Particles shared wiring; root applies serially with --apply."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod crosshair;','pub mod crosshair;\npub mod particles;','behavior export')
 p.replace(E+'lib.rs','mod crosshair_runtime;','mod crosshair_runtime;\n#[cfg(feature="behavior-hooks")]\nmod particles_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub crosshair:','    pub particles: liquidbounce_behaviors::particles::Particles,\n    pub crosshair:','state')
 p.replace(E+'behaviors.rs','            crosshair: Default::default(),','            particles: Default::default(),\n            crosshair: Default::default(),','default')
 p.replace(E+'behaviors.rs','            "Crosshair",','            "Crosshair",\n            "Particles",','registration')
 anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("DamageParticles") == Some(&id) => { self.damage_particles.clear(); }'
 p.replace(E+'behaviors.rs',anchor,anchor+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("Particles") == Some(&id) => { self.particles.clear(); }','disable clear')
 p.replace(E+'config.rs','#[path = "config/crosshair.rs"]','#[path = "config/particles.rs"]\nmod particles;\npub use particles::{ParticlesSetting,serialize_particles};\n#[path = "config/crosshair.rs"]','config export')
 p.replace(E+'config.rs','    Crosshair => ("Crosshair", "Render", "crosshair"),','    Crosshair => ("Crosshair", "Render", "crosshair"),\n    Particles => ("Particles", "Render", "particles"),','kind')
 p.replace(E+'config.rs','    Crosshair(CrosshairSetting),','    Crosshair(CrosshairSetting),\n    Particles(ParticlesSetting),','request and setting',2)
 p.replace(E+'config.rs','            | ModuleKind::Crosshair','            | ModuleKind::Crosshair\n            | ModuleKind::Particles','supported config')
 p.replace(E+'config.rs','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),\n        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),','parser')
 p.replace(E+'config.rs','        SettingChange::Crosshair(value) => {','        SettingChange::Particles(value) => { return ApplyResult::Runtime(RuntimeRequest::Particles(value.clone())); }\n        SettingChange::Crosshair(value) => {','apply')
 p.replace(E+'actor.rs','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),\n            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),','serializer')
 p.replace(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().particles.settings); }\n                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','actor setter')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.replace(E+'runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_particles(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientParticlesEvent<\'_>){self.particles(host,event);}\n'+anchor,'hook')
 anchor="    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {"
 p.replace(E+'runtime.rs',anchor,anchor+'\n        self.state().behaviors.borrow_mut().particles.clear();','world clear')
 p.replace(E+'runtime.rs','        self.criticals_attack_sprint(host);','        self.particles_attack(host,event.target as i32);\n        self.criticals_attack_sprint(host);','attack handler')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'crosshair-patch.py'","'crosshair-patch.py', 'particles-patch.py'",'native patch')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');p=a.parse_args();plan=prepare();print(f'Validated {len(plan.steps)} Particles changes across {len(plan.text)} shared files')
 if p.apply:plan.write();print('Integrated Particles')
