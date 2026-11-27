#!/usr/bin/env python3
"""Validate the whole AntiBot shared integration; --apply belongs to the integrator."""
from pathlib import Path
import argparse
import runpy
HERE=Path(__file__).resolve().parent
E="native/liquidbounce-horizon/engine/src/"
Plan=runpy.run_path(str(HERE/"integrate-super-knockback.py"))["Plan"]


def build_plan():
    p=Plan()
    p.replace("native/liquidbounce-behaviors/src/lib.rs","pub mod notifier;","pub mod notifier;\npub mod anti_bot;","behavior export")
    p.replace(E+"lib.rs","mod notifier_runtime;","mod notifier_runtime;\n#[cfg(feature=\"behavior-hooks\")]\nmod anti_bot_runtime;","runtime export")
    b=E+"behaviors.rs"
    p.replace(b,"    pub notifier:","    pub anti_bot:crate::anti_bot_runtime::AntiBotRuntime,\n    pub notifier:","module state")
    p.replace(b,'            "Notifier",','            "Notifier",\n            "AntiBot",',"registry")
    p.replace(b,"            notifier:Default::default(),","            notifier:Default::default(),\n            anti_bot:Default::default(),","state default")
    anchor='            ModuleCallback::CancelSequences(id) if self.query_ids.get("FastUse") == Some(&id)'
    p.replace(b,anchor,'            ModuleCallback::OnDisabled(id) if self.query_ids.get("AntiBot")==Some(&id)=>{self.anti_bot.state.reset();self.notifier.bot_entities.clear();}\n            ModuleCallback::CancelSequences(id) if self.query_ids.get("AntiBot")==Some(&id)=>{self.anti_bot.state.matrix.continuation=None;}\n'+anchor,"disable reset and coroutine cancellation")
    c=E+"config.rs"
    anchor="pub use notifier::{NotifierSetting,serialize_notifier};"
    p.replace(c,anchor,anchor+'\n#[path="config/anti_bot.rs"]\nmod anti_bot;\npub use anti_bot::{AntiBotSetting,serialize_anti_bot};',"configuration export")
    anchor='    Notifier=>("Notifier","Misc","notifier"),'
    p.replace(c,anchor,anchor+'\n    AntiBot=>("AntiBot","Misc","antiBot"),',"module kind")
    p.replace(c,"    Notifier(NotifierSetting),","    Notifier(NotifierSetting),\n    AntiBot(AntiBotSetting),","setting/request enums",2)
    p.replace(c,"            | ModuleKind::Notifier","            | ModuleKind::Notifier\n            | ModuleKind::AntiBot","dedicated serialization")
    anchor="        ModuleKind::Notifier=>notifier::parse_notifier(&mut patch,&root),"
    p.replace(c,anchor,anchor+"\n        ModuleKind::AntiBot=>anti_bot::parse_anti_bot(&mut patch,&root),","parser dispatch")
    anchor="        SettingChange::Notifier(value)=>"
    p.replace(c,anchor,"        SettingChange::AntiBot(value)=>return ApplyResult::Runtime(RuntimeRequest::AntiBot(value.clone())),\n"+anchor,"runtime request dispatch")
    a=E+"actor.rs"
    anchor="            ModuleKind::Notifier=>config::serialize_notifier(&b.notifier.model.settings,&common,format,&describe),"
    p.replace(a,anchor,anchor+"\n            ModuleKind::AntiBot=>config::serialize_anti_bot(&b.anti_bot.state,&common,format,&describe),","actor serialization")
    anchor="                ApplyResult::Runtime(RuntimeRequest::Notifier(setting))=>"
    p.replace(a,anchor,'                ApplyResult::Runtime(RuntimeRequest::AntiBot(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().anti_bot.state){eprintln!("[LiquidBounce] AntiBot configuration: {e}");}}\n'+anchor,"actor apply")
    anchor='            ModuleKind::AutoQueue=>Some('
    p.replace(a,anchor,'            ModuleKind::AntiBot=>Some(liquidbounce_behaviors::anti_bot::MODES[self.state.behaviors.borrow().anti_bot.state.settings.mode]),\n'+anchor,"active mode tag")
    p.replace(E+"coroutine_ticker.rs","    SuperKnockback,","    SuperKnockback,\n    AntiBotMatrix,","shared FIFO owner")
    t=E+"targeting.rs"
    p.replace(t,"    pub teams:","    pub anti_bot:Option<crate::anti_bot_runtime::Query>,\n    pub teams:","target query snapshot")
    p.replace(t,'        teams: crate::teams_runtime::facts(&value["teams"]),','        anti_bot:crate::anti_bot_runtime::query(&value["antiBot"]),\n        teams: crate::teams_runtime::facts(&value["teams"]),',"real target query decoder")
    r=E+"runtime.rs"
    anchor="        self.notifier_tick(host);"
    p.replace(r,anchor,"        self.anti_bot_tick(host);\n"+anchor,"critical custom/Matrix tick before Notifier")
    anchor='    fn client_notifier(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientNotifierEvent<\'_>){self.notifier_event(host,event);}'
    p.replace(r,anchor,anchor+'\n    #[cfg(feature="behavior-hooks")]\n    fn client_anti_bot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAntiBotEvent<\'_>){self.anti_bot_packet(host,event);}',"packet observation hook")
    anchor="                WaitOwner::SuperKnockback => self.super_knockback_callback(host, in_game, task.callback),"
    p.replace(r,anchor,anchor+"\n                WaitOwner::AntiBotMatrix=>self.anti_bot_callback(host,in_game,task.callback),","Matrix continuation dispatch")
    p.section(r,"    pub(crate) fn target_info(","    #[cfg(feature = \"webview\")]\n    pub(crate) fn camera_distance_changed(","            info\n","            if let Some(q)=&snapshot.anti_bot{if modules.anti_bot.state.is_bot(modules.query_running(&self.registry,\"AntiBot\"),&q.player,&q.online,q.max_health){info.ignore();}}\n            info\n","lazy TagEntityEvent ignore")
    p.section(r,"    fn client_attack_entity(","    fn client_attack_speed(","        let auto_buff_owns_slot=","        self.anti_bot_attack(event.target as i32);\n        let auto_buff_owns_slot=","attack history before cancellation")
    p.section(r,"    pub(crate) fn observe_world(","    fn feed_keyboard(","            modules.middle_click_action.world_change();","            modules.middle_click_action.world_change();\n            modules.anti_bot.state.reset();modules.notifier.bot_entities.clear();","world disappearance reset")
    p.section(r,"    fn client_world_change(","    fn client_timer_scale(","        self.state().behaviors.borrow_mut().notifier.model.world_change();","        self.state().behaviors.borrow_mut().notifier.model.world_change();\n        self.anti_bot_reset();","all mode world reset")
    p.section(r,"    fn client_disconnecting(","    fn settings(","            modules.middle_click_action.world_change();","            modules.middle_click_action.world_change();\n            modules.anti_bot.state.reset();modules.notifier.bot_entities.clear();","disconnect reset")
    p.replace("native/liquidbounce-horizon/make-behavior-patch.py","'notifier-patch.py',","'notifier-patch.py', 'anti-bot-patch.py',","native composition after Notifier")
    return p


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group()
    group.add_argument("--check",action="store_true")
    group.add_argument("--apply",action="store_true")
    args=parser.parse_args()
    plan=build_plan()
    print(f"AntiBot: {len(plan.steps)} changes/{len(plan.text)} files validated")
    if args.apply:
        plan.write()
        print("AntiBot integration applied")
    else:
        print("Check only; shared files unchanged")


if __name__=="__main__":
    main()
