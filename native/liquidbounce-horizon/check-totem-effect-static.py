"""Compose TotemEffect and prepare focused smoke; never invokes Rust/Cargo/GPU."""
from pathlib import Path
import argparse,hashlib,json,runpy,shutil
HERE=Path(__file__).resolve().parent
NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1]
ROOT=NATIVE.parents[1]
def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'totem_effect.rs':ROOT/'native/liquidbounce-behaviors/src/totem_effect.rs','totem_effect_runtime.rs':NATIVE/'engine/src/totem_effect_runtime.rs','totem_effect_config.rs':NATIVE/'engine/src/config/totem_effect.rs'}[name]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 assert not out.exists(),'Refusing to overwrite historical evidence'
 out.mkdir(parents=True)
 body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body'];plan=runpy.run_path(str(HERE/'integrate-totem-effect.py'))['prepare']()
 (out/'shared-plan.json').write_text(json.dumps({'files':[str(p.relative_to(ROOT))for p in plan.changed],'hashes':{str(p.relative_to(ROOT)):hashlib.sha256(t.encode()).hexdigest()for p,t in plan.changed.items()}},indent=2)+'\n',encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,old,new):assert t.count(old)==1,(old,t.count(old));return t.replace(old,new)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 metadata=runpy.run_path(str(HERE/'totem-effect-patch.py'))['apply'](edit,once)
 for path,t in patched.items():dest=out/'patched'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(t,encoding='utf8')
 native=patched['crates/horizon-client/src/game/totem_effect.rs']
 for token in ['p.event != 35','self.native_entity_position(handle)','self.native_entity_rotation(handle)','self.native_entity_dimensions(handle)','self.native_client_targeting_entity(p.id)','self.effective_camera_type().first_person()']:assert token in native,token
 remote=patched['crates/horizon-client/src/game/remote.rs'];assert remote.count('self.client_totem_effect_packet(&packet);')==1
 assert remote.index('self.native_client_packet(&packet);')<remote.index('self.client_totem_effect_packet(&packet);')<remote.index('self.handle_packet_direct(packet)',remote.index('self.client_totem_effect_packet(&packet);'))
 renderer=patched['crates/horizon-client-render/src/gfx/totem_effect.rs']
 for token in ['depth_write_enabled:Some(depth)','wgpu::CompareFunction::GreaterEqual','view:scene_depth','array_stride: 72','views: [wgpu::TextureView; 5]','pass(encoder,&targets.views[4],&self.capture[1],None,true,true);','let flipped=group(&targets.views[4]);','pub order: Vec<[usize; 2]>']:assert token in renderer,token
 for file in(NATIVE/'breadcrumbs-shaders').iterdir():assert patched['crates/horizon-client-render/src/gfx/totem-effect-shaders/'+file.name]==file.read_text(encoding='utf8')
 (out/'render.rs').write_text(renderer,encoding='utf8');(out/'totem-effect.wgsl').write_text(patched['crates/horizon-client-render/src/gfx/totem-effect.wgsl'],encoding='utf8');shutil.copytree(NATIVE/'breadcrumbs-shaders',out/'totem-effect-shaders')
 source=(ROOT/'native/liquidbounce-behaviors/src/target_renderer.rs').read_text(encoding='utf8');(out/'target_renderer.rs').write_text('use serde_json::Value;\n#[derive(Clone,Debug)]\n'+body(source,'pub struct Node')+'\n',encoding='utf8')
 source=(ROOT/'native/liquidbounce-behaviors/src/breadcrumbs.rs').read_text(encoding='utf8');(out/'breadcrumbs.rs').write_text(source.split('pub type Point')[0]+'\n'+body(source,'pub fn rgba(')+'\n',encoding='utf8')
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]TotemEffect}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.totemEffect"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{TotemEffect(TotemEffectSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n#[path="totem_effect_config.rs"]mod settings;pub use settings::*;\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 for name in ['totem_effect.rs','totem_effect_config.rs','totem_effect_runtime.rs']:
  source=owned(name).read_text(encoding='utf8')
  if name=='totem_effect.rs':source=source.replace('../vendor/minecraft-math/sine-table.bin',(ROOT/'native/liquidbounce-behaviors/vendor/minecraft-math/sine-table.bin').as_posix())
  (out/name).write_text(source,encoding='utf8')
 shutil.copyfile(HERE/'totem-effect-game.rs',out/'totem_effect_game.rs')
 source=(ROOT/'native/liquidbounce-behaviors/src/targeting.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0]
 source=source.replace('#[path = "targeting_facts.rs"]',f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/targeting_facts.rs").as_posix()}"]')
 source+='''
pub struct Snapshot{pub entity:TargetEntity}
pub fn snapshot(text:&str)->Option<Snapshot>{let v:serde_json::Value=serde_json::from_str(text).ok()?;Some(Snapshot{entity:TargetEntity{kind:EntityKind::for_type(v["kind"].as_str()?),alive:v["alive"].as_bool()?,invisible:v["invisible"].as_bool()?,sleeping:v["sleeping"].as_bool()?,is_local:v["is_local"].as_bool()?,has_local_passenger:v["has_local_passenger"].as_bool()?,within_border:v["within_border"].as_bool()?}})}
'''
 (out/'targeting.rs').write_text(source,encoding='utf8')
 lib='#![allow(dead_code,unused_imports,unused_mut,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;extern crate self as horizon_protocol;\nmod target_renderer;mod breadcrumbs;pub mod totem_effect;mod config;mod render;mod targeting;\n'
 for name in ['numeric','easing']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]mod {name};\n'
 lib+='''
pub mod mod_api{pub struct ClientTotemEffectEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}pub trait Host{}}
pub mod packets{pub mod play{pub struct EntityEvent{pub id:i32,pub event:u8}pub enum Clientbound{EntityEvent(EntityEvent)}}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};pub struct Modules{pub totem_effect:crate::totem_effect::TotemEffect,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{name=="TotemEffect"&&self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:(),pub global_targets:crate::targeting::GlobalTargets,pub info:RefCell<crate::targeting::TargetInfo>}impl State{pub fn target_info(&self,_:i32,_:&crate::targeting::Snapshot)->crate::targeting::TargetInfo{*self.info.borrow()}}#[derive(Clone)]pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{behaviors:RefCell::new(Modules{totem_effect:Default::default(),active:true}),registry:(),global_targets:Default::default(),info:Default::default()}))}}impl LiquidBounceHooks{pub fn state(&self)->&State{&self.0}}}
#[path="totem_effect_runtime.rs"]mod totem_effect_runtime;
pub mod gfx{pub mod totem_effect{thread_local!{static FRAME:std::cell::RefCell<Option<serde_json::Value>>=const{std::cell::RefCell::new(None)};}pub fn begin_frame(text:&str){FRAME.with_borrow_mut(|f|*f=serde_json::from_str::<serde_json::Value>(text).ok().filter(|v|!v.is_null()));}pub fn frame()->Option<serde_json::Value>{FRAME.with_borrow(Clone::clone)}}}
pub mod game{pub struct Game<'a>{pub hooks:crate::runtime::LiquidBounceHooks,marker:std::marker::PhantomData<&'a ()>}pub struct Camera;impl Camera{pub fn first_person(&self)->bool{true}}impl Game<'_>{pub fn new(hooks:crate::runtime::LiquidBounceHooks)->Self{Self{hooks,marker:std::marker::PhantomData}}pub fn native_window(&self)->bool{true}pub fn native_entity(&self,id:u32)->Option<u32>{(id==7).then_some(id)}pub fn native_entity_position(&self,_:u32)->Option<[f64;3]>{Some([1.,2.,3.])}pub fn native_entity_rotation(&self,_:u32)->Option<[f32;2]>{Some([30.,15.])}pub fn native_entity_dimensions(&self,_:u32)->Option<[f64;2]>{Some([0.6,1.8])}pub fn effective_camera_type(&self)->Camera{Camera}pub fn native_client_targeting_entity(&self,_:i32)->String{serde_json::json!({"kind":"minecraft:player","alive":true,"invisible":false,"sleeping":false,"is_local":false,"has_local_passenger":false,"within_border":true}).to_string()}pub fn packet(&self,p:&crate::packets::play::Clientbound){self.client_totem_effect_packet(p)}pub fn frame(&self,p:f64){self.client_totem_effect_frame(p)}pub fn native_client_totem_effect(&self,input:&str,geometry:&mut String){self.hooks.totem_effect(self,&mut crate::mod_api::ClientTotemEffectEvent{input,geometry});}}impl crate::mod_api::Host for Game<'_>{}#[path="GAME_SOURCE"]mod native;}
'''
 lib=lib.replace('GAME_SOURCE',(out/'totem_effect_game.rs').as_posix())+(HERE/'totem-effect-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 manifest=(ROOT/'build/jump-effect-revision-04/Cargo.toml').read_text(encoding='utf8').replace('name="jump-effect-smoke"','name="totem-effect-smoke"');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-totem-effect-locales.py'));counts={};locale=out/'localization';locale.mkdir()
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target);data=json.loads(target.read_text(encoding='utf8'));expected=updater['descriptions'](language);assert all(data['descriptions'].get(k)==v for k,v in expected.items());assert len(expected)==len(updater['paths']());counts[language]=len(expected)
 report={'feature':'TotemEffect','status':'static-passed-compile-and-smoke-deferred-to-parent-batch','sharedFiles':len(plan.changed),'nativeFiles':len(patched),'metadata':metadata,'localeKeys':counts,'testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'scope':'Complete Shockwave/Soul settings and inactive values; lowercase radius and lifetime alias; original expiration/clear semantics; actual EntityEvent35 native entity position/rotation/height snapshot, source visual targeting; ordered six-part standing WireframePlayer fill/outline geometry, full 144x12 torus, age/progress YMotion; real depth and depth-correct original ShaderLayer capture.','validation':'Read-only transactional shared/native composition; byte-identical frozen ShaderLayer files; actual shared config helpers, visual target model, runtime and native snapshot functions in prepared focused smoke. No Rust/Cargo/GPU execution. Declaration-only actor/native entity providers do not establish actual-game parity.'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
