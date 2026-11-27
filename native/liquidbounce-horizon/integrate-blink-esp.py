"""Transactional global BlinkManager ESP wiring, no module registry entry."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod crosshair;','pub mod crosshair;\npub mod blink_esp;','behavior export')
 p.replace(E+'lib.rs','mod crosshair_runtime;','mod crosshair_runtime;\n#[cfg(feature="behavior-hooks")]\nmod blink_esp_runtime;','runtime')
 p.replace(E+'behaviors.rs','    pub crosshair:','    pub blink_esp: liquidbounce_behaviors::blink_esp::BlinkEsp,\n    pub crosshair:','global settings')
 p.replace(E+'behaviors.rs','            crosshair: Default::default(),','            blink_esp: Default::default(),\n            crosshair: Default::default(),','default')
 p.replace(E+'config.rs','#[path = "config/inventory_manager.rs"]','#[path = "config/blink_esp.rs"]\nmod blink_esp;\npub use blink_esp::{serialize_global_blink,parse_global_blink,blink_changed_values};\n#[path = "config/inventory_manager.rs"]','global config')
 actor=E+'actor.rs'
 p.section(actor,'    pub fn global_settings(&self)','    pub fn set_global_settings(', '        tree\n','        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::Interop,&describe));\n        tree\n','GET global')
 p.section(actor,'    pub fn global_file(&self)','    pub fn save_global(', '        if let Some(groups) = self.state.global_preserved["value"].as_array() {','        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::File,&str::to_owned));\n        if let Some(groups) = self.state.global_preserved["value"].as_array() {','File global')
 anchor='        // Keep imported settings for systems that have not been ported yet.'
 p.replace(actor,anchor,'''        let blink_patch=config::parse_global_blink(value).map_err(|e|e.to_string())?;
        for issue in blink_patch.issues{eprintln!("[LiquidBounce] global config leaf skipped: {issue}");}
        for setting in blink_patch.changes {
            let before=config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::Interop,&|_|String::new());
            setting.apply(&mut self.state.behaviors.borrow_mut().blink_esp);
            let language=horizon_client::lang::code();
            let after=config::serialize_global_blink(&self.state.behaviors.borrow().blink_esp,JsonFormat::Interop,&|k|liquidbounce_interop::localization::describe(&language,k));
            for leaf in config::blink_changed_values(&before,&after){self.state.events.borrow_mut().push(("valueChanged".into(),json!({"value":leaf})));}
        }
'''+anchor,'apply and events')
 p.replace(actor,'"Targets" | "Enemies" | "Inventory"','"Targets" | "Enemies" | "Inventory" | "BlinkManager"','preserved exclusions',2)
 anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
 p.replace(E+'runtime.rs',anchor,'    #[cfg(feature="behavior-hooks")]\n    fn client_blink_esp(&self,host:&dyn Host,event:&mut horizon_client::mod_api::ClientBlinkEspEvent<\'_>){self.blink_esp(host,event);}\n'+anchor,'hook')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'crosshair-patch.py'","'crosshair-patch.py', 'blink-esp-patch.py'",'native patch')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} BlinkManager ESP changes across {len(p.text)} shared files');
 if args.apply:p.write();print('Integrated BlinkManager global ESP')
