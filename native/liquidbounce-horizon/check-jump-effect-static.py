"""Compose JumpEffect in memory and prepare one focused smoke; never invokes Rust/Cargo."""
from pathlib import Path
import argparse,base64,hashlib,json,runpy,shutil
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
ROOT=NATIVE.parents[1]
def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'jump_effect.rs':ROOT/'native/liquidbounce-behaviors/src/jump_effect.rs','jump_effect_runtime.rs':NATIVE/'engine/src/jump_effect_runtime.rs','jump_effect_config.rs':NATIVE/'engine/src/config/jump_effect.rs'}[name]

def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 if out.exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True)
 body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body']
 plan=runpy.run_path(str(HERE/'integrate-jump-effect.py'))['prepare']()
 (out/'shared-plan.json').write_text(json.dumps({'files':[str(p.relative_to(ROOT))for p in plan.changed],'hashes':{str(p.relative_to(ROOT)):hashlib.sha256(t.encode()).hexdigest()for p,t in plan.changed.items()}},indent=2)+'\n',encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,old,new):
  assert t.count(old)==1,(old,t.count(old));return t.replace(old,new)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 metadata=runpy.run_path(str(HERE/'jump-effect-patch.py'))['apply'](edit,once)
 for path,t in patched.items():
  dest=out/'patched'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(t,encoding='utf8')
 motion=patched['crates/horizon-client/src/motion.rs'];jump='crate::jump_effect_bridge::jump([self.player.position.x,self.player.position.y,self.player.position.z]);'
 assert motion.count(jump)==1
 guard='if self.player.client_high_jump_owner&&self.player.client_side&&self.player.native_client_queries_allowed(){'
 assert guard in motion and motion.index(guard)<motion.index(jump)<motion.index('crate::high_jump_bridge::jump(&mut power);')
 assert motion.index(jump)<motion.index('living_motion::jump_from_ground_with_power(self,power)?;')
 gfx=patched['crates/horizon-client-render/src/gfx.rs']
 assert 'self.jump_effect.draw(&self.device,&mut encoder,world_view,world_depth,'in gfx
 assert 'self.jump_effect.prepare(&self.device,&self.queue,jittered,frame.camera,breadcrumbs_size,frame.title);'in gfx
 renderer=patched['crates/horizon-client-render/src/gfx/jump_effect.rs']
 for token in ['depth_write_enabled:Some(depth)','wgpu::CompareFunction::GreaterEqual','depth_stencil_attachment:mesh.then_some','view:scene_depth','array_stride: 72','frame.texture=texture']:
  assert token in renderer,token
 for file in (NATIVE/'breadcrumbs-shaders').iterdir():assert patched['crates/horizon-client-render/src/gfx/jump-effect-shaders/'+file.name]==file.read_text(encoding='utf8')
 (out/'render.rs').write_text(renderer,encoding='utf8')
 for name in ['jump-effect.wgsl','jump-effect-textures.rs']:(out/name).write_text(patched['crates/horizon-client-render/src/gfx/'+name],encoding='utf8')
 shutil.copytree(NATIVE/'breadcrumbs-shaders',out/'jump-effect-shaders')
 # Reuse exact production schema/style declarations and shared config helpers.
 source=(ROOT/'native/liquidbounce-behaviors/src/target_renderer.rs').read_text(encoding='utf8')
 (out/'target_renderer.rs').write_text('use serde_json::Value;\n#[derive(Clone,Debug)]\n'+body(source,'pub struct Node')+'\n',encoding='utf8')
 source=(ROOT/'native/liquidbounce-behaviors/src/breadcrumbs.rs').read_text(encoding='utf8')
 source=source.split('pub type Point')[0]+'\n'+body(source,'pub fn rgba(')+'\n'
 (out/'breadcrumbs.rs').write_text(source,encoding='utf8')
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]JumpEffect}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.jumpEffect"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{JumpEffect(JumpEffectSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 config+='#[path="jump_effect_config.rs"]mod settings;pub use settings::*;\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 for src,dest in [('jump_effect_config.rs','jump_effect_config.rs'),('jump_effect.rs','jump_effect.rs'),('jump_effect_runtime.rs','jump_effect_runtime.rs'),('jump-effect-bridge.rs','jump_effect_bridge.rs'),('jump-effect-game.rs','jump_effect_game.rs')]:shutil.copyfile(owned(src),out/dest)
 hud_source=(ROOT/'upstream/Horizon/crates/horizon-client-render/src/hud.rs').read_text(encoding='utf8')
 hud='type Result<T>=std::result::Result<T,Box<dyn std::error::Error+Send+Sync>>;\nconst MAX_FETCHED_PIXELS:u64=4096*4096;\n'
 for anchor in ['enum ImageFormat','impl ImageFormat','fn check_fetched_size(','fn decode_webp(','fn decode_jpeg(','pub fn decode_image_bytes(','pub fn decode_bytes(','fn png_bytes_size(']:hud+=body(hud_source,anchor)+'\n'
 (out/'hud.rs').write_text(hud,encoding='utf8')
 lib='#![allow(dead_code,unused_imports)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_core;\nmod target_renderer;mod breadcrumbs;pub mod jump_effect;mod config;mod render;mod hud;pub mod jump_effect_bridge;\n'
 for name in ['numeric','easing']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]mod {name};\n'
 lib+='''
pub mod skins{pub fn base64_decode(text:&str)->Option<Vec<u8>>{use base64::Engine;base64::engine::general_purpose::STANDARD.decode(text).ok()}}
pub mod mod_api{pub struct ClientJumpEffectEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}pub trait Host{fn chat(&mut self,message:&str);}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};pub struct Modules{pub jump_effect:crate::jump_effect::JumpEffect,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{name=="JumpEffect"&&self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}#[derive(Clone)]pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{behaviors:RefCell::new(Modules{jump_effect:Default::default(),active:true}),registry:()}))}}impl LiquidBounceHooks{pub fn state(&self)->&State{&self.0}}}
#[path="jump_effect_runtime.rs"]mod jump_effect_runtime;
thread_local!{static JUMP_HOOK:std::cell::RefCell<Option<runtime::LiquidBounceHooks>>=const{std::cell::RefCell::new(None)};}
fn dispatch_jump_effect(position:[f64;3]){JUMP_HOOK.with_borrow(|v|v.as_ref().unwrap().jump_effect_jump(position));}
pub mod gfx{pub mod jump_effect{thread_local!{static FRAME:std::cell::RefCell<Option<serde_json::Value>>=const{std::cell::RefCell::new(None)};}pub fn begin_frame(text:&str){FRAME.with_borrow_mut(|f|*f=serde_json::from_str::<serde_json::Value>(text).ok().filter(|v|!v.is_null()));}pub fn frame()->Option<serde_json::Value>{FRAME.with_borrow(Clone::clone)}pub fn take_error()->Option<String>{None}}}
pub mod game{pub struct Game<'a>{pub hooks:crate::runtime::LiquidBounceHooks,marker:std::marker::PhantomData<&'a ()>}impl Game<'_>{pub fn new(hooks:crate::runtime::LiquidBounceHooks)->Self{Self{hooks,marker:std::marker::PhantomData}}pub fn native_window(&self)->bool{true}pub fn frame(&self,p:f64){self.client_jump_effect_frame(p)}pub fn native_client_jump_effect(&self,input:&str,geometry:&mut String){self.hooks.jump_effect(&mut crate::mod_api::ClientJumpEffectEvent{input,geometry});}}impl crate::mod_api::Host for Game<'_>{fn chat(&mut self,_:&str){}}#[path="jump_effect_game.rs"]mod native;}
'''
 # A nested module path is resolved relative to game/, so use absolute source.
 lib=lib.replace('#[path="jump_effect_game.rs"]',f'#[path="{(out/"jump_effect_game.rs").as_posix()}"]')
 lib+=(HERE/'jump-effect-smoke.rs').read_text(encoding='utf8')
 (out/'lib.rs').write_text(lib,encoding='utf8')
 render=(out/'render.rs').read_text(encoding='utf8')
 render+='\n#[cfg(test)]mod original_image_check{#[test]fn both_original_images_decode_with_actual_horizon_decoder(){for encoded in super::PNGS{let b=crate::skins::base64_decode(encoded).unwrap();let(w,h,rgba)=crate::hud::decode_bytes(&b).unwrap();assert!(w>0&&h>0);assert_eq!(rgba.len(),w as usize*h as usize*4);assert!(rgba.chunks_exact(4).any(|p|p[3]!=0));}}}\n'
 (out/'render.rs').write_text(render,encoding='utf8')
 manifest=(ROOT/'build/particles-smoke-01/Cargo.toml').read_text(encoding='utf8').replace('name="particles-smoke"','name="jump-effect-smoke"').replace('features=["glsl-in"]','features=["glsl-in","wgsl-in"]')
 (out/'Cargo.toml').write_text(manifest,encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-jump-effect-locales.py'));counts={};locale=out/'localization';locale.mkdir()
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target);data=json.loads(target.read_text(encoding='utf8'));expected=updater['descriptions'](language)
  assert all(data['descriptions'].get(k)==v for k,v in expected.items());assert len(expected)==len(updater['paths']());assert all('轨迹'not in v and'軌跡'not in v for v in expected.values());counts[language]=len(expected)
 textures={name:hashlib.sha256((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/jump_effect/{name}.png').read_bytes()).hexdigest()for name in ['liquidbounce','liquidbounce_with_logo']}
 report={'status':'static-passed-compile-and-focused-smoke-deferred','sharedFiles':len(plan.changed),'nativeFiles':len(patched),'metadata':metadata,'localeKeys':counts,'textureHashes':textures,'testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'scope':'Complete Simple/Image choices and inactive settings; synchronous guarded local jump HEAD feet snapshot; FIRST_PRIORITY expiration, current lifetime age, hue/fade; native renderer mesh and real reversed-Z scene depth; complete frozen ShaderLayer sources copied byte-for-byte; original PNGs and custom PNG/JPEG flow; actual shared config/decoder in prepared smoke. No Rust/Cargo/GPU execution. Frozen shader matrices are not rerun. Declaration-only actor/event providers do not establish actual-game parity.'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
