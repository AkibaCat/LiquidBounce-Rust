"""One focused BlockOutline compile; production runtime/config plus actual patch anchors."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};use crate::Easing;\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{BlockOutline}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.blockOutline"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{BlockOutline(settings::BlockOutlineSetting)}\n#[derive(Default)]pub struct SettingsPatch{pub changes:Vec<SettingChange>,pub issues:Vec<ConfigError>}\n'
 config+=f'#[path="{(HERE/"engine/src/config/block_outline.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='#[test]fn config_round_trip(){let s=crate::block_outline::BlockOutline::default();let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_block_outline(&s,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_block_outline(&mut patch,&nodes(&tree,"BlockOutline","BlockOutline").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);assert_eq!(patch.changes.len(),6);let mut restored=s.clone();for SettingChange::BlockOutline(setting)in patch.changes{setting.apply(&mut restored);}assert_eq!(restored,s);}}\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 for name in ['numeric','easing','block_outline']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 lib+='pub use easing::Easing;mod config;\n'
 lib+='''pub mod mod_api {pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientBlockOutlineEvent<'a>{pub input:&'a str,pub geometry:&'a mut String,pub active:bool}}
pub mod runtime {use std::{rc::Rc,cell::RefCell};pub struct Modules{pub block_outline:crate::block_outline::BlockOutline}impl Modules{pub fn query_running(&self,_r:&(),_name:&str)->bool{true}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{block_outline:Default::default()}),registry:()})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/block_outline_runtime.rs").as_posix()}"]mod block_outline_runtime;\n'
 lib+=(HERE/'block-outline-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="block-outline-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 error=None
 try:
  meta=runpy.run_path(str(HERE/'block-outline-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e);meta=None
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-block-outline-locales.py'))
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':meta,'scope':'Production behavior, surface mesh, engine runtime and config compiled with declaration-only registry/Host event providers; actual shared Tree/parsers extracted. One focused Cargo test, actual native patch applied in memory, copied Chinese dictionaries. Game compilation/visual proof remain aggregate.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
