"""Transactional Wings shared wiring; validate every anchor/original before writes."""
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
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod wings;');absent(E+'behaviors.rs','    pub wings:');absent(E+'config.rs','    Wings =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod hats;','pub mod hats;\npub mod wings;')
 edit(E+'lib.rs','mod hats_runtime;','mod hats_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod wings_runtime;')
 edit(E+'behaviors.rs','    pub hats:','    pub wings: liquidbounce_behaviors::wings::Wings,\n    pub hats:')
 edit(E+'behaviors.rs','            "Hats",','            "Hats",\n            "Wings",')
 edit(E+'behaviors.rs','            hats: Default::default(),','            wings: Default::default(),\n            hats: Default::default(),')
 edit(E+'config.rs','#[path = "config/hats.rs"]','#[path = "config/wings.rs"]\nmod wings;\npub use wings::{WingsSetting,serialize_wings};\n#[path = "config/hats.rs"]')
 edit(E+'config.rs','    Hats => ("Hats", "Render", "hats"),','    Hats => ("Hats", "Render", "hats"),\n    Wings => ("Wings", "Render", "wings"),')
 edit(E+'config.rs','    Hats(HatsSetting),','    Hats(HatsSetting),\n    Wings(WingsSetting),',2)
 edit(E+'config.rs','            | ModuleKind::Hats','            | ModuleKind::Hats\n            | ModuleKind::Wings')
 edit(E+'config.rs','        ModuleKind::Hats => hats::parse_hats(&mut patch,&root),','        ModuleKind::Hats => hats::parse_hats(&mut patch,&root),\n        ModuleKind::Wings => wings::parse_wings(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Hats(value) => {','        SettingChange::Wings(value) => { return ApplyResult::Runtime(RuntimeRequest::Wings(value.clone())); }\n        SettingChange::Hats(value) => {')
 edit(E+'actor.rs','            ModuleKind::Hats => config::serialize_hats(&b.hats,&common,format,&describe),','            ModuleKind::Hats => config::serialize_hats(&b.hats,&common,format,&describe),\n            ModuleKind::Wings => config::serialize_wings(&b.wings,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Hats(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Wings(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().wings); }\n                ApplyResult::Runtime(RuntimeRequest::Hats(setting)) => {')
 edit(E+'actor.rs','            ModuleKind::Hats => Some(liquidbounce_behaviors::hats::MODES[self.state.behaviors.borrow().hats.mode]),','            ModuleKind::Wings => Some("Lines"),\n            ModuleKind::Hats => Some(liquidbounce_behaviors::hats::MODES[self.state.behaviors.borrow().hats.mode]),')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_wings(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientWingsEvent<\'_>){self.wings(host,event);}\n'+anchor)
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'wings-patch.py'");edit(patch,"'semantic-pointer-patch.py'","'wings-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if(ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
 if not a.check:
  for path,text in changes.items():(ROOT/path).write_text(text,encoding='utf8')
 print(f'{"Validated"if a.check else"Integrated"} Wings across {len(changes)} shared files')
if __name__=='__main__':main()
