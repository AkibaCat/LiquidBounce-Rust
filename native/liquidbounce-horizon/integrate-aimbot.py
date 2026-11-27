"""Aimbot transactional engine wiring. --check never publishes shared edits."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod auto_rod;','pub mod auto_rod;\npub mod aimbot;\npub mod aimbot_ray;')
 math='../../../liquidbounce-behaviors/src/auto_bow_math.rs'
 def transform(old,new):p.edit(math,old,new)
 transform('pub fn projected_points(eye: V, b: B) -> Vec<V> {','pub fn projected_points(eye: V, b: B) -> Vec<V> { projected_points_limit(eye,b,128) }\npub fn projected_points_limit(eye: V, b: B, maximum: i32) -> Vec<V> {')
 transform('(true, false) => (1, 128),','(true, false) => (1, maximum),')
 transform('(false, true) => (128, 1),','(false, true) => (maximum, 1),')
 transform('((128. / (dz.len() / dy.len())).sqrt().floor() as i32).clamp(1, 128)','((f64::from(maximum) / (dz.len() / dy.len())).sqrt().floor() as i32).clamp(1, maximum)')
 transform('(n, (128 / n).max(1))','(n, (maximum / n).max(1))')
 p.edit('lib.rs','mod auto_rod_runtime;','mod auto_rod_runtime;\n#[cfg(feature="behavior-hooks")]\nmod aimbot_runtime;')
 p.edit('behaviors.rs','    pub auto_rod:crate::auto_rod_runtime::AutoRodRuntime,','    pub auto_rod:crate::auto_rod_runtime::AutoRodRuntime,\n    pub aimbot:crate::aimbot_runtime::AimbotRuntime,')
 p.edit('behaviors.rs','            "AutoRod",','            "AutoRod",\n            "Aimbot",')
 p.edit('behaviors.rs','            auto_rod:Default::default(),','            auto_rod:Default::default(),\n            aimbot:Default::default(),')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoRod")==Some(&id)=>{','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Aimbot")==Some(&id)=>self.aimbot.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoRod")==Some(&id)=>{')
 p.edit('config.rs','pub use auto_rod::{AutoRodSetting,serialize_auto_rod};','pub use auto_rod::{AutoRodSetting,serialize_auto_rod};\n#[path="config/aimbot.rs"]\nmod aimbot;\npub use aimbot::{AimbotSetting,serialize_aimbot};')
 p.edit('config.rs','    AutoRod=>("AutoRod","Combat","autoRod"),','    AutoRod=>("AutoRod","Combat","autoRod"),\n    Aimbot=>("Aimbot","Combat","aimbot"),')
 p.edit('config.rs','match name { "FastEat"','match name { "AimAssist"|"AutoAim"=>Some(Self::Aimbot), "FastEat"')
 p.edit('config.rs','    AutoRod(AutoRodSetting),','    AutoRod(AutoRodSetting),\n    Aimbot(AimbotSetting),',2)
 p.edit('config.rs','            | ModuleKind::AutoRod','            | ModuleKind::AutoRod\n            | ModuleKind::Aimbot')
 p.edit('config.rs','        ModuleKind::AutoRod=>auto_rod::parse_auto_rod(&mut patch,&root),','        ModuleKind::AutoRod=>auto_rod::parse_auto_rod(&mut patch,&root),\n        ModuleKind::Aimbot=>aimbot::parse_aimbot(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::AutoRod(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoRod(value.clone())),','        SettingChange::AutoRod(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoRod(value.clone())),\n        SettingChange::Aimbot(value)=>return ApplyResult::Runtime(RuntimeRequest::Aimbot(value.clone())),')
 p.edit('actor.rs','            ModuleKind::AutoRod=>config::serialize_auto_rod(&b.auto_rod,&common,format,&describe),','            ModuleKind::AutoRod=>config::serialize_auto_rod(&b.auto_rod,&common,format,&describe),\n            ModuleKind::Aimbot=>config::serialize_aimbot(&b.aimbot,&common,format,&describe),')
 p.edit('actor.rs','ModuleKind::FlagCheck=>vec!["FlagDetect"],','ModuleKind::Aimbot=>vec!["AimAssist","AutoAim"], ModuleKind::FlagCheck=>vec!["FlagDetect"],')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoRod(setting))=>{','                ApplyResult::Runtime(RuntimeRequest::Aimbot(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().aimbot){eprintln!("[LiquidBounce] Aimbot config leaf skipped: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoRod(setting))=>{')
 p.edit('inventory_runtime.rs','        modules.auto_rod.clear();','        modules.auto_rod.clear();\n        modules.aimbot.clear();')
 p.edit('timer_runtime.rs','    pub fn scale(&self, delta: f32) -> f32 {','    pub fn speed(&self)->f32 {\n        self.render_actions();\n        let providers=self.providers();\n        self.state.lock().expect("Timer poisoned").timer.speed(true,&mut |id|providers.get(&id).copied().unwrap_or(false))\n    }\n    pub fn scale(&self, delta: f32) -> f32 {')
 p.edit('runtime.rs','        self.auto_rod_prepare(host);','        self.aimbot_prepare(host);\n        self.auto_rod_prepare(host);')
 p.edit('runtime.rs','        modules.auto_rod.input.moved(event.cursor_x,event.cursor_y,behaviors::current_time_millis());','        modules.auto_rod.input.moved(event.cursor_x,event.cursor_y,behaviors::current_time_millis());\n        if modules.query_running(&state.registry,"Aimbot"){modules.aimbot.behavior.mouse(event.cursor_x,event.cursor_y);}')
 p.edit('runtime.rs','    fn client_target_renderer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTargetRendererEvent<\'_>){self.target_renderer(host,event);}','    fn client_target_renderer(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientTargetRendererEvent<\'_>){self.target_renderer(host,event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_aimbot_render(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientAimbotRenderEvent){self.aimbot_render(host,event);}')
 p.edit('target_renderer_runtime.rs','        // TARGET_RENDERER_OWNERS_END','        owner!("Aimbot", aimbot, true);\n        // TARGET_RENDERER_OWNERS_END')
 p.edit('../../make-behavior-patch.py',"'auto-rod-patch.py',","'auto-rod-patch.py', 'aimbot-patch.py',")
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('Aimbot '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
