#!/usr/bin/env python3
"""Validate every AutoRod shared anchor, then publish engine wiring transactionally."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_bow_math;','pub mod auto_bow_math;\npub mod auto_rod;\npub mod point_tracker;')
 p.edit('lib.rs','mod auto_bow_runtime;','mod auto_bow_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_rod_runtime;')
 p.edit('behaviors.rs','    pub auto_bow:crate::auto_bow_runtime::AutoBowRuntime,','    pub auto_bow:crate::auto_bow_runtime::AutoBowRuntime,\n    pub auto_rod:crate::auto_rod_runtime::AutoRodRuntime,')
 p.edit('behaviors.rs','            "AutoBuff",','            "AutoBuff",\n            "AutoRod",')
 p.edit('behaviors.rs','            auto_bow:Default::default(),','            auto_bow:Default::default(),\n            auto_rod:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoRod")==Some(&id)=>{let release=self.auto_rod.clear();if release{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoRodDisabled(true));}self.pending_slot_resets.push(crate::auto_rod_runtime::REQUESTER);},\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoBow")==Some(&id)=>self.auto_bow.clear(),')
 p.edit('behaviors.rs','            crate::auto_bow_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBow"),','            crate::auto_bow_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoBow"),\n            crate::auto_rod_runtime::ROTATION_PROVIDER=>self.query_running(registry,"AutoRod"),')
 p.edit('config.rs','pub use auto_bow::{AutoBowSetting,serialize_auto_bow};','pub use auto_bow::{AutoBowSetting,serialize_auto_bow};\n#[path="config/auto_rod.rs"]\nmod auto_rod;\npub use auto_rod::{AutoRodSetting,serialize_auto_rod};')
 p.edit('config.rs','    AutoBow=>("AutoBow","Combat","autoBow"),','    AutoBow=>("AutoBow","Combat","autoBow"),\n    AutoRod=>("AutoRod","Combat","autoRod"),')
 p.edit('config.rs','    AutoBow(AutoBowSetting),','    AutoBow(AutoBowSetting),\n    AutoRod(AutoRodSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoBow','            | ModuleKind::AutoBow\n            | ModuleKind::AutoRod')
 p.edit('config.rs','        ModuleKind::AutoBow=>auto_bow::parse_auto_bow(&mut patch,&root),','        ModuleKind::AutoBow=>auto_bow::parse_auto_bow(&mut patch,&root),\n        ModuleKind::AutoRod=>auto_rod::parse_auto_rod(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoBow(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBow(value.clone())),','        SettingChange::AutoBow(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoBow(value.clone())),\n        SettingChange::AutoRod(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoRod(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoBow=>config::serialize_auto_bow(&b.auto_bow,&common,format,&describe),','            ModuleKind::AutoBow=>config::serialize_auto_bow(&b.auto_bow,&common,format,&describe),\n            ModuleKind::AutoRod=>config::serialize_auto_rod(&b.auto_rod,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::AutoRod(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_rod,&self.state.item_registry){eprintln!("[LiquidBounce] AutoRod config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoBow(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.auto_bow.clear();','        modules.auto_bow.clear();\n        modules.auto_rod.clear();')
 p.edit('one_shots_runtime.rs','    LogoffSpotDisabled,','    LogoffSpotDisabled,\n    AutoRodDisabled(bool),')
 p.edit('one_shots_runtime.rs','            OneShotEffect::LogoffSpotDisabled=>self.logoff_spot_disabled(host),','            OneShotEffect::LogoffSpotDisabled=>self.logoff_spot_disabled(host),\n            OneShotEffect::AutoRodDisabled(release)=>self.auto_rod_disabled(host,release),')
 p.edit('coroutine_ticker.rs','    AutoBuff,','    AutoBuff,\n    AutoRod,')
 p.edit('runtime.rs','                WaitOwner::AutoBuff=>self.auto_buff_callback(host,in_game,task.callback),','                WaitOwner::AutoBuff=>self.auto_buff_callback(host,in_game,task.callback),\n                WaitOwner::AutoRod=>self.auto_rod_callback(host,in_game,task.callback),')
 p.edit('runtime.rs','        self.auto_buff_first_tick();','        self.auto_buff_first_tick();\n        self.auto_rod_first_tick(host);')
 p.edit('runtime.rs','        self.projectile_puncher_prepare(host);','        self.projectile_puncher_prepare(host);\n        self.auto_rod_prepare(host);')
 p.edit('runtime.rs','        self.auto_buff_tick(host);','        self.auto_buff_tick(host);\n        self.auto_rod_tick(host);')
 p.edit('runtime.rs','            let auto_bow_running=modules.query_running(&state.registry,"AutoBow");','            let auto_bow_running=modules.query_running(&state.registry,"AutoBow");\n            let auto_rod_running=modules.query_running(&state.registry,"AutoRod");')
 p.edit('runtime.rs','                        crate::auto_bow_runtime::ROTATION_PROVIDER=>auto_bow_running,','                        crate::auto_bow_runtime::ROTATION_PROVIDER=>auto_bow_running,\n                        crate::auto_rod_runtime::ROTATION_PROVIDER=>auto_rod_running,')
 p.edit('runtime.rs','                        } else if let Some(mode)=crate::auto_bow_runtime::AutoBowRuntime::processor_mode(processor){','                        } else if let Some(mode)=crate::auto_rod_runtime::AutoRodRuntime::processor_mode(processor){\n                            (&mut modules.auto_rod.rotations,mode)\n                        } else if let Some(mode)=crate::auto_bow_runtime::AutoBowRuntime::processor_mode(processor){')
 p.edit('runtime.rs','        self.crosshair_mouse(event.button,event.pressed);','        self.crosshair_mouse(event.button,event.pressed);\n        if let Some(button)=input::mouse_button(event.button){self.state().behaviors.borrow_mut().auto_rod.input.mouse_event(button,event.pressed,behaviors::current_time_millis());}')
 p.edit('runtime.rs','            .free_look\n            .mouse_rotation(running, event.cursor_x, event.cursor_y);','            .free_look\n            .mouse_rotation(running, event.cursor_x, event.cursor_y);\n        modules.auto_rod.input.moved(event.cursor_x,event.cursor_y,behaviors::current_time_millis());')
 p.edit('target_renderer_runtime.rs','        // TARGET_RENDERER_OWNERS_END','        owner!("AutoRod", auto_rod, true);\n        // TARGET_RENDERER_OWNERS_END')
 p.edit('../../make-behavior-patch.py',"'auto-bow-patch.py',","'auto-bow-patch.py', 'auto-rod-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('AutoRod '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
