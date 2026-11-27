"""One focused StorageESP usability check. Shared/stage files remain untouched."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--test-filter');a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]StorageEsp}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.storageESP"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{StorageEsp(StorageEspSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/storage_esp.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''pub fn roundtrip(){let mut s=crate::storage_esp::StorageEsp::default();s.glow=false;s.style=9;s.fade=[5.,10.,40.,50.];s.styles[2].values.insert("PulseSpeed".into(),99.);let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_storage_esp(&s,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_storage_esp(&mut patch,&nodes(&tree,"StorageESP","StorageESP").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=crate::storage_esp::StorageEsp::default();for SettingChange::StorageEsp(setting)in patch.changes{setting.apply(&mut restored);}assert_eq!(restored,s);}}
'''
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 for name in ['breadcrumbs','storage_esp']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 # Use the real native model vertex layout, not a replacement geometry format.
 vertex=body((ROOT/'upstream/Horizon/crates/horizon-client-render/src/mesher.rs').read_text(encoding='utf8'),'pub struct Vertex')
 lib+='pub mod mesher{#[repr(C)]#[derive(Clone,Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]'+vertex+'}\nmod config;\n'
 lib+='''pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientStorageEspEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
pub mod runtime{use std::{rc::Rc,cell::RefCell,collections::BTreeSet};#[derive(Default)]pub struct ChestStealer{pub interacted:BTreeSet<[i32;3]>}impl ChestStealer{pub fn storage_interacted(&self)->&BTreeSet<[i32;3]>{&self.interacted}}#[derive(Default)]pub struct Modules{pub storage_esp:crate::storage_esp::StorageEsp,pub chest_stealer:ChestStealer,pub disabled:bool,pub chest_running:bool}impl Modules{pub fn query_running(&self,_:&(),n:&str)->bool{match n{"StorageESP"=>!self.disabled,"ChestStealer"=>self.chest_running,_=>false}}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{behaviors:RefCell::default(),registry:()}))}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.0.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/storage_esp_runtime.rs").as_posix()}"]mod storage_esp_runtime;\n'
 renderer=runpy.run_path(str(HERE/'storage-esp-render.py'));(out/'storage-render.rs').write_text(renderer['source'](),encoding='utf8');(out/'storage-plain.rs').write_text(renderer['plain_source'](),encoding='utf8')
 for name in ['breadcrumbs.wgsl','storage-esp-model.wgsl','storage-esp-sobel.frag']:shutil.copyfile(HERE/name,out/name)
 shutil.copytree(HERE/'breadcrumbs-shaders',out/'breadcrumbs-shaders',dirs_exist_ok=True)
 lib+='mod storage_render{include!("storage-render-body.rs");}\nmod storage_plain{include!("storage-plain-body.rs");}\n'
 for name in ['storage-render','storage-plain']:(out/(name+'-body.rs')).write_text('\n'.join(l for l in(out/(name+'.rs')).read_text(encoding='utf8').splitlines()if not l.startswith('//!'))+'\n',encoding='utf8')
 lib+=(HERE/'storage-esp-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 manifest='''[package]
name="storage-esp-smoke"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
serde={version="1",features=["derive"]}
glam="0.30"
bytemuck={version="1.24",features=["derive"]}
naga={version="=30.0.1",features=["glsl-in"]}
wgpu={version="=30.0.1",default-features=false,features=["std","parking_lot","dx12","vulkan","wgsl","naga-ir"]}
pollster="0.4"
'''+f'\n[patch.crates-io]\nnaga={{path="{(ROOT/"upstream/Horizon/third_party/naga").as_posix()}"}}\nwgpu-hal={{path="{(ROOT/"upstream/Horizon/third_party/wgpu-hal").as_posix()}"}}\n'
 (out/'Cargo.toml').write_text(manifest,encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-storage-esp-locales.py'));counts={}
 for lang in ['zh_cn','zh_tw']:
  target=locale/f'display_{lang}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json',target);updater['update'](lang,target);counts[lang]=sum(k.startswith('liquidbounce.module.storageESP')for k in json.loads(target.read_text(encoding='utf8'))['descriptions'])
 patched={};base=ROOT/'build/staging/horizon';error=None;metadata=None
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'storage-esp-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e)
 plan=runpy.run_path(str(HERE/'integrate-storage-esp.py'))['prepare']();(out/'shared-plan.json').write_text(json.dumps({'steps':plan.steps,'files':list(plan.text)},indent=2)+'\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');command=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')];command+=([a.test_filter]if a.test_filter else[])
 r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':metadata,'patchedFiles':len(patched),'localeKeys':counts,'scope':'One production settings/geometry/runtime/renderer usability test. Actual Tree/parser helpers, all inactive settings roundtrip, thirteen categories, connected voxel union, ChestStealer interaction/running state, distance/alpha/tracers and model target selection. GPU readback uses native Vertex layout/atlas alpha with default Smooth and Vanilla only; frozen nine-style matrix not repeated. Native patch/shared wiring composed in memory. Native Game compilation and actual-game parity remain root-owned aggregate checks.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
