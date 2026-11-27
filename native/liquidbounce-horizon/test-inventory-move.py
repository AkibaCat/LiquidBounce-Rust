"""Single focused production policy/config/queue check and native patch composition."""
from pathlib import Path
import argparse,json,runpy,subprocess,shutil
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);out=ROOT/a.parse_args().out
 if (out/'report.json').exists():raise SystemExit('Evidence already exists')
 out.mkdir(parents=True,exist_ok=True)
 plan=runpy.run_path(str(HERE/'integrate-inventory-move.py'))['prepare']()
 source=(HERE/'make-behavior-patch.py').read_text(encoding='utf8')
 source=source.replace("'step-patch.py'):","'inventory-move-patch.py', 'step-patch.py'):")
 source=source.replace('destination = HERE / "patches/horizon-behavior-hooks.patch"',f'destination = Path({str(out/"hooks-preview.patch")!r})')
 exec(compile(source,str(HERE/'make-behavior-patch.py'),'exec'),{'__file__':str(HERE/'make-behavior-patch.py'),'__name__':'__main__'})
 shared=(HERE/'engine/src/config.rs').read_text(encoding='utf8');body=runpy.run_path(str(HERE/'test-reset-vl.py'))['body']
 config='use std::collections::BTreeMap;use serde_json::{Value,Map,Number,json};\n#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum JsonFormat{File,Interop}\n#[derive(Clone,Copy,Debug,Default)]pub enum ModuleKind{#[default]InventoryMove}impl ModuleKind{fn key(self)->&\'static str{"liquidbounce.module.inventoryMove"}}\n'
 for anchor in ['pub struct CommonSettings','pub struct ConfigError','impl ConfigError',"struct Tree<'a>","impl Tree<'_>",'fn scalar_string(',"fn nodes<'a>(",'fn float_json(',"fn value_of<'a>(",'fn parse_float(','fn parse_int(','fn decimal_int(','fn parse_bool(','\nfn leaf(']:
  if anchor=='pub struct ConfigError':config+='#[derive(Debug)]\n'
  config+=body(shared,anchor)+'\n'
 config+='type Nodes<\'a>=BTreeMap<String,&\'a Value>;#[derive(Debug)]pub enum SettingChange{InventoryMove(InventoryMoveSetting)}\n#[derive(Default)]'+body(shared,'pub struct SettingsPatch')+'\n'
 config+=f'#[path="{(HERE/"engine/src/config/inventory_move.rs").as_posix()}"]mod settings;pub use settings::*;\n'
 config+='''#[test]fn roundtrip(){let s=crate::inventory_move_model::Settings{behavior:3,passthrough_sneak:true,sprint_control:true,client_sprint:1,server_sprint:2,sneak_control:true,client_sneak:2,timer:true,speed:1.3,blink:true,maximum_time:9123};let c=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};for f in [JsonFormat::File,JsonFormat::Interop]{let tree=serialize_inventory_move(&s,&c,f,&str::to_owned).unwrap();let mut p=SettingsPatch::default();settings::parse_inventory_move(&mut p,&nodes(&tree,"InventoryMove","InventoryMove").unwrap());assert!(p.issues.is_empty(),"{:?}",p.issues);assert_eq!(p.changes.len(),11);let mut restored=crate::inventory_move_model::Settings::default();for SettingChange::InventoryMove(v)in p.changes{v.apply(&mut restored).unwrap();}assert_eq!(s,restored);}let mut s=crate::inventory_move_model::Settings::default();assert!(InventoryMoveSetting("Behavior".into(),json!("bad")).apply(&mut s).is_err());InventoryMoveSetting("Timer.Speed".into(),json!(100)).apply(&mut s).unwrap();assert_eq!(s.speed,2.);InventoryMoveSetting("Blink.MaximumTime".into(),json!(-1)).apply(&mut s).unwrap();assert_eq!(s.maximum_time,0);}'''
 (out/'config.rs').write_text(config,encoding='utf8')
 source='#![allow(dead_code,unused_imports,irrefutable_let_patterns)]\nextern crate self as liquidbounce_behaviors;extern crate self as horizon_protocol;\n'
 source+=f'#[path="{(ROOT/"native/liquidbounce-behaviors/src/inventory_move.rs").as_posix()}"]pub mod inventory_move_model;\npub use inventory_move_model as inventory_move;\n'
 source=source.replace('pub use inventory_move_model as inventory_move;','')
 source+='pub mod packets{pub mod play{#[derive(Clone)]pub enum Serverbound{ContainerClick(()),ContainerButtonClick(()),SetCreativeModeSlot(()),ContainerSlotStateChanged(()),ContainerClose(()),Other}}}\n'
 # The native bridge and behavior intentionally share one actual model source.
 config=config.replace('liquidbounce_behaviors::inventory_move::','crate::inventory_move_model::')
 production=(HERE/'engine/src/config/inventory_move.rs').read_text(encoding='utf8').replace('liquidbounce_behaviors::inventory_move::','crate::inventory_move_model::')
 (out/'settings.rs').write_text(production,encoding='utf8')
 config=config.replace((HERE/'engine/src/config/inventory_move.rs').as_posix(),(out/'settings.rs').as_posix())
 (out/'config.rs').write_text(config,encoding='utf8')
 source+=f'#[path="{(HERE/"inventory-move-bridge.rs").as_posix()}"]pub mod inventory_move;\nmod config;\n'
 source+=(HERE/'inventory-move-smoke.rs').read_text(encoding='utf8');(out/'lib.rs').write_text(source,encoding='utf8')
 candidates=list((ROOT/'build/staging/horizon/target/debug/deps').glob('libserde_json-*.rlib'));dependency=max(candidates,key=lambda p:p.stat().st_mtime_ns)
 command=['rustc','+stable','--edition=2024','--test',str(out/'lib.rs'),'-L','dependency='+str(dependency.parent),'--extern','serde_json='+str(dependency),'-o',str(out/'smoke.exe')]
 compiled=subprocess.run(command,capture_output=True,text=True,encoding='utf8',errors='replace');log=compiled.stdout+compiled.stderr
 result=compiled
 if compiled.returncode==0:
  result=subprocess.run([str(out/'smoke.exe'),'--test-threads=1'],capture_output=True,text=True,encoding='utf8',errors='replace');log+=result.stdout+result.stderr
 (out/'test.log').write_text(log,encoding='utf8')
 loc=out/'localization';loc.mkdir(exist_ok=True)
 for code in ['zh_cn','zh_tw']:shutil.copyfile(HERE.parent/f'liquidbounce-interop/resources/localization/display_{code}.json',loc/f'display_{code}.json')
 update=runpy.run_path(str(HERE/'update-inventory-move-locales.py'))['update'];update(loc)
 for p in loc.glob('*.json'):
  d=json.loads(p.read_text(encoding='utf8'));assert len([k for k in d['descriptions']if k.startswith('liquidbounce.module.inventoryMove.')])>=11;assert '\ufffd'not in p.read_text(encoding='utf8')
 report={'passed':result.returncode==0,'exitCode':result.returncode,'sharedEdits':len(plan.steps),'scope':'Production four policies, Safe click/key transitions, sprint precedence, exact blink deadline, five container packet classes, next-loop StopOnAction replay and disable, real config eleven-leaf File/Interop roundtrip and range/error guards, full cumulative native patch generation, both Chinese locales. No whole suite or gameplay run.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report));raise SystemExit(result.returncode)
if __name__=='__main__':main()
