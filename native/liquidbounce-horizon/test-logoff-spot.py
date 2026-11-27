"""One focused production-native/config/runtime/packet usability test, no official build."""
from pathlib import Path
import argparse,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--test-filter');args=p.parse_args();out=ROOT/args.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite historical evidence')
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug)]pub enum ModuleKind{LogoffSpot}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.logoffSpot"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{LogoffSpot(LogoffSpotSetting)}\n'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/logoff_spot.rs").as_posix()}"]mod logoff_spot;pub use logoff_spot::*;pub fn parse_logoff_for_test(p:&mut SettingsPatch,r:&Nodes){{logoff_spot::parse_logoff_spot(p,r)}}\n'
 config=config.replace('pub struct SettingsPatch','#[derive(Default)]pub struct SettingsPatch').replace('pub enum ModuleKind{LogoffSpot}','pub enum ModuleKind{LogoffSpot}impl Default for ModuleKind{fn default()->Self{Self::LogoffSpot}}').replace("fn nodes<'a>(","pub fn nodes<'a>(");(out/'config.rs').write_text(config,encoding='utf8')
 locale=out/'interop/resources/localization';locale.mkdir(parents=True,exist_ok=True);updater=runpy.run_path(str(HERE/'update-logoff-spot-locales.py'));counts={}
 for name in ['en_us','zh_cn','zh_tw','module_keys','display_zh_cn','display_zh_tw']:shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/{name}.json',locale/f'{name}.json')
 for lang in ['zh_cn','zh_tw']:
  target=locale/f'display_{lang}.json';updater['update'](lang,target);counts[lang]=sum(k.startswith(updater['KEY'])for k in json.loads(target.read_text(encoding='utf8'))['descriptions'])
 (out/'interop/src').mkdir(exist_ok=True);(out/'interop/src/localization.rs').write_text((ROOT/'native/liquidbounce-interop/src/localization.rs').read_text(encoding='utf8').split('#[cfg(test)]')[0],encoding='utf8')
 lib='#![allow(dead_code,irrefutable_let_patterns,unused_imports)]\nextern crate self as horizon_client;extern crate self as liquidbounce_interop;\n#[path="interop/src/localization.rs"]pub mod localization;\nmod config;\n'
 lib+='''pub mod lang{pub fn code()->String{"en_us".into()}}pub mod mod_api{pub trait Host{fn chat(&mut self,s:&str);fn client_logoff_spot_clear(&mut self);}pub struct ClientLogoffSpotEvent<'a>{pub input:&'a str,pub output:&'a mut String}pub struct Target{pub id:i32}pub struct Spectator{pub target:Option<i32>}pub enum ServerboundPlayPacket{Attack(Target),Interact(Target),SpectatorAction(Spectator),Other}}
pub mod packets{pub struct PacketEvent<'a,P>{pub packet:&'a mut P,pub cancelled:bool}impl<'a,P>PacketEvent<'a,P>{pub fn new(packet:&'a mut P)->Self{Self{packet,cancelled:false}}pub fn cancel(&mut self){self.cancelled=true;}}pub trait PacketListener{fn outgoing(&self,event:&mut PacketEvent<'_,crate::mod_api::ServerboundPlayPacket>);}}
pub mod runtime{use std::{rc::Rc,cell::RefCell,sync::Arc};pub struct Modules{pub logoff_spot:Arc<crate::logoff_spot_runtime::LogoffSpotService>,pub active:bool}impl Modules{pub fn query_running(&self,_:&(),name:&str)->bool{assert_eq!(name,"LogoffSpot");self.active}}pub struct State{pub behaviors:RefCell<Modules>,pub registry:()}#[derive(Clone)]pub struct LiquidBounceHooks(Rc<State>);impl Default for LiquidBounceHooks{fn default()->Self{Self(Rc::new(State{behaviors:RefCell::new(Modules{logoff_spot:Arc::default(),active:true}),registry:()}))}}impl LiquidBounceHooks{pub fn state(&self)->Rc<State>{self.0.clone()}}}
'''
 lib+=f'#[path="{(HERE/"engine/src/logoff_spot_runtime.rs").as_posix()}"]mod logoff_spot_runtime;\n'
 fixture=(HERE/'logoff-spot-native-fixture.rs').read_text(encoding='utf8').replace('@@GAME@@',(HERE/'logoff-spot-game.rs').as_posix()).replace('// @@HOST@@',(HERE/'logoff-spot-host.rs').read_text(encoding='utf8')).replace('// @@REMOTE@@',(HERE/'logoff-spot-remote.rs').read_text(encoding='utf8'))
 lib+=fixture.rstrip()[:-1]+'\n#[cfg(test)]mod usability{use super::*;\n'+(HERE/'logoff-spot-smoke.rs').read_text(encoding='utf8')+'\n}}\n';(out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="logoff-spot-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\nserde={version="1",features=["derive"]}\n',encoding='utf8')
 patched={};error=None;metadata=None;base=ROOT/'build/staging/horizon'
 def once(t,o,n):assert t.count(o)==1,(o,t.count(o));return t.replace(o,n)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 try:
  metadata=runpy.run_path(str(HERE/'logoff-spot-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as e:error=repr(e)
 plan=runpy.run_path(str(HERE/'integrate-logoff-spot.py'))['prepare']();(out/'shared-plan.json').write_text(json.dumps({'steps':plan.steps,'files':list(plan.text)},indent=2)+'\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');command=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')];command+=([args.test_filter]if args.test_filter else[])
 r=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 report={'passed':r.returncode==0 and error is None,'compileExit':r.returncode,'patchError':error,'metadata':metadata,'patchedFiles':len(patched),'localeKeys':counts,'scope':'One focused usability test compiling production config, packet listener, runtime, native Game/Host bridge, and actual locale resolver. Declaration-only entity/world/Host/packet providers exercise fresh native player snapshot, visual gear/skin, scoreboard health, duplicate capture exclusion, Blink/fake exclusion, random entity UUID, signed local IDs, unload-before-reappearance, cleanup, settings and messages. Native patch/shared integration composed in memory only. No official build, no game-parity claim; aggregate game check remains root-owned.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
