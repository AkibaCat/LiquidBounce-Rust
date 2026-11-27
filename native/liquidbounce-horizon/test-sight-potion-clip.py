"""One short compile per module, actual config helpers and actual patch anchors."""
from pathlib import Path
import argparse,runpy,subprocess,os,json,hashlib,shutil
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--module',choices=['potion_spoof','no_clip','true_sight'],required=True);p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);module=a.module;name={'potion_spoof':'PotionSpoof','no_clip':'NoClip','true_sight':'TrueSight'}[module]
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{'+name+'}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.'+name[0].lower()+name[1:]+'"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{'+name+'(settings::'+name+'Setting)}\n#[derive(Default)]pub struct SettingsPatch{pub changes:Vec<SettingChange>,pub issues:Vec<ConfigError>}\n'
 config+=f'#[path="{(HERE/f"engine/src/config/{module}.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='#[test]fn complete_config_smoke(){let state=crate::'+module+'::'+name+'::default();let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_'+module+'(&state,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_'+module+'(&mut patch,&nodes(&tree,"'+name+'","'+name+'").unwrap());assert!(patch.issues.is_empty());let mut restored=state.clone();for change in patch.changes{let SettingChange::'+name+'(setting)=change;setting.apply(&mut restored);}assert_eq!(restored,state);}}\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;\n'+f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{module}.rs").as_posix()}"]pub mod {module};\nmod config;\n'
 if module=='no_clip':
  lib+='''extern crate self as horizon_client;
pub mod mod_api {pub enum ClientboundPlayPacket {PlayerPosition(()),Other}}
pub mod packets {
 pub struct Context(pub bool);impl Context{pub fn original(&self)->bool{self.0}}
 pub struct PacketEvent<'a,P>{pub context:Context,pub packet:&'a mut P,pub cancelled:bool}
 pub trait PacketListener{fn incoming(&self,event:&mut PacketEvent<'_,crate::mod_api::ClientboundPlayPacket>);}
}
'''+f'#[path="{(HERE/"engine/src/no_clip_packets.rs").as_posix()}"]mod no_clip_packets;\n'
 lib+=(HERE/f'{module.replace("_","-")}-smoke.rs').read_text(encoding='utf8')
 (out/'lib.rs').write_text(lib,encoding='utf8');(out/'Cargo.toml').write_text('[package]\nname="'+module.replace('_','-')+'-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,old,new):assert t.count(old)==1,(old,t.count(old));return t.replace(old,new)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 patch_error=None
 try:
  meta=runpy.run_path(str(HERE/f'{module.replace("_","-")}-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as error:patch_error=repr(error);meta=None
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-sight-potion-clip-locales.py'))
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 report={'passed':r.returncode==0 and patch_error is None,'compileExit':r.returncode,'patchError':patch_error,'metadata':meta,'scope':'Exact owned behavior/config files with shared Tree/parser bodies; short behavior smoke; owned patch applied to current staged actual sources in memory; locale updater on copies. Runtime/native full Game compilation remains final aggregate validation.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
