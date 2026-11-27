"""Transactional Crosshair shared wiring; no writes until all anchors pass."""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2];E='native/liquidbounce-horizon/engine/src/'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();originals={};changes={}
 def read(path):
  if path not in changes:originals[path]=(ROOT/path).read_text(encoding='utf8');changes[path]=originals[path]
  return changes[path]
 def edit(path,old,new,count=1):
  text=read(path)
  if text.count(old)!=count:raise ValueError(f'{path}: expected {count} of {old[:70]!r}, found {text.count(old)}')
  changes[path]=text.replace(old,new)
 def absent(path,token):
  if token in read(path):raise ValueError(f'{path}: already contains {token}')
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod crosshair;');absent(E+'behaviors.rs','    pub crosshair:');absent(E+'config.rs','    Crosshair =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod wings;','pub mod wings;\npub mod crosshair;')
 edit(E+'lib.rs','mod wings_runtime;','mod wings_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod crosshair_runtime;')
 edit(E+'behaviors.rs','    pub wings:','    pub crosshair: liquidbounce_behaviors::crosshair::Crosshair,\n    pub wings:')
 edit(E+'behaviors.rs','            "Wings",','            "Wings",\n            "Crosshair",')
 edit(E+'behaviors.rs','            wings: Default::default(),','            crosshair: Default::default(),\n            wings: Default::default(),')
 edit(E+'config.rs','#[path = "config/wings.rs"]','#[path = "config/crosshair.rs"]\nmod crosshair;\npub use crosshair::{CrosshairSetting,serialize_crosshair};\n#[path = "config/wings.rs"]')
 edit(E+'config.rs','    Wings => ("Wings", "Render", "wings"),','    Wings => ("Wings", "Render", "wings"),\n    Crosshair => ("Crosshair", "Render", "crosshair"),')
 edit(E+'config.rs','    Wings(WingsSetting),','    Wings(WingsSetting),\n    Crosshair(CrosshairSetting),',2)
 edit(E+'config.rs','            | ModuleKind::Wings','            | ModuleKind::Wings\n            | ModuleKind::Crosshair')
 edit(E+'config.rs','        ModuleKind::Wings => wings::parse_wings(&mut patch,&root),','        ModuleKind::Wings => wings::parse_wings(&mut patch,&root),\n        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Wings(value) => {','        SettingChange::Crosshair(value) => { return ApplyResult::Runtime(RuntimeRequest::Crosshair(value.clone())); }\n        SettingChange::Wings(value) => {')
 edit(E+'actor.rs','            ModuleKind::Wings => config::serialize_wings(&b.wings,&common,format,&describe),','            ModuleKind::Wings => config::serialize_wings(&b.wings,&common,format,&describe),\n            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Wings(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().crosshair); }\n                ApplyResult::Runtime(RuntimeRequest::Wings(setting)) => {')
 edit(E+'actor.rs','            ModuleKind::Wings => Some("Lines"),','            ModuleKind::Crosshair => Some(if self.state.behaviors.borrow().crosshair.mode()=="CS2"{"CS2"}else{"Circle"}),\n            ModuleKind::Wings => Some("Lines"),')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_crosshair(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientCrosshairEvent<\'_>){self.crosshair(host,event);}\n'+anchor)
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'crosshair-patch.py'");edit(patch,"'crosshair-clicker-patch.py'","'crosshair-clicker-patch.py', 'crosshair-patch.py'")
 for path,original in originals.items():
  if(ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
 if not a.check:
  for path,text in changes.items():(ROOT/path).write_text(text,encoding='utf8')
 print(f'{"Validated"if a.check else"Integrated"} Crosshair across {len(changes)} shared files')
if __name__=='__main__':main()
