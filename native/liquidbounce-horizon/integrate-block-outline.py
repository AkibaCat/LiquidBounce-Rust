#!/usr/bin/env python3
"""One-time BlockOutline wiring. Validate all replacements before any write."""
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
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod block_outline;');absent(E+'behaviors.rs','    pub block_outline:');absent(E+'config.rs','    BlockOutline =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod block_outline;')
 edit(E+'lib.rs','mod auto_break_runtime;','mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod block_outline_runtime;')
 edit(E+'behaviors.rs','    pub auto_leave:','    pub block_outline: liquidbounce_behaviors::block_outline::BlockOutline,\n    pub auto_leave:')
 edit(E+'behaviors.rs','            "AutoLeave",','            "AutoLeave",\n            "BlockOutline",')
 edit(E+'behaviors.rs','            auto_leave: Default::default(),','            block_outline: Default::default(),\n            auto_leave: Default::default(),')
 edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/block_outline.rs"]\nmod block_outline;\npub use block_outline::{BlockOutlineSetting,serialize_block_outline};\n#[path = "config/auto_weapon.rs"]')
 edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    BlockOutline => ("BlockOutline", "Render", "blockOutline"),')
 edit(E+'config.rs','"MultiTask" => Some(Self::MultiActions),','"BlockOverlay" => Some(Self::BlockOutline), "MultiTask" => Some(Self::MultiActions),')
 edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    BlockOutline(BlockOutlineSetting),',2)
 edit(E+'config.rs','            | ModuleKind::AutoLeave','            | ModuleKind::AutoLeave\n            | ModuleKind::BlockOutline')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::BlockOutline => block_outline::parse_block_outline(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::BlockOutline(value) => { return ApplyResult::Runtime(RuntimeRequest::BlockOutline(*value)); }\n        SettingChange::Damage(value) => {')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::BlockOutline => config::serialize_block_outline(&b.block_outline,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::BlockOutline(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().block_outline); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'''    #[cfg(feature = "behavior-hooks")]
    fn client_block_outline(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlockOutlineEvent<'_>){
        self.block_outline(host,event);
    }
'''+anchor)
 edit(E+'runtime.rs',"    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {",'''    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {
        {
            let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0,|v|v.as_millis() as i64);
            self.state().behaviors.borrow_mut().block_outline.world_change(now);
        }''')
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'block-outline-patch.py'")
 edit(patch,"'semantic-pointer-patch.py'","'block-outline-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if (ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation; no files written')
 if not args.check:
  for path,text in changes.items():(ROOT/path).write_text(text,encoding='utf8')
 print(f'{"Validated" if args.check else "Integrated"} BlockOutline across {len(changes)} shared files')
if __name__=='__main__':main()
