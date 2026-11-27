"""One focused global BlinkManager ESP smoke, no official build or shader matrix."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True,exist_ok=True)
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug)]pub enum ModuleKind{Breadcrumbs}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.breadcrumbs"}}\n'
 for anchor in['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{Breadcrumbs(BreadcrumbsSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
 for name in['breadcrumbs','blink_esp']:config+=f'#[path="{(HERE/f"engine/src/config/{name}.rs").as_posix()}"]mod {name};pub use {name}::*;\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 for name in['breadcrumbs','rotations','blink_esp']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 lib+='mod config;\n'+'''pub mod mod_api{pub trait Host{fn client_blink_visual(&self,partial_ticks:f32)->String;}pub struct TestHost;impl Host for TestHost{fn client_blink_visual(&self,p:f32)->String{assert_eq!(p,0.4);serde_json::json!({"firstPerson":false,"width":0.6,"height":1.8,"pose":0,"swimAmount":0.}).to_string()}}pub struct ClientBlinkEspEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
pub mod runtime{use std::{cell::RefCell,rc::Rc,sync::Mutex};pub struct Criticals;impl Criticals{pub fn positions(&self)->Vec<[f64;3]>{vec![[12.,70.,-8.],[13.,71.,-7.]]}}pub struct Modules{pub blink_esp:crate::blink_esp::BlinkEsp,pub criticals:Criticals,pub rotations:Mutex<crate::rotations::RotationManager>}pub struct State{pub behaviors:RefCell<Modules>}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{let mut s=crate::blink_esp::BlinkEsp::default();s.mode=1;let mut r=crate::rotations::RotationManager::default();r.actual_server=crate::rotations::Rotation{yaw:134.,pitch:-27.,is_normalized:false};r.theoretical_server=crate::rotations::Rotation{yaw:3.,pitch:9.,is_normalized:false};Self{state:Rc::new(State{behaviors:RefCell::new(Modules{blink_esp:s,criticals:Criticals,rotations:Mutex::new(r)})})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/blink_esp_runtime.rs").as_posix()}"]mod blink_esp_runtime;\n'
 lib+=(HERE/'blink-esp-smoke.rs').read_text(encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-blink-esp-locales.py'));counts={}
 for language in['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target);d=json.loads(target.read_text(encoding='utf8'));counts[language]=sum(k.startswith('liquidbounce.settings.blinkManager')for k in d['descriptions'])
 lib+='''#[test]fn all_global_localization_keys_resolve(){fn check(v:&serde_json::Value,d:&serde_json::Value){if let Some(k)=v["key"].as_str(){assert!(d["descriptions"][format!("{k}.description")].as_str().is_some(),"{k}");}match v{serde_json::Value::Object(o)=>for v in o.values(){check(v,d)},serde_json::Value::Array(a)=>for v in a{check(v,d)},_=>{}}}let tree=config::serialize_global_blink(&blink_esp::BlinkEsp::default(),config::JsonFormat::Interop,&str::to_owned);for source in[include_str!("localization/display_zh_cn.json"),include_str!("localization/display_zh_tw.json")]{check(&tree,&serde_json::from_str(source).unwrap());}}\n'''
 (out/'lib.rs').write_text(lib,encoding='utf8')
 model=runpy.run_path(str(HERE/'blink-esp-game.py'))['source']();(out/'blink_native.rs').write_text(model,encoding='utf8');render=runpy.run_path(str(HERE/'blink-esp-render.py'));(out/'blink_renderer.rs').write_text(render['source'](),encoding='utf8');(out/'blink_esp.wgsl').write_text(render['shader'](),encoding='utf8')
 subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true','--check',str(out/'blink_native.rs')],capture_output=True)
 patched={};base=ROOT/'build/staging/horizon';error=None;metadata=None
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'blink-esp-patch.py'))['apply'](edit,once)
  for path,text in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf8')
 except Exception as e:error=repr(e)
 (out/'Cargo.toml').write_text('[package]\nname="blink-esp-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\nserde={version="1",features=["derive"]}\nnaga={version="30.0.1",features=["wgsl-in"]}\n[patch.crates-io]\nnaga={path="'+(ROOT/'upstream/Horizon/third_party/naga').as_posix()+'"}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':metadata,'patchedFiles':len(patched),'localeKeys':counts,'scope':'One focused usability smoke of production settings, behavior and runtime with declaration-only Host/shared manager providers; actual native player extraction generated from pinned source and checked for clone/skin/equipment/partial-zero/light semantics; single new radial WGSL validation, reused frozen shader styles without another GPU matrix. Native/engine official build and actual in-game parity remain root-owned.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
