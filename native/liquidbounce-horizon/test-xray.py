"""One focused XRay check; build native patch in memory, no staging or shared writes."""
from pathlib import Path
import argparse,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 if (out/'report.json').exists():raise SystemExit('Evidence already exists')
 out.mkdir(parents=True,exist_ok=True)
 texts={};base=ROOT/'build/staging/horizon'
 def once(t,o,n):
  if t.count(o)!=1:raise AssertionError((o,t.count(o)))
  return t.replace(o,n)
 def edit(name,f):texts[name]=f(texts.get(name,(base/name).read_text(encoding='utf8')if(base/name).exists()else''))
 meta=runpy.run_path(str(HERE/'xray-patch.py'))['apply'](edit,once)
 plan=runpy.run_path(str(HERE/'integrate-xray.py'))['prepare']()
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]XRay}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.xRay"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;#[derive(Debug)]pub enum SettingChange{XRay(XRaySetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/xray.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''#[test]fn roundtrip(){let mut s=crate::xray::Settings::default();s.exposed_only=true;s.background_opacity=83;s.blocks.insert("minecraft:stone".into());let c=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for f in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_xray(&s,&c,f,&str::to_owned).unwrap();let mut p=SettingsPatch::default();settings::parse_xray(&mut p,&nodes(&tree,"XRay","XRay").unwrap());assert!(p.issues.is_empty(),"{:?}",p.issues);let mut restored=crate::xray::Settings::default();for SettingChange::XRay(v)in p.changes{v.apply(&mut restored).unwrap();}assert_eq!(s,restored);}}'''
 (out/'config.rs').write_text(config,encoding='utf8')
 source='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;\n'
 source+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/xray.rs").as_posix()}"]pub mod xray;\nmod config;\n'
 vertex=body((ROOT/'upstream/Horizon/crates/horizon-client-render/src/mesher.rs').read_text(encoding='utf8'),'pub struct TerrainVertex')
 source+='pub mod mesher{#[derive(Clone,Copy)]'+vertex+'pub const OPAQUE_LAYER:u16=1<<15;#[derive(Default)]pub struct SectionMesh{pub solid:Vec<TerrainVertex>,pub translucent:Vec<TerrainVertex>,pub solid_tints:Vec<u32>}}\n'
 source+=f'#[path="{(HERE/"xray-render.rs").as_posix()}"]pub mod native;\n'
 source+=(HERE/'xray-smoke.rs').read_text(encoding='utf8').replace('#[path="../liquidbounce-behaviors/src/xray.rs"] mod xray;','')
 source+='''#[test]fn native_policy(){use native::*;let states=||vec![State{name:"minecraft:air".into(),air:true,conductor:false},State{name:"minecraft:stone".into(),air:false,conductor:true},State{name:"minecraft:diamond_ore".into(),air:false,conductor:true}];let text=r#"{"active":true,"fullBright":true,"exposed":true,"opacity":83,"blocks":["minecraft:diamond_ore"]}"#;assert!(configure(text,states));assert!(!configure(text,states));let p=current().unwrap();assert!(!p.render(2,[1;6]));assert!(p.render(2,[1,1,0,1,1,1]));assert!(p.background(1)&&!p.background(0)&&!p.background(2));let v=crate::mesher::TerrainVertex{pos:[0.;3],uv:[0;2],layer:4,light:0xf0f0,color:0xffaabbcc};let mut m=crate::mesher::SectionMesh{solid:vec![v;8],translucent:vec![v;4],solid_tints:vec![0;2]};p.background_vertices(&mut m,[4,4]);assert_eq!(m.solid.len(),4);assert_eq!(m.translucent.len(),8);assert_eq!(m.solid_tints.len(),1);assert_eq!(m.translucent[4].color>>24,83);assert_ne!(m.translucent[4].layer&0x8000,0);assert!(configure("null",||panic!()));assert!(current().is_none());assert!(p.render(2,[0]));}'''
 render=ROOT/'upstream/Horizon/crates/horizon-client-render/src'
 terrain=texts['crates/horizon-client-render/src/terrain.wgsl']
 for name,shader in [('terrain',(render/'hdr_light.wgsl').read_text(encoding='utf8')+terrain),('oit',(render/'hdr_light.wgsl').read_text(encoding='utf8')+terrain+(render/'oit.wgsl').read_text(encoding='utf8')+texts['crates/horizon-client-render/src/oit_world.wgsl'])]:
  path=out/(name+'.wgsl');path.write_text(shader,encoding='utf8');source+=f'#[test]fn shader_{name}(){{let source=include_str!("{name}.wgsl");let module=naga::front::wgsl::parse_str(source).unwrap_or_else(|e|panic!("{{}}",e.emit_to_string(source)));naga::valid::Validator::new(naga::valid::ValidationFlags::all(),naga::valid::Capabilities::all()).validate(&module).unwrap();}}\n'
 (out/'lib.rs').write_text(source,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="xray-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\nnaga={version="=30.0.1",features=["wgsl-in"]}\n'+f'\n[patch.crates-io]\nnaga={{path="{(ROOT/"upstream/Horizon/third_party/naga").as_posix()}"}}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
 result=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
 (out/'test.log').write_text(result.stdout+result.stderr,encoding='utf8')
 report={'passed':result.returncode==0,'exitCode':result.returncode,'patchedFiles':len(texts),'sharedEdits':len(plan.steps),'metadata':meta,'scope':'One focused production model/config/native-policy/alpha test, native patch composition in memory, actual terrain and OIT WGSL validation. No whole suite or actual game parity claim.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(result.returncode)
if __name__=='__main__':main()
