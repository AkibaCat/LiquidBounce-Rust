"""Transactionally publish the complete AutoShoot implementation for a grouped batch."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_rod;','pub mod auto_rod;\npub mod auto_shoot;')
 p.edit('lib.rs','mod auto_rod_runtime;','mod auto_rod_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_shoot_runtime;')
 p.edit('behaviors.rs','    pub auto_rod:crate::auto_rod_runtime::AutoRodRuntime,','    pub auto_rod:crate::auto_rod_runtime::AutoRodRuntime,\n    pub auto_shoot:crate::auto_shoot_runtime::AutoShootRuntime,')
 p.edit('behaviors.rs','            "AutoRod",','            "AutoRod",\n            "AutoShoot",')
 p.edit('behaviors.rs','            auto_rod:Default::default(),','            auto_rod:Default::default(),\n            auto_shoot:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),','            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoShoot")==Some(&id)=>{self.auto_shoot.behavior.disabled();self.pending_slot_resets.push(crate::auto_shoot_runtime::REQUESTER);},\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),')
 p.edit('behaviors.rs','            crate::auto_rod_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoRod"),','            crate::auto_rod_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoRod"),\n            crate::auto_shoot_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoShoot"),')
 p.edit('config.rs','pub use auto_rod::{AutoRodSetting,serialize_auto_rod};','pub use auto_rod::{AutoRodSetting,serialize_auto_rod};\n#[path="config/auto_shoot.rs"]\nmod auto_shoot;\npub use auto_shoot::{AutoShootSetting,serialize_auto_shoot};')
 p.edit('config.rs','    AutoRod=>("AutoRod","Combat","autoRod"),','    AutoRod=>("AutoRod","Combat","autoRod"),\n    AutoShoot=>("AutoShoot","Combat","autoShoot"),')
 p.edit('config.rs','    AutoRod(AutoRodSetting),','    AutoRod(AutoRodSetting),\n    AutoShoot(AutoShootSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoRod','            | ModuleKind::AutoRod\n            | ModuleKind::AutoShoot')
 p.edit('config.rs','        ModuleKind::AutoRod=>auto_rod::parse_auto_rod(&mut patch,&root),','        ModuleKind::AutoRod=>auto_rod::parse_auto_rod(&mut patch,&root),\n        ModuleKind::AutoShoot=>auto_shoot::parse_auto_shoot(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoRod(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoRod(value.clone())),','        SettingChange::AutoRod(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoRod(value.clone())),\n        SettingChange::AutoShoot(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoShoot(value.clone())),')
 p.edit('config/crosshair_clicker.rs','fn click_tree(','pub(super) fn click_tree(')
 p.edit('actor.rs','            ModuleKind::AutoRod=>config::serialize_auto_rod(&b.auto_rod,&common,format,&describe),','            ModuleKind::AutoRod=>config::serialize_auto_rod(&b.auto_rod,&common,format,&describe),\n            ModuleKind::AutoShoot=>config::serialize_auto_shoot(&b.auto_shoot,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::AutoShoot(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_shoot,&self.state.item_registry){eprintln!("[LiquidBounce] AutoShoot config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{')
 p.edit('actor.rs','            ModuleKind::AntiVoid=>Some(','            ModuleKind::AutoShoot=>Some(["Auto","Linear","Projectile"][usize::from(self.state.behaviors.borrow().auto_shoot.behavior.settings.gravity.min(2))]),\n            ModuleKind::AntiVoid=>Some(')
 p.edit('runtime.rs','        self.auto_rod_first_tick(host);','        self.auto_rod_first_tick(host);\n        self.auto_shoot_first_tick();')
 p.edit('runtime.rs','        self.auto_rod_prepare(host);','        self.auto_rod_prepare(host);\n        self.auto_shoot_prepare(host);')
 p.edit('runtime.rs','        self.auto_rod_tick(host);','        self.auto_rod_tick(host);\n        self.auto_shoot_tick(host);')
 p.edit('runtime.rs','            let auto_rod_running=modules.query_running(&state.registry,"AutoRod");','            let auto_rod_running=modules.query_running(&state.registry,"AutoRod");\n            let auto_shoot_running=modules.query_running(&state.registry,"AutoShoot");')
 p.edit('runtime.rs','                        crate::auto_rod_runtime::ROTATION_PROVIDER=>auto_rod_running,','                        crate::auto_rod_runtime::ROTATION_PROVIDER=>auto_rod_running,\n                        crate::auto_shoot_runtime::ROTATION_PROVIDER=>auto_shoot_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_rod_runtime::AutoRodRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::auto_shoot_runtime::AutoShootRuntime::processor_mode(processor){\n                            (&mut modules.auto_shoot.rotations,mode)\n                        } else if let Some(mode)=crate::auto_rod_runtime::AutoRodRuntime::processor_mode(processor){')
 p.edit('crosshair_clicker_runtime.rs','self.hooks.projectile_puncher_other_click(id);','self.hooks.projectile_puncher_other_click(id);self.hooks.auto_shoot_other_click(id);')
 p.edit('crosshair_clicker_runtime.rs','        self.projectile_puncher_player_press(attack,open,pressed);','        self.projectile_puncher_player_press(attack,open,pressed);\n        self.auto_shoot_player_press(use_key,open,pressed);')
 p.edit('target_renderer_runtime.rs','        // TARGET_RENDERER_OWNERS_END','        owner!("AutoShoot", auto_shoot, true);\n        // TARGET_RENDERER_OWNERS_END')
 p.edit('../../make-behavior-patch.py',"'auto-rod-patch.py',","'auto-rod-patch.py', 'auto-shoot-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoShoot '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
