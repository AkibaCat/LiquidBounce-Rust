"""One focused compilation/module; copied dictionary and actual patch anchors only."""
from pathlib import Path
import argparse,runpy,subprocess,os,json,shutil
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--module',choices=['mob_owners','combine_mobs','better_tab'],required=True);p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 out.mkdir(parents=True,exist_ok=True);module=a.module;slug=module.replace('_','-');name={'mob_owners':'MobOwners','combine_mobs':'CombineMobs','better_tab':'BetterTab'}[module]
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{'+name+'}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.'+name[0].lower()+name[1:]+'"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;\n#[derive(Debug)]pub enum SettingChange{'+name+'(settings::'+name+'Setting)}\n#[derive(Default)]pub struct SettingsPatch{pub changes:Vec<SettingChange>,pub issues:Vec<ConfigError>}\n'
 config+=f'#[path="{(HERE/f"engine/src/config/{module}.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='#[test]fn complete_config_smoke(){let state=crate::'+module+'::'+name+'::default();let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for format in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_'+module+'(&state,&common,format,&str::to_owned).unwrap();let mut patch=SettingsPatch::default();settings::parse_'+module+'(&mut patch,&nodes(&tree,"'+name+'","'+name+'").unwrap());assert!(patch.issues.is_empty(),"{:?}",patch.issues);let mut restored=state.clone();for change in patch.changes{let SettingChange::'+name+'(setting)=change;setting.apply(&mut restored);}assert_eq!(restored,state);}}\n'
 (out/'config.rs').write_text(config,encoding='utf8')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;\n'+f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{module}.rs").as_posix()}"]pub mod {module};\nmod config;\n'
 if module=='better_tab':lib+=f'#[path="{(HERE/"better-tab-render.rs").as_posix()}"]mod better_tab_render;\n'
 if module=='mob_owners':
  shutil.copyfile(HERE/'mob-owners-cache.rs',out/'mob_owners_cache.rs');shutil.copyfile(HERE/'mob-owners-types.rs',out/'mob_owners_types.rs');lib+='mod mob_owner_cache {include!("mob_owners_cache.rs");}\n'
 lib+=(HERE/f'{slug}-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(lib,encoding='utf8')
 dependencies='serde_json={version="1",features=["arbitrary_precision"]}\njava_regex={path="'+(ROOT/'native/liquidbounce-behaviors/vendor/java_regex').as_posix()+'"}\n'
 if module=='mob_owners':dependencies+='ureq={version="2",default-features=false,features=["tls"]}\n'
 (out/'Cargo.toml').write_text('[package]\nname="'+slug+'-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[dependencies]\n'+dependencies,encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 patched={};base=ROOT/'build/staging/horizon'
 def once(t,old,new):assert t.count(old)==1,(old,t.count(old));return t.replace(old,new)
 def edit(path,op):patched[path]=op(patched.get(path,(base/path).read_text(encoding='utf8')if(base/path).exists()else''))
 patch_error=None
 try:
  meta=runpy.run_path(str(HERE/f'{slug}-patch.py'))['apply'](edit,once)
  for path,t in patched.items():target=out/'patched'/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(t,encoding='utf8')
 except Exception as error:patch_error=repr(error);meta=None
 locale=out/'localization';locale.mkdir(exist_ok=True);updater=runpy.run_path(str(HERE/'update-owner-combine-tab-locales.py'))
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 report={'passed':r.returncode==0 and patch_error is None,'compileExit':r.returncode,'patchError':patch_error,'metadata':meta,'scope':'Actual owned behavior/config and shared Tree/parser; short behavioral smoke; BetterTab compiles actual render policy against declaration-only text/player types; MobOwners compiles actual ureq cache without network requests. Actual patches applied in memory to staged source; dictionaries tested on copies. Full Game/native compilation remains aggregate validation.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
