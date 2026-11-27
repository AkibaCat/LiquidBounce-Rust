"""Prepare one batch harness without compiling any module individually."""
from pathlib import Path
import argparse,json,runpy
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 source='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\npub mod clicker_math{pub mod fdlibm{pub fn log(v:f64)->f64{v.ln()}}}\n'
 for name in ['clock','timed_pickup_tracker','extinguish','auto_shoot','fake_lag','auto_bow','auto_bow_math','point_tracker','random','clicking','rotations','no_rotate_set','target_renderer']:
  source+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 rod=(HERE/'engine/src/auto_rod_runtime.rs').read_text(encoding='utf8')
 samples="pub struct NativeSamples<'a>(pub &'a mut JavaRandom);\n"+body(rod,"impl Samples for NativeSamples<'_>")
 source+='mod auto_rod_runtime{use crate::{clicking::JavaRandom,point_tracker::Samples,random};'+samples+'}\n'
 shoot=(HERE/'engine/src/auto_shoot_runtime.rs').read_text(encoding='utf8')
 struct=body(shoot,'pub struct AutoShootRuntime');default=body(shoot,'impl Default for AutoShootRuntime')
 source+='mod auto_shoot_runtime{use crate::{auto_shoot::AutoShoot,clicking::{Clicker,JavaRandom,Math,Settings as ClickSettings},point_tracker::PointTracker,no_rotate_set::RotationsSettings,target_renderer::{TargetRendererRuntime,TargetRendererSettings},auto_rod_runtime::NativeSamples};fn click_math()->Math{Math::default()}'+struct+default+'}\n'
 ext=(HERE/'engine/src/extinguish_runtime.rs').read_text(encoding='utf8')
 source+='mod extinguish_runtime{use crate::{extinguish::Extinguish,no_rotate_set::RotationsSettings};use serde_json::Value;#[derive(Default)]'+body(ext,'pub struct ExtinguishRuntime')+'}\n'
 shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 cfg='use std::{fmt,collections::BTreeMap};use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Default)]enum ModuleKind{#[default]NoRotateSet,Extinguish,AutoShoot}impl ModuleKind{fn key(self)->&\'static str{match self{Self::NoRotateSet=>"liquidbounce.module.noRotateSet",Self::Extinguish=>"liquidbounce.module.extinguish",Self::AutoShoot=>"liquidbounce.module.autoShoot"}}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(',"fn pair_members<'a>(",'fn parse_int_pair(','fn parse_float_pair(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':cfg+='#[derive(Debug)]\n'
  cfg+=body(shared,anchor)+'\n'
 cfg+=body(shared,'fn float_range(')+'\n'
 cfg+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;enum SettingChange{NoRotateSet(NoRotateSetSetting),Extinguish(extinguish_config::ExtinguishSetting),AutoShoot(auto_shoot_config::AutoShootSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 for name,file in [('no_rotate_set','no_rotate_set'),('target_renderer','target_renderer'),('extinguish_config','extinguish'),('auto_shoot_config','auto_shoot')]:
  cfg+=f'#[path="{(HERE/f"engine/src/config/{file}.rs").as_posix()}"]mod {name};\n'
 cfg+='use no_rotate_set::{NoRotateSetSetting,serialize_no_rotate_set};\n'
 cross=(HERE/'engine/src/config/crosshair_clicker.rs').read_text(encoding='utf8')
 cfg+='mod crosshair_clicker{use super::*;use crate::clicking::{Settings as ClickSettings,Technique};'+body(cross,'fn choice(')+body(cross,'fn click_tree(').replace('fn click_tree(','pub(super) fn click_tree(',1)+'}\n'
 cfg+='''pub fn check(){
let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};
let mut extinguish=crate::extinguish_runtime::ExtinguishRuntime::default();
extinguish.behavior.settings.cooldown=2.5;extinguish.behavior.settings.pickup_span=[0.2,3.];extinguish.behavior.settings.not_during_combat=false;
extinguish.rotations.active=crate::no_rotate_set::SmoothMode::Acceleration;extinguish.rotations.ticks_until_reset=3;
let mut shoot=crate::auto_shoot_runtime::AutoShootRuntime::default();shoot.behavior.settings.custom=true;shoot.behavior.settings.items=["minecraft:stick".into()].into();shoot.behavior.settings.gravity=1;shoot.behavior.settings.range=[2.,42.];shoot.behavior.settings.constant_lag=true;shoot.behavior.settings.slot_reset=4;shoot.clicker.settings.cps=[8,12];shoot.point_tracker.settings.delay=true;shoot.point_tracker.settings.delay_ticks=[1,3];shoot.rotations.active=crate::no_rotate_set::SmoothMode::Sigmoid;
for format in [JsonFormat::File,JsonFormat::Interop]{
let tree=extinguish_config::serialize_extinguish(&extinguish,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::Extinguish,..Default::default()};extinguish_config::parse_extinguish(&mut patch,&nodes(&tree,"Extinguish","Extinguish").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=crate::extinguish_runtime::ExtinguishRuntime::default();for change in patch.changes{if let SettingChange::Extinguish(v)=change{v.apply(&mut restored).unwrap();}}assert_eq!(restored.behavior.settings,extinguish.behavior.settings);assert_eq!(restored.rotations,extinguish.rotations);
let tree=auto_shoot_config::serialize_auto_shoot(&shoot,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::AutoShoot,..Default::default()};auto_shoot_config::parse_auto_shoot(&mut patch,&nodes(&tree,"AutoShoot","AutoShoot").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=crate::auto_shoot_runtime::AutoShootRuntime::default();for change in patch.changes{if let SettingChange::AutoShoot(v)=change{v.apply(&mut restored,&["minecraft:egg".into(),"minecraft:snowball".into(),"minecraft:stick".into()]).unwrap();}}assert_eq!(restored.behavior.settings,shoot.behavior.settings);assert_eq!(restored.clicker.settings,shoot.clicker.settings);assert_eq!(restored.point_tracker.settings,shoot.point_tracker.settings);assert_eq!(restored.rotations,shoot.rotations);assert_eq!(restored.target_renderer.values,shoot.target_renderer.values);
}
assert!(extinguish_config::ExtinguishSetting::Value("Pickup.PickupSpan".into(),json!({"from":5,"to":1})).apply(&mut extinguish).is_err());
assert!(auto_shoot_config::AutoShootSetting::Value("Target.Priority".into(),json!([])).apply(&mut shoot,&[]).is_err());
println!("Extinguish/AutoShoot File and Interop configuration round trips passed");}
'''
 (out/'config.rs').write_text(cfg,encoding='utf8')
 source+='mod fake_lag_runtime{pub struct FakeLagService(pub crate::fake_lag::Settings);impl FakeLagService{pub fn settings(&self)->crate::fake_lag::Settings{self.0}}}\nmod config;fn main(){config::check();}\n'
 cfg=cfg.replace('NoRotateSet,Extinguish,AutoShoot}', 'NoRotateSet,Extinguish,AutoShoot,FakeLag}').replace('Self::AutoShoot=>"liquidbounce.module.autoShoot"','Self::AutoShoot=>"liquidbounce.module.autoShoot",Self::FakeLag=>"liquidbounce.module.fakeLag"')
 cfg=cfg.replace('AutoShoot(auto_shoot_config::AutoShootSetting)}','AutoShoot(auto_shoot_config::AutoShootSetting),FakeLag(fake_lag_config::FakeLagSetting)}')
 cfg+=body(shared,'fn parse_multi_mask(')+'\n'+f'#[path="{(HERE/"engine/src/config/fake_lag.rs").as_posix()}"]mod fake_lag_config;\n'
 cfg=cfg.replace('println!("Extinguish/AutoShoot File and Interop configuration round trips passed");}', '''let mut settings=crate::fake_lag::Settings::default();settings.range=[1.,9.];settings.delay=[14,999];settings.recoil=3;settings.constant=true;settings.flush_on=5;
for format in [JsonFormat::File,JsonFormat::Interop]{let service=crate::fake_lag_runtime::FakeLagService(settings);let tree=fake_lag_config::serialize_fake_lag(&service,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::FakeLag,..Default::default()};fake_lag_config::parse_fake_lag(&mut patch,&nodes(&tree,"FakeLag","FakeLag").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=crate::fake_lag::Settings::default();for change in patch.changes{if let SettingChange::FakeLag(v)=change{v.apply(&mut restored).unwrap();}}assert_eq!(restored,settings);}
println!("Extinguish/AutoShoot/FakeLag File and Interop configuration round trips passed");}''')
 (out/'config.rs').write_text(cfg,encoding='utf8');(out/'main.rs').write_text(source,encoding='utf8')
 (out/'preparation.json').write_text(json.dumps({'status':'prepared-no-compiler-invoked','scope':'Two new modules only; actual production config trees/helpers and models; Host side effects remain in official grouped check.'},indent=2)+'\n',encoding='utf8')
 print('Prepared one combined Extinguish/AutoShoot config smoke; no compiler invoked')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();prepare(ROOT/a.out)
