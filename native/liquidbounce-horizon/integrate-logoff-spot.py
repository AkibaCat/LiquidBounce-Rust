"""Transactional LogoffSpot wiring; integration owner applies serially."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace(E+'lib.rs','mod crosshair_runtime;','mod crosshair_runtime;\n#[cfg(feature="behavior-hooks")]\nmod logoff_spot_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub crosshair:','    pub logoff_spot:std::sync::Arc<crate::logoff_spot_runtime::LogoffSpotService>,\n    pub crosshair:','state')
 p.replace(E+'behaviors.rs','            crosshair: Default::default(),','            logoff_spot: Default::default(),\n            crosshair: Default::default(),','default')
 p.replace(E+'behaviors.rs','            "Crosshair",','            "Crosshair",\n            "LogoffSpot",','registration')
 p.replace(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.logoff_spot.set_running(self.query_running(registry,"LogoffSpot"));','packet running')
 anchor='            ModuleCallback::OnDisabled(id) if self.query_ids.get("Blink")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BlinkDisabled);}'
 p.replace(E+'behaviors.rs',anchor,anchor+'\n            ModuleCallback::OnDisabled(id) if self.query_ids.get("LogoffSpot")==Some(&id)=>{self.logoff_spot.clear();self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::LogoffSpotDisabled);}','disable clear')
 p.replace(E+'one_shots_runtime.rs','    BlinkDisabled,','    BlinkDisabled,\n    LogoffSpotDisabled,','disable effect')
 p.replace(E+'one_shots_runtime.rs','            OneShotEffect::BlinkDisabled=>self.blink_disabled(host),','            OneShotEffect::BlinkDisabled=>self.blink_disabled(host),\n            OneShotEffect::LogoffSpotDisabled=>self.logoff_spot_disabled(host),','disable host cleanup')
 p.replace(E+'config.rs','#[path = "config/crosshair.rs"]','#[path = "config/logoff_spot.rs"]\nmod logoff_spot;\npub use logoff_spot::{LogoffSpotSetting,serialize_logoff_spot};\n#[path = "config/crosshair.rs"]','config export')
 p.replace(E+'config.rs','    Crosshair => ("Crosshair", "Render", "crosshair"),','    Crosshair => ("Crosshair", "Render", "crosshair"),\n    LogoffSpot => ("LogoffSpot", "Render", "logoffSpot"),','kind')
 p.replace(E+'config.rs','    Crosshair(CrosshairSetting),','    Crosshair(CrosshairSetting),\n    LogoffSpot(LogoffSpotSetting),','request and setting',2)
 p.replace(E+'config.rs','            | ModuleKind::Crosshair','            | ModuleKind::Crosshair\n            | ModuleKind::LogoffSpot','supported config')
 p.replace(E+'config.rs','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),','        ModuleKind::Crosshair => crosshair::parse_crosshair(&mut patch,&root),\n        ModuleKind::LogoffSpot => logoff_spot::parse_logoff_spot(&mut patch,&root),','parser')
 p.replace(E+'config.rs','        SettingChange::Crosshair(value) => {','        SettingChange::LogoffSpot(value) => { return ApplyResult::Runtime(RuntimeRequest::LogoffSpot(*value)); }\n        SettingChange::Crosshair(value) => {','apply')
 p.replace(E+'actor.rs','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),','            ModuleKind::Crosshair => config::serialize_crosshair(&b.crosshair,&common,format,&describe),\n            ModuleKind::LogoffSpot => config::serialize_logoff_spot(b.logoff_spot.send_in_chat(),&common,format,&describe),','serializer')
 p.replace(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::LogoffSpot(setting)) => { self.state.behaviors.borrow().logoff_spot.set_send_in_chat(setting.0); }\n                ApplyResult::Runtime(RuntimeRequest::Crosshair(setting)) => {','setter')
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.replace(E+'runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_logoff_spot(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientLogoffSpotEvent<\'_>){self.logoff_spot(host,event);}\n'+anchor,'hook')
 anchor="    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<'_>) {"
 p.replace(E+'runtime.rs',anchor,anchor+'\n        self.state().behaviors.borrow().logoff_spot.clear();','world clear')
 p.replace(E+'runtime.rs','            pipeline.add_listener(0, modules.flag_check.clone());','            pipeline.add_listener(0, modules.flag_check.clone());\n            pipeline.add_listener(0,modules.logoff_spot.clone());','packet listener')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'blink-patch.py'","'blink-patch.py', 'logoff-spot-patch.py'",'native patch')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} LogoffSpot changes across {len(p.text)} shared files')
 if args.apply:p.write();print('Integrated LogoffSpot')
