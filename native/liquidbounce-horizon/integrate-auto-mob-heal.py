#!/usr/bin/env python3
"""Idempotently wire the complete AutoMobHeal after validating every seam."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
BasePlan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def integration_applied():
 root=HERE.parents[1]
 checks=(
  (root/'native/liquidbounce-behaviors/src/lib.rs','pub mod auto_mob_heal;'),
  (root/'native/liquidbounce-horizon/engine/src/behaviors.rs','pub auto_mob_heal:'),
  (root/'native/liquidbounce-horizon/engine/src/config.rs','ModuleKind::AutoMobHeal'),
  (root/'native/liquidbounce-horizon/engine/src/actor.rs','RuntimeRequest::AutoMobHeal'),
  (root/'native/liquidbounce-horizon/engine/src/runtime.rs','self.auto_mob_heal_tick(host);'),
  (root/'native/liquidbounce-horizon/make-behavior-patch.py',"'auto-mob-heal-patch.py'"),
 )
 return all(path.exists() and marker in path.read_text(encoding='utf8') for path,marker in checks)
class Plan(BasePlan):
 def edit(self,name,old,new,count=1):
  path=HERE/'engine/src'/name
  source=self.changed.get(path)
  if source is None: source=path.read_text(encoding='utf8')
  if new in source:return
  super().edit(name,old,new,count)
def prepare():
 if integration_applied():
  return Plan()
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_armor;','pub mod auto_armor;\npub mod auto_mob_heal;')
 p.edit('lib.rs','mod auto_armor_runtime;','mod auto_armor_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_mob_heal_runtime;')
 p.edit('behaviors.rs','    pub auto_armor:crate::auto_armor_runtime::AutoArmorRuntime,','    pub auto_armor:crate::auto_armor_runtime::AutoArmorRuntime,\n    pub auto_mob_heal:crate::auto_mob_heal_runtime::AutoMobHealRuntime,')
 p.edit('behaviors.rs','            "AutoArmor",','            "AutoArmor",\n            "AutoMobHeal",')
 p.edit('behaviors.rs','            auto_armor:Default::default(),','            auto_armor:Default::default(),\n            auto_mob_heal:Default::default(),')
 p.edit('behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoMobHeal")==Some(&id)=>{self.auto_mob_heal.clear();self.pending_slot_resets.push(crate::auto_mob_heal_runtime::REQUESTER);},
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoMobHeal")==Some(&id)=>self.auto_mob_heal.clear(),''')
 p.edit('coroutine_ticker.rs','    AutoArmor,','    AutoArmor,\n    AutoMobHeal,')
 p.edit('config.rs','pub use auto_armor::{AutoArmorSetting,serialize_auto_armor};','pub use auto_armor::{AutoArmorSetting,serialize_auto_armor};\n#[path="config/auto_mob_heal.rs"]\nmod auto_mob_heal;\npub use auto_mob_heal::{AutoMobHealSetting,serialize_auto_mob_heal};')
 p.edit('config.rs','    AutoArmor => ("AutoArmor","Combat","autoArmor"),','    AutoArmor => ("AutoArmor","Combat","autoArmor"),\n    AutoMobHeal => ("AutoMobHeal","World","autoMobHeal"),')
 p.edit('config.rs','"InventoryManager"=>Some(Self::InventoryCleaner),','"InventoryManager"=>Some(Self::InventoryCleaner), "AutoGolemRepair"=>Some(Self::AutoMobHeal),')
 p.edit('config.rs','    AutoArmor(AutoArmorSetting),','    AutoArmor(AutoArmorSetting),\n    AutoMobHeal(AutoMobHealSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoArmor','            | ModuleKind::AutoArmor\n            | ModuleKind::AutoMobHeal')
 p.edit('config.rs','        ModuleKind::AutoArmor => auto_armor::parse_auto_armor(&mut patch,&root),','        ModuleKind::AutoArmor => auto_armor::parse_auto_armor(&mut patch,&root),\n        ModuleKind::AutoMobHeal => auto_mob_heal::parse_auto_mob_heal(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoArmor(value)=>{return ApplyResult::Runtime(RuntimeRequest::AutoArmor(value.clone()));}','        SettingChange::AutoArmor(value)=>{return ApplyResult::Runtime(RuntimeRequest::AutoArmor(value.clone()));}\n        SettingChange::AutoMobHeal(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoMobHeal(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoArmor => config::serialize_auto_armor(&b.auto_armor.settings,&common,format,&describe),','            ModuleKind::AutoArmor => config::serialize_auto_armor(&b.auto_armor.settings,&common,format,&describe),\n            ModuleKind::AutoMobHeal => config::serialize_auto_mob_heal(&b.auto_mob_heal.settings,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoArmor(setting))=>{','''                ApplyResult::Runtime(RuntimeRequest::AutoMobHeal(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_mob_heal.settings){eprintln!("[LiquidBounce] AutoMobHeal config: {e}");}}
                ApplyResult::Runtime(RuntimeRequest::AutoArmor(setting))=>{''')
 p.edit('actor.rs','ModuleKind::Aimbot=>vec!["AimAssist","AutoAim"],','ModuleKind::Aimbot=>vec!["AimAssist","AutoAim"], ModuleKind::AutoMobHeal=>vec!["AutoGolemRepair"],')
 p.edit('runtime.rs','                WaitOwner::AutoArmor=>self.auto_armor_callback(host,in_game,task.callback),','                WaitOwner::AutoArmor=>self.auto_armor_callback(host,in_game,task.callback),\n                WaitOwner::AutoMobHeal=>self.auto_mob_heal_callback(in_game,task.callback),')
 p.edit('runtime.rs','        self.auto_rod_tick(host);','        self.auto_rod_tick(host);\n        self.auto_mob_heal_tick(host);')
 p.edit('runtime.rs','        self.auto_queue_world_change();','        self.auto_queue_world_change();\n        self.state().behaviors.borrow_mut().auto_mob_heal.clear();')
 p.edit('../../make-behavior-patch.py',"'auto-armor-patch.py',","'auto-armor-patch.py', 'auto-mob-heal-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoMobHeal '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
