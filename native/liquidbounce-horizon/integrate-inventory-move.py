"""Transactional InventoryMove integration. Only the integration owner applies."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent;E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod xray;','pub mod xray;\npub mod inventory_move;','model')
 p.replace(E+'lib.rs','mod xray_runtime;','mod xray_runtime;\n#[cfg(feature="behavior-hooks")]\nmod inventory_move_runtime;','runtime')
 b=E+'behaviors.rs'
 p.replace(b,'    pub xray:','    pub inventory_move:std::sync::Arc<crate::inventory_move_runtime::InventoryMoveService>,\n    pub xray:','state')
 p.replace(b,'            "XRay",','            "XRay",\n            "InventoryMove",','registration')
 p.replace(b,'            xray:Default::default(),','            xray:Default::default(),\n            inventory_move:Default::default(),','default')
 p.replace(b,'    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.inventory_move.publish(self.query_running(registry,"InventoryMove"),self.inventory.manager.inventory_open_server_side);','running')
 c=E+'config.rs'
 p.replace(c,'pub use xray::{XRaySetting,serialize_xray};','pub use xray::{XRaySetting,serialize_xray};\n#[path="config/inventory_move.rs"]\nmod inventory_move;\npub use inventory_move::{InventoryMoveSetting,serialize_inventory_move};','codec')
 p.replace(c,'    XRay=>("XRay","Render","xRay"),','    XRay=>("XRay","Render","xRay"),\n    InventoryMove=>("InventoryMove","Movement","inventoryMove"),','kind')
 p.replace(c,'    XRay(XRaySetting),','    XRay(XRaySetting),\n    InventoryMove(InventoryMoveSetting),','enum',2)
 p.replace(c,'            | ModuleKind::XRay','            | ModuleKind::XRay\n            | ModuleKind::InventoryMove','supported')
 p.replace(c,'        ModuleKind::XRay=>xray::parse_xray(&mut patch,&root),','        ModuleKind::XRay=>xray::parse_xray(&mut patch,&root),\n        ModuleKind::InventoryMove=>inventory_move::parse_inventory_move(&mut patch,&root),','parse')
 p.replace(c,'        SettingChange::XRay(value)=>','        SettingChange::InventoryMove(value)=>return ApplyResult::Runtime(RuntimeRequest::InventoryMove(value.clone())),\n        SettingChange::XRay(value)=>','request')
 a=E+'actor.rs'
 p.replace(a,'            ModuleKind::XRay=>config::serialize_xray(&b.xray.settings,&common,format,&describe),','            ModuleKind::XRay=>config::serialize_xray(&b.xray.settings,&common,format,&describe),\n            ModuleKind::InventoryMove=>config::serialize_inventory_move(&b.inventory_move.settings(),&common,format,&describe),','serialize')
 p.replace(a,'                ApplyResult::Runtime(RuntimeRequest::XRay(setting))=>','                ApplyResult::Runtime(RuntimeRequest::InventoryMove(setting))=>{let b=self.state.behaviors.borrow();b.inventory_move.update(|s|{if let Err(e)=setting.apply(s){eprintln!("[LiquidBounce] InventoryMove config: {e}");}});b.sync_packet_modules(&self.state.registry);}\n                ApplyResult::Runtime(RuntimeRequest::XRay(setting))=>','apply')
 p.replace(a,'            ModuleKind::AutoQueue=>Some(','            ModuleKind::InventoryMove=>Some(liquidbounce_behaviors::inventory_move::BEHAVIORS[self.state.behaviors.borrow().inventory_move.settings().behavior as usize]),\n            ModuleKind::AutoQueue=>Some(','tagBy')
 r=E+'runtime.rs'
 p.replace(r,'            pipeline.add_listener(i32::MAX, modules.plugins.clone());','            pipeline.add_listener(i32::MAX, modules.plugins.clone());\n            pipeline.add_listener(1000,modules.inventory_move.clone());','FIRST_PRIORITY')
 p.replace(r,'        self.auto_buff_tick(host);','        self.inventory_move_tick(host);\n        self.auto_buff_tick(host);','tick')
 p.replace(r,'    fn client_directional_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {','    fn client_directional_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {\n        self.inventory_move_input(host,event);','physical input')
 p.replace(r,'        let running = modules.query_running(&state.registry, "Sprint");\n        let input = DirectionalInput {','        event.shift=horizon_client::inventory_move::sneak(event.shift);\n        let running = modules.query_running(&state.registry, "Sprint");\n        let input = DirectionalInput {','sneak MODEL_STATE')
 p.replace(r,'        #[cfg(feature = "webview")]\n        let anti_hunger_input = event.sprint;','        event.sprint=horizon_client::inventory_move::sprint(0,transformed.is_moving(),event.sprint);\n        #[cfg(feature = "webview")]\n        let anti_hunger_input = event.sprint;','sprint INPUT')
 p.replace(r,'            event.omnidirectional = modules.sprint.omnidirectional(running, false);','            event.movement_override=horizon_client::inventory_move::movement_sprint(modules.directional_input.is_moving(),event.movement_override);\n            event.omnidirectional = modules.sprint.omnidirectional(running, false);','sprint MOVEMENT_TICK')
 p.replace(r,'        if criticals_stop_sprint{event.sprinting=false;}','        event.sprinting=horizon_client::inventory_move::sprint(2,modules.directional_input.is_moving(),event.sprinting);\n        if criticals_stop_sprint{event.sprinting=false;}','sprint NETWORK')
 p.replace(r,'        self.state().behaviors.borrow_mut().auto_buff.record_input(event);','        self.inventory_move_final(event);\n        self.state().behaviors.borrow_mut().auto_buff.record_input(event);','FINAL_DECISION')
 p.replace(r,'        self.reset_inventory();','        horizon_client::inventory_move::clear();\n        self.reset_inventory();','world reset')
 blink=E+'criticals_runtime.rs'
 p.replace(blink,'pass: bool, moving:Option<bool>) -> BlinkAction','pass: bool, moving:Option<bool>, container:bool) -> BlinkAction','shared manager signature')
 p.replace(blink,'        // BlinkPacketEvent.action is monotonic: QUEUE > PASS > FLUSH.','        let inventory=if outgoing{horizon_client::inventory_move::blink(container)}else{0};\n        if inventory==2{return BlinkAction::Queue;}\n        // BlinkPacketEvent.action is monotonic: QUEUE > PASS > FLUSH.','shared QUEUE')
 p.replace(blink,'        action\n    }\n    fn probe','        if inventory==1&&action==BlinkAction::Flush{BlinkAction::Pass}else{action}\n    }\n    fn probe','shared PASS')
 p.replace(blink,'outgoing, false,None)','outgoing, false,None,false)','probe')
 p.replace(blink,'true, pass(event.packet),Some(movement(event.packet)))','true, pass(event.packet),Some(movement(event.packet)),horizon_client::inventory_move::container_packet(event.packet))','outgoing')
 p.replace(blink,'false, false,None)','false, false,None,false)','incoming')
 p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'inventory-move-patch.py', 'step-patch.py'):",'native')
 browser=E+'browser.rs'
 p.replace(browser,'        if let Some(response) = self.ui.request(request) {','        if let Some(response) = self.ui.request(request) {\n            horizon_client::inventory_move::SEARCH_FOCUSED.store(self.ui.typing,std::sync::atomic::Ordering::Release);','original ClickGUI typing route')
 p.replace(browser,'        browser.screen(host.mod_screen() == Some(MOD_ID));','        browser.screen(host.mod_screen() == Some(MOD_ID));\n        horizon_client::inventory_move::SEARCH_FOCUSED.store(browser.ui.typing,std::sync::atomic::Ordering::Release);','browser screen focus')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare()
 if args.apply:p.write()
 print('InventoryMove',len(p.steps),'edits',len(p.text),'files','applied'if args.apply else'validated')
