#!/usr/bin/env python3
"""Transactional NewChunks wiring. All anchors and current contents validated before any write."""
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
  if text.count(old)!=count:raise ValueError(f'{path}: expected {count} of {old[:80]!r}, got {text.count(old)}')
  changes[path]=text.replace(old,new)
 def absent(path,token):
  if token in read(path):raise ValueError(f'{path}: already contains {token}')
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod new_chunks;');absent(E+'behaviors.rs','    pub new_chunks:');absent(E+'config.rs','    NewChunks =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod new_chunks;')
 edit(E+'lib.rs','mod breadcrumbs_runtime;','mod breadcrumbs_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod new_chunks_runtime;')
 edit(E+'behaviors.rs','    pub breadcrumbs:','    pub new_chunks: liquidbounce_behaviors::new_chunks::NewChunks,\n    pub breadcrumbs:')
 edit(E+'behaviors.rs','            "Breadcrumbs",','            "Breadcrumbs",\n            "NewChunks",')
 edit(E+'behaviors.rs','            breadcrumbs: Default::default(),','            new_chunks: Default::default(),\n            breadcrumbs: Default::default(),')
 edit(E+'config.rs','#[path = "config/breadcrumbs.rs"]','#[path = "config/new_chunks.rs"]\nmod new_chunks;\npub use new_chunks::{NewChunksSetting,serialize_new_chunks};\n#[path = "config/breadcrumbs.rs"]')
 edit(E+'config.rs','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),\n    NewChunks => ("NewChunks", "Render", "newChunks"),')
 edit(E+'config.rs','    Breadcrumbs(BreadcrumbsSetting),','    Breadcrumbs(BreadcrumbsSetting),\n    NewChunks(NewChunksSetting),',2)
 edit(E+'config.rs','            | ModuleKind::Breadcrumbs','            | ModuleKind::Breadcrumbs\n            | ModuleKind::NewChunks')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::NewChunks => new_chunks::parse_new_chunks(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::NewChunks(value) => { return ApplyResult::Runtime(RuntimeRequest::NewChunks(*value)); }\n        SettingChange::Damage(value) => {')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::NewChunks => config::serialize_new_chunks(&b.new_chunks,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::NewChunks(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().new_chunks); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
 edit(E+'behaviors.rs','            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {','            ModuleCallback::OnDisabled(id) if self.query_ids.get("NewChunks") == Some(&id) => { self.new_chunks.clear(); }\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("CombineMobs") == Some(&id) => {')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_new_chunks(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientNewChunksEvent<\'_>){self.new_chunks(host,event);}\n'+anchor)
 edit(E+'runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {","    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {\n        self.state().behaviors.borrow_mut().new_chunks.clear();")
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'new-chunks-patch.py'")
 edit(patch,"'semantic-pointer-patch.py'","'new-chunks-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if (ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
 if not a.check:
  for path,text in changes.items():(ROOT/path).write_text(text,encoding='utf8')
 print(f'{"Validated" if a.check else "Integrated"} NewChunks across {len(changes)} shared files')
if __name__=='__main__':main()
