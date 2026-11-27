#!/usr/bin/env python3
"""Actual TargetStrafe planner/config/native bridge fixture. No compiler without --run."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{TargetStrafe}
impl ModuleKind{fn name(self)->&'static str{"TargetStrafe"}fn key(self)->&'static str{"liquidbounce.module.targetStrafe"}}
'''
 for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),('fn parse_int(',''),('fn decimal_int(',''),('fn parse_multi_mask(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
  config+=(f'#[derive({derive})]\n'if derive else '')+BODY(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),TargetStrafe(TargetStrafeSetting)}
'''
 config+=BODY(shared,'fn leaf(\n    patch:')+'\n'
 entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')];common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
 config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'target_strafe_config::parse_target_strafe(&mut patch,&root);Ok(patch)}\n'
 actual=HERE/'engine/src/config/target_strafe.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
 (out/'target-strafe-config.rs').write_text(original,encoding='utf8');config+='#[path="target-strafe-config.rs"]mod target_strafe_config;pub use target_strafe_config::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 model=ROOT/'native/liquidbounce-behaviors/src/target_strafe.rs'
 math=model.parent/'auto_bow_math.rs';math_source=math.read_text(encoding='utf8');math_source=math_source.split('#[cfg(test)]',1)[0]
 (out/'math.rs').write_text(math_source,encoding='utf8')
 bridge=HERE/'target-strafe-bridge.rs';smoke=HERE/'target-strafe-smoke.rs'
 (out/'bridge.rs').write_text(bridge.read_text(encoding='utf8')+'\n'+smoke.read_text(encoding='utf8'),encoding='utf8')
 (out/'lib.rs').write_text(f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="math.rs"]pub mod auto_bow_math;
pub use auto_bow_math as target_strafe_math;
#[path="{model.as_posix()}"]pub mod target_strafe;
mod config;
#[path="bridge.rs"]pub mod target_strafe_bridge;
''',encoding='utf8')
 (out/'Cargo.toml').write_text('''[package]
name="target-strafe-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
''',encoding='utf8')
 sources=[HERE/'engine/src/config.rs',actual,model,math,bridge,smoke,HERE/'target-strafe-game.rs',HERE/'target-strafe-motion.rs',HERE/'target-strafe-patch.py',HERE/'engine/src/target_strafe_runtime.rs']
 report={'status':'prepared-no-compiler-invoked','tests':10,'scope':'Actual production TargetStrafe planner, all settings and exact shared config parsers plus native thread-local bridge. Terrain mock exercises validation, adaptive radius, sight side selection, input cancellation and real event speed transport. Full Horizon compilation, actual block collision/line-of-sight, GPU ring, mouse/keyboard, multiplayer and SpeedHypixel shared shouldStrafe have not been executed by this fixture.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}};(out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args();out=ROOT/args.out;prepare(out)
 if not args.run:print(f'TargetStrafe fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
