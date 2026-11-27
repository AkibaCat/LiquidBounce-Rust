#!/usr/bin/env python3
"""Prepare real NameProtect model/service/config tests without compiling the engine. --run compiles."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']


def prepare(out,horizon,oracle=None):
    out.mkdir(parents=True,exist_ok=True)
    shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
    config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{NameProtect}
impl ModuleKind{fn name(self)->&'static str{"NameProtect"}fn key(self)->&'static str{"liquidbounce.module.nameProtect"}}
'''
    for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('\nfn leaf(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
        body=BODY(shared,anchor)
        if anchor=='\nfn leaf(':assert '&self'not in body and 'patch: &mut SettingsPatch'in body
        config+=(f'#[derive({derive})]\n'if derive else '')+body+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),NameProtect(NameProtectSetting)}
'''
    entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')]
    common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
    config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'name_protect::parse_name_protect(&mut patch,&root);Ok(patch)}\n'
    actual=HERE/'engine/src/config/name_protect.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
    (out/'name-protect-config.rs').write_text(original,encoding='utf8')
    config+='#[path="name-protect-config.rs"]mod name_protect;pub use name_protect::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
    actual_runtime=HERE/'engine/src/name_protect_runtime.rs';runtime=actual_runtime.read_text(encoding='utf8')
    # Only LiquidBounceHooks/Host integration is left out. All service bodies and original tests remain.
    extracted=runtime[:runtime.index('impl LiquidBounceHooks')]+runtime[runtime.index('#[cfg(test)]'):]
    extracted=extracted.replace('use crate::runtime::LiquidBounceHooks;\n','').replace('mod_api::Host,','')
    assert extracted.count('#[test]')==3
    (out/'name-protect-runtime.rs').write_text(extracted,encoding='utf8')
    model=ROOT/'native/liquidbounce-behaviors/src/name_protect.rs'
    lib=f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
#[path="{model.as_posix()}"]pub mod name_protect;
#[path="name-protect-runtime.rs"]mod name_protect_runtime;
mod config;
'''
    tests=13
    if oracle:
        shutil.copyfile(oracle,out/'oracle.json')
        lib+='''#[cfg(test)]mod source_differential{
 #[test]fn deterministic_username_matches_unchanged_pinned_generator(){
  let vectors:serde_json::Value=serde_json::from_str(include_str!("oracle.json")).unwrap();
  for row in vectors.as_array().unwrap(){let name=row["name"].as_str().unwrap();assert_eq!(crate::name_protect::pseudonym(name),row["replacement"].as_str().unwrap(),"{name}");}
 }
}
''';tests+=1
    (out/'lib.rs').write_text(lib,encoding='utf8')
    (out/'Cargo.toml').write_text(f'''[package]
name="name-protect-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={{version="1",features=["arbitrary_precision"]}}
horizon-client={{package="horizon-client-core",path="{(horizon/'crates/horizon-client-core').as_posix()}"}}
''',encoding='utf8')
    sources=[HERE/'engine/src/config.rs',actual,actual_runtime,model,model.with_name('name_protect_username.rs')]
    report={'status':'prepared-no-compiler-invoked','tests':tests,'scope':'Actual production model plus original six model/two username tests, all production text service bodies plus three original tests, exact config plus two original tests and real shared parser functions. Actual staged Horizon client-core text styles/service, no substitution. Registration narrowed; LiquidBounceHooks/Host actor boundary omitted; no engine/game claim.','horizon':str(horizon),'oracle':str(oracle)if oracle else None,'sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}}
    (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--horizon',default='build/staging/horizon');parser.add_argument('--oracle');parser.add_argument('--run',action='store_true');parser.add_argument('--filter',default='');args=parser.parse_args()
    out=ROOT/args.out;prepare(out,ROOT/args.horizon,ROOT/args.oracle if args.oracle else None)
    if not args.run:print(f'NameProtect focused fixture prepared: {out}; no compiler invoked');return
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    cmd=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]
    if args.filter:cmd.append(args.filter)
    cmd+=['--','--test-threads=1'];r=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    (out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode,'filter':args.filter},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)


if __name__=='__main__':main()
