"""One production HoleESP usability smoke; no official build or GPU matrix."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]HoleEsp}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.holeESP"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{HoleEsp(HoleEspSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/hole_esp.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 smoke=(HERE/'hole-esp-smoke.rs').read_text(encoding='utf8');marker='#[test]fn full_file_and_interop_roundtrip';offset=smoke.index(marker);config+='#[cfg(test)]mod tests{use super::*;use crate::hole_esp;'+smoke[offset:].replace('    use config::*;','')+'}\n'
 (out/'config.rs').write_text(config,encoding='utf8');(out/'tests.rs').write_text(smoke[:offset],encoding='utf8')
 lib='#![allow(dead_code,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_world;\n'
 for name in ['hole_tracker','hole_esp']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 lib+='mod config;\n'+'''pub mod mod_api{pub trait Host{}pub struct TestHost;impl Host for TestHost{}pub struct ClientHoleEspEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};pub struct Modules{pub hole_esp:crate::hole_esp::HoleEsp,pub running:bool}impl Modules{pub fn query_running(&mut self,_registry:&(),name:&str)->bool{assert_eq!(name,"HoleESP");self.running}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{hole_esp:Default::default(),running:true}),registry:()})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/hole_esp_runtime.rs").as_posix()}"]mod hole_esp_runtime;\n'
 fixture=(HERE/'hole-esp-native-fixture.rs').read_text(encoding='utf8').replace('// @@NATIVE_MODULE@@',f'#[path="{(HERE/"hole-esp-game.rs").as_posix()}"]mod hole_esp;')
 lib+=fixture+'\ninclude!("tests.rs");\n';(out/'lib.rs').write_text(lib,encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-hole-esp-locales.py'));counts={}
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target);d=json.loads(target.read_text(encoding='utf8'));counts[language]=sum(k.startswith('liquidbounce.module.holeESP')for k in d['descriptions'])
 patched={};base=ROOT/'build/staging/horizon';error=None;metadata=None
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'hole-esp-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e)
 plan=runpy.run_path(str(HERE/'integrate-hole-esp.py'))['prepare']();(out/'shared-plan.json').write_text(json.dumps({'steps':plan.steps,'files':list(plan.text)},indent=2)+'\n',encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="hole-esp-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':metadata,'patchedFiles':len(patched),'localeKeys':counts,'scope':'One focused usability smoke of production HoleTracker/HoleManager, settings, geometry, engine runtime and native COW snapshot bridge. Only Host/registry/resident-world declarations supplied. All inactive settings roundtrip. Frozen plain renderer reused; no new shader, GPU matrix, official build or actual in-game parity claim. Shared integration remains root-owned.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
