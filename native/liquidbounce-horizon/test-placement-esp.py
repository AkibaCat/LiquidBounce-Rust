"""One focused ProphuntESP/VoidESP compile and smoke with production bridge."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=ROOT/args.out
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
    config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};use crate::Easing;\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,PartialEq)]enum ModuleKind{ProphuntEsp,VoidEsp}impl ModuleKind{fn key(self)->&\'static str{match self{Self::ProphuntEsp=>"liquidbounce.module.prophuntESP",Self::VoidEsp=>"liquidbounce.module.voidEsp"}}}\n'
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','fn parse_multi_mask(','\nfn leaf(']:
        if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
        config+=body(shared,anchor)+'\n'
    config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug,PartialEq)]pub enum SettingChange{ProphuntEsp(settings::ProphuntEspSetting),VoidEsp(settings::VoidEspSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
    config+=f'#[path="{(HERE/"engine/src/config/placement_esp.rs").as_posix()}"]mod settings;pub use settings::*;\n'
    config+='''#[test]fn both_module_settings_roundtrip(){let p=crate::prophunt_esp::ProphuntEsp::default();let mut v=crate::void_esp::VoidEsp::default();v.y_threshold=31;v.range_side=0;v.range_facing=32;v.render.start_curve=crate::Easing::ExponentialIn;v.render.out_time=250;let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let t=serialize_prophunt_esp(&p,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::ProphuntEsp,changes:vec![],issues:vec![]};settings::parse_prophunt_esp(&mut patch,&nodes(&t,"ProphuntESP","ProphuntESP").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);assert_eq!(patch.changes.len(),13);let mut restored=crate::prophunt_esp::ProphuntEsp::default();for c in patch.changes{if let SettingChange::ProphuntEsp(s)=c{s.apply(&mut restored);}}assert_eq!(p,restored);if format==JsonFormat::Interop{assert_eq!(t["value"][2]["canBeNone"],false);assert_eq!(t["value"][2]["key"],"liquidbounce.module.prophuntESP.tracking");}let t=serialize_void_esp(&v,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::VoidEsp,changes:vec![],issues:vec![]};settings::parse_void_esp(&mut patch,&nodes(&t,"VoidESP","VoidESP").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);assert_eq!(patch.changes.len(),15);let mut restored=crate::void_esp::VoidEsp::default();for c in patch.changes{if let SettingChange::VoidEsp(s)=c{s.apply(&mut restored);}}assert_eq!(v,restored);}}\n'''
    (out/'config.rs').write_text(config,encoding='utf8')
    patched={};base=ROOT/'build/staging/horizon';error=None;meta=None
    def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
    def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
    try:
        meta=runpy.run_path(str(HERE/'placement-esp-patch.py'))['apply'](edit,once)
        for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
    except Exception as e:error=repr(e)
    primitives=(base/'crates/horizon-client/src/world_primitives.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0];(out/'world_primitives.rs').write_text(primitives,encoding='utf8')
    lib='#![allow(dead_code,non_snake_case)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_core;extern crate self as horizon_protocol;\n'
    for name in ['numeric','easing','placement_renderer','prophunt_esp','void_esp','void_search']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
    lib+='pub use easing::Easing;mod config;mod world_primitives;\n'
    lib+='''pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientPlacementEspEvent<'a>{pub input:&'a str,pub output:&'a mut String}}
pub mod runtime{use std::{rc::Rc,cell::RefCell};pub struct Modules{pub prophunt_esp:crate::prophunt_esp::ProphuntEsp,pub void_esp:crate::void_esp::VoidEsp,pub prophunt_running:bool,pub void_running:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{if name=="ProphuntESP"{self.prophunt_running}else{self.void_running}}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{prophunt_esp:Default::default(),void_esp:Default::default(),prophunt_running:true,void_running:true}),registry:()})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
pub mod packets{pub mod play{pub struct BlockUpdate{pub pos:[i32;3],pub state:i32}pub struct SectionBlocksUpdate{pub section:[i32;3],pub changes:Vec<i64>}pub enum Clientbound{BlockUpdate(BlockUpdate),SectionBlocksUpdate(SectionBlocksUpdate),Other}}}
'''
    lib+=f'#[path="{(HERE/"engine/src/placement_esp_runtime.rs").as_posix()}"]mod placement_esp_runtime;\n'
    lib+=(HERE/'placement-esp-native-fixture.rs').read_text(encoding='utf8').replace('NATIVE_PATH',(HERE/'placement-esp-game.rs').as_posix()).replace('SEARCH_PATH',(ROOT/'native/liquidbounce-behaviors/src/void_search.rs').as_posix());lib+=(HERE/'placement-esp-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
    (out/'Cargo.toml').write_text('[package]\nname="placement-esp-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
    locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-placement-esp-locales.py'))
    for language in ['zh_cn','zh_tw']:target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
    report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':meta,'patchedFiles':len(patched),'scope':'Production PlacementRenderer/BlockCuller, ProphuntESP, VoidESP, shared native search, both configs, runtime and native Game bridge compiled with actual WorldPrimitives API. Game/Host/registry/level providers are declarations. One focused Cargo smoke; native patch composed six files in memory; Chinese updaters on copies. Aggregate real native/support/render validation remains root-owned.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)

if __name__=='__main__':main()
