#!/usr/bin/env python3
"""Prepare production ItemTags model/config and composed WGSL tests; --run compiles."""
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
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{ItemTags}
impl ModuleKind{fn name(self)->&'static str{"ItemTags"}fn key(self)->&'static str{"liquidbounce.module.itemTags"}}
'''
    for anchor,derive in [('pub enum JsonFormat','Clone,Copy,Debug,PartialEq,Eq'),('pub struct CommonSettings','Clone,Debug,PartialEq'),('pub struct ConfigError','Clone,Debug,PartialEq,Eq'),('impl ConfigError',''),('impl fmt::Display for ConfigError',''),("struct Tree<'a>",''),("impl Tree<'_>",''),('fn float_json(',''),('fn scalar_string(',''),("fn nodes<'a>(",''),("fn value_of<'a>(",''),('fn parse_bool(',''),('fn parse_float(',''),('fn parse_int(',''),('fn decimal_int(',''),('\nfn leaf(',''),('pub struct SettingsPatch','Clone,Debug,PartialEq')]:
        part=BODY(shared,anchor)
        if anchor=='\nfn leaf(':assert '&self'not in part and 'patch: &mut SettingsPatch'in part
        config+=(f'#[derive({derive})]\n'if derive else '')+part+'\n'
    config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),ItemTags(ItemTagsSetting)}
'''
    entry=BODY(shared,'pub fn parse_settings(');common=entry[entry.index('    let root ='):entry.index('    match module {')]
    common=common.replace('nodes(json, group_name, module.name())?','nodes(json,module.name(),module.name())?')
    config+='pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{\n'+common+'item_tags::parse_item_tags(&mut patch,&root);Ok(patch)}\n'
    actual=HERE/'engine/src/config/item_tags.rs';original=actual.read_text(encoding='utf8');assert original.count('#[test]')==2
    (out/'item-tags-config.rs').write_text(original,encoding='utf8')
    config+='#[path="item-tags-config.rs"]mod item_tags;pub use item_tags::*;\n';(out/'config.rs').write_text(config,encoding='utf8')
    static=runpy.run_path(str(HERE/'test-item-tags.py'));native=static['compose_native']();static['validate_native'](native)
    shader=native['crates/horizon-client-render/src/hud.wgsl'];(out/'hud.wgsl').write_text(shader,encoding='utf8')
    model=ROOT/'native/liquidbounce-behaviors/src/item_tags.rs';curve=ROOT/'native/liquidbounce-behaviors/src/tracers_curve.rs'
    lib=f'''#![allow(dead_code,unused_imports)]
extern crate self as liquidbounce_behaviors;
pub mod tracers{{#[path="{curve.as_posix()}"]pub mod curve;}}
#[path="{model.as_posix()}"]pub mod item_tags;
mod config;
#[cfg(test)]mod composed_shader{{
 #[test]fn actual_composed_hud_shader_parses_and_validates(){{
  let source=include_str!("hud.wgsl");
  let module=naga::front::wgsl::parse_str(source).unwrap_or_else(|e|panic!("{{}}",e.emit_to_string(source)));
  naga::valid::Validator::new(naga::valid::ValidationFlags::all(),naga::valid::Capabilities::all()).validate(&module).unwrap();
 }}
}}
'''
    (out/'lib.rs').write_text(lib,encoding='utf8')
    (out/'Cargo.toml').write_text('''[package]
name="item-tags-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={version="1",features=["arbitrary_precision"]}
naga={version="30.0.1",features=["wgsl-in"]}
''',encoding='utf8')
    sources=[HERE/'engine/src/config.rs',actual,model,model.with_name('item_tags_layout.rs'),curve,HERE/'item-tags-patch.py',HERE/'item-tags-hud.rs',HERE/'item-tags-shader.wgsl']
    sources.append(HERE/'item-stack-list-game.rs')
    report={'status':'prepared-no-compiler-invoked','tests':6,'scope':'Exact production model/layout/curve with original three tests; exact production config with original two tests and real shared Tree/common/parser functions. Registration narrowed to ItemTags; no engine actor, native game or GPU execution claim. Native shared planned queue/ChestStealer connection is statically composed, not executed by this fixture. One Naga parse/validation test uses the complete HUD shader from all actual native generators composed in memory.','composedShaderSha256':hashlib.sha256(shader.encode()).hexdigest(),'sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in sources}}
    (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');parser.add_argument('--filter',default='');args=parser.parse_args()
    out=ROOT/args.out;prepare(out)
    if not args.run:print(f'ItemTags focused fixture prepared: {out}; no compiler invoked');return
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target')
    cmd=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]
    if args.filter:cmd.append(args.filter)
    cmd+=['--','--test-threads=1'];r=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace')
    (out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode,'filter':args.filter},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)


if __name__=='__main__':main()
