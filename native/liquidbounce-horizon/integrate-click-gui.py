#!/usr/bin/env python3
"""Transactional ClickGUI module, browser, settings, keybind and FPS wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod debug;','pub mod debug;\npub mod click_gui;')
 p.edit('browser.rs','use crate::{','#[path="click_gui_browser.rs"]\nmod click_gui;\nuse crate::{')
 p.edit('browser.rs','            url: theme_url.clone(),','            url: theme_url.clone(),\n            click_gui_url:Some(format!("{theme_url}#/clickgui?static")),')
 p.edit('browser.rs','        if !visible {\n            self.buttons = 0;','        if !visible {\n            push(self,Input::SelectPage(liquidbounce_webview::input::PageTarget::Shared));\n            self.buttons = 0;')
 p.edit('behaviors.rs','    pub debug:crate::debug_runtime::DebugRuntime,','    pub debug:crate::debug_runtime::DebugRuntime,\n    pub click_gui:liquidbounce_behaviors::click_gui::ClickGui,')
 p.edit('behaviors.rs','            "Debug",','            "Debug",\n            "ClickGUI",')
 p.edit('behaviors.rs','            debug:Default::default(),','            debug:Default::default(),\n            click_gui:Default::default(),')
 p.edit('behaviors.rs','                        if matches!(name, "NameCollector" | "Kick" | "Damage") {','                        if name=="ClickGUI"{definition.disable_activation=true;definition.bind=crate::input::configured_bind("key.keyboard.right.shift").map(|key|InputBind::new(key,BindAction::Toggle));}\n                        if matches!(name, "NameCollector" | "Kick" | "Damage") {')
 anchor='            ModuleCallback::OnEnabled(id) if self.query_ids.get("Debug")==Some(&id)=>self.debug.enabled(),'
 p.edit('behaviors.rs',anchor,'            ModuleCallback::OnEnabled(id) if self.query_ids.get("ClickGUI")==Some(&id)=>self.click_gui.enabled(registry.world().in_game()),\n'+anchor)
 p.edit('config.rs','pub use debug::{DebugSetting,serialize_debug};','pub use debug::{DebugSetting,serialize_debug};\n#[path="config/click_gui.rs"]\nmod click_gui;\npub use click_gui::{ClickGuiSetting,serialize_click_gui};')
 p.edit('config.rs','    Debug=>("Debug","Render","debug"),','    Debug=>("Debug","Render","debug"),\n    ClickGui=>("ClickGUI","Render","clickGui"),')
 p.edit('config.rs','    Debug(DebugSetting),','    Debug(DebugSetting),\n    ClickGui(ClickGuiSetting),',2)
 p.edit('config.rs','            | ModuleKind::Debug\n            | ModuleKind::BedPlates','            | ModuleKind::Debug\n            | ModuleKind::ClickGui\n            | ModuleKind::BedPlates')
 p.edit('config.rs','        ModuleKind::Debug=>debug::parse_debug(&mut patch,&root),','        ModuleKind::Debug=>debug::parse_debug(&mut patch,&root),\n        ModuleKind::ClickGui=>click_gui::parse_click_gui(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Debug(v)=>return ApplyResult::Runtime(RuntimeRequest::Debug(v.clone())),','        SettingChange::Debug(v)=>return ApplyResult::Runtime(RuntimeRequest::Debug(v.clone())),\n        SettingChange::ClickGui(v)=>return ApplyResult::Runtime(RuntimeRequest::ClickGui(v.clone())),')
 p.edit('actor.rs','            ModuleKind::Debug=>config::serialize_debug(&b.debug.model.settings,&common,format,&describe),','            ModuleKind::Debug=>config::serialize_debug(&b.debug.model.settings,&common,format,&describe),\n            ModuleKind::ClickGui=>config::serialize_click_gui(&b.click_gui.settings,&common,format,&describe,b.click_gui.refresh_hz),')
 p.edit('actor.rs','value["running"] = json!(if kind == ModuleKind::AutoAccount {','value["running"] = json!(if kind==ModuleKind::ClickGui {true} else if kind == ModuleKind::AutoAccount {')
 anchor='                ApplyResult::Runtime(RuntimeRequest::Debug(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().debug.model){eprintln!("[LiquidBounce] Debug config: {e}");}},'
 branch='''                ApplyResult::Runtime(RuntimeRequest::ClickGui(v))=>{
                    let change=v.apply(&mut self.state.behaviors.borrow_mut().click_gui);
                    if change.scale {let scale=self.state.behaviors.borrow().click_gui.settings.scale;self.state.events.borrow_mut().push(("clickGuiScaleChange".into(),json!({"value":scale})));}
                    if change.values {if let Some(tree)=self.settings("ClickGUI"){self.state.events.borrow_mut().push(("clickGuiValueChange".into(),json!({"configurable":tree})));}}
                },'''
 p.edit('actor.rs',anchor,anchor+'\n'+branch)
 # Original common input handler activates ClickGUI; only closing an already open screen is intercepted here.
 old='''        if code == KeyCode::ShiftRight && state == ElementState::Pressed && !repeat {
            if own {
                host.mod_screen_close(MOD_ID);
            } else {
                host.mod_screen_open(MOD_ID);
            }
            return true;
        }'''
 new='''        if own && state==ElementState::Pressed && !repeat && crate::input::keyboard_scan_code(code).is_some_and(|scan|self.click_gui_bound(liquidbounce_core::InputKey::Keyboard(scan),modifiers)) {
            host.mod_screen_close(MOD_ID);
            return true;
        }'''
 p.edit('browser.rs',old,new)
 p.edit('browser.rs','        if let Some(response) = self.ui.request(request) {','''        if request.path=="/api/v1/client/modules/settings"&&request.query.get("name").is_some_and(|n|n=="ClickGUI"){
            if let Some(response)=module_request(actor,request){return response;}
        }
        if let Some(response) = self.ui.request(request) {''')
 p.edit('browser.rs','SEARCH_FOCUSED.store(self.ui.typing,','SEARCH_FOCUSED.store(liquidbounce_behaviors::click_gui::ClickGui::in_search_bar(self.ui.typing,self.visible),')
 p.edit('browser.rs','SEARCH_FOCUSED.store(browser.ui.typing,','SEARCH_FOCUSED.store(liquidbounce_behaviors::click_gui::ClickGui::in_search_bar(browser.ui.typing,browser.visible),')
 p.edit('browser.rs','''                if let Err(error) = actor.load(&browser.modules_file) {''','''                browser.migrate_click_gui(&mut actor);
                if let Err(error) = actor.load(&browser.modules_file) {''')
 p.edit('browser.rs','''        let mut state = self.state();
        let actor = Actor {''','''        browser.click_gui_frame(self,host,client);
        let mut state = self.state();
        let actor = Actor {''')
 p.edit('browser.rs','''        // Screen.Pre cancellation prevents Horizon's generic Mouse.Post hook.''','''        if !up && self.click_gui_bound(liquidbounce_core::InputKey::Mouse(match button{Button::Left=>1,Button::Middle=>2,Button::Right=>3}),modifiers){
            self.state().behaviors.borrow_mut().click_gui.close_requested=true;
            return Flow::Cancel;
        }
        // Screen.Pre cancellation prevents Horizon's generic Mouse.Post hook.''')
 p.edit('browser.rs','''        let modules = actor.state.behaviors.borrow();''','''        let pages=liquidbounce_webview::runtime::page_status_current_thread().map(|s|json!({"active":format!("{:?}",s.active),"sharedOpen":s.shared_open,"clickGuiOpen":s.click_gui_open,"sharedBrowserId":s.shared_browser_id,"clickGuiBrowserId":s.click_gui_browser_id,"retiredPages":s.retired_pages,"createdPages":s.created_pages,"closedPages":s.closed_pages,"fps":s.fps,"sharedFps":s.shared_fps,"clickGuiFps":s.click_gui_fps}));
        let modules = actor.state.behaviors.borrow();''')
 p.edit('browser.rs','"page":page},"requests"','"page":page,"pages":pages},"requests"')
 load='''    fn client_level_load(
        &self,
        host: &mut dyn Host,
        _event: &mut horizon_client::mod_api::ClientLevelLoadEvent<'_>,
    ) {'''
 p.edit('runtime.rs',load,load+'''
        self.state().behaviors.borrow_mut().click_gui.level_loaded(self.started.elapsed().as_millis().min(u64::MAX as u128)as u64);''')
 p.edit('runtime.rs','            modules.directional_input = Default::default();','            modules.click_gui.disconnected();\n            modules.directional_input = Default::default();')
 p.edit('runtime.rs','''    #[cfg(feature = "behavior-hooks")]
    fn client_level_load(''','''    #[cfg(feature="webview")]
    fn framerate_limit(&self,host:&dyn Host,event:&mut horizon_client::mod_api::FramerateLimitEvent){self.click_gui_limit(host,event);}

    #[cfg(feature = "behavior-hooks")]
    fn client_level_load(''')
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'click-gui-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args();p=prepare()
 if not args.check:p.commit()
 print(f'ClickGUI {"validated" if args.check else "integrated"}: {len(p.changed)} shared files')
