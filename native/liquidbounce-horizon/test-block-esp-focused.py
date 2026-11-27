#!/usr/bin/env python3
"""Narrow actual BlockESP behavior/config fixture; --run is the only Cargo entry."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{BlockEsp}
impl ModuleKind{fn name(self)->&'static str{"BlockESP"}fn key(self)->&'static str{"liquidbounce.module.blockESP"}}
'''
 for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),('fn parse_int(',''),('fn decimal_int(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
  config+=(f'#[derive({derive})]\n'if derive else '')+BODY(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),BlockEsp(BlockEspSetting)}
'''
 config+=BODY(shared,'fn leaf(\n    patch:')+'\n'
 entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')];common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
 config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'block_esp_config::parse_block_esp(&mut patch,&root);Ok(patch)}\n'
 actual=HERE/'engine/src/config/block_esp.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
 (out/'block-esp-config.rs').write_text(original,encoding='utf8');config+='#[path="block-esp-config.rs"]mod block_esp_config;pub use block_esp_config::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 model=ROOT/'native/liquidbounce-behaviors/src/block_esp.rs';
 (out/'lib.rs').write_text(f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="{model.with_name('breadcrumbs.rs').as_posix()}"]pub mod breadcrumbs;
#[path="{model.with_name('storage_esp.rs').as_posix()}"]pub mod storage_esp;
#[path="{model.as_posix()}"]pub mod block_esp;
mod config;
''',encoding='utf8')
 (out/'Cargo.toml').write_text('''[package]
name="block-esp-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
''',encoding='utf8')
 sources=[HERE/'engine/src/config.rs',actual,model,model.with_name('breadcrumbs.rs'),model.with_name('storage_esp.rs'),model.with_name('storage_esp_surface.rs'),HERE/'block-esp-game.rs',HERE/'block-esp-patch.py',HERE/'engine/src/block_esp_runtime.rs']
 report={'status':'prepared-no-compiler-invoked','tests':4,'scope':'Exact production BlockESP model/config with real StorageESP voxel-union mesh and Breadcrumbs style/HSB implementations. Shared actual Tree/common/scalar/bool/number parsers are extracted verbatim. No native scan, actor lifecycle or GPU execution claim.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}};(out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args();out=ROOT/args.out;prepare(out)
 if not args.run:print(f'BlockESP fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
