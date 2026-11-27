#!/usr/bin/env python3
"""Check TriggerBot/AutoClicker shared integration in memory; owner uses --apply."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('crosshair_shared',HERE/'integrate-super-knockback.py')
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
ENGINE='native/liquidbounce-horizon/engine/src/'
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod clicking;\n','pub mod clicking;\npub mod crosshair_clicker;\n','behavior export')
    p.replace(ENGINE+'lib.rs','mod spear_kill_runtime;\n','mod spear_kill_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod crosshair_clicker_runtime;\n','runtime export')
    b=ENGINE+'behaviors.rs'
    anchor='    pub spear_kill: liquidbounce_behaviors::spear_kill::SpearKill,\n'
    p.replace(b,anchor,anchor+'    pub crosshair_clicker: liquidbounce_behaviors::crosshair_clicker::State,\n    pub crosshair_break_observer: std::sync::Arc<crate::crosshair_clicker_runtime::BreakObserver>,\n','fields')
    p.replace(b,'            "SpearKill",\n','            "SpearKill",\n            "AutoClicker",\n            "TriggerBot",\n','registration')
    p.replace(b,'            spear_kill: Default::default(),\n','            spear_kill: Default::default(),\n            crosshair_clicker: Default::default(),\n            crosshair_break_observer: Default::default(),\n','state defaults')
    p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.crosshair_break_observer.set_running(self.query_running(registry,"AutoClicker"));\n','packet lifecycle')
    c=ENGINE+'config.rs'
    anchor='pub use spear_kill::{SpearKillSetting, serialize_spear_kill};\n'
    p.replace(c,anchor,anchor+'#[path="config/crosshair_clicker.rs"]\nmod crosshair_clicker;\npub use crosshair_clicker::{CrosshairClickerSetting,serialize_trigger_bot,serialize_auto_clicker};\n','config export')
    anchor='    SpearKill => ("SpearKill", "Combat", "spearKill"),\n'
    p.replace(c,anchor,anchor+'    TriggerBot => ("TriggerBot", "Combat", "triggerBot"),\n    AutoClicker => ("AutoClicker", "Combat", "autoClicker"),\n','module kinds')
    p.replace(c,'    SpearKill(SpearKillSetting),\n','    SpearKill(SpearKillSetting),\n    CrosshairClicker(CrosshairClickerSetting),\n','request variants',count=2)
    p.replace(c,'            | ModuleKind::SpearKill\n','            | ModuleKind::SpearKill\n            | ModuleKind::TriggerBot\n            | ModuleKind::AutoClicker\n','dedicated serializer')
    anchor='        ModuleKind::SpearKill => spear_kill::parse_spear_kill(&mut patch, &root),\n'
    p.replace(c,anchor,anchor+'        ModuleKind::TriggerBot => crosshair_clicker::parse_trigger_bot(&mut patch,&root),\n        ModuleKind::AutoClicker => crosshair_clicker::parse_auto_clicker(&mut patch,&root),\n','config parse')
    anchor='        SettingChange::SpearKill(value) => {\n'
    p.replace(c,anchor,'        SettingChange::CrosshairClicker(value) => {\n            return ApplyResult::Runtime(RuntimeRequest::CrosshairClicker(value.clone()));\n        }\n'+anchor,'runtime dispatch')
    a=ENGINE+'actor.rs'
    anchor='            ModuleKind::SpearKill => config::serialize_spear_kill(&b.spear_kill, &common, format, &describe),\n'
    p.replace(a,anchor,anchor+'            ModuleKind::TriggerBot => config::serialize_trigger_bot(&b.crosshair_clicker,&common,format,&describe),\n            ModuleKind::AutoClicker => config::serialize_auto_clicker(&b.crosshair_clicker,&common,format,&describe),\n','actor trees')
    anchor='                ApplyResult::Runtime(RuntimeRequest::SpearKill(setting)) => {\n'
    p.replace(a,anchor,'                ApplyResult::Runtime(RuntimeRequest::CrosshairClicker(setting)) => {\n                    if let Err(error)=setting.apply(&mut self.state.behaviors.borrow_mut().crosshair_clicker,&self.state.item_registry,&self.state.block_registry){eprintln!("[LiquidBounce] config leaf skipped: {error}");}\n                }\n'+anchor,'actor mutation')
    anchor='        let mut browser = browser.lock().unwrap_or_else(|e| e.into_inner());\n        let language_generation = horizon_client::lang::generation();\n'
    p.replace(ENGINE+'browser.rs',anchor,'        self.crosshair_initialize(host);\n'+anchor,'menu registry defaults')
    r=ENGINE+'runtime.rs'
    anchor='            pipeline.add_listener(0, modules.combat_cooldown.clone());\n'
    p.replace(r,anchor,anchor+'            pipeline.add_listener(0, modules.crosshair_break_observer.clone());\n','packet listener')
    anchor='                WaitOwner::SuperKnockback => self.super_knockback_callback(host, in_game, task.callback),\n'
    p.replace(r,anchor,anchor+'                WaitOwner::TriggerBot => self.crosshair_callback(host,in_game,liquidbounce_behaviors::crosshair_clicker::Id::Trigger,task.callback),\n                WaitOwner::AutoClickerAttack => self.crosshair_callback(host,in_game,liquidbounce_behaviors::crosshair_clicker::Id::Attack,task.callback),\n                WaitOwner::AutoClickerUse => self.crosshair_callback(host,in_game,liquidbounce_behaviors::crosshair_clicker::Id::Use,task.callback),\n','coroutine dispatch')
    anchor='        self.state().behaviors.borrow().timer.first_game_tick();\n'
    p.replace(r,anchor,anchor+'        self.crosshair_first_tick(host); // FIRST_PRIORITY GameTick\n','plan tick')
    p.replace(r,'        self.smart_eat_tick(host);\n','        self.smart_eat_tick(host);\n        self.crosshair_tick(host);\n','module ticks')
    anchor='        let auto_walk_sprint = self.auto_walk_wants_sprint(host);\n'
    p.replace(r,anchor,anchor+'        let crosshair_stop_sprint=self.crosshair_stop_sprint(host);\n','read-only sprint query',count=2)
    # The two crosshair SprintEvent handlers follow Sprint, and precede
    # SuperKnockback in the source combat registration order.
    anchor='            let knockback_running = modules.query_running(&state.registry, "SuperKnockback");\n'
    p.replace(r,anchor,'            if crosshair_stop_sprint {event.movement_override=0;}\n'+anchor,'sprint override')
    anchor='        if modules.super_knockback.stop_sprint(knockback_running) {\n            event.sprint = false;\n'
    p.replace(r,anchor,'        if crosshair_stop_sprint {event.sprint=false;}\n'+anchor,'input sprint override')
    anchor='        self.feed_keyboard(host, code, action, event.modifiers);\n'
    p.replace(r,anchor,'        #[cfg(feature="behavior-hooks")]\n        if let Some(key)=horizon_client::keybinds::key_name(code){self.crosshair_player_press(host,key,action==KeyAction::Press);}\n'+anchor,'physical keyboard presses')
    anchor='''    fn mouse_button_pre(
        &self,
        _host: &mut dyn Host,
        event: &mut horizon_client::mod_api::MouseButtonPreEvent,
    ) -> horizon_client::mod_api::Flow {
'''
    p.replace(r,anchor,anchor.replace('_host:','host:')+'''        self.crosshair_mouse(event.button,event.pressed);
        if let Some(button)=input::mouse_button(event.button) && let Some((_,key))=horizon_client::key_names::MOUSE.iter().find(|(b,_)|*b==button){self.crosshair_player_press(host,key,event.pressed);}
''','physical mouse presses')
    anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_use_key_pressed(\n'
    p.replace(r,anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_crosshair_clicker_attack_key(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientCrosshairClickerAttackKeyEvent){self.crosshair_attack_key(&mut event.pressed);}

'''+anchor,'simulated attack isDown')
    p.replace(ENGINE+'coroutine_ticker.rs','    SuperKnockback,\n','    SuperKnockback,\n    TriggerBot,\n    AutoClickerAttack,\n    AutoClickerUse,\n','wait owners')
    anchor='    pub fn progress(&self, rate: f32, delay: f32) -> f32 {\n'
    p.replace(ENGINE+'super_knockback_runtime.rs',anchor,'''    pub fn snapshot(&self, rate:f32, attack_delay:f32, has_cooldown:bool)->liquidbounce_behaviors::clicking::Cooldown {
        let real_ticks=self.now();let mut state=self.state.lock().expect("CombatCooldown poisoned");state.drain(real_ticks);
        liquidbounce_behaviors::clicking::Cooldown {real_ticks,reset_ticks:state.reset_ticks,server_tick_speed:if state.tps.is_nan(){1.0}else{state.tps/f64::from(rate)},attack_delay,has_cooldown}
    }
'''+anchor,'reuse cooldown observer')
    # Crosshair falling depends on edge, reverse-step and terrain bridges. Step
    # remains last because it rewrites every added MotionWorld literal.
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'crosshair-clicker-patch.py', 'step-patch.py'):",'native extension')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);mode=parser.add_mutually_exclusive_group();mode.add_argument('--apply',action='store_true');mode.add_argument('--check',action='store_true');args=parser.parse_args()
    p=build_plan();print(f'Validated {len(p.steps)} crosshair clicker changes across {len(p.text)} shared files')
    if args.apply:p.write();print('Applied TriggerBot / AutoClicker integration')
    else:print('Check only; no shared file writes')
if __name__=='__main__':main()
