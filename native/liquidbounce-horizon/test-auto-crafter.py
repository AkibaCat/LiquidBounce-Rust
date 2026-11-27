"""Prepare focused actual AutoCrafter model/config; compile only with --run in a batch."""
from pathlib import Path
import argparse, hashlib, json, runpy, subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]

def prepare(out):
    out.mkdir(parents=True,exist_ok=True)
    body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
    source=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
    config='''use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}
#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]AutoCrafter}
impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.autoCrafter"}}
'''
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(',"fn pair_members<'a>(",'fn parse_int_pair(','fn parse_multi_mask(','\nfn leaf(']:
        if anchor in ['pub struct ConfigError','pub struct CommonSettings']:config+='#[derive(Debug)]\n'
        config+=body(source,anchor)+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug)]pub enum SettingChange{AutoCrafter(AutoCrafterSetting)}
#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}
'''
    config+=f'#[path="{(HERE/"engine/src/config/auto_crafter.rs").as_posix()}"]mod settings;use settings::*;\nuse crate::auto_crafter::{{Settings,OnFull}};\n'
    config+=(HERE/'auto-crafter-config-smoke.rs').read_text(encoding='utf8')
    (out/'config.rs').write_text(config,encoding='utf8')
    model=ROOT/'native/liquidbounce-behaviors/src/auto_crafter.rs'
    (out/'lib.rs').write_text(f'#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;\n#[path="{model.as_posix()}"]pub mod auto_crafter;\nmod config;\n',encoding='utf8')
    (out/'Cargo.toml').write_text('[package]\nname="auto-crafter-focused"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
    report={'status':'prepared-no-compiler-invoked','tests':6,'scope':'Actual four production AutoCrafter coroutine tests and two config tests using actual Tree and shared parsers. Native recipe/actions remain subject to official batch check and final game testing.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in [model,HERE/'engine/src/config/auto_crafter.rs',HERE/'auto-crafter-game.rs']}}
    (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args();out=ROOT/args.out;prepare(out)
    if not args.run:print('AutoCrafter fixture prepared; no compiler invoked');return
    r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'],capture_output=True,text=True,encoding='utf8',errors='replace')
    (out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode,'tests':6},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
if __name__=='__main__':main()
