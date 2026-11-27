#!/usr/bin/env python3
"""Prepare actual BedPlates model/config tests; only --run invokes Cargo."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BODY=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
def prepare(out):
    out.mkdir(parents=True,exist_ok=True);shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
    config='''use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{BedPlates}
impl ModuleKind{fn name(self)->&'static str{"BedPlates"}fn key(self)->&'static str{"liquidbounce.module.bedPlates"}}
'''
    for anchor,derive in[('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),('fn parse_int(',''),('fn decimal_int(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
        config+=(f'#[derive({derive})]\n'if derive else '')+BODY(shared,anchor)+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),BedPlates(BedPlatesSetting)}
'''
    config+=BODY(shared,'fn leaf(\n    patch:')+'\n'
    entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')];common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
    config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'bed_plates::parse_bed_plates(&mut patch,&root);Ok(patch)}\n'
    actual=HERE/'engine/src/config/bed_plates.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
    (out/'bed-plates-config.rs').write_text(original,encoding='utf8');config+='#[path="bed-plates-config.rs"]mod bed_plates;pub use bed_plates::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
    model=ROOT/'native/liquidbounce-behaviors/src/bed_plates.rs';curve=model.with_name('tracers_curve.rs');tags=model.with_name('item_tags.rs');camera=model.with_name('camera_clip.rs');keys=model.with_name('camera_clip_keys.rs')
    enums='';tags_source=tags.read_text(encoding='utf8')
    for name in['Filter','Background']:enums+='#[derive(Clone,Copy,Debug,PartialEq,Eq)]\n'+BODY(tags_source,f'pub enum {name}')+'\n'+BODY(tags_source,f'impl {name}')+'\n'
    modifier=f'#[path="{keys.as_posix()}"]mod keys;\n';camera_source=camera.read_text(encoding='utf8')
    for anchor,derive in[('pub struct ModifierKey','Clone,Debug,PartialEq,Eq'),('impl Default for ModifierKey',''),('impl ModifierKey','')]:modifier+=(f'#[derive({derive})]\n'if derive else '')+BODY(camera_source,anchor)+'\n'
    lib=f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
pub mod tracers{{#[path="{curve.as_posix()}"]pub mod curve;}}
pub mod item_tags{{{enums}}}
pub mod camera_clip{{{modifier}}}
#[path="{model.as_posix()}"]pub mod bed_plates;
mod config;
'''
    (out/'lib.rs').write_text(lib,encoding='utf8');(out/'Cargo.toml').write_text('''[package]
name="bed-plates-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
''',encoding='utf8')
    sources=[HERE/'engine/src/config.rs',actual,model,model.with_name('bed_plates_scan.rs'),curve,tags,camera,keys,HERE/'bed-plates-game.rs',HERE/'bed-plates-patch.py']
    report={'status':'prepared-no-compiler-invoked','tests':6,'scope':'Exact production BedPlates model/scan/curve with its four original tests, exact config with its two original tests and shared actual Tree/common/scalar/bool/number parsers. Only module registration/dispatch and dependency modules are narrowed; original ItemTags enums and original ModifierKey are extracted verbatim, with actual key tables. No native game/engine actor/GPU execution claim.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}};(out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');parser.add_argument('--filter',default='');args=parser.parse_args();out=ROOT/args.out;prepare(out)
    if not args.run:print(f'BedPlates focused fixture prepared: {out}; no compiler invoked');return
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');cmd=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]
    if args.filter:cmd.append(args.filter)
    cmd+=['--','--test-threads=1'];r=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode,'filter':args.filter},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
if __name__=='__main__':main()
