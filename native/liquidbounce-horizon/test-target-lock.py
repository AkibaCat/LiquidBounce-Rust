#!/usr/bin/env python3
"""One focused compile and smoke of owned behavior/config/runtime/native helpers."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--source',default='build/target-lock-source-01');a=p.parse_args();out=ROOT/a.out;source=ROOT/a.source
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 assert json.loads((source/'report.json').read_text())['passed'];out.mkdir(parents=True,exist_ok=True)
 for name in ['inputs.json','oracle.json']:shutil.copyfile(source/name,out/name)
 body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body'];shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8')
 config='use std::collections::BTreeMap;\nuse serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\nenum ModuleKind{TargetLock}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.targetLock"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='''type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Debug)]pub enum SettingChange{TargetLock(settings::TargetLockSetting)}
#[derive(Default)]pub struct SettingsPatch{pub changes:Vec<SettingChange>,pub issues:Vec<ConfigError>}
'''
 config+=f'#[path="{(HERE/"engine/src/config/target_lock.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''pub fn parse_tree(tree:&Value)->SettingsPatch{let mut patch=SettingsPatch::default();settings::parse_target_lock(&mut patch,&nodes(tree,"TargetLock","TargetLock").unwrap());patch}
pub fn apply_names(state:&mut crate::target_lock::TargetLock,value:Value)->bool{let tree=json!({"name":"TargetLock","value":[{"name":"Mode","active":"Filter","value":[],"choices":{"Temporary":{"name":"Temporary","value":[]},"Filter":{"name":"Filter","value":[{"name":"Usernames","value":value}]}}}]});let patch=parse_tree(&tree);for item in patch.changes{if let SettingChange::TargetLock(TargetLockSetting::Usernames(v))=item{state.settings.usernames=v;}}!patch.issues.is_empty()}
'''
 (out/'config.rs').write_text(config,encoding='utf8');shutil.copyfile(HERE/'target-lock-tests.rs',out/'tests.rs');shutil.copyfile(HERE/'target-lock-runtime-fixture.rs',out/'runtime-fixture.rs')
 lib='#![allow(dead_code)]\nextern crate self as liquidbounce_behaviors;\nextern crate self as horizon_client;\n'
 for name in ['target_lock','targeting']:lib+=f'#[path="{(ROOT/f"native/liquidbounce-behaviors/src/{name}.rs").as_posix()}"]pub mod {name};\n'
 lib+=f'#[path="{(HERE/"engine/src/target_lock_runtime.rs").as_posix()}"]mod target_lock_runtime;\n'
 native=(HERE/'target-lock-native-fixture.rs').read_text(encoding='utf8').replace('__OWNED_HELPERS__',(HERE/'target-lock-host.rs').read_text(encoding='utf8'))
 (out/'native-fixture.rs').write_text(native,encoding='utf8')
 lib+='include!("runtime-fixture.rs");\ninclude!("native-fixture.rs");\nmod config;\n#[cfg(test)]mod tests;\n'
 (out/'lib.rs').write_text(lib,encoding='utf8');(out/'Cargo.toml').write_text('[package]\nname="target-lock-smoke"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[lib]\npath="lib.rs"\n[features]\ndefault=["webview"]\nwebview=[]\n[dependencies]\nserde_json={version="1",features=["arbitrary_precision"]}\n',encoding='utf8')
 env=os.environ.copy();env['CARGO_TARGET_DIR']=str(HERE/'packet-tests/target');r=subprocess.run(['cargo','+stable','test','--offline','--manifest-path',str(out/'Cargo.toml')],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'test.log').write_text(r.stdout+r.stderr,encoding='utf8')
 # The locale updater is exercised on copies only.
 locale=out/'localization';locale.mkdir(exist_ok=True)
 updater=runpy.run_path(str(HERE/'update-target-lock-locales.py'))
 for language in ['zh_cn','zh_tw']:
  target=locale/f'display_{language}.json';shutil.copyfile(ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json',target);updater['update'](language,target)
 # Execute the owned patch against a copy containing Hitbox's actual helper.
 patched={'crates/horizon-client/src/game/native_hooks.rs':(HERE/'hitbox-host-methods.rs').read_text(encoding='utf8')+'\n    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'}
 def once(source,old,new):
  assert source.count(old)==1,(old,source.count(old));return source.replace(old,new)
 def edit(path,operation):patched[path]=operation(patched[path])
 metadata=runpy.run_path(str(HERE/'target-lock-patch.py'))['apply'](edit,once)
 assert metadata['helper_names']==['client_target_lock_player','client_target_lock_dead']
 assert '"abstract_client_player":lock_player["name"].is_string()'in patched['crates/horizon-client/src/game/native_hooks.rs']
 owned=[ROOT/'native/liquidbounce-behaviors/src/target_lock.rs',HERE/'engine/src/config/target_lock.rs',HERE/'engine/src/target_lock_runtime.rs',HERE/'target-lock-host.rs',HERE/'target-lock-patch.py']
 report={'passed':r.returncode==0,'commands':[{'name':'focused-test','exitCode':r.returncode}],'scope':'Owned behavior/config/runtime/native helper bodies with actual shared Tree/parser/targeting bodies; explicit config/Host/State/Game scalar providers. Short per-step untouched Kotlin/ListValue oracle. Owned patch tested with actual Hitbox helper copy. Locale updater on copies. Concrete Game and complete engine deferred to final cumulative validation.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in owned},'scenarios':json.loads((source/'report.json').read_text())['scenarios'],'steps':json.loads((source/'report.json').read_text())['steps']}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
