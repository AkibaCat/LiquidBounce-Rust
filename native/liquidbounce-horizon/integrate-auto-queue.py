#!/usr/bin/env python3
"""Validate AutoQueue shared wiring; integration owner applies exactly once."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('auto_queue_shared',HERE/'integrate-super-knockback.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
E='native/liquidbounce-horizon/engine/src/'
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod report_helper;\n','pub mod report_helper;\npub mod auto_queue;\n','behavior export')
    p.replace(E+'lib.rs','mod report_helper_runtime;\n','mod report_helper_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_queue_runtime;\n','runtime export')
    b=E+'behaviors.rs'
    p.replace(b,'    pub report_helper:liquidbounce_behaviors::report_helper::State,\n','    pub report_helper:liquidbounce_behaviors::report_helper::State,\n    pub auto_queue:liquidbounce_behaviors::auto_queue::State,\n','state')
    p.replace(b,'            "ReportHelper",\n','            "ReportHelper",\n            "AutoQueue",\n','registration')
    p.replace(b,'            report_helper:Default::default(),\n','            report_helper:Default::default(),\n            auto_queue:Default::default(),\n','defaults')
    anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("ReportHelper")==Some(&id)=>{self.report_helper.disabled();}\n'
    p.replace(b,anchor,'''            ModuleCallback::OnEnabled(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.enabled();if self.auto_queue.settings.preset==1{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoQueueEnabled);}}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.disabled();if self.auto_queue.reset_gomme{self.pending_slot_resets.push(crate::auto_queue_runtime::GOMME_REQUESTER);self.auto_queue.reset_gomme=false;}}
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.cancel();}
'''+anchor,'lifecycle')
    c=E+'config.rs'
    anchor='pub use report_helper::{ReportHelperSetting,serialize_report_helper};\n'
    p.replace(c,anchor,anchor+'#[path="config/auto_queue.rs"]\nmod auto_queue;\npub use auto_queue::{AutoQueueSetting,serialize_auto_queue};\n','config export')
    p.replace(c,'    ReportHelper => ("ReportHelper", "Player", "reportHelper"),\n','    ReportHelper => ("ReportHelper", "Player", "reportHelper"),\n    AutoQueue => ("AutoQueue", "Player", "autoQueue"),\n','category')
    p.replace(c,'    ReportHelper(ReportHelperSetting),\n','    ReportHelper(ReportHelperSetting),\n    AutoQueue(AutoQueueSetting),\n','requests',count=2)
    p.replace(c,'            | ModuleKind::ReportHelper\n','            | ModuleKind::ReportHelper\n            | ModuleKind::AutoQueue\n','dedicated tree')
    p.replace(c,'        ModuleKind::ReportHelper=>report_helper::parse_report_helper(&mut patch,&root),\n','        ModuleKind::ReportHelper=>report_helper::parse_report_helper(&mut patch,&root),\n        ModuleKind::AutoQueue=>auto_queue::parse_auto_queue(&mut patch,&root),\n','parse')
    anchor='        SettingChange::ReportHelper(value)=>return ApplyResult::Runtime(RuntimeRequest::ReportHelper(value.clone())),\n'
    p.replace(c,anchor,anchor+'        SettingChange::AutoQueue(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoQueue(value.clone())),\n','runtime request')
    a=E+'actor.rs'
    anchor='            ModuleKind::ReportHelper=>config::serialize_report_helper(&b.report_helper,&common,format,&describe),\n'
    p.replace(a,anchor,anchor+'            ModuleKind::AutoQueue=>config::serialize_auto_queue(&b.auto_queue,&common,format,&describe),\n','serialize')
    p.replace(a,' ModuleKind::ReportHelper=>vec!["AutoReport"],',' ModuleKind::ReportHelper=>vec!["AutoReport"], ModuleKind::AutoQueue=>vec!["AutoPlay"],','alias')
    anchor='            ModuleKind::Hats => Some(liquidbounce_behaviors::hats::MODES[self.state.behaviors.borrow().hats.mode]),\n'
    p.replace(a,anchor,anchor+'            ModuleKind::AutoQueue=>Some(liquidbounce_behaviors::auto_queue::PRESETS[self.state.behaviors.borrow().auto_queue.settings.preset]),\n','preset tag')
    anchor='                ApplyResult::Runtime(RuntimeRequest::ReportHelper(setting))=>{\n'
    p.replace(a,anchor,'''                ApplyResult::Runtime(RuntimeRequest::AutoQueue(setting))=>{
                    let mut b=self.state.behaviors.borrow_mut();let running=b.query_running(&self.state.registry,"AutoQueue");let old=b.auto_queue.settings.preset;
                    if let Err(error)=setting.clone().apply(&mut b.auto_queue,&self.state.item_registry){eprintln!("[LiquidBounce] AutoQueue config leaf skipped: {error}");}
                    if old!=b.auto_queue.settings.preset{if old==1{b.pending_slot_resets.push(crate::auto_queue_runtime::GOMME_REQUESTER);b.auto_queue.reset_gomme=false;}if running&&b.auto_queue.settings.preset==1{b.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoQueueEnabled);}}
                }
'''+anchor,'actor update')
    p.replace(E+'coroutine_ticker.rs','    ReportHelperConfirm,\n','    ReportHelperConfirm,\n    AutoQueue,\n','FIFO owner')
    r=E+'runtime.rs'
    anchor='                WaitOwner::ReportHelperConfirm=>self.report_helper_confirm_callback(host,in_game,task.callback),\n'
    p.replace(r,anchor,anchor+'                WaitOwner::AutoQueue=>self.auto_queue_callback(host,in_game,task.callback),\n','FIFO callback')
    anchor='        if event.source==0{self.report_helper_message(host,event.message);}\n'
    p.replace(r,anchor,anchor+'        if event.source==1||event.source==2{self.auto_queue_message(host,event.source,0,event.message);}\n','title and subtitle')
    anchor='    #[cfg(feature="behavior-hooks")]\n    fn client_report_helper_screen('
    p.replace(r,anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_auto_queue_message(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAutoQueueMessageEvent<'_>){self.auto_queue_message(host,0,event.kind,event.message);}
    #[cfg(feature="behavior-hooks")]
    fn client_auto_queue_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientAutoQueueWakeEvent){self.auto_queue_wake(host);}
'''+anchor,'typed chat and Main wake')
    anchor='        self.middle_click_action_tick(host);\n'
    p.replace(r,anchor,anchor+'        self.auto_queue_tick(host);\n','GameTick')
    anchor='    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {\n'
    p.replace(r,anchor,anchor+'        self.auto_queue_world_change();\n','real WorldChange')
    anchor='            modules.combat_cooldown.disconnect();\n'
    p.replace(r,anchor,anchor+'            modules.auto_queue.cancel();\n','disconnect')
    o=E+'one_shots_runtime.rs'
    p.replace(o,'    VehicleControlEnabled,\n','    VehicleControlEnabled,\n    AutoQueueEnabled,\n','immediate enable effect')
    anchor='            OneShotEffect::VehicleControlEnabled=>self.vehicle_control_enabled(host),\n'
    p.replace(o,anchor,anchor+'            OneShotEffect::AutoQueueEnabled=>self.auto_queue_enabled_info(host),\n','enable messages')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'report-helper-patch.py',","'report-helper-patch.py', 'auto-queue-patch.py',",'native extension')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);g=parser.add_mutually_exclusive_group();g.add_argument('--apply',action='store_true');g.add_argument('--check',action='store_true');args=parser.parse_args();p=build_plan();print(f'Validated {len(p.steps)} AutoQueue changes across {len(p.text)} shared files')
    if args.apply:p.write();print('Applied AutoQueue integration')
    else:print('Check only; no shared writes')
if __name__=='__main__':main()
