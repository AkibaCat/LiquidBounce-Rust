"""One focused shared TargetRenderer check; never writes staging or canonical locales."""
from pathlib import Path
import argparse,base64,hashlib,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]

def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--test');a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
 shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy)]pub struct ModuleKind;impl ModuleKind{fn key(self)->&\'static str{"unused"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n'
 config+=f'#[path="{(HERE/"engine/src/config/target_renderer.rs").as_posix()}"]pub mod target_renderer;\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 renderer=runpy.run_path(str(HERE/'target-renderer-render.py'))
 (out/'render.rs').write_text(renderer['source']().replace('include_str!("target_renderer.wgsl")',f'include_str!("{(HERE/"target-renderer.wgsl").as_posix()}")')+'\n'+(HERE/'target-renderer-custom-smoke.rs').read_text(encoding='utf8'),encoding='utf8')
 shutil.copyfile(ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/target_renderer/target.png',out/'custom-original.png')
 (out/'target_renderer-textures.rs').write_text(renderer['textures'](),encoding='utf8')
 (out/'target-renderer-font-data.rs').write_text(runpy.run_path(str(HERE/'target-renderer-patch.py'))['fonts'](),encoding='utf8')
 font_source=(ROOT/'upstream/Horizon/crates/horizon-client-render/src/font.rs').read_text(encoding='utf8')
 font='use ab_glyph::{Font as _,ScaleFont as _};use std::{cell::RefCell,collections::HashMap,path::PathBuf};\n#[derive(Clone,Copy,Debug,Default)]pub struct Rect{pub u0:f32,pub v0:f32,pub u1:f32,pub v1:f32,pub w:u32,pub h:u32}\n'
 font+='#[derive(Clone,Copy,Debug,Default)]'+body(font_source,'pub struct Glyph')+'\n#[derive(Clone,Debug)]'+body(font_source,'pub struct Patch')+'\n'
 font+='type Provider=();\n'+body(font_source,'struct FontSet').replace('crate::fast_hash::FxHashMap','HashMap')+'\n'+body(font_source,'struct Region')+'\n'+body(font_source,'impl Region')+'\n'+body(font_source,'fn glyph_rect(')+'\n'
 font+='struct Cache{sets:Vec<(String,FontSet)>,region:Region}\npub struct Font{cache:RefCell<Cache>,atlas:(u32,u32),region:(u32,u32),missing:Glyph,metrics_only:bool}\n'
 font+='impl Default for Font{fn default()->Self{Self{cache:RefCell::new(Cache{sets:vec![],region:Region{cursor:(0,0),row:0,pending:vec![],full:false}}),atlas:(512,512),region:(0,512),missing:Glyph::default(),metrics_only:false}}}\n'
 font+=(HERE/'target-renderer-font.rs').read_text(encoding='utf8').replace('include!("target-renderer-font-data.rs");','include!("target-renderer-font-data.rs");')
 font+='\n#[test]fn original_inter_font_raster_cache_and_atlas(){let f=Font::default();let(a,h)=f.liquidbounce_target_metrics();assert!(a>30.&&h>=43.);let g=f.liquidbounce_target_glyph(\'T\',false);assert!(g.advance>15.&&g.rect.is_some());let count=f.cache.borrow().region.pending.len();let g2=f.liquidbounce_target_glyph(\'T\',false);assert_eq!(g.advance,g2.advance);assert_eq!(f.cache.borrow().region.pending.len(),count);let bold=f.liquidbounce_target_glyph(\'T\',true);assert!(bold.rect.is_some());let cjk=f.liquidbounce_target_glyph(\'目\',false);assert!(cjk.advance>0.);let cache=f.cache.borrow();for p in &cache.region.pending{assert!(p.x+p.width<=512&&p.y+p.height<=512);assert!(p.rgba.chunks(4).any(|c|c[3]>0));}}\n'
 (out/'font.rs').write_text(font,encoding='utf8')
 hud_source=(ROOT/'upstream/Horizon/crates/horizon-client-render/src/hud.rs').read_text(encoding='utf8');hud='type Result<T>=std::result::Result<T,Box<dyn std::error::Error+Send+Sync>>;const MAX_FETCHED_PIXELS:u64=4096*4096;\n'
 for anchor in ['enum ImageFormat','impl ImageFormat','fn check_fetched_size(','fn decode_webp(','fn decode_jpeg(','pub fn decode_image_bytes(','pub fn decode_bytes(','fn png_bytes_size(']:hud+=body(hud_source,anchor)+'\n'
 (out/'hud.rs').write_text(hud,encoding='utf8')
 lib='#![allow(dead_code,unused_imports)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 lib+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/target_renderer.rs").as_posix()}"]pub mod target_renderer;\nmod config;mod render;mod font;mod hud;\n'
 lib+='pub mod skins{pub fn base64_decode(text:&str)->Option<Vec<u8>>{use base64::Engine;base64::engine::general_purpose::STANDARD.decode(text).ok()}}\n'
 lib+='pub mod mod_api{pub trait Host{}pub struct ClientTargetRendererEvent<\'a>{pub input:&\'a str,pub geometry:&\'a mut String}}\n'
 lib+='''pub mod runtime{use std::{cell::{RefCell,RefMut},rc::Rc,time::Instant};use crate::target_renderer::*;#[derive(Default)]pub struct Settings{pub aimbot:bool}#[derive(Default)]pub struct Behavior{pub target:Option<i32>,pub settings:Settings}#[derive(Default)]pub struct Owner{pub behavior:Behavior,pub target_renderer:TargetRendererSettings,pub target_render_runtime:TargetRendererRuntime}pub struct Modules{pub auto_bow:Owner,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),_:&str)->bool{self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks{state:Rc<RefCell<State>>,pub started:Instant}impl Default for LiquidBounceHooks{fn default()->Self{let mut auto_bow=Owner::default();auto_bow.behavior.settings.aimbot=true;Self{state:Rc::new(RefCell::new(State{behaviors:RefCell::new(Modules{auto_bow,active:true}),registry:()})),started:Instant::now()}}}impl LiquidBounceHooks{pub fn state(&self)->RefMut<'_,State>{self.state.borrow_mut()}}}\n'''
 lib+=f'#[path="{(HERE/"engine/src/target_renderer_runtime.rs").as_posix()}"]mod target_renderer_runtime;\n'
 lib+=(HERE/'target-renderer-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 manifest=(ROOT/'build/particles-smoke-01/Cargo.toml').read_text(encoding='utf8').replace('name="particles-smoke"','name="target-renderer-smoke"').replace('pollster="0.4"','pollster="0.4"\nab_glyph="0.2"');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 patched={};error=None;metadata=None;base=ROOT/'build/staging/horizon'
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'target-renderer-patch.py'))['apply'](edit,once)
  assert 'fn native_client_target_renderer(' in patched['crates/horizon-client/src/game/native_hooks.rs']
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e)
 plan=runpy.run_path(str(HERE/'integrate-target-renderer.py'))['prepare']();(out/'shared-plan.json').write_text(json.dumps({'steps':plan.steps,'files':list(plan.text)},indent=2)+'\n',encoding='utf8')
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-target-renderer-locales.py'));counts={}
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target);d=json.loads(target.read_text(encoding='utf8'));counts[language]=len(updater['descriptions'](language));assert all(d['descriptions'].get(k)for k in updater['descriptions'](language))
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');command=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')];command+=([a.test]if a.test else[]);r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'testFilter':a.test,'patchError':error,'metadata':metadata,'patchedFiles':len(patched),'localeKeys':counts,'scope':'One focused shared TargetRenderer invocation: production behavior/schema/parser/serializer/runtime, real original Inter font glyph rasterization and native Region atlas placement, GPU shader/pipeline/draw/readback with original builtin images, annulus and partial heart with actual scene depth. Filtered follow-ups test only the named item. Owner declarations are fixtures. Native Game integration composed in memory; full native Game/overlay compile and real-game visual parity remain root-owned. No frozen renderer matrix rerun.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
