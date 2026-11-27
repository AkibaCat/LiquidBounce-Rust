"""One focused Crosshair smoke: source behavior, complete settings, native HUD and locales."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug)]enum ModuleKind{Crosshair}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.crosshair"}}\n'
 for anchor in['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{Crosshair(settings::CrosshairSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/crosshair.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''#[test]fn all_settings_inactive_mode_and_source_tree_roundtrip(){let mut s=crate::crosshair::Crosshair::default();for field in crate::crosshair::fields(){let v=match field.kind{"MODE"=>json!("CS2"),"BOOLEAN"=>json!(!field.default.as_bool().unwrap()),"FLOAT"=>json!(0.75),"INT"=>json!(777),"COLOR"=>json!(0x88776655u32 as i32),"FLOAT_RANGE"=>json!({"from":4.5,"to":9.5}),"MULTI_CHOOSE"=>json!(["Charge","Attack"]),_=>unreachable!()};s.values.insert(field.path,v);}let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in[JsonFormat::File,JsonFormat::Interop]{let tree=serialize_crosshair(&s,&common,format,&str::to_owned).unwrap();let mut p=SettingsPatch{module:ModuleKind::Crosshair,changes:vec![],issues:vec![]};settings::parse_crosshair(&mut p,&nodes(&tree,"Crosshair","Crosshair").unwrap());assert!(p.issues.is_empty(),"{:?}",p.issues);let mut restored=crate::crosshair::Crosshair::default();for SettingChange::Crosshair(v)in p.changes{v.apply(&mut restored)}assert_eq!(s.values,restored.values);let values=tree["value"].as_array().unwrap();let mode=values.iter().find(|v|v["name"]=="Mode").unwrap();assert_eq!(mode["active"],"CS2");assert!(mode["choices"]["Circle"]["value"].as_array().unwrap().iter().any(|v|v["name"]=="Radius"));let bar=values.iter().find(|v|v["name"]=="CooldownBar").unwrap();assert!(bar["value"].as_array().unwrap().iter().any(|v|v["name"]=="ReadyFlash"));}}\n'''
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_client;\n'
 lib+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/crosshair.rs").as_posix()}"]pub mod crosshair;\nmod config;\n'
 lib+='''pub mod mod_api{pub trait Host{}pub struct NullHost;impl Host for NullHost{}pub struct ClientCrosshairEvent<'a>{pub input:&'a str,pub geometry:&'a mut String}}
pub mod runtime{use std::{cell::{Cell,RefCell},rc::Rc};pub struct Module(pub bool);impl Module{pub fn enabled(&self)->bool{self.0}}pub struct Registry{pub enabled:Cell<bool>,pub running:Cell<bool>}impl Registry{pub fn module(&self,_:u32)->Option<Module>{Some(Module(self.enabled.get()))}}pub struct Modules{pub crosshair:crate::crosshair::Crosshair,pub query_ids:std::collections::BTreeMap<&'static str,u32>}impl Modules{pub fn query_running(&self,r:&Registry,_:&str)->bool{r.running.get()}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:Registry}pub struct LiquidBounceHooks{state:Rc<State>}impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Rc::new(State{behaviors:RefCell::new(Modules{crosshair:Default::default(),query_ids:std::collections::BTreeMap::from_iter([("Crosshair",0)])}),registry:Registry{enabled:Cell::new(true),running:Cell::new(true)}})}}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.state.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/crosshair_runtime.rs").as_posix()}"]mod crosshair_runtime;\n'
 attack=(ROOT/'upstream/Horizon/crates/horizon-client/src/attack_strength.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0];(out/'attack_strength.rs').write_text(attack,encoding='utf8');lib+='pub mod attack_strength;\n'
 native=(HERE/'crosshair-native-fixture.rs').read_text(encoding='utf8').replace('NATIVE_PATH',(out/'crosshair.rs').as_posix()).replace('CHARGE_TICKS',body((ROOT/'upstream/Horizon/crates/horizon-client/src/game/crossbow.rs').read_text(encoding='utf8'),'    pub(super) fn charge_ticks('))
 native=native.replace('NATIVE_FILL',body((ROOT/'upstream/Horizon/crates/horizon-client/src/game/native_hooks.rs').read_text(encoding='utf8'),'    pub(super) fn native_fill('))
 lib+=native+(HERE/'crosshair-smoke.rs').read_text(encoding='utf8')
 # The owned native file includes its copied sibling raster at this exact path.
 shutil.copyfile(HERE/'crosshair-game.rs',out/'crosshair.rs');shutil.copyfile(HERE/'crosshair-raster.rs',out/'crosshair_raster.rs')
 patched={};base=ROOT/'build/staging/horizon';error=None;metadata=None
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'crosshair-patch.py'))['apply'](edit,once)
  for path,text in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text,encoding='utf8')
 except Exception as e:error=repr(e)
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-crosshair-locales.py'));counts={}
 for language in['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target);d=json.loads(target.read_text(encoding='utf8'));counts[language]=sum(k.startswith('liquidbounce.module.crosshair')for k in d['descriptions']);assert not any(k.startswith('liquidbounce.module.crosshair')for k in d)
 loc=(ROOT/'native/liquidbounce-interop/src/localization.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0]
 for name in ['display_zh_cn','display_zh_tw','en_us','zh_cn','zh_tw','module_keys']:
  target=locale/f'{name}.json'if name.startswith('display_')else ROOT/f'native/liquidbounce-interop/resources/localization/{name}.json';loc=loc.replace(f'../resources/localization/{name}.json',target.as_posix())
 (out/'localization.rs').write_text(loc,encoding='utf8');lib+='mod localization;\n#[test]fn actual_localization_dictionary_resolves_all_crosshair_keys(){assert_eq!(localization::display_label("zh_cn","ReadyFlash"),"就绪闪色");assert_eq!(localization::display_label("zh_tw","DynamicRadiusMultiplier"),"動態半徑倍率");for l in["zh_cn","zh_tw"]{for f in crate::crosshair::fields(){let key=format!("{}.description",crate::config::crosshair_key(&f.path));assert!(!localization::describe(l,&key).contains("liquidbounce"),"{key}");}}}\n';(out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="crosshair-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\nserde={version="1",features=["derive"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':metadata,'patchedFiles':len(patched),'localeKeys':counts,'scope':'One focused usability smoke. Exact production behavior, full File/Interop both-mode settings and production Runtime/native HUD bridge with declaration-only Game providers; native AttackStrength and Crossbow charge function bodies; actual localization source over copied dictionaries. Circle evaluates pinned 256-angle LUT, quantized inner radius and physical-pixel derivative AA, using existing native HudBatch alpha pipeline. No additional GPU/shader matrix. Official native/engine aggregate and actual in-game parity remain root-owned.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
