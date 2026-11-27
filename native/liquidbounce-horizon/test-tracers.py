"""One focused Tracers compile/smoke, in-memory native composition, copied locales."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{Breadcrumbs,Tracers}impl ModuleKind{fn key(self)->&\'static str{match self{Self::Breadcrumbs=>"liquidbounce.module.breadcrumbs",Self::Tracers=>"liquidbounce.module.tracers"}}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{Breadcrumbs(breadcrumbs::BreadcrumbsSetting),Tracers(settings::TracersSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/breadcrumbs.rs").as_posix()}"]mod breadcrumbs;pub use breadcrumbs::*;\n#[path="{(HERE/"engine/src/config/tracers.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''#[test]fn all_inactive_modes_styles_and_curve_roundtrip(){let mut s=crate::tracers::Tracers::default();s.line_width=7.;s.hue=vec![[0.,25.],[65.,230.],[200.,30.]];s.render.shader=true;let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_tracers(&s,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::Tracers,changes:vec![],issues:vec![]};settings::parse_tracers(&mut patch,&nodes(&tree,"Tracers","Tracers").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=crate::tracers::Tracers::default();for c in patch.changes{if let SettingChange::Tracers(setting)=c{setting.apply(&mut restored);}}assert_eq!(restored,s);assert!(!tree.to_string().contains("liquidbounce.module.breadcrumbs"));}}\n'''
 (out/'config.rs').write_text(config,encoding='utf8');renderer=runpy.run_path(str(HERE/'tracers-render.py'))['source']();renderer=renderer.replace('include_str!("breadcrumbs',f'include_str!("{HERE.as_posix()}/breadcrumbs');(out/'render.rs').write_text(renderer,encoding='utf8')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 for name in ['breadcrumbs','tracers','combine_mobs','teams']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 targeting=(ROOT/'native/liquidbounce-behaviors/src/targeting.rs').read_text(encoding='utf8');assert targeting.count('#[cfg(test)]\nmod tests {')==1
 targeting=targeting.split('#[cfg(test)]\nmod tests {')[0].replace('"targeting_facts.rs"',f'"{(ROOT/"native/liquidbounce-behaviors/src/targeting_facts.rs").as_posix()}"');(out/'targeting.rs').write_text(targeting,encoding='utf8');lib+='mod targeting;\n'
 # The actual targeting decoder expects liquidbounce_behaviors::targeting. Keep
 # that module and expose the engine decoder separately at its actual path.
 lib=lib.replace('pub mod behavior_targeting;','pub mod targeting;')
 engine=(HERE/'engine/src/tracers_runtime.rs').read_text(encoding='utf8').replace('crate::targeting::snapshot','crate::targeting_decode::snapshot');(out/'runtime.rs').write_text(engine,encoding='utf8')
 lib+=f'#[path="{(HERE/"engine/src/targeting.rs").as_posix()}"]pub mod targeting_decode;\nmod config;mod render;\n'
 lib+='''pub mod teams_runtime{pub fn facts(_:&serde_json::Value)->Option<crate::teams::Facts>{None}}
 pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientTracersEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
 pub mod runtime{use std::{cell::RefCell,rc::Rc};use crate::targeting::*;pub struct Module;impl Module{pub fn enabled(&self)->bool{false}}pub struct Registry;impl Registry{pub fn module(&self,_:u32)->Option<Module>{None}}pub struct Modules{pub tracers:crate::tracers::Tracers,pub combine_mobs:crate::combine_mobs::CombineMobs,pub query_ids:std::collections::BTreeMap<&'static str,u32>}impl Modules{pub fn query_running(&self,_:&Registry,_:&str)->bool{true}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:Registry,pub global_targets:GlobalTargets,pub friends:FriendBook}impl State{pub fn target_info(&mut self,_:i32,s:&crate::targeting_decode::Snapshot)->TargetInfo{self.friends.tag(s.entity.kind,s.player_name.as_deref())}}pub struct LiquidBounceHooks{state:Rc<RefCell<State>>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(RefCell::new(State{behaviors:RefCell::new(Modules{tracers:Default::default(),combine_mobs:Default::default(),query_ids:std::collections::BTreeMap::from_iter([("FreeLook",0)])}),registry:Registry,global_targets:Default::default(),friends:Default::default()}))}}}impl LiquidBounceHooks{pub fn state(&self)->std::cell::RefMut<'_,State>{self.state.borrow_mut()}}}
 mod runtime_bridge{include!("runtime.rs");}
 '''
 lib+=(HERE/'tracers-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 manifest=(ROOT/'build/breadcrumbs-smoke-03/Cargo.toml').read_text(encoding='utf8').replace('name="breadcrumbs-smoke"','name="tracers-smoke"');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 error=None;meta=None
 try:
  meta=runpy.run_path(str(HERE/'tracers-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e)
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-tracers-locales.py'))
 for language in ['zh_cn','zh_tw']:target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':meta,'patchedFiles':len(patched),'scope':'Production pure behavior, actual engine runtime/config/target decoder and independent renderer compiled. Shared actual Tree/parsers extracted; declaration-only actor/Host providers. One focused Cargo smoke for colors, spline, tick target cache, CombineMobs, config roundtrip and width. Native patch composed in memory, locales copied. Native Game compilation and actual game/GPU visual comparison remain aggregate; frozen Breadcrumbs shader GPU evidence is reused without claiming Tracers visual parity.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
