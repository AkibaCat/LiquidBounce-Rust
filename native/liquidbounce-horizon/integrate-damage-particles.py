#!/usr/bin/env python3
"""Transactional DamageParticles wiring. All anchors and current contents validated before any write."""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2];E='native/liquidbounce-horizon/engine/src/'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();originals={};changes={}
 def read(path):
  if path not in changes:originals[path]=(ROOT/path).read_text(encoding='utf8');changes[path]=originals[path]
  return changes[path]
 def edit(path,old,new,count=1):
  text=read(path)
  if text.count(old)!=count:raise ValueError(f'{path}: expected {count} of {old[:80]!r}, got {text.count(old)}')
  changes[path]=text.replace(old,new)
 def absent(path,token):
  if token in read(path):raise ValueError(f'{path}: already contains {token}')
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod damage_particles;');absent(E+'behaviors.rs','    pub damage_particles:');absent(E+'config.rs','    DamageParticles =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod damage_particles;')
 edit(E+'lib.rs','mod breadcrumbs_runtime;','mod breadcrumbs_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod damage_particles_runtime;')
 edit(E+'behaviors.rs','    pub breadcrumbs:','    pub damage_particles: liquidbounce_behaviors::damage_particles::DamageParticles,\n    pub breadcrumbs:')
 edit(E+'behaviors.rs','            "Breadcrumbs",','            "Breadcrumbs",\n            "DamageParticles",')
 edit(E+'behaviors.rs','            breadcrumbs: Default::default(),','            damage_particles: Default::default(),\n            breadcrumbs: Default::default(),')
 edit(E+'config.rs','#[path = "config/breadcrumbs.rs"]','#[path = "config/damage_particles.rs"]\nmod damage_particles;\npub use damage_particles::{DamageParticlesSetting,serialize_damage_particles};\n#[path = "config/breadcrumbs.rs"]')
 edit(E+'config.rs','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),\n    DamageParticles => ("DamageParticles", "Render", "damageParticles"),')
 edit(E+'config.rs','    Breadcrumbs(BreadcrumbsSetting),','    Breadcrumbs(BreadcrumbsSetting),\n    DamageParticles(DamageParticlesSetting),',2)
 edit(E+'config.rs','            | ModuleKind::Breadcrumbs','            | ModuleKind::Breadcrumbs\n            | ModuleKind::DamageParticles')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::DamageParticles => damage_particles::parse_damage_particles(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::DamageParticles(value) => { return ApplyResult::Runtime(RuntimeRequest::DamageParticles(*value)); }\n        SettingChange::Damage(value) => {')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::DamageParticles => config::serialize_damage_particles(&b.damage_particles,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::DamageParticles(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().damage_particles); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
 edit(E+'behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {','            ModuleCallback::OnDisabled(id) if self.query_ids.get("DamageParticles") == Some(&id) => { self.damage_particles.clear(); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_damage_particles(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientDamageParticlesEvent<\'_>){self.damage_particles(host,event);}\n'+anchor)
 edit(E+'runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {","    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {\n        self.state().behaviors.borrow_mut().damage_particles.clear();")
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'damage-particles-patch.py'")
 edit(patch,"'semantic-pointer-patch.py'","'damage-particles-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if (ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
 if not a.check:
  for path,text in changes.items():(ROOT/path).write_text(text,encoding='utf8')
 print(f'{"Validated" if a.check else "Integrated"} DamageParticles across {len(changes)} shared files')
if __name__=='__main__':main()
