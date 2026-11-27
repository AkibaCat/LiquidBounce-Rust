"""One focused compile/smoke of production ProtectionZones and actual native storage."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,PartialEq)]enum ModuleKind{ProtectionZones}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.protectionZones"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{ProtectionZones(settings::ProtectionZonesSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/protection_zones.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''#[test]fn all_twelve_settings_file_interop_roundtrip(){let mut s=crate::protection_zones::ProtectionZones::default();s.protection_blocks=vec!["minecraft:diamond_block".into(),"minecraft:emerald_block".into()];s.radius_x=1;s.radius_z=256;s.radius_y=21;s.render_limit=3;s.hold_block_to_render=true;s.snap_y=true;s.zone_fill=0x12345678;s.zone_outline=1;s.center_zone_outline=2;s.indicator_outline=3;s.indicator_fill=4;let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in[JsonFormat::File,JsonFormat::Interop]{let t=serialize_protection_zones(&s,&common,format,&str::to_owned).unwrap();let mut p=SettingsPatch{module:ModuleKind::ProtectionZones,changes:vec![],issues:vec![]};settings::parse_protection_zones(&mut p,&nodes(&t,"ProtectionZones","ProtectionZones").unwrap());assert!(p.issues.is_empty(),"{:?}",p.issues);assert_eq!(p.changes.len(),12);let mut restored=crate::protection_zones::ProtectionZones::default();for SettingChange::ProtectionZones(v)in p.changes{v.apply(&mut restored);}assert_eq!(restored,s);if format==JsonFormat::Interop{assert_eq!(t["value"][2]["registry"],"block");assert_eq!(t["value"][2]["innerValueType"],"BLOCK");assert_eq!(t["value"][3]["value"][0]["range"],json!({"from":1,"to":256}));assert_eq!(t["value"][3]["value"][0]["key"],"liquidbounce.module.protectionZones.protectionRadius.radiusX");}}}\n'''
 (out/'config.rs').write_text(config,encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon';patch_error=None;metadata=None
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'protection-zones-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:patch_error=repr(e)
 for source,target in [(base/'crates/horizon-client/src/world_primitives.rs','world_primitives.rs'),(ROOT/'upstream/Horizon/crates/horizon-core/src/paletted.rs','paletted.rs'),(ROOT/'upstream/Horizon/crates/horizon-core/src/storage.rs','storage.rs')]:
  (out/target).write_text(source.read_text(encoding='utf8').split('#[cfg(test)]')[0],encoding='utf8')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_world;\n'
 lib+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/protection_zones.rs").as_posix()}"]pub mod protection_zones;\nmod config;mod world_primitives;mod paletted;mod storage;\n'
 lib+='''pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientProtectionZonesEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
pub mod runtime{use std::{rc::Rc,cell::RefCell};pub struct Modules{pub protection_zones:crate::protection_zones::ProtectionZones,pub running:bool}impl Modules{pub fn query_running(&self,_:&(),_:&str)->bool{self.running}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{protection_zones:Default::default(),running:true}),registry:()})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/protection_zones_runtime.rs").as_posix()}"]mod protection_zones_runtime;\n'
 lib+=(HERE/'protection-zones-native-fixture.rs').read_text(encoding='utf8').replace('NATIVE_PATH',(HERE/'protection-zones-game.rs').as_posix());lib+=(HERE/'protection-zones-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="protection-zones-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-protection-zones-locales.py'))
 for language in['zh_cn','zh_tw']:target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 report={'passed':r.returncode==0 and patch_error is None,'compileExit':r.returncode,'patchError':patch_error,'metadata':metadata,'patchedFiles':len(patched),'scope':'Production behavior, twelve-setting config, engine runtime and native Game bridge; actual Horizon PalettedContainer/SimpleBitStorage and WorldPrimitives. Game/world/items/facts providers are declaration fixtures. One focused Cargo smoke; patch composed in memory; locale updaters run on copies. Aggregate native and visual validation remain root-owned.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
