#!/usr/bin/env python3
"""Validate AntiVoid shared integration; --apply is reserved for the serial integration owner."""
from pathlib import Path
import argparse
import runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']

def integration_applied():
    """Return true when the serial integration has already committed all seams."""
    root=HERE.parents[1]
    checks=(
        (root/'native/liquidbounce-behaviors/src/lib.rs','pub mod anti_void;'),
        (root/'native/liquidbounce-horizon/engine/src/lib.rs','mod anti_void_runtime;'),
        (root/'native/liquidbounce-horizon/engine/src/behaviors.rs','pub anti_void:'),
        (root/'native/liquidbounce-horizon/engine/src/config.rs','ModuleKind::AntiVoid'),
        (root/'native/liquidbounce-horizon/engine/src/actor.rs','RuntimeRequest::AntiVoid'),
        (root/'native/liquidbounce-horizon/make-behavior-patch.py',"'anti-void-patch.py'"),
        (root/'native/liquidbounce-horizon/step-patch.py',"'anti-void-late-patch.py'"),
    )
    return all(path.exists() and marker in path.read_text(encoding='utf8') for path,marker in checks)

def build_plan():
    p=Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod anti_bot;','pub mod anti_bot;\npub mod anti_void;','behavior export')
    p.replace(E+'lib.rs','mod anti_bot_runtime;','mod anti_bot_runtime;\n#[cfg(feature="behavior-hooks")]\nmod anti_void_runtime;','runtime export')
    b=E+'behaviors.rs'
    p.replace(b,'    pub anti_bot:','    pub anti_void:std::sync::Arc<crate::anti_void_runtime::AntiVoidService>,\n    pub rescue_guards:std::sync::Arc<crate::anti_void_runtime::RescueGuards>,\n    pub anti_bot:','AntiVoid and shared Clutch guard state')
    p.replace(b,'            "AntiBot",','            "AntiBot",\n            "AntiVoid",','registry')
    p.replace(b,'        let no_slow=std::sync::Arc::new(crate::no_slow_runtime::NoSlowService::default());','        let rescue_guards=std::sync::Arc::new(crate::anti_void_runtime::RescueGuards::default());\n        let anti_void=std::sync::Arc::new(crate::anti_void_runtime::AntiVoidService::new(rescue_guards.clone()));\n        let no_slow=std::sync::Arc::new(crate::no_slow_runtime::NoSlowService::default());','shared guard and queue creation')
    p.replace(b,'freeze.clone(),blink.clone(),no_slow.clone()));','freeze.clone(),blink.clone(),no_slow.clone(),anti_void.clone()));','shared BlinkManager AntiVoid service')
    p.replace(b,'            anti_bot:Default::default(),','            anti_bot:Default::default(),\n            anti_void,\n            rescue_guards,','state initialization')
    a='        self.blink.set_running(self.query_running(registry,"Blink"));'
    p.replace(b,a,a+'\n        self.anti_void.set_context(self.query_running(registry,"AntiVoid"),self.query_running(registry,"Fly"),self.query_running(registry,"Clutch"),self.query_running(registry,"Scaffold"));','live registry queue context')
    a='            ModuleCallback::OnDisabled(id) if self.query_ids.get("AntiBot")'
    p.replace(b,a,'            ModuleCallback::OnEnabled(id) if self.query_ids.get("AntiVoid")==Some(&id)=>{self.anti_void.state.lock().expect("AntiVoid").enabled();}\n'+a,'source enable reset only')
    c=E+'config.rs'
    a='pub use anti_bot::{AntiBotSetting,serialize_anti_bot};'
    p.replace(c,a,a+'\n#[path="config/anti_void.rs"]\nmod anti_void;\npub use anti_void::{AntiVoidSetting,serialize_anti_void};','configuration export')
    a='    AntiBot=>("AntiBot","Misc","antiBot"),'
    p.replace(c,a,a+'\n    AntiVoid=>("AntiVoid","Player","antiVoid"),','module kind')
    p.replace(c,'    AntiBot(AntiBotSetting),','    AntiBot(AntiBotSetting),\n    AntiVoid(AntiVoidSetting),','setting and runtime request',2)
    p.replace(c,'            | ModuleKind::AntiBot','            | ModuleKind::AntiBot\n            | ModuleKind::AntiVoid','dedicated serialization')
    a='        ModuleKind::AntiBot=>anti_bot::parse_anti_bot(&mut patch,&root),'
    p.replace(c,a,a+'\n        ModuleKind::AntiVoid=>anti_void::parse_anti_void(&mut patch,&root),','parser dispatch')
    a='        SettingChange::AntiBot(value)=>'
    p.replace(c,a,'        SettingChange::AntiVoid(value)=>return ApplyResult::Runtime(RuntimeRequest::AntiVoid(value.clone())),\n'+a,'runtime setting dispatch')
    a=E+'actor.rs'
    old='            ModuleKind::AntiBot=>config::serialize_anti_bot(&b.anti_bot.state,&common,format,&describe),'
    p.replace(a,old,old+'\n            ModuleKind::AntiVoid=>config::serialize_anti_void(&b.anti_void.state.lock().expect("AntiVoid"),&common,format,&describe),','actor serialization')
    old='                ApplyResult::Runtime(RuntimeRequest::AntiBot(setting))=>'
    p.replace(a,old,'                ApplyResult::Runtime(RuntimeRequest::AntiVoid(setting))=>{let service=self.state.behaviors.borrow().anti_void.clone();if let Err(e)=setting.apply(&mut service.state.lock().expect("AntiVoid"),&self.state.item_registry){eprintln!("[LiquidBounce] AntiVoid configuration: {e}");}}\n'+old,'actor apply item registry')
    old='            ModuleKind::AntiBot=>Some('
    p.replace(a,old,'            ModuleKind::AntiVoid=>Some(liquidbounce_behaviors::anti_void::MODES[self.state.behaviors.borrow().anti_void.state.lock().expect("AntiVoid").settings.mode]),\n'+old,'active mode tag')
    k=E+'criticals_runtime.rs'
    old='\n    no_slow:Arc<crate::no_slow_runtime::NoSlowService>,'
    p.replace(k,old,old+'\n    anti_void:Arc<crate::anti_void_runtime::AntiVoidService>,','BlinkManager queue owner')
    old='\n        no_slow:Arc<crate::no_slow_runtime::NoSlowService>,'
    p.replace(k,old,old+'\n        anti_void:Arc<crate::anti_void_runtime::AntiVoidService>,','constructor queue owner')
    p.replace(k,'            no_slow,','            no_slow,\n            anti_void,','store shared queue owner')
    old='if self.blink_module.queues(outgoing)||self.freeze.blink_queue(outgoing)'
    p.replace(k,old,'if self.blink_module.queues(outgoing)||self.freeze.blink_queue(outgoing)||self.anti_void.queues(outgoing)','monotonic outgoing queue action')
    old='        self.0.live(event);\n        let mut s = self.0.state.lock().expect("Criticals");'
    p.replace(k,old,'        if let Some(dead)=event.client_player.and_then(|p|p.anti_void_dead()){self.0.anti_void.observe_dead(dead);}\n'+old,'live packet player death before queue')
    r=E+'runtime.rs'
    old='        self.anti_bot_tick(host);'
    p.replace(r,old,'        self.anti_void_tick(host);\n'+old,'GameTick handler')
    old='        self.state().behaviors.borrow_mut().auto_wind_charge.record_input(event);'
    p.replace(r,old,old+'\n        self.state().behaviors.borrow().anti_void.record_input(event);','actual final movement input')
    old='    fn client_anti_bot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAntiBotEvent<\'_>){self.anti_bot_packet(host,event);}'
    p.replace(r,old,old+'\n    #[cfg(feature="behavior-hooks")]\n    fn client_anti_void_shape(&self,event:&mut horizon_client::mod_api::ClientAntiVoidShapeEvent<\'_>){self.anti_void_shape(event.active,event.floor);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_anti_void_network(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAntiVoidNetworkEvent<\'_>){self.anti_void_network(host,event.pre,event.y);}','native global shape and sendPosition events')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'anti-void-patch.py', 'step-patch.py'):",'generic composition before final Step extension')
    p.replace('native/liquidbounce-horizon/step-patch.py'," edit('crates/horizon-client/src/game/remote.rs',remote)\n return {"," edit('crates/horizon-client/src/game/remote.rs',remote)\n # Late sendPosition hooks run after Step has rewritten the final sender.\n runpy.run_path(str(HERE/'anti-void-late-patch.py'))['apply'](edit,once)\n return {",'late sendPosition wiring after final Step sender rewrite')
    return p

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group();group.add_argument('--apply',action='store_true');group.add_argument('--check',action='store_true')
    args=parser.parse_args()
    if integration_applied():
        print('AntiVoid: already applied; shared files unchanged')
        return
    plan=build_plan()
    print(f'AntiVoid: {len(plan.steps)} edits/{len(plan.text)} files validated')
    if args.apply:plan.write();print('AntiVoid integration applied')
    else:print('Check only; shared files unchanged')

if __name__=='__main__':main()
