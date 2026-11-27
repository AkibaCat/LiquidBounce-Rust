#!/usr/bin/env python3
"""Actual BetterChat model/config/layout/HTTP-worker fixture; --run is the only Cargo entry."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{BetterChat}
impl ModuleKind{fn name(self)->&'static str{"BetterChat"}fn key(self)->&'static str{"liquidbounce.module.betterChat"}}
'''
 for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),('fn parse_int(',''),('fn decimal_int(',''),('fn parse_multi_mask(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
  config+=(f'#[derive({derive})]\n'if derive else '')+BODY(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),BetterChat(BetterChatSetting)}
'''
 config+=BODY(shared,'fn leaf(\n    patch:')+'\n'
 entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')];common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
 config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'better_chat_config::parse_better_chat(&mut patch,&root);Ok(patch)}\n'
 actual=HERE/'engine/src/config/better_chat.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
 (out/'better-chat-config.rs').write_text(original,encoding='utf8');config+='#[path="better-chat-config.rs"]mod better_chat_config;pub use better_chat_config::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 runtime=(HERE/'engine/src/better_chat_runtime.rs').read_text(encoding='utf8')
 worker='''use liquidbounce_behaviors::better_chat::{self,Settings,TranslationSettings,Translation,TranslationCache};
use std::{collections::VecDeque,sync::mpsc::{self,Receiver},time::Duration};
'''+BODY(runtime,'struct Reply')+'\n#[derive(Default)]'+BODY(runtime,'pub struct BetterChatRuntime')+'\n'+BODY(runtime,'impl BetterChatRuntime')+'\n'+BODY(runtime,'pub(crate) fn request(')+'\n'+(HERE/'better-chat-smoke.rs').read_text(encoding='utf8')
 (out/'runtime.rs').write_text(worker,encoding='utf8')
 model=ROOT/'native/liquidbounce-behaviors/src/better_chat.rs';regex=model.parent.parent/'vendor/java_regex';layout=HERE/'better-chat-layout.rs'
 (out/'lib.rs').write_text(f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="{model.as_posix()}"]pub mod better_chat;
#[path="{layout.as_posix()}"]pub mod layout;
mod config;
mod runtime;
''',encoding='utf8')
 (out/'Cargo.toml').write_text('''[package]
name="better-chat-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
ureq={version="2",default-features=false,features=["tls"]}
url="2"
'''+f'java_regex={{path="{regex.as_posix()}"}}\n',encoding='utf8')
 sources=[HERE/'engine/src/config.rs',actual,model,layout,HERE/'better-chat-game.rs',HERE/'better-chat-render.rs',HERE/'better-chat-patch.py',HERE/'engine/src/better_chat_runtime.rs',HERE/'better-chat-smoke.rs']
 report={'status':'prepared-no-compiler-invoked','tests':9,'scope':'Exact production BetterChat model/config, native hit/bounds/rect and HTTP request/queue/cache methods. Includes a real loopback LibreTranslate HTTP worker and exact shared Tree/common/scalar/multichoice parsers. Does not claim in-game native chat storage, actor callbacks, Google availability or UI/GPU execution.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}};(out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args();out=ROOT/args.out;prepare(out)
 if not args.run:print(f'BetterChat fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
