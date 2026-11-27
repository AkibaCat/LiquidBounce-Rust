#!/usr/bin/env python3
"""Original HighJump/ticker traces, shared config and actual native jump consumers."""
import hashlib,json,os,runpy,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/high-jump-tests'
UPSTREAM=ROOT/'upstream/Horizon'
def compose():
    g=HERE/'make-behavior-patch.py';source=g.read_text(encoding='utf-8');ns={'__file__':str(g),'__name__':'high_jump_composition'}
    exec(compile(source.split('\noutput = []\n',1)[0],str(g),'exec'),ns)
    patch=runpy.run_path(str(HERE/'high-jump-patch.py'))
    contract=patch['apply'](ns['edit'],ns['replace_once']) if 'client_high_jump_state' not in ns['helper_names'] else {'alreadyInProduction':True}
    paths=[]
    for path,(_,content)in ns['changes'].items():
        if path.endswith('.rs'):
            dest=OUT/'composition'/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(content,encoding='utf-8');paths.append(str(dest))
    for start in range(0,len(paths),48):
        r=subprocess.run(['rustfmt','+stable','--edition','2024','--config','skip_children=true','--emit','files',*paths[start:start+48]],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace')
        if r.returncode:raise SystemExit(r.stdout+r.stderr)
    (OUT/'composition.json').write_text(json.dumps(dict(contract=contract,rustSources=len(paths),generatorSha256=hashlib.sha256(source.encode()).hexdigest(),scope='cumulative production plus HighJump; syntax only'),indent=2)+'\n',encoding='utf-8')
    return ns['changes']
def main():
    OUT.mkdir(parents=True,exist_ok=True)
    if not (OUT/'oracle.json').is_file():raise SystemExit('Run collect-high-jump-original.py first')
    block=runpy.run_path(str(HERE/'test-attack-patch.py'))['block'];changes=compose()
    (OUT/'living-motion.rs').write_text(changes['crates/horizon-world/src/living_motion.rs'][1],encoding='utf-8')
    native=(HERE/'high-jump-native-fixture.rs').read_text(encoding='utf-8').replace('__BRIDGE__',(HERE/'high-jump-bridge.rs').as_posix()).replace('__JUMP__',block(changes['crates/horizon-client/src/motion.rs'][1],'    pub fn jump_from_ground('))
    (OUT/'native-fixture.rs').write_text(native,encoding='utf-8')
    shared=(HERE/'engine/src/config.rs').read_text(encoding='utf-8')
    config='''use std::collections::BTreeMap;
use serde_json::{Value,Map,Number,json};
use crate::high_jump;
#[derive(Clone,Copy,PartialEq,Eq)]enum JsonFormat{File,Interop}
enum ModuleKind{HighJump}impl ModuleKind{fn key(self)->&'static str{"liquidbounce.module.highJump"}}
'''
    for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_bool(','fn parse_float(','\nfn leaf(']:
        if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
        config+=block(shared,anchor)+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug,PartialEq)]enum SettingChange{HighJump(terrain::HighJumpSetting)}
#[derive(Default)]struct SettingsPatch{changes:Vec<SettingChange>,issues:Vec<ConfigError>}
'''
    config+=f'#[path="{(HERE/"engine/src/config/high_jump.rs").as_posix()}"]mod terrain;\nuse terrain::*;\n'
    config+=(HERE/'high-jump-config-tests.rs').read_text(encoding='utf-8');(OUT/'config-fixture.rs').write_text(config,encoding='utf-8')
    (OUT/'tests.rs').write_text((HERE/'high-jump-tests.rs').read_text(encoding='utf-8'),encoding='utf-8')
    lib='#![cfg(test)]\n#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;\n'+f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/high_jump.rs").as_posix()}"]pub mod high_jump;\n#[path="tests.rs"]mod tests;\n#[path="config-fixture.rs"]mod config;\ninclude!("native-fixture.rs");\n'
    (OUT/'lib.rs').write_text(lib,encoding='utf-8')
    manifest='[package]\nname="liquidbounce-high-jump-tests"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json="1"\n'
    for name in ['horizon-core','horizon-world']:manifest+=name+'={path="'+(UPSTREAM/'crates'/name).as_posix()+'"}\n'
    (OUT/'Cargo.toml').write_text(manifest,encoding='utf-8');env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/no-entity-interact-tests/target');commands=[]
    for name,argv in [('test',['cargo','+stable','test','--offline','--manifest-path',str(OUT/'Cargo.toml')]),('clippy',['cargo','+stable','clippy','--offline','--manifest-path',str(OUT/'Cargo.toml'),'--all-targets','--','-D','warnings'])]:
        r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/(name+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name=name,exitCode=r.returncode))
        if r.returncode:break
    owned=[Path(__file__),ROOT/'native/liquidbounce-behaviors/src/high_jump.rs',HERE/'engine/src/config/high_jump.rs',HERE/'engine/src/high_jump_runtime.rs',*[HERE/p for p in ['high-jump-patch.py','high-jump-bridge.rs','high-jump-host.rs','high-jump-native-fixture.rs','high-jump-tests.rs','high-jump-config-tests.rs','collect-high-jump-original.py','test-high-jump-native.py']]]
    report=dict(passed=len(commands)==2 and all(c['exitCode']==0 for c in commands),commands=commands,oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),sourceSha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in owned},scope='untouched HighJump+CoroutineTicker 48 scenarios; actual motion/living consumer; storage and Host providers; full client differential not_run')
    (OUT/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
