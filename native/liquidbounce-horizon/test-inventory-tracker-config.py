#!/usr/bin/env python3
"""Prepare the two unchanged production InventoryTracker config tests; --run explicitly compiles."""
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
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{InventoryTracker}
impl ModuleKind{fn name(self)->&'static str{"InventoryTracker"}fn key(self)->&'static str{"liquidbounce.module.inventoryTracker"}}
'''
    for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('\nfn leaf(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
        body=BODY(shared,anchor)
        if anchor=='\nfn leaf(':assert '&self'not in body and 'patch: &mut SettingsPatch'in body
        config+=(f'#[derive({derive})]\n'if derive else '')+body+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),InventoryTracker(InventoryTrackerSetting)}
'''
    entry=BODY(shared,'pub fn parse_settings(')
    common=entry[entry.index('    let root ='):entry.index('    match module {')]
    common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
    config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'inventory_tracker::parse_inventory_tracker(&mut patch,&root);Ok(patch)}\n'
    actual=HERE/'engine/src/config/inventory_tracker.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
    (out/'inventory-tracker-config.rs').write_text(original,encoding='utf8')
    config+='#[path="inventory-tracker-config.rs"]mod inventory_tracker;pub use inventory_tracker::*;\n'
    (out/'config.rs').write_text(config,encoding='utf8')
    path=HERE/'engine/src/inventory_tracker_runtime.rs';source=path.read_text(encoding='utf8')
    runtime='use liquidbounce_behaviors::inventory_tracker as model;\n#[derive(Default)]\n'+BODY(source,'pub struct InventoryTrackerRuntime')+'\n'+BODY(source,'impl InventoryTrackerRuntime')+'\n'
    (out/'inventory-tracker-runtime.rs').write_text(runtime,encoding='utf8')
    (out/'lib.rs').write_text('#![allow(dead_code,unused_imports)]\n#[path="inventory-tracker-runtime.rs"]mod inventory_tracker_runtime;\nmod config;\n',encoding='utf8')
    (out/'Cargo.toml').write_text(f'''[package]
name="inventory-tracker-config-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={{version="1",features=["arbitrary_precision"]}}
liquidbounce-behaviors={{path="{(ROOT/'native/liquidbounce-behaviors').as_posix()}"}}
''',encoding='utf8')
    report={'status':'prepared-no-compiler-invoked','tests':2,'scope':'Byte-identical production InventoryTracker config and original two tests; actual shared Tree/common-leaf/parser functions and real InventoryTracker runtime settings/reset/save methods and behavior dependency. Registration narrowed, actor/Host/native boundary omitted. No complete engine/game claim.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in [HERE/'engine/src/config.rs',actual,path,ROOT/'native/liquidbounce-behaviors/src/inventory_tracker.rs']}}
    (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args()
    out=ROOT/args.out;prepare(out)
    if not args.run:print(f'InventoryTracker focused fixture prepared: {out}; no compiler invoked');return
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'inventory_tracker_config_tests','--','--test-threads=1'],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    (out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode,'tests':2},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)


if __name__=='__main__':main()
