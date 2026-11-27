"""Prepare the full ESP/OutlineGlow focused fixture. No Rust/Cargo/GPU execution."""
from pathlib import Path
import argparse,hashlib,json,runpy,shutil,zipfile
HERE=Path(__file__).resolve().parent;NATIVE=HERE if(HERE/'integrate-chest-stealer.py').is_file()else HERE.parents[1];ROOT=NATIVE.parents[1]
def owned(name):
 if(HERE/name).is_file():return HERE/name
 return {'esp.rs':ROOT/'native/liquidbounce-behaviors/src/esp.rs','esp_runtime.rs':NATIVE/'engine/src/esp_runtime.rs','esp_config.rs':NATIVE/'engine/src/config/esp.rs'}[name]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);args=a.parse_args();out=ROOT/args.out;assert not out.exists(),'Preserve historical evidence';out.mkdir(parents=True)
 body=runpy.run_path(str(NATIVE/'test-reset-vl.py'))['body'];plan=runpy.run_path(str(HERE/'integrate-esp.py'))['prepare']()
 (out/'shared-plan.json').write_text(json.dumps({'files':[str(p.relative_to(ROOT))for p in plan.changed],'sha256':{str(p.relative_to(ROOT)):hashlib.sha256(s.encode()).hexdigest()for p,s in plan.changed.items()}},indent=2)+'\n',encoding='utf8')
 base=ROOT/'build/staging/horizon';patched={}
 def once(t,a,b):assert t.count(a)==1,(a,t.count(a));return t.replace(a,b)
 def edit(path,fn):patched[path]=fn(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 meta=runpy.run_path(str(HERE/'esp-patch.py'))['apply'](edit,once)
 esp_count=len(patched);item_meta=runpy.run_path(str(NATIVE/'item-esp-patch.py'))['apply'](edit,once)
 for path,text in patched.items():dest=out/'patched'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(text,encoding='utf8')
 game=patched['crates/horizon-client/src/game.rs'];assert game.index('outline_glow::begin_frame();')<game.index('self.client_item_esp_frame(alpha,camera);')<game.index('self.client_esp_frame(camera,alpha);')<game.index('self.client_storage_esp_frame(camera,alpha);')<game.index('let mut entities = std::mem::take(&mut self.entity_vertices);')
 checks={'crates/horizon-client/src/game/creatures/mob_geometry.rs':['storage_esp::capture_entity(mob.id','outline_glow::capture_entity(id','projectile_outline'], 'crates/horizon-client/src/game/perspective.rs':['outline_glow::capture_entity(self.own_entity_id()'], 'crates/horizon-client/src/game/armor_stands.rs':['outline_glow::capture_entity(a.id'], 'crates/horizon-client/src/game/end.rs':['outline_glow::capture_entity(crystal.id'], 'crates/horizon-client/src/game/dragon.rs':['outline_glow::capture_entity(d.id','outline_glow::capture_entity(b.id'], 'crates/horizon-client/src/game/boats.rs':['storage_esp::capture_entity(b.id'], 'crates/horizon-client/src/game/minecarts.rs':['storage_esp::capture_entity(c.id'], 'crates/horizon-client/src/game/potions.rs':['outline_glow::capture_entity(t.id'], 'crates/horizon-client/src/game/fireworks.rs':['outline_glow::capture_entity(r.id'], 'crates/horizon-client/src/game/fishing.rs':['outline_glow::capture_entity(id'], 'crates/horizon-client/src/game/trident.rs':['outline_glow::capture_entity(t.id'], 'crates/horizon-client/src/game/item_draw.rs':['outline_glow::capture_entity(item.id','item_outline']}
 for path,needles in checks.items():
  for needle in needles:assert needle in patched[path],(path,needle)
 assert 'Owner::StorageEsp' in patched['crates/horizon-client/src/game/storage_esp.rs']
 assert 'super::outline_glow::capture_entity(id,vertices)' in patched['crates/horizon-client-render/src/gfx/storage_esp.rs']
 shared=(NATIVE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub enum ModuleKind{#[default]Esp,Breadcrumbs,Tracers,StorageEsp}impl ModuleKind{fn key(self)->&\'static str{match self{Self::Esp=>"liquidbounce.module.esp",Self::Breadcrumbs=>"liquidbounce.module.breadcrumbs",Self::Tracers=>"liquidbounce.module.tracers",Self::StorageEsp=>"liquidbounce.module.storageESP"}}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{Esp(EspSetting),Breadcrumbs(BreadcrumbsSetting),Tracers(TracersSetting),StorageEsp(StorageEspSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
 for name in ['breadcrumbs','tracers','storage_esp']:
  config+=f'#[path="{(NATIVE/f"engine/src/config/{name}.rs").as_posix()}"]mod {name};pub use {name}::*;\n'
 config+='#[path="esp_config.rs"]mod esp;pub use esp::*;\n'
 # Expose production parser only inside this fixture's configuration module.
 config+='pub fn roundtrip(s:&crate::esp::Esp,format:JsonFormat)->(crate::esp::Esp,Value){let tree=serialize_esp(s,&CommonSettings{enabled:true,hidden:false,key_bind:json!({})},format,&str::to_owned).unwrap();let mut patch=SettingsPatch{module:ModuleKind::Esp,changes:vec![],issues:vec![]};esp::parse_esp(&mut patch,&nodes(&tree,"ESP","ESP").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=crate::esp::Esp::default();for c in patch.changes{if let SettingChange::Esp(v)=c{v.apply(&mut restored);}}(restored,tree)}\n'
 (out/'config.rs').write_text(config,encoding='utf8');shutil.copyfile(owned('esp_config.rs'),out/'esp_config.rs');shutil.copyfile(owned('esp.rs'),out/'esp.rs')
 lib='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 for name in ['breadcrumbs','tracers','combine_mobs','teams','storage_esp']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 lib+='pub mod esp;mod config;\n'
 vertex=body((base/'crates/horizon-client-render/src/mesher.rs').read_text(encoding='utf8'),'pub struct Vertex');lib+='pub mod mesher{#[repr(C)]#[derive(Clone,Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]'+vertex+'}\n'
 targeting=(ROOT/'native/liquidbounce-behaviors/src/targeting.rs').read_text(encoding='utf8').split('#[cfg(test)]\nmod tests {')[0].replace('"targeting_facts.rs"',f'"{(ROOT/"native/liquidbounce-behaviors/src/targeting_facts.rs").as_posix()}"');(out/'targeting.rs').write_text(targeting,encoding='utf8')
 lib+='pub mod targeting;\n'+f'#[path="{(NATIVE/"engine/src/targeting.rs").as_posix()}"]pub mod targeting_decode;\n'
 bridge=owned('esp_runtime.rs').read_text(encoding='utf8').replace('crate::targeting::snapshot','crate::targeting_decode::snapshot');(out/'esp_runtime.rs').write_text(bridge,encoding='utf8')
 lib+='''pub mod teams_runtime{pub fn facts(_:&serde_json::Value)->Option<crate::teams::Facts>{None}}
pub mod anti_bot_runtime{pub struct Query;pub fn query(_:&serde_json::Value)->Option<Query>{None}}
pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientEspEvent<'a>{pub input:&'a str,pub output:&'a mut String}}
pub mod runtime{use std::{cell::RefCell,rc::Rc};use crate::targeting::*;pub struct Module;impl Module{pub fn enabled(&self)->bool{false}}pub struct Registry;impl Registry{pub fn module(&self,_:u32)->Option<Module>{None}}pub struct Modules{pub esp:crate::esp::Esp,pub combine_mobs:crate::combine_mobs::CombineMobs,pub query_ids:std::collections::BTreeMap<&'static str,u32>,pub disabled:bool}impl Modules{pub fn query_running(&self,_:&Registry,n:&str)->bool{match n{"ESP"=>!self.disabled,"CombineMobs"=>true,_=>false}}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:Registry,pub global_targets:GlobalTargets,pub friends:FriendBook}impl State{pub fn target_info(&mut self,_:i32,s:&crate::targeting_decode::Snapshot)->TargetInfo{self.friends.tag(s.entity.kind,s.player_name.as_deref())}}pub struct LiquidBounceHooks{state:Rc<RefCell<State>>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(RefCell::new(State{behaviors:RefCell::new(Modules{esp:Default::default(),combine_mobs:Default::default(),query_ids:std::collections::BTreeMap::from_iter([("FreeLook",0)]),disabled:false}),registry:Registry,global_targets:Default::default(),friends:Default::default()}))}}}impl LiquidBounceHooks{pub fn state(&self)->std::cell::RefMut<'_,State>{self.state.borrow_mut()}}}
mod runtime_bridge{include!("esp_runtime.rs");}
'''
 for name in ['outline_owner','outline_glow','esp_plain','storage_esp']:
  code=patched[f'crates/horizon-client-render/src/gfx/{name}.rs'];(out/(name+'-body.rs')).write_text('\n'.join(l for l in code.splitlines()if not l.startswith('//!'))+'\n',encoding='utf8')
  if name=='outline_glow':
   with(out/(name+'-body.rs')).open('a',encoding='utf8')as f:f.write('pub(crate)fn fixture_models()->Vec<crate::mesher::Vertex>{REGISTRY.with_borrow(|r|r.models.clone())}\n')
 for name in ['breadcrumbs.wgsl','storage-esp-model.wgsl','storage-esp-sobel.frag']:shutil.copyfile(NATIVE/name,out/name)
 for name in ['outline-glow.wgsl','outline-glow-blur-h.frag','outline-glow-blur-v.frag']:shutil.copyfile(HERE/name,out/name)
 shutil.copytree(NATIVE/'breadcrumbs-shaders',out/'breadcrumbs-shaders')
 lib+='mod gfx{pub mod outline_owner{include!("outline_owner-body.rs");}pub mod outline_glow{include!("outline_glow-body.rs");}pub mod esp_plain{include!("esp_plain-body.rs");}pub mod storage_esp{include!("storage_esp-body.rs");}}\n'
 lib+=(HERE/'esp-smoke.rs').read_text(encoding='utf8');lib+='\n#[cfg(test)]#[path="esp-gpu.rs"]mod esp_gpu;\n';(out/'lib.rs').write_text(lib,encoding='utf8');shutil.copyfile(HERE/'esp-gpu.rs',out/'esp-gpu.rs')
 manifest=(ROOT/'build/tracers-smoke-01/Cargo.toml').read_text(encoding='utf8').replace('name="tracers-smoke"','name="esp-smoke"').replace('features=["glsl-in"]','features=["glsl-in","wgsl-in"]');(out/'Cargo.toml').write_text(manifest,encoding='utf8')
 updater=runpy.run_path(str(HERE/'update-esp-locales.py'));locale=out/'localization';locale.mkdir();counts={}
 for lang in ['zh_cn','zh_tw']:
  target=locale/f'display_{lang}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{lang}.json',target);updater['update'](lang,target);data=json.loads(target.read_text(encoding='utf8'));expected=updater['descriptions'](lang);assert all(data['descriptions'].get(k)==v for k,v in expected.items());counts[lang]=len(expected)
 source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce';sources=[source/'features/module/modules/render/esp/ModuleESP.kt',*(source/'features/module/modules/render/esp/modes').glob('*.kt'),source/'render/effect/OutlineGlow.kt',source/'utils/render/Legacy2DMarkerRenderer.kt',source/'utils/math/KeyedAabb.kt']
 jar=Path.home()/'.gradle/caches/fabric-loom/26.3/minecraft-client.jar';vanilla={}
 with zipfile.ZipFile(jar)as z:
  for name in ['assets/minecraft/post_effect/entity_outline.json','assets/minecraft/shaders/post/entity_sobel.fsh','assets/minecraft/shaders/post/entity_outline_box_blur.fsh']:vanilla[name]=hashlib.sha256(z.read(name)).hexdigest()
 report={'feature':'ESP + shared OutlineGlow','status':'static-passed-compile-smoke-deferred-parent-batch','sharedFiles':len(plan.changed),'espNativeFiles':esp_count,'coexistingNativeFiles':len(patched),'metadata':meta,'itemEspMetadata':item_meta,'localeDescriptions':counts,'preparedTests':lib.count('#[test]'),'testManifest':str((out/'Cargo.toml').relative_to(ROOT)),'sourceSha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sources},'vanillaResources':vanilla,'scope':'Four complete ESP modes and all inactive settings; visual targeting/friend/tag/scoreboard facts, tick pose cache, strict range, original merge/2D/Legacy2D geometry. One real tagged atlas-alpha silhouette shared by ESP, ItemESP, StorageESP entities with original mask/over/gamma order and native model capture. StorageESP block meshes remain independent; its Vanilla chain gains the official H/V blur after Sobel.','validation':'No Rust/Cargo/GPU executed. Shared integration and complete ESP+ItemESP native patch composition are read-only. Focused fixture compiles production model/config/runtime/target decoder, generated renderers and real native Vertex with declared actor/Host providers. Actual native Game compilation and actual-game visual parity belong to parent grouped validation.','deferredChecks':['Official grouped native/API build with ESP and ItemESP.','Prepared focused fixture: all settings/modes/styles, cache/filter/colors, geometry, registry priorities, original GLSL and WGSL validation.','Focused GPU coexistence of ESP/ItemESP/StorageESP model silhouettes and Vanilla three-pass chain; no full frozen style matrix rerun.'],'limits':['Legacy2D JOML/native trigonometric precision may differ at rounding boundaries.','Native object containers without JoinSeq use native entity ids for identity; id reuse between tick preparation and frame is constrained by the native lifetime model.','Raw GUI scale 0 preserves source division boundary; JSON cannot encode nonfinite rectangle coordinates and the native overlay discards them.']}
 report['validation']=report['validation'].replace('No Rust/Cargo/GPU executed.','This preparation executes no Cargo or GPU work.').replace('Focused fixture compiles production','Focused fixture includes production')
 report['preparedIgnoredGpuTests']=1
 report['gpuSource']='esp-gpu.rs'
 report['focusedCommand']=['cargo','+stable','test','--offline','--manifest-path',str((out/'Cargo.toml').relative_to(ROOT))]
 report['gpuCommand']=report['focusedCommand']+['focused_gpu_esp_item_storage_masks_atlas_and_vanilla','--','--ignored','--nocapture']
 (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf8');print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
