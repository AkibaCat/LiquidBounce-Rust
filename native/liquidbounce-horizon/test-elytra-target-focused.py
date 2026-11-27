#!/usr/bin/env python3
"""Prepare the ElytraTarget production model/settings fixture; Cargo only with --run."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']

def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{ElytraTarget}
impl ModuleKind{fn name(self)->&'static str{"ElytraTarget"}fn key(self)->&'static str{"liquidbounce.module.elytraTarget"}}
'''
 for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),("fn pair_members<'a>(",''),('fn parse_float_pair(',''),('fn parse_int_pair(',''),('fn parse_int(',''),('fn decimal_int(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
  config+=(f'#[derive({derive})]\n'if derive else'')+BODY(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),ElytraTarget(ElytraTargetSetting)}
'''
 config+=BODY(shared,'fn leaf(\n    patch:')+'\n'
 entry=BODY(shared,'pub fn parse_settings(')
 common=entry[entry.index('    let root ='):entry.index('    match module {')].replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
 config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'elytra_target_config::parse_elytra_target(&mut patch,&root);Ok(patch)}\n'
 actual=HERE/'engine/src/config/elytra_target.rs'
 (out/'elytra-target-config.rs').write_text(actual.read_text(encoding='utf8'),encoding='utf8')
 rendering=HERE/'engine/src/config/target_renderer.rs'
 (out/'target-renderer-config.rs').write_text(rendering.read_text(encoding='utf8').split('#[cfg(test)]',1)[0],encoding='utf8')
 config+='#[path="target-renderer-config.rs"]mod target_renderer;\n#[path="elytra-target-config.rs"]mod elytra_target_config;pub use elytra_target_config::*;\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 runtime=(HERE/'engine/src/elytra_target_runtime.rs').read_text(encoding='utf8')
 (out/'runtime.rs').write_text('use crate::elytra_target::ElytraTarget;use crate::target_renderer::{TargetRendererSettings,TargetRendererRuntime};\n'+BODY(runtime,'pub struct ElytraTargetRuntime')+'\n'+BODY(runtime,'impl Default for ElytraTargetRuntime'),encoding='utf8')
 model=ROOT/'native/liquidbounce-behaviors/src/elytra_target.rs'
 (out/'lib.rs').write_text(f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
pub use behavior_core::{{auto_bow,auto_bow_math,rotations,target_renderer}};
#[path="{model.as_posix()}"]pub mod elytra_target;
mod behaviors{{pub fn current_time_millis()->i64{{0}}}}
#[path="runtime.rs"]mod elytra_target_runtime;
mod config;
''',encoding='utf8')
 core=ROOT/'native/liquidbounce-behaviors'
 (out/'Cargo.toml').write_text('''[package]
name="elytra-target-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
'''+f'behavior_core={{package="liquidbounce-behaviors",path="{core.as_posix()}"}}\n',encoding='utf8')
 sources=[actual,rendering,model,HERE/'engine/src/elytra_target_runtime.rs',HERE/'elytra-target-native.rs',HERE/'elytra-target-patch.py']
 report={'status':'prepared-no-compiler-invoked','tests':7,'scope':'Production model, nested setting trees, shared parsers and TargetRenderer configuration. Actual collision, packet/server behavior, AlwaysGlide event timing and GPU rendering require game validation.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}}
 (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
 return report

if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--out',required=True);ap.add_argument('--run',action='store_true');a=ap.parse_args();out=ROOT/a.out;prepare(out)
 if not a.run:print(f'ElytraTarget fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
 r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
 (out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
