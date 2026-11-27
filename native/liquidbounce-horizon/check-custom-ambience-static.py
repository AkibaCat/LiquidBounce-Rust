"""Read-only transactional composition and focused fixture preparation. Never runs Rust/Cargo/GPU."""
from pathlib import Path
import argparse,base64,hashlib,json,runpy,shutil
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1]

def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'custom_ambience.rs':ROOT/'native/liquidbounce-behaviors/src/custom_ambience.rs','custom_ambience_runtime.rs':NATIVE/'engine/src/custom_ambience_runtime.rs','custom_ambience_config.rs':NATIVE/'engine/src/config/custom_ambience.rs'}[name]

def main():
 parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args();out=ROOT/args.out;assert not out.exists(),'Preserve historical evidence';out.mkdir(parents=True)
 body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body'];plan=runpy.run_path(str(HERE/'integrate-custom-ambience.py'))['prepare']()
 (out/'shared-plan.json').write_text(json.dumps({'files':[str(p.relative_to(ROOT))for p in plan.changed],'sha256':{str(p.relative_to(ROOT)):hashlib.sha256(s.encode()).hexdigest()for p,s in plan.changed.items()}},indent=2)+'\n',encoding='utf8')
 base=ROOT/'build/staging/horizon';patched={}
 def once(t,a,b):assert t.count(a)==1,(a,t.count(a));return t.replace(a,b)
 def edit(path,fn):patched[path]=fn(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 patch=runpy.run_path(str(HERE/'custom-ambience-patch.py'));metadata=patch['apply'](edit,once)
 for path,text in patched.items():dest=out/'patched'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(text,encoding='utf8')
 native=patched['crates/horizon-client/src/game/custom_ambience.rs'];renderer=patched['crates/horizon-client-render/src/gfx/custom_ambience.rs'];gfx=patched['crates/horizon-client-render/src/gfx.rs'];game=patched['crates/horizon-client/src/game.rs']
 compact=lambda text:''.join(text.split())
 for token in ['static SNAPSHOT:RefCell<Value>','pub(super) fn snapshot_time','definition!="minecraft:overworld"','else{Some(snapshot())}','default_clock.as_deref()','if self.native_window()&&self.title.is_none()']:
  assert compact(token) in compact(native),token
 # Ordinary reads contain no dispatch, even while a different mod owns behaviors.borrow_mut().
 query=body(native,'fn client_custom_ambience_query(');assert query.count('native_client_custom_ambience')==1 and 'input["action"]=="frame"'in compact(query)
 assert 'self.client_custom_ambience_refresh();'in body(game,'fn render_frame(')
 hooks=patched['crates/horizon-client/src/game/native_hooks.rs']
 for anchor in ['pub(super) fn native_client_tick_pre(', 'pub(super) fn native_frame(']:
  function=body(hooks,anchor);assert function.index('self.client_custom_ambience_refresh();')<function.index('mod_api::dispatch_')
 for token in ['ambience_fog_sky=sky','self.client_custom_ambience_sky(&mut sky)','self.frame_fog(camera, &ambience_fog_sky, biome, water_fog)']:assert token in game,token
 for token in ['array_stride: 40','wgpu::CompareFunction::GreaterEqual','depth_write_enabled: Some(i == 1)','self.pending.as_ref()','address_mode_u: wgpu::AddressMode::Repeat','mag_filter: wgpu::FilterMode::Nearest','min_filter: wgpu::FilterMode::Nearest']:assert token in renderer,token
 assert 'std::fs::read'not in renderer and 'std::thread::spawn'not in renderer
 for token in ['custom_ambience::clear_color(clear)','custom_ambience::lightmap_color','night_color * night_vision','custom_ambience::lightmap_fill(&mut pixels)']:assert token in gfx,token
 assert gfx.index('custom_ambience::clear_color(clear)')<gfx.index('linear_blend::stored_clear(clear')
 shader=patched['crates/horizon-client-render/src/gfx/custom-ambience.wgsl'];assert'if color.a==0.0{discard;}'in shader
 (out/'render.rs').write_text(renderer+'\npub(crate)fn assets()-> &\'static [(&\'static str,&\'static str)]{&PNGS}\n',encoding='utf8')
 for name in ['custom-ambience.wgsl','custom-ambience-textures.rs']:(out/name).write_text(patched['crates/horizon-client-render/src/gfx/'+name],encoding='utf8')
 assets={}
 for name,file in patch['ASSETS']:
  path=ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/particles/{file}.png';data=path.read_bytes();assert data.startswith(b'\x89PNG\r\n\x1a\n');assert base64.b64encode(data).decode()in patched['crates/horizon-client-render/src/gfx/custom-ambience-textures.rs'];assets[file+'.png']=hashlib.sha256(data).hexdigest()
 (out/'assets.json').write_text(json.dumps(assets,indent=2)+'\n',encoding='utf8')
 source=(ROOT/'native/liquidbounce-behaviors/src/target_renderer.rs').read_text(encoding='utf8');(out/'target_renderer.rs').write_text('use serde_json::Value;\n#[derive(Clone,Debug)]\n'+body(source,'pub struct Node')+'\n',encoding='utf8')
 for name in ['custom_ambience.rs','custom_ambience_runtime.rs','custom_ambience_config.rs']:
  source=owned(name).read_text(encoding='utf8').replace('../vendor/minecraft-math/sine-table.bin',(ROOT/'native/liquidbounce-behaviors/vendor/minecraft-math/sine-table.bin').as_posix());(out/name).write_text(source,encoding='utf8')
 shutil.copyfile(ROOT/'build/jump-effect-revision-04/hud.rs',out/'hud.rs');(out/'native_game.rs').write_text(native,encoding='utf8')
 shutil.copyfile(HERE/'custom-ambience-vectors.json',out/'custom-ambience-vectors.json')
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8');config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]CustomAmbience}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.customAmbience"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{CustomAmbience(CustomAmbienceSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n#[path="custom_ambience_config.rs"]mod settings;pub use settings::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code,unused_imports)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;mod target_renderer;pub mod custom_ambience;mod render;mod config;mod hud;mod native_game;\n'
 for name in ['easing','numeric']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]mod {name};\n'
 for module,path,anchor in [('environment','crates/horizon-client-render/src/environment.rs','pub struct Sky'),('fog','crates/horizon-client-render/src/fog.rs','pub struct FogData')]:
  source=(base/path).read_text(encoding='utf8');lib+='pub mod '+module+'{#[derive(Clone,Copy,Debug,Default,PartialEq,serde::Serialize,serde::Deserialize)]'+body(source,anchor)+'}\n'
 lib+='''
pub mod skins{pub fn base64_decode(s:&str)->Option<Vec<u8>>{use base64::Engine;base64::engine::general_purpose::STANDARD.decode(s).ok()}}
pub mod gfx{pub mod custom_ambience{pub use crate::render::begin_frame;}}
pub mod mod_api{pub struct ClientCustomAmbienceEvent<'a>{pub input:&'a str,pub output:&'a mut String}pub trait Host{}impl Host for(){}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};pub struct Modules{pub custom_ambience:crate::custom_ambience::CustomAmbience,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{name=="CustomAmbience"&&self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}#[derive(Clone)]pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{behaviors:RefCell::new(Modules{custom_ambience:Default::default(),active:true}),registry:()}))}}impl LiquidBounceHooks{pub fn state(&self)->&State{&self.0}}}
#[path="custom_ambience_runtime.rs"]mod custom_ambience_runtime;
pub struct Player{pub position:glam::DVec3}pub struct Dimension{pub default_clock:Option<String>}
pub struct Game<'a>{pub hooks:runtime::LiquidBounceHooks,pub title:Option<()>,pub player:Option<Player>,pub native:bool,pub default_clock:Option<String>,pub marker:std::marker::PhantomData<&'a ()>}
impl Default for Game<'_>{fn default()->Self{Self{hooks:Default::default(),title:None,player:Some(Player{position:glam::DVec3::ZERO}),native:true,default_clock:Some("minecraft:overworld".into()),marker:std::marker::PhantomData}}}
impl Game<'_>{fn native_window(&self)->bool{self.native}fn dimension_type(&self)->Dimension{Dimension{default_clock:self.default_clock.clone()}}fn camera_orientation(&self)->glam::Quat{glam::Quat::IDENTITY}fn native_client_custom_ambience(&self,input:&str,output:&mut String){self.hooks.custom_ambience(&(),&mut mod_api::ClientCustomAmbienceEvent{input,output});}}
'''
 lib+=(HERE/'custom-ambience-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 manifest=(ROOT/'build/potion-fx-static-04/Cargo.toml').read_text(encoding='utf8').replace('name="potion-fx-smoke"','name="custom-ambience-smoke"');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-custom-ambience-locales.py'));locale=out/'localization';locale.mkdir();counts={};keys=['liquidbounce.module.customAmbience.'+p for p in updater['TEXTS']]
 (out/'description-keys.json').write_text(json.dumps(keys,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
 for lang in ['zh_cn','zh_tw']:
  target=locale/f'display_{lang}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json',target);updater['update'](lang,target);data=json.loads(target.read_text(encoding='utf8'));i=1 if lang=='zh_cn'else 2
  assert all(data['descriptions'].get('liquidbounce.module.customAmbience.'+k)==v[i]for k,v in updater['TEXTS'].items());counts[lang]=len(keys)
 source_checks=['ModuleCustomAmbience.kt','worldeffects/WorldParticles.kt','worldeffects/WorldParticlesMode.kt','worldeffects/modes/WorldParticlesSimple.kt'];source_hashes={}
 for source in source_checks:
  file=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/features/module/modules/render/customambience'/source;source_hashes[source]=hashlib.sha256(file.read_bytes()).hexdigest()
 report={'feature':'CustomAmbience','status':'static-passed-compile-smoke-deferred-parent-batch','sharedFiles':len(plan.changed),'nativeFiles':len(patched),'ownedFiles':12,'metadata':metadata,'assets':len(assets),'localeDescriptions':counts,'preparedTests':lib.count('#[test]'),'sourceOracleVectors':len(json.loads((HERE/'custom-ambience-vectors.json').read_text(encoding='utf8'))),'sourceOracleReport':'build/custom-ambience-source-oracle-01/report.json','testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'sourceSha256':source_hashes,'scope':'Complete source schema, weather/time/precipitation, actual fog/sky/background/lightmap paths, Simple world-particle lifetime, original 11 PNGs, camera billboard renderer and independent native snapshot for nonreentrant provider reads. Client clock consumers include native API, item clocks, environment sky brightness, shader packs, CEM and random textures. Canonical server clocks remain unchanged.','validation':'No Rust/Cargo/GPU executed. Transactional shared/native composition is read-only. Prepared fixture includes actual production dispatch, owned native game methods and real Sky/FogData declarations with a declared Game/Host actor provider; this does not establish actual-game parity. Actual MC 26.3 Mth/Vec3 Java oracle produced 32 shell/hash vectors.','limits':['Native RNG preserves source distribution, not the Kotlin random stream.','Equal Radius endpoints cause source Random.nextDouble to throw after lastSpawnTime update; native leaves no particle after updating timestamp.','Java cbrt/libm and JOML/native billboard precision can differ at float rounding boundaries; shell positions use 1e-8 comparison, Vec3 hash uses exact oracle inputs.','Snapshot refresh occurs at client tick/frame boundaries before other mod dispatch; settings changes become observable at the next boundary.']}
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
