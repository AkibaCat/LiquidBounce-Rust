#!/usr/bin/env python3
"""Transactional complete PotionFX wiring; --check is read-only."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
Plan=runpy.run_path(str(NATIVE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod particles;','pub mod particles;\npub mod potion_fx;')
 p.edit('lib.rs','mod particles_runtime;','mod particles_runtime;\n#[cfg(feature="behavior-hooks")]\nmod potion_fx_runtime;')
 p.edit('behaviors.rs','    pub particles: liquidbounce_behaviors::particles::Particles,','    pub particles: liquidbounce_behaviors::particles::Particles,\n    pub potion_fx: liquidbounce_behaviors::potion_fx::PotionFX,')
 p.edit('behaviors.rs','            "Particles",','            "Particles",\n            "PotionFX",')
 p.edit('behaviors.rs','            particles: Default::default(),','            particles: Default::default(),\n            potion_fx:Default::default(),')
 anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("Particles") == Some(&id) => { self.particles.clear(); }'
 p.edit('behaviors.rs',anchor,anchor+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("PotionFX")==Some(&id)=>self.potion_fx.clear(),')
 p.edit('config.rs','pub use particles::{ParticlesSetting,serialize_particles};','pub use particles::{ParticlesSetting,serialize_particles};\n#[path="config/potion_fx.rs"]\nmod potion_fx;\npub use potion_fx::{PotionFXSetting,serialize_potion_fx};')
 p.edit('config.rs','    Particles => ("Particles", "Render", "particles"),','    Particles => ("Particles", "Render", "particles"),\n    PotionFX=>("PotionFX","Render","potionFX"),')
 p.edit('config.rs','    Particles(ParticlesSetting),','    Particles(ParticlesSetting),\n    PotionFX(PotionFXSetting),',2)
 p.edit('config.rs','            | ModuleKind::Particles','            | ModuleKind::Particles\n            | ModuleKind::PotionFX')
 p.edit('config.rs','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),','        ModuleKind::Particles => particles::parse_particles(&mut patch,&root),\n        ModuleKind::PotionFX=>potion_fx::parse_potion_fx(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Particles(value) => {','        SettingChange::PotionFX(value)=>return ApplyResult::Runtime(RuntimeRequest::PotionFX(value.clone())),\n        SettingChange::Particles(value) => {')
 p.edit('actor.rs','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),','            ModuleKind::Particles => config::serialize_particles(&b.particles.settings,&common,format,&describe),\n            ModuleKind::PotionFX=>config::serialize_potion_fx(&b.potion_fx,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::PotionFX(setting))=>setting.apply(&mut self.state.behaviors.borrow_mut().potion_fx),\n                ApplyResult::Runtime(RuntimeRequest::Particles(setting)) => {')
 p.edit('runtime.rs','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {','    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {\n        self.potion_fx_tick(); // ExpiringList FIRST_PRIORITY')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.edit('runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_potion_fx(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientPotionFxEvent<\'_>){self.potion_fx(host,event);}\n'+anchor)
 p.edit('../../make-behavior-patch.py',"'target-renderer-patch.py',","'target-renderer-patch.py', 'potion-fx-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('PotionFX '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
