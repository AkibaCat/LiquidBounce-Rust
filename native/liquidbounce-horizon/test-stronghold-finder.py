"""Prepare a single focused StrongholdFinder batch smoke; compile only with --run."""
from pathlib import Path
import argparse,ast,json,runpy,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
JAVA='''import java.util.SplittableRandom;
public class StrongholdOracle {
 public static void main(String[]args){StringBuilder out=new StringBuilder("[");
  for(int h=0;h<2;h++){if(h>0)out.append(',');out.append('[');var random=new SplittableRandom(0x51f15eedL+h);float pi=(float)Math.PI*2;double angle=random.nextDouble()*pi;int ring=0,spread=3,inRing=0;
   for(int i=0;i<128;i++){double radius=128+192*ring+(random.nextDouble()-.5)*80;int x=(int)Math.floor(Math.cos(angle)*radius+.5),z=(int)Math.floor(Math.sin(angle)*radius+.5);x+=random.nextInt(-7,8);z+=random.nextInt(-7,8);if(i>0)out.append(',');out.append('[').append(x).append(',').append(z).append(']');angle+=pi/spread;inRing++;if(inRing==spread){ring++;inRing=0;spread+=2*spread/(ring+1);spread=Math.min(spread,128-i);angle+=random.nextDouble()*pi;}}
   out.append(']');}out.append(']');System.out.print(out);}
}'''
def prepare(out):
 out.mkdir(parents=True,exist_ok=True);body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]StrongholdFinder}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.strongholdFinder"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor in ['pub struct ConfigError','pub struct CommonSettings']:config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{StrongholdFinder(StrongholdFinderSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/stronghold_finder.rs").as_posix()}"]mod settings;pub use settings::*;\npub use settings::parse_stronghold_finder;\n'
 config=config.replace('pub(super) fn parse_stronghold_finder','pub fn parse_stronghold_finder') # production child visibility is handled by local wrapper below
 config=config.replace('pub use settings::parse_stronghold_finder;','pub fn parse_stronghold_finder(p:&mut SettingsPatch,r:&Nodes<\'_>){settings::parse_stronghold_finder(p,r);}')
 config=config.replace('fn nodes<','pub fn nodes<')
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;\n'+f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/stronghold_finder.rs").as_posix()}"]pub mod stronghold_finder;\nmod config;\n'+(HERE/'stronghold-finder-smoke.rs').read_text(encoding='utf8')
 (out/'lib.rs').write_text(lib,encoding='utf8');(out/'StrongholdOracle.java').write_text(JAVA,encoding='utf8')
 (out/'Cargo.toml').write_text('[package]\nname="stronghold-finder-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 for name in ['integrate-stronghold-finder.py','stronghold-finder-patch.py','update-stronghold-finder-locales.py']:ast.parse((HERE/name).read_text(encoding='utf8'))
 (out/'preparation.json').write_text(json.dumps({'status':'prepared-no-compiler-invoked','scope':'Production model/config plus one small translated numeric fixture using real JDK SplittableRandom; not an unmodified original-client oracle or game test.'},indent=2)+'\n',encoding='utf8')
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--run',action='store_true');args=p.parse_args();out=ROOT/args.out;prepare(out)
 if not args.run:print('StrongholdFinder smoke prepared; no compiler invoked');return
 jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin';commands=[('java-compile',[str(jdk/'javac.exe'),str(out/'StrongholdOracle.java')]),('oracle',[str(jdk/'java.exe'),'-cp',str(out),'StrongholdOracle']),('rust-test',['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml'),'--','--test-threads=1'])]
 checks=[]
 for name,command in commands:
  result=subprocess.run(command,capture_output=True,text=True,encoding='utf8',errors='replace');(out/f'{name}.log').write_text(result.stdout+result.stderr,encoding='utf8');checks.append({'name':name,'exitCode':result.returncode})
  if name=='oracle' and result.returncode==0:json.loads(result.stdout);(out/'oracle.json').write_text(result.stdout,encoding='utf8')
  if result.returncode:break
 passed=len(checks)==3 and all(c['exitCode']==0 for c in checks);(out/'report.json').write_text(json.dumps({'passed':passed,'checks':checks,'scope':'Focused production model/config; translated JDK numeric fixture. No game/full-suite.'},indent=2)+'\n',encoding='utf8');print(checks);raise SystemExit(0 if passed else 1)
if __name__=='__main__':main()
