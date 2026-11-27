"""Transactional Hats shared wiring; validate all anchors/originals before writes."""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2];E='native/liquidbounce-horizon/engine/src/'
def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();originals={};changes={}
 def read(path):
  if path not in changes:originals[path]=(ROOT/path).read_text(encoding='utf8');changes[path]=originals[path]
  return changes[path]
 def edit(path,old,new,count=1):
  t=read(path)
  if t.count(old)!=count:raise ValueError(f'{path}: expected {count} of {old[:80]!r}, got {t.count(old)}')
  changes[path]=t.replace(old,new)
 def absent(path,token):
  if token in read(path):raise ValueError(f'{path}: already contains {token}')
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod hats;');absent(E+'behaviors.rs','    pub hats:');absent(E+'config.rs','    Hats =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod hats;')
 edit(E+'lib.rs','mod breadcrumbs_runtime;','mod breadcrumbs_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod hats_runtime;')
 edit(E+'behaviors.rs','    pub breadcrumbs:','    pub hats: liquidbounce_behaviors::hats::Hats,\n    pub breadcrumbs:')
 edit(E+'behaviors.rs','            "Breadcrumbs",','            "Breadcrumbs",\n            "Hats",')
 edit(E+'behaviors.rs','            breadcrumbs: Default::default(),','            hats: Default::default(),\n            breadcrumbs: Default::default(),')
 edit(E+'config.rs','#[path = "config/breadcrumbs.rs"]','#[path = "config/hats.rs"]\nmod hats;\npub use hats::{HatsSetting,serialize_hats};\n#[path = "config/breadcrumbs.rs"]')
 edit(E+'config.rs','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),\n    Hats => ("Hats", "Render", "hats"),')
 edit(E+'config.rs','    Breadcrumbs(BreadcrumbsSetting),','    Breadcrumbs(BreadcrumbsSetting),\n    Hats(HatsSetting),',2)
 edit(E+'config.rs','            | ModuleKind::Breadcrumbs','            | ModuleKind::Breadcrumbs\n            | ModuleKind::Hats')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::Hats => hats::parse_hats(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::Hats(value) => { return ApplyResult::Runtime(RuntimeRequest::Hats(value.clone())); }\n        SettingChange::Damage(value) => {')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::Hats => config::serialize_hats(&b.hats,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Hats(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().hats); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
 edit(E+'actor.rs','            ModuleKind::Timer => Some(self.state.behaviors.borrow().timer.state.lock().expect("Timer poisoned").settings.mode.tag()),','            ModuleKind::Hats => Some(liquidbounce_behaviors::hats::MODES[self.state.behaviors.borrow().hats.mode]),\n            ModuleKind::Timer => Some(self.state.behaviors.borrow().timer.state.lock().expect("Timer poisoned").settings.mode.tag()),')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_hats(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientHatsEvent<\'_>){self.hats(host,event);}\n'+anchor)
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'hats-patch.py'")
 edit(patch,"'semantic-pointer-patch.py'","'hats-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if(ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
 if not a.check:
  for path,t in changes.items():(ROOT/path).write_text(t,encoding='utf8')
 print(f'{"Validated"if a.check else"Integrated"} Hats across {len(changes)} shared files')
if __name__=='__main__':main()
