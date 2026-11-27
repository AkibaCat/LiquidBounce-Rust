#!/usr/bin/env python3
"""Narrow actual ClickGui behavior/config fixture; --run is the only Cargo entry."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{ClickGui}
impl ModuleKind{fn name(self)->&'static str{"ClickGUI"}fn key(self)->&'static str{"liquidbounce.module.clickGui"}}
'''
 for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),('fn parse_int(',''),('fn decimal_int(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
  config+=(f'#[derive({derive})]\n'if derive else '')+BODY(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),ClickGui(ClickGuiSetting)}
'''
 config+=BODY(shared,'fn leaf(\n    patch:')+'\n'
 entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')];common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
 config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'click_gui_config::parse_click_gui(&mut patch,&root);Ok(patch)}\n'
 actual=HERE/'engine/src/config/click_gui.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
 (out/'click-gui-config.rs').write_text(original,encoding='utf8');config+='#[path="click-gui-config.rs"]mod click_gui_config;pub use click_gui_config::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
 model=ROOT/'native/liquidbounce-behaviors/src/click_gui.rs'
 (out/'lib.rs').write_text(f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="{model.as_posix()}"]pub mod click_gui;
mod config;
''',encoding='utf8')
 (out/'Cargo.toml').write_text('''[package]
name="click-gui-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
''',encoding='utf8')
 sources=[HERE/'engine/src/config.rs',actual,model,HERE/'click-gui-patch.py',HERE/'engine/src/click_gui_browser.rs']
 report={'status':'prepared-no-compiler-invoked','tests':5,'scope':'Exact production ClickGui model/config and shared common parsers. Shared actual Tree/common/scalar/bool/number parsers are extracted verbatim. This fixture does not execute native renderer, actor lifecycle or GPU drawing.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}};(out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args();out=ROOT/args.out;prepare(out)
 if not args.run:print(f'ClickGui fixture prepared: {out}; no compiler invoked');raise SystemExit()
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
