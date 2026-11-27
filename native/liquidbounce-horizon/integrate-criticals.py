#!/usr/bin/env python3
"""Check Criticals and shared Freeze/Blink integration; owner applies once."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('criticals_shared',HERE/'integrate-super-knockback.py')
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
ENGINE='native/liquidbounce-horizon/engine/src/'
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod crosshair_clicker;\n','pub mod crosshair_clicker;\npub mod criticals;\n','behavior export')
    p.replace(ENGINE+'lib.rs','mod crosshair_clicker_runtime;\n','mod crosshair_clicker_runtime;\n#[cfg(feature="behavior-hooks")]\nmod criticals_runtime;\n','runtime export')
    b=ENGINE+'behaviors.rs'
    anchor='    pub crosshair_break_observer: std::sync::Arc<crate::crosshair_clicker_runtime::BreakObserver>,\n'
    p.replace(b,anchor,anchor+'    pub criticals:std::sync::Arc<crate::criticals_runtime::CriticalsService>,\n','service field')
    p.replace(b,'            "TriggerBot",\n','            "TriggerBot",\n            "Criticals",\n','module registration')
    anchor='        let freeze = std::sync::Arc::new(crate::freeze_runtime::FreezeService::new(timer.clone()));\n'
    p.replace(b,anchor,anchor+'        let combat_cooldown=std::sync::Arc::new(crate::super_knockback_runtime::CombatCooldown::new(timer.clone()));\n        let criticals=std::sync::Arc::new(crate::criticals_runtime::CriticalsService::new(timer.clone(),combat_cooldown.clone(),freeze.clone()));\n','shared services')
    p.replace(b,'            combat_cooldown: std::sync::Arc::new(crate::super_knockback_runtime::CombatCooldown::new(timer)),\n','            combat_cooldown,\n            criticals,\n','service defaults')
    p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.criticals.set_running(self.query_running(registry,"Criticals"));\n','running transitions')
    c=ENGINE+'config.rs'
    anchor='pub use crosshair_clicker::{CrosshairClickerSetting,serialize_trigger_bot,serialize_auto_clicker};\n'
    p.replace(c,anchor,anchor+'#[path="config/criticals.rs"]\nmod criticals;\npub use criticals::{CriticalsSetting,serialize_criticals};\n','config export')
    anchor='    TriggerBot => ("TriggerBot", "Combat", "triggerBot"),\n'
    p.replace(c,anchor,anchor+'    Criticals => ("Criticals", "Combat", "criticals"),\n','module kind')
    p.replace(c,'    CrosshairClicker(CrosshairClickerSetting),\n','    CrosshairClicker(CrosshairClickerSetting),\n    Criticals(CriticalsSetting),\n','request variants',count=2)
    p.replace(c,'            | ModuleKind::TriggerBot\n','            | ModuleKind::TriggerBot\n            | ModuleKind::Criticals\n','dedicated tree')
    anchor='        ModuleKind::TriggerBot => crosshair_clicker::parse_trigger_bot(&mut patch,&root),\n'
    p.replace(c,anchor,anchor+'        ModuleKind::Criticals=>criticals::parse_criticals(&mut patch,&root),\n','parse dispatch')
    anchor='        SettingChange::CrosshairClicker(value) => {\n'
    p.replace(c,anchor,'        SettingChange::Criticals(value)=>return ApplyResult::Runtime(RuntimeRequest::Criticals(value.clone())),\n'+anchor,'runtime settings request')
    a=ENGINE+'actor.rs'
    anchor='            ModuleKind::TriggerBot => config::serialize_trigger_bot(&b.crosshair_clicker,&common,format,&describe),\n'
    p.replace(a,anchor,anchor+'            ModuleKind::Criticals=>config::serialize_criticals(&b.criticals.settings(),&common,format,&describe),\n','actor tree')
    anchor='                ApplyResult::Runtime(RuntimeRequest::CrosshairClicker(setting)) => {\n'
    p.replace(a,anchor,'''                ApplyResult::Runtime(RuntimeRequest::Criticals(setting))=>{
                    self.state.behaviors.borrow().criticals.update(|s|{if let Err(error)=setting.apply(s){eprintln!("[LiquidBounce] Criticals config leaf skipped: {error}");}});
                }
'''+anchor,'actor setting update')
    r=ENGINE+'runtime.rs'
    anchor='            pipeline.add_listener(0, modules.crosshair_break_observer.clone());\n'
    p.replace(r,anchor,anchor+'''            pipeline.add_listener(0,std::sync::Arc::new(crate::criticals_runtime::NoGroundListener(modules.criticals.clone())));
            pipeline.add_listener(-500,std::sync::Arc::new(crate::criticals_runtime::BlinkListener(modules.criticals.clone())));
            pipeline.add_listener(-1000,std::sync::Arc::new(crate::criticals_runtime::RewoundListener(modules.criticals.clone())));
''','packet priorities')
    p.replace(r,'            pipeline.add_listener(-900, modules.freeze.clone());\n','            pipeline.add_listener(0, modules.freeze.clone());\n','Freeze ordinary packet priority')
    anchor='            modules.combat_cooldown.world_change();\n'
    p.replace(r,anchor,anchor+'            modules.criticals.disconnect();\n','world disconnect')
    p.replace(r,'        self.super_knockback_attack(host, event.target);\n','        self.criticals_attack_sprint(host);\n','high priority attack sprint')
    p.replace(r,'        self.mace_kill_attack(host);\n','        self.criticals_attack(host,event.target,event.living);\n        self.super_knockback_attack(host,event.target);\n        self.mace_kill_attack(host);\n','ordinary attack registration order')
    p.replace(r,'        self.high_jump_jump(&mut event.motion);\n','        self.state().behaviors.borrow().criticals.jump(&mut event.motion);\n        self.high_jump_jump(&mut event.motion);\n','jump motion')
    p.replace(r,'        self.sneak_input(host,event);\n','        self.criticals_input(host,&mut event.jump);\n        self.sneak_input(host,event);\n','jump movement input')
    p.replace(r,'        self.middle_click_action_tick(host);\n','        self.criticals_tick(host);\n        self.middle_click_action_tick(host);\n','mode tick')
    p.section(r,'    fn client_freeze_frame(','    fn client_freeze_flag(','self.freeze_frame(host);}','self.freeze_frame(host);self.criticals_frame(host);}','frame probe')
    anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_freeze_frame('
    p.replace(r,anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_criticals_receive_probe(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientCriticalsReceiveProbeEvent){self.criticals_receive(host);}\n'+anchor,'receive probe')
    p.replace(r,'        let crosshair_stop_sprint=self.crosshair_stop_sprint(host);\n','        let crosshair_stop_sprint=self.crosshair_stop_sprint(host);\n        let criticals_stop_sprint=self.criticals_stop_sprint(host,0);\n','sprint input reads',count=2)
    # The MOVEMENT_TICK callback has a separate query source.
    p.section(r,'    fn client_sprint_queries(','    fn client_sprint_network(','self.criticals_stop_sprint(host,0)','self.criticals_stop_sprint(host,1)','movement sprint source')
    p.replace(r,'        if crosshair_stop_sprint {event.sprint=false;}\n','        if criticals_stop_sprint {event.sprint=false;}\n        if crosshair_stop_sprint {event.sprint=false;}\n','input sprint handler')
    p.replace(r,'            if crosshair_stop_sprint {event.movement_override=0;}\n','            if criticals_stop_sprint {event.movement_override=0;}\n            if crosshair_stop_sprint {event.movement_override=0;}\n','movement sprint handler')
    p.section(r,'    fn client_sprint_network(','    fn client_interaction_range(', '        use liquidbounce_behaviors::sprint::Source;\n','        use liquidbounce_behaviors::sprint::Source;\n        let criticals_stop_sprint=self.criticals_stop_sprint(host,2);\n','network sprint read')
    p.section(r,'    fn client_sprint_network(','    fn client_interaction_range(', '        #[cfg(feature = "webview")]\n        let anti_hunger_input', '        if criticals_stop_sprint{event.sprinting=false;}\n        #[cfg(feature = "webview")]\n        let anti_hunger_input','network sprint handler')
    click=ENGINE+'crosshair_clicker_runtime.rs'
    anchor='        let (cooldown, v) = self.crosshair_cooldown(host)?;\n'
    p.replace(click,anchor,anchor+'        let (jump_height,adjust_next_jump,blink_in_state)=self.state().behaviors.borrow().criticals.selection();\n','shared selection state')
    p.replace(click,'            jumping: v["jumping"].as_bool()?,\n','            jumping: v["jumping"].as_bool()?||adjust_next_jump,\n','automatic jump prediction')
    p.replace(click,'            jump_height: 0.42,\n            blink_in_state: false,\n','            jump_height,\n            blink_in_state,\n','Criticals selection parameters')
    freeze=ENGINE+'freeze_runtime.rs'
    anchor='    pub fn clear(&self) {\n'
    p.replace(freeze,anchor,'''    pub(crate) fn blink_queue(&self,outgoing:bool)->bool{
        let s=self.state.lock().expect("Freeze");self.running.load(Ordering::Acquire)&&s.settings.mode==Mode::Queue&&s.settings.queue_origins&(if outgoing{2}else{1})!=0
    }
    pub(crate) fn blink_own_hurt(&self,packet:&C)->bool{
        let C::Sound(p)=packet else{return false;};let s=self.state.lock().expect("Freeze");p.sound.1.as_ref().is_some_and(|v|v.0=="minecraft:entity.player.hurt")||p.sound.1.is_none()&&p.sound.0==s.hurt_sound
    }
'''+anchor,'Freeze Blink action adapter')
    text=p.read(freeze)
    start=text.index('            Mode::Queue if settings.queue_origins & 1 != 0 && !event.is_cancelled() => {')
    end=text.index('            _ => {}\n        }\n    }\n    fn outgoing',start)
    p.replace(freeze,text[start:end],'','shared incoming queue')
    text=p.read(freeze)
    start=text.index('            Mode::Queue if settings.queue_origins & 2 != 0 && !event.is_cancelled() => {')
    end=text.index('            Mode::Stationary => {',start)
    p.replace(freeze,text[start:end],'','shared outgoing queue')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'criticals-patch.py', 'step-patch.py'):",'native extension')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);m=parser.add_mutually_exclusive_group();m.add_argument('--apply',action='store_true');m.add_argument('--check',action='store_true');args=parser.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Criticals changes across {len(p.text)} shared files');
    if args.apply:p.write();print('Applied Criticals integration')
    else:print('Check only; no shared file writes')
if __name__=='__main__':main()
