"""Prepare or run one focused AutoChatGame check as part of a module batch."""
from pathlib import Path
import argparse,json,re,runpy,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
 config_source=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{File,Interop}\n#[derive(Default)]enum ModuleKind{#[default]AutoChatGame}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.autoChatGame"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(',"fn pair_members<'a>(",'fn parse_int_pair(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(config_source,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;#[derive(Debug)]enum SettingChange{AutoChatGame(AutoChatGameSetting)}\n#[derive(Default)]'+body(config_source,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/auto_chat_game.rs").as_posix()}"]mod settings;use settings::*;\n'
 config+='''#[test]fn config_roundtrip(){let mut s=liquidbounce_behaviors::auto_chat_game::Settings::default();s.protocol=1;s.base_url="http://localhost/v1".into();s.api_key="local-only".into();s.model="mock".into();s.reaction_time=[0,10000];s.cooldown=60;s.buffer_time=500;s.include_trigger=false;s.trigger_sentence="开始".into();s.server_name="本地".into();s.answer_template="/answer %s".into();s.prompt="Question {SERVER_NAME}".into();let c=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for f in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_auto_chat_game(&s,&c,f,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_auto_chat_game(&mut patch,&nodes(&tree,"AutoChatGame","AutoChatGame").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);assert_eq!(patch.changes.len(),12);let mut restored=liquidbounce_behaviors::auto_chat_game::Settings::default();for SettingChange::AutoChatGame(v)in patch.changes{v.apply(&mut restored).unwrap();}assert!(s==restored);}assert!(AutoChatGameSetting{name:"ReactionTime".into(),value:json!({"from":5,"to":1})}.apply(&mut s).is_err());}'''
 (out/'config.rs').write_text(config,encoding='utf8')
 source='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;\n'
 for name in ['auto_chat_game','auto_chat_game_wire','report_helper_format']:source+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 source+='use auto_chat_game_wire as wire;use std::time::Duration;mod config;\n'
 runtime=(HERE/'engine/src/auto_chat_game_runtime.rs').read_text(encoding='utf8')
 source+=body(runtime,'fn redact(')+'\n'+body(runtime,'pub(crate) fn request(')+'\n'
 source+=(HERE/'auto-chat-game-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(source,encoding='utf8')
 return out/'lib.rs'
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);a.add_argument('--prepare-only',action='store_true');args=a.parse_args();out=ROOT/args.out
 if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 source=prepare(out)
 if args.prepare_only:print('AutoChatGame harness prepared; no Rust compiler invoked');return
 deps=ROOT/'build/staging/horizon/target/debug/deps';command=['rustc','+stable','--edition=2024','--test',str(source),'-L','dependency='+str(deps),'-o',str(out/'smoke.exe')]
 for name in ['serde_json','ureq','url']:
  candidates=list(deps.glob(f'lib{name}-*.rlib'))
  if name=='ureq':
   # The game already uses ureq 3. The production module explicitly owns
   # ureq 2; choose its real Cargo artifact rather than the newest same-name lib.
   candidates=[p for p in candidates if (d:=deps/(p.stem.removeprefix('lib')+'.d')).is_file() and re.search(r'[/\\]ureq-2\.',d.read_text(encoding='utf8'))]
  if not candidates:raise SystemExit(f'Batch build must first provide {name} rlib')
  dependency=max(candidates,key=lambda p:p.stat().st_mtime_ns);command+=['--extern',f'{name}={dependency}']
 result=subprocess.run(command,capture_output=True,text=True,encoding='utf8',errors='replace');log=result.stdout+result.stderr
 if result.returncode==0:
  result=subprocess.run([str(out/'smoke.exe'),'--test-threads=1'],capture_output=True,text=True,encoding='utf8',errors='replace');log+=result.stdout+result.stderr
 (out/'test.log').write_text(log,encoding='utf8');report={'passed':result.returncode==0,'scope':'Buffer/strict cooldown/GG/cancel persistence, real loopback HTTP both pinned protocols, provider error redaction, Java answer formatter and twelve-leaf File/Interop round trip. No external API calls, full suite or game.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(result.returncode)
if __name__=='__main__':main()
