#!/usr/bin/env python3
"""AntiHunger policy/original, real packet pipeline and native jump consumption."""
import hashlib,json,os,runpy,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/anti-hunger-tests'
def compose():
 generator=HERE/'make-behavior-patch.py';source=generator.read_text(encoding='utf-8');ns={'__file__':str(generator),'__name__':'anti_hunger_composition'};exec(compile(source.split('\noutput = []\n',1)[0],str(generator),'exec'),ns)
 contracts=[]
 for extension,member,kind in [('timer','client_timer_scale','names'),('terrain-speed','client_terrain_speed','names'),('high-jump','client_high_jump_state','helper_names'),('anti-hunger','client_anti_hunger_swimming','helper_names')]:
  if member not in ns[kind]:contracts.append(runpy.run_path(str(HERE/(extension+'-patch.py')))['apply'](ns['edit'],ns['replace_once']))
 paths=[]
 for path,(_,content)in ns['changes'].items():
  if path.endswith('.rs'):
   dest=OUT/'composition'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(content,encoding='utf-8');paths.append(str(dest))
 for start in range(0,len(paths),48):
  r=subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true','--emit','files',*paths[start:start+48]],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
  if r.returncode:raise SystemExit(r.stdout+r.stderr)
 (OUT/'composition.json').write_text(json.dumps(dict(contracts=contracts,rustSources=len(paths),generatorSha256=hashlib.sha256(source.encode()).hexdigest(),scope='cumulative production plus HighJump and AntiHunger; syntax only'),indent=2)+'\n',encoding='utf-8');return ns['changes']
def main():
 OUT.mkdir(parents=True,exist_ok=True)
 if not(OUT/'oracle.json').is_file():raise SystemExit('Run collect-anti-hunger-original.py first')
 block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block'];changes=compose()
 (OUT/'packet-bridge.rs').write_text(changes['crates/horizon-client/src/packet_bridge.rs'][1],encoding='utf-8')
 (OUT/'living-motion.rs').write_text(changes['crates/horizon-world/src/living_motion.rs'][1],encoding='utf-8')
 native=(HERE/'high-jump-native-fixture.rs').read_text(encoding='utf-8').replace('__BRIDGE__',(HERE/'high-jump-bridge.rs').as_posix()).replace('__JUMP__',block(changes['crates/horizon-client/src/motion.rs'][1],'    pub fn jump_from_ground('))
 (OUT/'native-fixture.rs').write_text(native,encoding='utf-8')
 shared=(HERE/'engine/src/config.rs').read_text(encoding='utf-8');config='''use std::collections::BTreeMap;
use serde_json::{Value,Map,Number,json};use crate::anti_hunger;
#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{File,Interop}
enum ModuleKind{AntiHunger}impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.antiHunger"}}
'''
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=block(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug,PartialEq)]enum SettingChange{AntiHunger(terrain::AntiHungerSetting)}
#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}
'''
 config+=f'#[path="{(HERE/"engine/src/config/anti_hunger.rs").as_posix()}"]mod terrain;use terrain::*;\n';config+=(HERE/'anti-hunger-config-tests.rs').read_text(encoding='utf-8');(OUT/'config-fixture.rs').write_text(config,encoding='utf-8')
 lib='''#![cfg(test)]
#![allow(dead_code)]
extern crate self as liquidbounce_behaviors;extern crate self as horizon_client;
pub mod mod_api{pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket,Serverbound as ServerboundPlayPacket};}
pub mod lang{pub fn code()->String{"zh_cn".into()}}
'''
 sources={'anti_hunger':ROOT/'native/liquidbounce-behaviors/src/anti_hunger.rs','high_jump':ROOT/'native/liquidbounce-behaviors/src/high_jump.rs','no_swing':ROOT/'native/liquidbounce-behaviors/src/no_swing.rs','anti_hunger_bridge':HERE/'anti-hunger-bridge.rs','anti_hunger_packets':HERE/'engine/src/anti_hunger_packets.rs','anti_hunger_notifications':HERE/'engine/src/anti_hunger_notifications.rs','packets':HERE/'engine/src/packets.rs','packet_bridge':OUT/'packet-bridge.rs','tests':HERE/'anti-hunger-tests.rs','packet_tests':HERE/'anti-hunger-packet-tests.rs','config':OUT/'config-fixture.rs'}
 for name,path in sources.items():lib+=f'#[path="{path.as_posix()}"]pub mod {name};\n'
 lib+='include!("native-fixture.rs");\n';(OUT/'lib.rs').write_text(lib,encoding='utf-8')
 manifest='[package]\nname="liquidbounce-anti-hunger-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n'
 for name in ['horizon-core','horizon-world','horizon-protocol']:manifest+=name+'={path="'+(ROOT/'upstream/Horizon/crates'/name).as_posix()+'"}\n'
 (OUT/'Cargo.toml').write_text(manifest,encoding='utf-8');env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-entity-interact-tests/target');commands=[]
 for name,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
  r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name=name,exitCode=r.returncode))
  if r.returncode:break
 owned=[Path(__file__),*sources.values(),HERE/'anti-hunger-native.rs',HERE/'anti-hunger-patch.py',HERE/'engine/src/config/anti_hunger.rs',HERE/'anti-hunger-config-tests.rs',HERE/'collect-anti-hunger-original.py',HERE/'test-anti-hunger-native.py']
 report=dict(passed=len(commands)==2 and all(c['exitCode']==0 for c in commands),commands=commands,oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),sourceSha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in owned},scope='2280 untouched AntiHunger/Chronometer cases, actual PacketPipeline and native jump consumption; full client differential not_run')
 (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
