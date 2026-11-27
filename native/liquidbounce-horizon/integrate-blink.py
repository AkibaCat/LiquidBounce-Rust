#!/usr/bin/env python3
"""ModuleBlink and the shared Criticals/Freeze/NoSlow BlinkManager integration."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']

def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod fast_exp;','pub mod fast_exp;\npub mod blink;','model')
 p.replace(E+'lib.rs','mod fast_exp_runtime;','mod fast_exp_runtime;\n#[cfg(feature="behavior-hooks")]\nmod blink_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,','    pub fast_exp:crate::fast_exp_runtime::FastExpRuntime,\n    pub blink:std::sync::Arc<crate::blink_runtime::BlinkService>,','field')
 p.replace(b,'            "FastExp",','            "FastExp",\n            "Blink",','registry')
 p.replace(b,'            fast_exp:Default::default(),','            fast_exp:Default::default(),\n            blink,','default')
 anchor='        let criticals=std::sync::Arc::new(crate::criticals_runtime::CriticalsService::new(timer.clone(),combat_cooldown.clone(),freeze.clone()));'
 p.replace(b,anchor,'        let blink=std::sync::Arc::new(crate::blink_runtime::BlinkService::default());\n        let no_slow=std::sync::Arc::new(crate::no_slow_runtime::NoSlowService::default());\n        let criticals=std::sync::Arc::new(crate::criticals_runtime::CriticalsService::new(timer.clone(),combat_cooldown.clone(),freeze.clone(),blink.clone(),no_slow.clone()));','shared manager')
 p.replace(b,'            no_slow:Default::default(),','            no_slow,','shared NoSlow')
 p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.blink.set_running(self.query_running(registry,"Blink"));','running')
 anchor='            ModuleCallback::OnEnabled(id) if self.query_ids.get("VehicleControl")==Some(&id)=>'
 p.replace(b,anchor,'''            ModuleCallback::OnEnabled(id) if self.query_ids.get("Blink")==Some(&id)=>{self.blink.enabled();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BlinkEnabled);}
            ModuleCallback::OnDisabled(id) if self.query_ids.get("Blink")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BlinkDisabled);}
'''+anchor,'lifecycle')
 c=E+'config.rs'
 p.replace(c,'pub use fast_exp::{FastExpSetting,serialize_fast_exp};','pub use fast_exp::{FastExpSetting,serialize_fast_exp};\n#[path="config/blink.rs"]\nmod blink;\npub use blink::{BlinkSetting,serialize_blink};','config')
 p.replace(c,'    FastExp=>("FastExp","Player","fastExp"),','    FastExp=>("FastExp","Player","fastExp"),\n    Blink=>("Blink","Player","blink"),','kind')
 p.replace(c,'    FastExp(FastExpSetting),','    FastExp(FastExpSetting),\n    Blink(BlinkSetting),','requests',2)
 p.replace(c,'            | ModuleKind::FastExp','            | ModuleKind::FastExp\n            | ModuleKind::Blink','dedicated')
 p.replace(c,'        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),','        ModuleKind::FastExp=>fast_exp::parse_fast_exp(&mut patch,&root),\n        ModuleKind::Blink=>blink::parse_blink(&mut patch,&root),','parse')
 anchor='        SettingChange::FastExp(value)=>'
 p.replace(c,anchor,'        SettingChange::Blink(value)=>return ApplyResult::Runtime(RuntimeRequest::Blink(value.clone())),\n'+anchor,'runtime request')
 a=E+'actor.rs'
 anchor='            ModuleKind::FastExp=>config::serialize_fast_exp(&b.fast_exp,&common,format,&describe),'
 p.replace(a,anchor,anchor+'\n            ModuleKind::Blink=>config::serialize_blink(&b.blink,&common,format,&describe),','serialize')
 anchor='                ApplyResult::Runtime(RuntimeRequest::FastExp(setting))=>'
 p.replace(a,anchor,'''                ApplyResult::Runtime(RuntimeRequest::Blink(setting))=>{self.state.behaviors.borrow().blink.update(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] Blink setting: {e}");}});}
'''+anchor,'setting')
 r=E+'runtime.rs'
 p.replace(r,'            pipeline.add_listener(-500,std::sync::Arc::new(crate::no_slow_runtime::NoSlowBlink(modules.no_slow.clone())));','            pipeline.add_listener(-10,std::sync::Arc::new(crate::blink_runtime::AmbushListener(modules.blink.clone(),modules.criticals.clone())));','single shared packet queue')
 p.replace(r,'        self.auto_queue_tick(host);','        self.auto_queue_tick(host);\n        self.blink_tick(host);','GameTick')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_freeze_movement('
 p.replace(r,anchor,'    #[cfg(feature = "behavior-hooks")]\n    fn client_blink_movement(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientBlinkMovementEvent){self.blink_movement(host);}\n'+anchor,'aiStep HEAD')
 p.replace(r,'self.no_slow_frame(host);self.freeze_frame(host);self.criticals_frame(host);','self.blink_pending(host);self.no_slow_frame(host);self.freeze_frame(host);self.criticals_frame(host);','Main ambush disable')
 o=E+'one_shots_runtime.rs'
 p.replace(o,'    VehicleControlEnabled,','    VehicleControlEnabled,\n    BlinkEnabled,\n    BlinkDisabled,','lifecycle variants')
 p.replace(o,'            OneShotEffect::VehicleControlEnabled=>self.vehicle_control_enabled(host),','            OneShotEffect::VehicleControlEnabled=>self.vehicle_control_enabled(host),\n            OneShotEffect::BlinkEnabled=>self.blink_enabled(host),\n            OneShotEffect::BlinkDisabled=>self.blink_disabled(host),','lifecycle effects')
 n=E+'no_slow_runtime.rs'
 anchor='    pub fn settings(&self) -> NoSlow {'
 p.replace(n,anchor,'''    pub(crate) fn blink_active(&self)->bool{
        let raw=bridge::context();let s=self.state.lock().expect("NoSlow");
        Self::blink(&s,self.running.load(Ordering::Acquire),raw)
    }
'''+anchor,'NoSlow shared action')
 cr=E+'criticals_runtime.rs'
 p.replace(cr,'    freeze: Arc<FreezeService>,\n}', '    freeze: Arc<FreezeService>,\n    blink_module:Arc<crate::blink_runtime::BlinkService>,\n    no_slow:Arc<crate::no_slow_runtime::NoSlowService>,\n}','shared action services')
 p.replace(cr,'        freeze: Arc<FreezeService>,\n    ) -> Self {','        freeze: Arc<FreezeService>,\n        blink_module:Arc<crate::blink_runtime::BlinkService>,\n        no_slow:Arc<crate::no_slow_runtime::NoSlowService>,\n    ) -> Self {','constructor')
 p.replace(cr,'            freeze,\n        }','            freeze,\n            blink_module,\n            no_slow,\n        }','service ownership')
 p.replace(cr,'    fn action(&self, s: &mut State, outgoing: bool, pass: bool) -> BlinkAction {','    fn action(&self, s: &mut State, outgoing: bool, pass: bool, moving:Option<bool>) -> BlinkAction {','action signature')
 p.replace(cr,'        if self.freeze.blink_queue(outgoing) {\n            BlinkAction::Queue\n        } else {\n            action\n        }','''        // BlinkPacketEvent.action is monotonic: QUEUE > PASS > FLUSH.
        if self.blink_module.queues(outgoing)||self.freeze.blink_queue(outgoing){return BlinkAction::Queue;}
        if outgoing&&self.no_slow.blink_active(){
            if moving==Some(true){return BlinkAction::Queue;}
            if action==BlinkAction::Flush{return BlinkAction::Pass;}
        }
        action''','merged BlinkPacketEvent')
 p.replace(cr,'self.action(&mut s, outgoing, false)','self.action(&mut s, outgoing, false,None)','probe')
 p.replace(cr,'self.0.action(&mut s, true, pass(event.packet))','self.0.action(&mut s, true, pass(event.packet),Some(movement(event.packet)))','outgoing action')
 p.replace(cr,'self.0.action(&mut s, false, false)','self.0.action(&mut s, false, false,None)','incoming action')
 p.replace(cr,'''            s.ready.extend(ready);
            return;
        }
        if action == BlinkAction::Pass''','''            drop(s);
            let mut deferred=Vec::new();
            for p in ready {if p.session==event.context.session {
                if let Packet::Incoming(packet)=&p.packet {if !event.replay_before(packet.clone()){deferred.push(p);}}
            }}
            self.0.state.lock().expect("BlinkManager").ready.extend(deferred);
            return;
        }
        if action == BlinkAction::Pass''','incoming FIFO before trigger')
 anchor='    pub fn jump(&self, motion: &mut f32) {'
 methods=(HERE/'blink-manager-methods.rs').read_text(encoding='utf8')
 p.replace(cr,anchor,methods+'\n'+anchor,'shared manager operations')
 anchor='    pub(crate) fn criticals_frame(&self, host: &mut dyn Host) {'
 p.replace(cr,anchor,'''    pub(crate) fn criticals_flush_ready(&self,host:&mut dyn Host){
        let service=self.criticals_service();let ready=std::mem::take(&mut service.state.lock().expect("BlinkManager").ready);self.criticals_replay(host,ready);
    }
'''+anchor,'explicit flush')
 fx=E+'fast_exp_runtime.rs'
 p.replace(fx,'        // Wire to BlinkManager and Backtrack packet queues when those services land.\n        let lag = false;','        let lag=m.criticals.is_lagging(); // Backtrack adds its independent incoming queue when integrated.','real lag state')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'blink-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');a=a.parse_args();p=prepare()
 if a.apply:p.write()
 print(f'Blink {"integrated"if a.apply else"validated"}: {len(p.steps)} changes/{len(p.text)} shared files')
