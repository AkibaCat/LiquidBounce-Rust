"""Atomic owned AirPlace registry/config/actual useItem/preview wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent; E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_dodge;','pub mod auto_dodge;\npub mod air_place;','model')
 p.replace(E+'lib.rs','mod auto_dodge_runtime;','mod auto_dodge_runtime;\n#[cfg(feature="behavior-hooks")]\nmod air_place_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub auto_dodge:crate::auto_dodge_runtime::AutoDodgeRuntime,','    pub auto_dodge:crate::auto_dodge_runtime::AutoDodgeRuntime,\n    pub air_place:liquidbounce_behaviors::air_place::Settings,','field')
 p.replace(b,'            "AutoDodge",','            "AutoDodge",\n            "AirPlace",','registry')
 p.replace(b,'            auto_dodge:Default::default(),','            auto_dodge:Default::default(),\n            air_place:Default::default(),','default')
 c=E+'config.rs'
 p.replace(c,'pub use auto_dodge::{AutoDodgeSetting,serialize_auto_dodge};','pub use auto_dodge::{AutoDodgeSetting,serialize_auto_dodge};\n#[path="config/air_place.rs"]\nmod air_place;\npub use air_place::{AirPlaceSetting,serialize_air_place};','config')
 p.replace(c,'    AutoDodge=>("AutoDodge","Combat","autoDodge"),','    AutoDodge=>("AutoDodge","Combat","autoDodge"),\n    AirPlace=>("AirPlace","World","airPlace"),','kind')
 p.replace(c,'    AutoDodge(AutoDodgeSetting),','    AutoDodge(AutoDodgeSetting),\n    AirPlace(AirPlaceSetting),','setting request',2)
 p.replace(c,'            | ModuleKind::AutoDodge','            | ModuleKind::AutoDodge\n            | ModuleKind::AirPlace','dedicated')
 p.replace(c,'        ModuleKind::AutoDodge=>auto_dodge::parse_auto_dodge(&mut patch,&root),','        ModuleKind::AutoDodge=>auto_dodge::parse_auto_dodge(&mut patch,&root),\n        ModuleKind::AirPlace=>air_place::parse_air_place(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::AutoDodge(value)=>','        SettingChange::AirPlace(value)=>return ApplyResult::Runtime(RuntimeRequest::AirPlace(value.clone())),\n        SettingChange::AutoDodge(value)=>','dispatch')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::AutoDodge=>config::serialize_auto_dodge(&b.auto_dodge.settings,&common,format,&describe),','            ModuleKind::AutoDodge=>config::serialize_auto_dodge(&b.auto_dodge.settings,&common,format,&describe),\n            ModuleKind::AirPlace=>config::serialize_air_place(&b.air_place,&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::AutoDodge(setting))=>','                ApplyResult::Runtime(RuntimeRequest::AirPlace(setting))=>{if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().air_place){eprintln!("[LiquidBounce] AirPlace setting: {e}");}}\n                ApplyResult::Runtime(RuntimeRequest::AutoDodge(setting))=>','apply')
 r=E+'runtime.rs'
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_mouse_scroll_raw('
 p.replace(r,anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_air_place_configuration(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientAirPlaceConfigurationEvent<'_>){self.air_place_configuration(event.configuration);}
'''+anchor,'native settings')
 p.section(r,'    fn client_mouse_scroll_raw(','    fn client_dank_bobbing(','        let modifier_scan = self.state().behaviors.borrow().camera_clip.modifier.value;','        self.air_place_scroll(host,event.vertical);\n        let modifier_scan = self.state().behaviors.borrow().camera_clip.modifier.value;','raw scroll')
 p.section(r,'    fn mouse_scrolling(','    fn client_player_tick_post(','        let modifier_scan = self.state().behaviors.borrow().camera_clip.modifier.value;','        let air_place_cancel=self.air_place_cancel_scroll(host);\n        let modifier_scan = self.state().behaviors.borrow().camera_clip.modifier.value;','hotbar scroll')
 p.replace(r,'        if cancel || clip_cancel {','        if cancel || clip_cancel || air_place_cancel {','cancel')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'air-place-patch.py', 'step-patch.py'):",'native')
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--apply',action='store_true');a=parser.parse_args();p=prepare()
 if a.apply:p.write()
 print(f'AirPlace {"integrated" if a.apply else "validated"}: {len(p.steps)} changes/{len(p.text)} shared files')
