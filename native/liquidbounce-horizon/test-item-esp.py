"""Prepare eight focused production ItemESP checks; compilation is opt-in."""
from pathlib import Path
import argparse,ast,json,os,runpy,subprocess
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def prepare(out):
 out.mkdir(parents=True,exist_ok=True)
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
 shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 c='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]ItemEsp}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.itemEsp"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','fn leaf(\n    patch:']:
  if anchor in ['pub struct CommonSettings','pub struct ConfigError']:c+='#[derive(Debug)]\n'
  c+=body(shared,anchor)+'\n'
 c+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{ItemEsp(ItemEspSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 c+=f'#[path="{(HERE/"engine/src/config/item_esp.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 c+="pub fn parse_item_esp(p:&mut SettingsPatch,n:&Nodes<'_>){settings::parse_item_esp(p,n);}\n"
 c=c.replace('fn nodes<','pub fn nodes<');(out/'config.rs').write_text(c,encoding='utf8')
 lib='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;pub use lb::{breadcrumbs,storage_esp};\n'
 lib+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/item_esp.rs").as_posix()}"]pub mod item_esp;\nmod config;\n'+(HERE/'item-esp-smoke.rs').read_text(encoding='utf8')
 (out/'lib.rs').write_text(lib,encoding='utf8')
 (out/'Cargo.toml').write_text(f'''[package]
name="item-esp-focused"
version="0.0.0"
edition="2024"
[workspace]
[lib]
path="lib.rs"
[dependencies]
serde_json={{version="1",features=["arbitrary_precision"]}}
lb={{package="liquidbounce-behaviors",path="{(ROOT/'native/liquidbounce-behaviors').as_posix()}"}}
''',encoding='utf8')
 for name in ['item-esp-patch.py','item-esp-render.py','integrate-item-esp.py','update-item-esp-locales.py']:ast.parse((HERE/name).read_text(encoding='utf8'))
 (out/'preparation.json').write_text(json.dumps({'status':'prepared-no-compiler-invoked','tests':8,'scope':'Actual production ItemESP model, full settings/config and shared Tree/parsers. Native model captures/common OutlineGlow will be checked in the combined official build and final game validation.'},indent=2)+'\n',encoding='utf8')
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--run',action='store_true');p.add_argument('--filter',default='');args=p.parse_args();out=ROOT/args.out;prepare(out)
 if not args.run:print('ItemESP fixture prepared, no compiler invoked');raise SystemExit(0)
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'build/staging/horizon/target')
 cmd=['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')]
 if args.filter:cmd.append(args.filter)
 cmd+=['--','--test-threads=1'];r=subprocess.run(cmd,env=env,cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8');(out/'report.json').write_text(json.dumps({'passed':r.returncode==0,'exitCode':r.returncode,'filter':args.filter},indent=2)+'\n',encoding='utf8');print(r.stdout+r.stderr);raise SystemExit(r.returncode)
