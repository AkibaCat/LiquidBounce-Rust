#!/usr/bin/env python3
"""One-time Breadcrumbs wiring. Validate all replacements before any write."""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2];E='native/liquidbounce-horizon/engine/src/'

def main():
 p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args();originals={};changes={}
 def read(path):
  if path not in changes:originals[path]=(ROOT/path).read_text(encoding='utf8');changes[path]=originals[path]
  return changes[path]
 def edit(path,old,new,count=1):
  text=read(path)
  if text.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:80]!r}, got {text.count(old)}')
  changes[path]=text.replace(old,new)
 def absent(path,token):
  if token in read(path):raise ValueError(f'{path}: already contains {token!r}; refuse partial/repeated integration')
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;');absent(E+'behaviors.rs','    pub breadcrumbs:');absent(E+'config.rs','    Breadcrumbs =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod breadcrumbs;')
 edit(E+'lib.rs','mod auto_break_runtime;','mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod breadcrumbs_runtime;')
 edit(E+'behaviors.rs','    pub auto_leave:','    pub breadcrumbs: liquidbounce_behaviors::breadcrumbs::Breadcrumbs,\n    pub auto_leave:')
 edit(E+'behaviors.rs','            "AutoLeave",','            "AutoLeave",\n            "Breadcrumbs",')
 edit(E+'behaviors.rs','            auto_leave: Default::default(),','            breadcrumbs: Default::default(),\n            auto_leave: Default::default(),')
 edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/breadcrumbs.rs"]\nmod breadcrumbs;\npub use breadcrumbs::{BreadcrumbsSetting,serialize_breadcrumbs};\n#[path = "config/auto_weapon.rs"]')
 edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),')
 edit(E+'config.rs','"MultiTask" => Some(Self::MultiActions),','"PlayerTrails" => Some(Self::Breadcrumbs), "MultiTask" => Some(Self::MultiActions),')
 edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    Breadcrumbs(BreadcrumbsSetting),',2)
 edit(E+'config.rs','            | ModuleKind::AutoLeave','            | ModuleKind::AutoLeave\n            | ModuleKind::Breadcrumbs')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::Breadcrumbs => breadcrumbs::parse_breadcrumbs(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::Breadcrumbs(value) => { return ApplyResult::Runtime(RuntimeRequest::Breadcrumbs(value.clone())); }\n        SettingChange::Damage(value) => {')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::Breadcrumbs => config::serialize_breadcrumbs(&b.breadcrumbs,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Breadcrumbs(setting)) => { setting.clone().apply(&mut self.state.behaviors.borrow_mut().breadcrumbs); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {','            ModuleCallback::OnDisabled(id) if self.query_ids.get("Breadcrumbs") == Some(&id) => { self.breadcrumbs.clear(); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {')
 edit(E+'runtime.rs',anchor,'''    #[cfg(feature = "behavior-hooks")]
    fn client_breadcrumbs(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientBreadcrumbsEvent<'_>){
        self.breadcrumbs(host,event);
    }
'''+anchor)
 edit(E+'runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {",'''    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {
        {
            self.state().behaviors.borrow_mut().breadcrumbs.clear();
        }''')
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'breadcrumbs-patch.py'")
 edit(patch,"'semantic-pointer-patch.py'","'breadcrumbs-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if (ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation; no files written')
 if not args.check:
  for path,text in changes.items():(ROOT/path).write_text(text,encoding='utf8')
 print(f'{"Validated" if args.check else "Integrated"} Breadcrumbs across {len(changes)} shared files')
if __name__=='__main__':main()
