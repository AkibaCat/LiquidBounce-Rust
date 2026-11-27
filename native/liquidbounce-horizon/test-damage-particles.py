"""One focused compile/smoke of production behavior/config/runtime and true native setter capture."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};use crate::Easing;\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{DamageParticles}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.damageParticles"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{DamageParticles(settings::DamageParticlesSetting)}\n#[derive(Default)]pub struct SettingsPatch{pub changes:Vec<SettingChange>,pub issues:Vec<ConfigError>}\n'
 config+=f'#[path="{(HERE/"engine/src/config/damage_particles.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='#[test]fn config_round_trip(){let s=crate::damage_particles::DamageParticles::default();let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_damage_particles(&s,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_damage_particles(&mut patch,&nodes(&tree,"DamageParticles","DamageParticles").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);assert_eq!(patch.changes.len(),10);let mut restored=s.clone();for SettingChange::DamageParticles(setting)in patch.changes{setting.apply(&mut restored);}assert_eq!(restored,s);}}\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 error=None;meta=None
 try:
  meta=runpy.run_path(str(HERE/'damage-particles-patch.py'))['apply'](edit,once)
  assert 'self.client_lb_set_health(self.health - (through));' in patched['crates/horizon-client/src/mobs.rs']
  assert 'self.client_lb_set_health(self.health - (damage));' in patched['crates/horizon-client/src/game/dragon.rs']
  assert sum(t.count('client_lb_set_health(') for t in patched.values())>=20
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e)
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_core;extern crate self as horizon_world;\n'
 for name in ['numeric','easing','damage_particles']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 lib+='pub use easing::Easing;mod config;\n'
 lib+='''pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientDamageParticlesEvent<'a>{pub input:&'a str,pub draws:&'a mut String}}
pub mod runtime{use std::{rc::Rc,cell::RefCell};pub struct Modules{pub damage_particles:crate::damage_particles::DamageParticles}impl Modules{pub fn query_running(&self,_:&(),_:&str)->bool{true}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{damage_particles:Default::default()}),registry:()})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
pub mod geometry{#[derive(Clone,Copy,Default)]pub struct Vec3{pub x:f64,pub y:f64,pub z:f64}pub struct Bounds;impl Bounds{pub fn center(&self)->Vec3{Vec3::default()}}}
pub mod motion{use crate::geometry::*;pub struct Health{pub tick_count:i32}pub struct PlayerMotion{pub health:Health,pub velocity:Vec3}impl Default for PlayerMotion{fn default()->Self{Self{health:Health{tick_count:1},velocity:Vec3::default()}}}impl PlayerMotion{pub fn bounds(&self)->Bounds{Bounds}}}
pub mod mobs{use crate::geometry::*;pub struct Mob{pub age:u32,pub health:f32,pub velocity:Vec3}impl Mob{pub fn bounds(&self)->Bounds{Bounds}}pub mod finalize{pub fn max_health(_:&super::Mob)->f32{20.}}}
pub mod entities{use crate::geometry::*;pub struct ArmorStand{pub first_tick:bool,pub health:f32,pub velocity:Vec3}impl ArmorStand{pub fn bounds(&self)->Bounds{Bounds}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/damage_particles_runtime.rs").as_posix()}"]mod damage_particles_runtime;\n'
 lib+=f'#[path="{(HERE/"damage-particles-capture.rs").as_posix()}"]pub mod damage_particles_capture;\n'
 if not error:lib+=f'#[path="{(out/"patched/crates/horizon-world/src/living_health.rs").as_posix()}"]pub mod living_health;\n'
 lib+=(HERE/'damage-particles-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="damage-particles-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-damage-particles-locales.py'))
 for language in ['zh_cn','zh_tw']:target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':meta,'patchedFiles':len(patched),'scope':'Production behavior, engine runtime/config and native capture compiled with declaration-only Host/registry/native entity providers; actual shared Tree/parsers and patched living_health setter. One focused Cargo smoke, native patch composed in memory, copied Chinese dictionaries. Actual game build and visual validation remain aggregate.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
