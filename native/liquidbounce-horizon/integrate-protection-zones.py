"""Transactional ProtectionZones shared wiring; validate every anchor before any write."""
from pathlib import Path
import argparse
ROOT=Path(__file__).resolve().parents[2]; E='native/liquidbounce-horizon/engine/src/'
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
 absent('native/liquidbounce-behaviors/src/lib.rs','pub mod protection_zones;');absent(E+'behaviors.rs','    pub protection_zones:');absent(E+'config.rs','    ProtectionZones =>')
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod breadcrumbs;','pub mod breadcrumbs;\npub mod protection_zones;')
 edit(E+'lib.rs','mod breadcrumbs_runtime;','mod breadcrumbs_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod protection_zones_runtime;')
 edit(E+'behaviors.rs','    pub breadcrumbs:','    pub protection_zones: liquidbounce_behaviors::protection_zones::ProtectionZones,\n    pub breadcrumbs:')
 edit(E+'behaviors.rs','            "Breadcrumbs",','            "Breadcrumbs",\n            "ProtectionZones",')
 edit(E+'behaviors.rs','            breadcrumbs: Default::default(),','            protection_zones: Default::default(),\n            breadcrumbs: Default::default(),')
 edit(E+'config.rs','#[path = "config/breadcrumbs.rs"]','#[path = "config/protection_zones.rs"]\nmod protection_zones;\npub use protection_zones::{ProtectionZonesSetting,serialize_protection_zones};\n#[path = "config/breadcrumbs.rs"]')
 edit(E+'config.rs','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),','    Breadcrumbs => ("Breadcrumbs", "Render", "breadcrumbs"),\n    ProtectionZones => ("ProtectionZones", "Render", "protectionZones"),')
 edit(E+'config.rs','    Breadcrumbs(BreadcrumbsSetting),','    Breadcrumbs(BreadcrumbsSetting),\n    ProtectionZones(ProtectionZonesSetting),',2)
 edit(E+'config.rs','            | ModuleKind::Breadcrumbs','            | ModuleKind::Breadcrumbs\n            | ModuleKind::ProtectionZones')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::ProtectionZones => protection_zones::parse_protection_zones(&mut patch,&root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::ProtectionZones(value) => { return ApplyResult::Runtime(RuntimeRequest::ProtectionZones(value.clone())); }\n        SettingChange::Damage(value) => {')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::ProtectionZones => config::serialize_protection_zones(&b.protection_zones,&common,format,&describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::ProtectionZones(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().protection_zones); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 edit(E+'runtime.rs',anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_protection_zones(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientProtectionZonesEvent<\'_>){self.protection_zones(host,event);}\n'+anchor)
 patch='native/liquidbounce-horizon/make-behavior-patch.py';absent(patch,"'protection-zones-patch.py'")
 edit(patch,"'semantic-pointer-patch.py'","'protection-zones-patch.py', 'semantic-pointer-patch.py'")
 for path,original in originals.items():
  if (ROOT/path).read_text(encoding='utf8')!=original:raise ValueError(f'{path}: changed during validation, nothing written')
 if not a.check:
  for path,t in changes.items():(ROOT/path).write_text(t,encoding='utf8')
 print(f'{"Validated" if a.check else "Integrated"} ProtectionZones across {len(changes)} shared files')
if __name__=='__main__':main()
