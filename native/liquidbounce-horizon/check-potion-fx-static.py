"""Compose complete PotionFX and prepare focused checks; never executes Rust/Cargo/GPU."""
from pathlib import Path
import argparse,base64,hashlib,json,runpy,shutil
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1]
def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'potion_fx.rs':ROOT/'native/liquidbounce-behaviors/src/potion_fx.rs','potion_fx_runtime.rs':NATIVE/'engine/src/potion_fx_runtime.rs','potion_fx_config.rs':NATIVE/'engine/src/config/potion_fx.rs'}[name]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out;assert not out.exists(),'Preserve historical evidence';out.mkdir(parents=True)
 body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body'];plan=runpy.run_path(str(HERE/'integrate-potion-fx.py'))['prepare']()
 (out/'shared-plan.json').write_text(json.dumps({'files':[str(p.relative_to(ROOT))for p in plan.changed],'sha256':{str(p.relative_to(ROOT)):hashlib.sha256(s.encode()).hexdigest()for p,s in plan.changed.items()}},indent=2)+'\n',encoding='utf8')
 base=ROOT/'build/staging/horizon';patched={}
 def once(t,a,b):assert t.count(a)==1,(a,t.count(a));return t.replace(a,b)
 def edit(path,fn):patched[path]=fn(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 metadata=runpy.run_path(str(HERE/'potion-fx-patch.py'))['apply'](edit,once)
 for path,text in patched.items():dest=out/'patched'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(text,encoding='utf8')
 native=patched['crates/horizon-client/src/game/potion_fx.rs']
 for token in ['Clientbound::LevelEvent(p) if p.kind == 2002','self.potions.thrown_entities()','"minecraft:splash_potion"','self.client_potion_fx_ground(center)','self.previous_position','self.client_potion_fx_clouds()','visible.iter().map','self.liquidbounce_potion_fx.effects']:assert token in native,token
 ground=patched['crates/horizon-client/src/game/potion_fx_ground.rs']
 for token in ['CollisionContext::empty(false)','RayBlockMode::Collider','RayFluidMode::None','Vec3::new(0., -1., 0.)']:assert token in ground,token
 remote=patched['crates/horizon-client/src/game/remote.rs'];assert remote.count('self.client_potion_fx_packet(&packet);')==1;assert remote.index('self.native_client_packet(&packet);')<remote.index('self.client_potion_fx_packet(&packet);')<remote.index('self.handle_packet_direct(packet)',remote.index('self.client_potion_fx_packet(&packet);'))
 clouds=patched['crates/horizon-client/src/game/remote_projectiles.rs'];assert'liquidbounce_potion_fx_age:0'in clouds;assert'for c in &self.breath_clouds'in clouds;assert'pack(&potion.particle())'in clouds
 renderer=patched['crates/horizon-client-render/src/gfx/potion_fx.rs']
 for token in ['array_stride: 40','wgpu::CompareFunction::GreaterEqual','depth_write_enabled: Some(i == 1)','let [Some(main), Some(secondary)] = resolved','slots: [Slot; 6]','if load && self.slots[slot].selected','self.pipelines[usize::from(draw.depth)]','address_mode_u: wgpu::AddressMode::Repeat','mag_filter: wgpu::FilterMode::Nearest','min_filter: wgpu::FilterMode::Nearest']:assert token in renderer,token
 shader=patched['crates/horizon-client-render/src/gfx/potion-fx.wgsl'];assert 'if color.a==0.0{discard;}' in shader and 'color.a<0.1' not in shader
 (out/'render.rs').write_text(renderer.replace('const PNGS:', 'const PNGS:'),encoding='utf8')
 for name in ['potion-fx.wgsl','potion-fx-textures.rs']:(out/name).write_text(patched['crates/horizon-client-render/src/gfx/'+name],encoding='utf8')
 source=runpy.run_path(str(HERE/'potion-fx-render.py'));assets={}
 for name,path in source['ASSETS']:
  file=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce'/path;data=file.read_bytes();assert data.startswith(b'\x89PNG\r\n\x1a\n');assert base64.b64encode(data).decode()in patched['crates/horizon-client-render/src/gfx/potion-fx-textures.rs'];assets[path]=hashlib.sha256(data).hexdigest()
 (out/'assets.json').write_text(json.dumps(assets,indent=2)+'\n',encoding='utf8')
 source=(ROOT/'native/liquidbounce-behaviors/src/target_renderer.rs').read_text(encoding='utf8');(out/'target_renderer.rs').write_text('use serde_json::Value;\n#[derive(Clone,Debug)]\n'+body(source,'pub struct Node')+'\n',encoding='utf8')
 for name in ['potion_fx.rs','potion_fx_config.rs','potion_fx_runtime.rs']:(out/name).write_text(owned(name).read_text(encoding='utf8'),encoding='utf8')
 shutil.copyfile(ROOT/'build/jump-effect-revision-04/hud.rs',out/'hud.rs')
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8');config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]PotionFX}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.potionFX"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{PotionFX(PotionFXSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n#[path="potion_fx_config.rs"]mod settings;pub use settings::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code,unused_imports)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;mod target_renderer;pub mod potion_fx;mod render;mod config;mod hud;\n'
 for name in ['easing','numeric']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]mod {name};\n'
 lib+='''
pub mod skins{pub fn base64_decode(s:&str)->Option<Vec<u8>>{use base64::Engine;base64::engine::general_purpose::STANDARD.decode(s).ok()}}
pub mod mod_api{pub struct ClientPotionFxEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}pub trait Host{}impl Host for(){}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};pub struct Modules{pub potion_fx:crate::potion_fx::PotionFX,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{name=="PotionFX"&&self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}#[derive(Clone)]pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{behaviors:RefCell::new(Modules{potion_fx:Default::default(),active:true}),registry:()}))}}impl LiquidBounceHooks{pub fn state(&self)->&State{&self.0}}}
#[path="potion_fx_runtime.rs"]mod potion_fx_runtime;
'''
 lib+=(HERE/'potion-fx-smoke.rs').read_text(encoding='utf8')
 lib+=(HERE/'potion-fx-packet-fixture.rs').read_text(encoding='utf8').replace('// ACTUAL_PACKET_FUNCTION',body(native,'pub(super) fn client_potion_fx_packet('))
 (out/'lib.rs').write_text(lib,encoding='utf8')
 with(out/'render.rs').open('a',encoding='utf8')as f:f.write('\npub(crate)fn assets()-> &\'static [(&\'static str,&\'static str)]{&PNGS}\n')
 with(out/'lib.rs').open('a',encoding='utf8')as f:f.write('\nfn assets()-> &\'static [(&\'static str,&\'static str)]{render::assets()}\n')
 manifest=(ROOT/'build/jump-effect-revision-04/Cargo.toml').read_text(encoding='utf8').replace('name="jump-effect-smoke"','name="potion-fx-smoke"');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-potion-fx-locales.py'));locale=out/'localization';locale.mkdir();counts={}
 for lang in ['zh_cn','zh_tw']:
  target=locale/f'display_{lang}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json',target);updater['update'](lang,target);data=json.loads(target.read_text(encoding='utf8'));expected=updater['descriptions'](lang);assert all(data['descriptions'].get(k)==v for k,v in expected.items());assert len(expected)==len(updater['paths']());counts[lang]=len(expected)
 report={'feature':'PotionFX','status':'static-passed-compile-smoke-deferred-parent-batch','sharedFiles':len(plan.changed),'nativeFiles':len(patched),'metadata':metadata,'assets':len(assets),'localeDescriptions':counts,'testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'scope':'Complete three independent toggles, six full source settings, original nine PNGs, player interpolation/effect-color snapshots, 2002 actual nearby splash entity plus empty-context collider ground ray, lifetime/fade/flash and current-position cloud snapshots, native GPU texture consumer with real depth and six independent async custom slots.','validation':'No Rust/Cargo/GPU execution; read-only transactional native/shared composition. Focused runtime fixture uses actual production dispatch with declared actor/Host; does not establish actual-game parity. Non-ColorParticleOption clouds cannot provide the original source accessor integer color and are omitted.'};(out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
