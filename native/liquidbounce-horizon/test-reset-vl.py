#!/usr/bin/env python3
"""ResetVL exact source differential; declaration-only registration providers."""
from pathlib import Path
import argparse, hashlib, json, os, shutil, subprocess, time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent

def body(text,signature):
    start=text.index(signature);end=text.index('{',start)+1;depth=1
    while depth:depth+=(text[end]=='{')-(text[end]=='}');end+=1
    return text[start:end]

def main():
    p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--source',default='build/reset-vl-source-03');a=p.parse_args();out=ROOT/a.out;source=ROOT/a.source
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    assert json.loads((source/'report.json').read_text(encoding='utf8'))['passed']
    out.mkdir(parents=True,exist_ok=True)
    for name in ['inputs.json','oracle.json']:shutil.copyfile(source/name,out/name)
    ticker=(HERE/'engine/src/coroutine_ticker.rs').read_text(encoding='utf8')
    # Only the not-yet-integrated enum declaration is supplied; FIFO body untouched.
    assert ticker.count('    InventoryManager,')==1
    (out/'coroutine-ticker.rs').write_text(ticker if '    ResetVl,'in ticker else ticker.replace('    InventoryManager,','    InventoryManager,\n    ResetVl,'),encoding='utf8')
    shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
    config='''use std::collections::BTreeMap;
use serde_json::{Value,Map,Number,json};
use crate::reset_vl;
#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{File,Interop}
enum ModuleKind{ResetVl}impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.resetVL"}}
'''
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','\nfn leaf(']:
        if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
        config+=body(shared,anchor)+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug,PartialEq)]enum SettingChange{ResetVl(settings::ResetVlSetting)}
#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}
'''
    config+=f'#[path="{(HERE/"engine/src/config/reset_vl.rs").as_posix()}"]mod settings;\nuse settings::*;\n'
    config+=(HERE/'reset-vl-config-tests.rs').read_text(encoding='utf8');(out/'config.rs').write_text(config,encoding='utf8')
    shutil.copyfile(HERE/'reset-vl-tests.rs',out/'tests.rs')
    lib='#![cfg(test)]\n#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;\n'
    for name in ['reset_vl','timer']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
    lib+='#[path="coroutine-ticker.rs"]mod coroutine_ticker;\n#[path="config.rs"]mod config;\nmod tests;\n'
    (out/'lib.rs').write_text(lib,encoding='utf8')
    (out/'Cargo.toml').write_text('[package]\nname="reset-vl-source-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(out/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        start=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf8');commands.append({'name':name,'exitCode':r.returncode,'seconds':round(time.monotonic()-start,3)})
        if r.returncode:break
    owned=[Path(__file__),HERE/'reset-vl-tests.rs',HERE/'reset-vl-config-tests.rs',HERE/'engine/src/config/reset_vl.rs',ROOT/'native/liquidbounce-behaviors/src/reset_vl.rs']
    report={'passed':len(commands)==2 and all(c['exitCode']==0 for c in commands),'commands':commands,'scenarios':len(json.loads((source/'inputs.json').read_text(encoding='utf8'))),'steps':sum(map(len,json.loads((source/'inputs.json').read_text(encoding='utf8')))),
        'scope':'Exact production ResetVL and Timer bodies with shared CoroutineTicker body; declaration-only ResetVl WaitOwner/config registration providers. Per-step differential against byte-identical original Kotlin/full coroutine runtime/Timer. File+Interop config extraction with actual shared parsers. No complete engine or game claim.',
        'oracleSha256':hashlib.sha256((source/'oracle.json').read_bytes()).hexdigest(),'sources':{path.relative_to(ROOT).as_posix():hashlib.sha256(path.read_bytes()).hexdigest() for path in owned},'tickerShim':'Only ResetVl WaitOwner variant declaration added; no FIFO implementation substitution.'}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed'] else 1)

if __name__=='__main__':main()
