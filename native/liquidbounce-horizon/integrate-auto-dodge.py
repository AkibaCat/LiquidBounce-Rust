"""AutoDodge's actual movement event and shared Blink projectile evasion."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod blink;','pub mod blink;\npub mod auto_dodge;','model')
 p.replace(E+'lib.rs','mod blink_runtime;','mod blink_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_dodge_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub blink:std::sync::Arc<crate::blink_runtime::BlinkService>,','    pub blink:std::sync::Arc<crate::blink_runtime::BlinkService>,\n    pub auto_dodge:crate::auto_dodge_runtime::AutoDodgeRuntime,','field')
 p.replace(b,'            "Blink",','            "Blink",\n            "AutoDodge",','registry')
 p.replace(b,'            blink,','            blink,\n            auto_dodge:Default::default(),','default')
 p.replace(b,'        self.blink.set_running(self.query_running(registry,"Blink"));','        self.blink.set_running(self.query_running(registry,"Blink"));\n        self.timer.set_provider_running(crate::auto_dodge_runtime::TIMER_PROVIDER,self.query_running(registry,"AutoDodge")&&!self.query_running(registry,"Blink")&&!self.auto_dodge.murder_mystery_disallows);','Timer running')
 c=E+'config.rs'
 p.replace(c,'pub use blink::{BlinkSetting,serialize_blink};','pub use blink::{BlinkSetting,serialize_blink};\n#[path="config/auto_dodge.rs"]\nmod auto_dodge;\npub use auto_dodge::{AutoDodgeSetting,serialize_auto_dodge};','config')
 p.replace(c,'    Blink=>("Blink","Player","blink"),','    Blink=>("Blink","Player","blink"),\n    AutoDodge=>("AutoDodge","Combat","autoDodge"),','kind')
 p.replace(c,'    Blink(BlinkSetting),','    Blink(BlinkSetting),\n    AutoDodge(AutoDodgeSetting),','setting request',2)
 p.replace(c,'            | ModuleKind::Blink','            | ModuleKind::Blink\n            | ModuleKind::AutoDodge','dedicated')
 p.replace(c,'        ModuleKind::Blink=>blink::parse_blink(&mut patch,&root),','        ModuleKind::Blink=>blink::parse_blink(&mut patch,&root),\n        ModuleKind::AutoDodge=>auto_dodge::parse_auto_dodge(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::Blink(value)=>','        SettingChange::AutoDodge(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoDodge(value.clone())),\n        SettingChange::Blink(value)=>','dispatch')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::Blink=>config::serialize_blink(&b.blink,&common,format,&describe),','            ModuleKind::Blink=>config::serialize_blink(&b.blink,&common,format,&describe),\n            ModuleKind::AutoDodge=>config::serialize_auto_dodge(&b.auto_dodge.settings,&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::Blink(setting))=>','                ApplyResult::Runtime(RuntimeRequest::AutoDodge(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_dodge.settings){eprintln!("[LiquidBounce] AutoDodge setting: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::Blink(setting))=>','apply')
 r=E+'runtime.rs'
 p.replace(r,'        self.criticals_input(host,&mut event.jump);','        self.criticals_input(host,&mut event.jump);\n        let auto_dodge_jump=self.auto_dodge_input(host,event);','MovementInput')
 p.replace(r,'        self.vehicle_control_input(host,event);','        self.vehicle_control_input(host,event);\n        if auto_dodge_jump{event.jump=true;}','once next MovementInput')
 bl=E+'blink_runtime.rs'
 old='''        let predictor = service.avoidance.lock().expect("Blink avoidance").clone();
        let Some(predictor) = predictor else {
            return;
        };
        if !predictor.threatened(first) {
            return;
        }
        if let Some((index, impact)) = predictor.avoiding_position(&positions) {'''
 new='''        let query=serde_json::json!({"mode":"blink","positions":positions});
        let Ok(result)=serde_json::from_str::<serde_json::Value>(&host.client_auto_dodge_query(&query.to_string()))else{return;};
        if result["threatened"]!=true{return;}
        if let Some(index)=result["index"].as_i64(){
            let index=index as i32;let impact=result["impact"]==true;'''
 p.replace(bl,old,new,'real synchronous Blink arrow forecast')
 p.replace(bl,'        let Some(first) = positions.first().copied() else {','        let Some(_) = positions.first().copied() else {','empty queue')
 p.replace(bl,'        let (service, manager, active) = {','        let (manager, active) = {','native predictor ownership')
 p.section(bl,'    pub(crate) fn blink_tick','    pub(crate) fn blink_enabled','                b.blink.clone(),\n                b.criticals.clone(),\n                b.query_running','                b.criticals.clone(),\n                b.query_running','manager ownership')
 # The prediction now runs synchronously within the real native Game scope.
 p.replace(bl,'    avoidance: Mutex<Option<Arc<dyn BlinkAvoidance>>>,\n','', 'remove unused predictor storage')
 start='/// AutoDodge installs its real projectile predictor here when its module lands.\n'
 text=p.read(bl);a=text.index(start);z=text.index('    pub fn settings(&self)',a)
 old=text[a:z]
 p.replace(bl,old,'impl BlinkService {\n','remove deferred trait')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'auto-dodge-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--apply',action='store_true');a=parser.parse_args();p=prepare()
 if a.apply:p.write()
 print(f'AutoDodge {"integrated"if a.apply else"validated"}: {len(p.steps)} changes/{len(p.text)} shared files')
