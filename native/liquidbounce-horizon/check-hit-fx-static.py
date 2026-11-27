"""Prepare one HitFX focused smoke and in-memory native/shared patch composition, no Rust/Cargo."""
from pathlib import Path
import argparse,base64,hashlib,json,runpy,shutil
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'engine').is_dir()else HERE.parents[1];ROOT=NATIVE.parents[1]
def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'hit_fx.rs':ROOT/'native/liquidbounce-behaviors/src/hit_fx.rs','hit_fx_sounds.rs':ROOT/'native/liquidbounce-behaviors/src/hit_fx_sounds.rs','hit_fx_runtime.rs':NATIVE/'engine/src/hit_fx_runtime.rs','hit_fx_config.rs':NATIVE/'engine/src/config/hit_fx.rs'}[name]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out
 if out.exists():raise SystemExit('Refusing overwrite of historical evidence')
 out.mkdir(parents=True);body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body'];plan=runpy.run_path(str(HERE/'integrate-hit-fx.py'))['prepare']();base=ROOT/'build/staging/horizon';patched={}
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 meta=runpy.run_path(str(HERE/'hit-fx-patch.py'))['apply'](edit,once)
 for path,t in patched.items():p=out/'patched'/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(t,encoding='utf8')
 native=patched['crates/horizon-client/src/game/remote.rs'];assert native.index('!self.client_hit_fx_packet(')<native.index('&& !self.pvp_mutes_sound(&name)',native.index('!self.client_hit_fx_packet('));assert 'let source = horizon_client::settings::sound_source(self.remote_hurt_sound(p.source_type));'in native
 assert 'source:usize' in patched['crates/horizon-client-render/src/audio.rs'];assert 'self.play_seeded(event,at,volume,pitch,seed)'in patched['crates/horizon-client-render/src/audio.rs'];assert 'registry.add_file("liquidbounce",include_bytes!("hit-fx-sounds.json")'in patched['crates/horizon-client-render/src/sound_events.rs']
 registry=json.loads((HERE/'hit-fx-sounds.json').read_text(encoding='utf8'));assert len(registry)==36
 import re
 encoded=dict(re.findall(r'\("([^"]+)","([^"]+)"\)',(HERE/'hit-fx-audio-data.rs').read_text(encoding='utf8')));hashes={}
 assert set(encoded)==set(registry)
 for name,text in encoded.items():
  original=ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/sounds/{name}.ogg';data=base64.b64decode(text);assert data==original.read_bytes()and data.startswith(b'OggS');hashes[name]=hashlib.sha256(data).hexdigest()
 for name in ['hit_fx.rs','hit_fx_sounds.rs','hit_fx_config.rs','hit_fx_runtime.rs']:shutil.copyfile(owned(name),out/name)
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8');config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]HitFx}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.hitFX"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','fn parse_multi_mask(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{HitFx(HitFxSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n#[path="hit_fx_config.rs"]mod settings;pub use settings::*;pub fn parse_fixture(tree:&Value)->SettingsPatch{let mut p=SettingsPatch::default();settings::parse_hit_fx(&mut p,&nodes(tree,"HitFX","HitFX").unwrap());p}\n';(out/'config.rs').write_text(config,encoding='utf8')
 audio='use std::{fs,path::Path};\n'+body(patched['crates/horizon-client-render/src/audio.rs'],'fn decode(').replace('fn decode(','pub(crate) fn decode(',1)+'\n'+(HERE/'hit-fx-audio-data.rs').read_text(encoding='utf8').replace('const HIT_FX_OGG','pub(crate) const HIT_FX_OGG');(out/'audio_fixture.rs').write_text(audio,encoding='utf8')
 lib='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;pub mod hit_fx;mod config;mod audio_fixture;\n'+f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/random.rs").as_posix()}"]mod random;\n'
 lib+='''pub mod skins{pub fn base64_decode(s:&str)->Option<Vec<u8>>{use base64::Engine;base64::engine::general_purpose::STANDARD.decode(s).ok()}}
pub mod mod_api{pub trait Host{fn client_hit_fx_context(&self,id:i32)->String;}pub struct ClientHitFxEvent<'a>{pub input:&'a str,pub output:&'a mut String}}
pub mod runtime{use std::cell::RefCell;pub struct Modules{pub hit_fx:crate::hit_fx::HitFx,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{name=="HitFX"&&self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}pub struct LiquidBounceHooks(State);impl Default for LiquidBounceHooks{fn default()->Self{Self(State{behaviors:RefCell::new(Modules{hit_fx:Default::default(),active:true}),registry:()})}}impl LiquidBounceHooks{pub fn state(&self)->&State{&self.0}}}
#[path="hit_fx_runtime.rs"]mod hit_fx_runtime;
'''+(HERE/'hit-fx-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="hit-fx-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\nbase64="0.22"\nsymphonia={version="0.5",default-features=false,features=["ogg","vorbis"]}\n',encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-hit-fx-locales.py'));locales=out/'localization';locales.mkdir();counts={}
 for lang in ['zh_cn','zh_tw']:
  dest=locales/f'display_{lang}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json',dest);updater['update'](lang,dest);d=json.loads(dest.read_text(encoding='utf8'));assert all(d['descriptions'].get(k)==v for k,v in updater['descriptions'](lang).items());counts[lang]=len(updater['descriptions'](lang))
 (out/'shared-plan.json').write_text(json.dumps({'files':[str(p.relative_to(ROOT))for p in plan.changed]},indent=2)+'\n',encoding='utf8')
 report={'status':'static-passed-compile-and-focused-smoke-deferred','sharedFiles':len(plan.changed),'nativeFiles':len(patched),'metadata':meta,'soundGroups':26,'originalOgg':len(hashes),'soundHashes':hashes,'localeKeys':counts,'testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'scope':'Complete original HitFX settings and all 26 sound groups/36 Ogg assets, last-target attack and alive/distance/source/sound gates, native tracking emitters and redstone destroy particles, seeded source-preserving replacement sound, actual local-player DamageEvent self sound, real Horizon decoder bytes from original resources. In-memory shared/native integration only; no Rust/Cargo/GPU; actual-game parity deferred.'}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps({k:v for k,v in report.items()if k!='soundHashes'},ensure_ascii=False))
if __name__=='__main__':main()
