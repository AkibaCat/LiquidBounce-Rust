#!/usr/bin/env python3
"""One-time canonical wiring for the owned MiddleClickAction implementation."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} anchors {old[:60]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_account;','pub mod auto_account;\npub mod middle_click_action;')
edit(E+'lib.rs','mod auto_account_runtime;','mod auto_account_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod middle_click_action_runtime;')
edit(E+'behaviors.rs','    pub auto_account:','    pub middle_click_action: liquidbounce_behaviors::middle_click_action::MiddleClickAction,\n    pub auto_account:')
edit(E+'behaviors.rs','            "AutoAccount",','            "AutoAccount",\n            "MiddleClickAction",')
edit(E+'behaviors.rs','            auto_account: Default::default(),','            middle_click_action: Default::default(),\n            auto_account: Default::default(),')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::CancelSequences(id) if self.query_ids.get("MiddleClickAction") == Some(&id) => {
                self.middle_click_action.pearl_disable();
                self.pending_slot_resets.push(crate::middle_click_action_runtime::PEARL_REQUESTER);
            }''')
edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/middle_click_action.rs"]\nmod middle_click_action;\npub use middle_click_action::{MiddleClickActionSetting,serialize_middle_click_action};\n#[path = "config/auto_weapon.rs"]')
edit(E+'config.rs','    AutoAccount => ("AutoAccount", "Misc", "autoAccount"),','    AutoAccount => ("AutoAccount", "Misc", "autoAccount"),\n    MiddleClickAction => ("MiddleClickAction", "Misc", "middleClickAction"),')
edit(E+'config.rs','    AutoAccount(AutoAccountSetting),','    AutoAccount(AutoAccountSetting),\n    MiddleClickAction(MiddleClickActionSetting),',2)
edit(E+'config.rs','            | ModuleKind::AutoAccount','            | ModuleKind::AutoAccount\n            | ModuleKind::MiddleClickAction')
edit(E+'config.rs','        ModuleKind::AutoAccount => auto_account::parse_auto_account(&mut patch, &root),','        ModuleKind::AutoAccount => auto_account::parse_auto_account(&mut patch, &root),\n        ModuleKind::MiddleClickAction => middle_click_action::parse_middle_click_action(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::AutoAccount(value) => {','        SettingChange::MiddleClickAction(value) => { return ApplyResult::Runtime(RuntimeRequest::MiddleClickAction(*value)); }\n        SettingChange::AutoAccount(value) => {')
edit(E+'actor.rs','            ModuleKind::AutoAccount => config::serialize_auto_account(&b.auto_account,&common,format,&describe),','            ModuleKind::AutoAccount => config::serialize_auto_account(&b.auto_account,&common,format,&describe),\n            ModuleKind::MiddleClickAction => config::serialize_middle_click_action(&b.middle_click_action,&common,format,&describe),')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::AutoAccount(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::MiddleClickAction(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().middle_click_action); }\n                ApplyResult::Runtime(RuntimeRequest::AutoAccount(setting)) => {')
edit(E+'actor.rs','ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"],','ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"], ModuleKind::MiddleClickAction=>vec!["FriendClicker","MiddleClickPearl"],')
edit(E+'actor.rs','value["running"] = json!(self.state.registry.running(self.id(kind)));','value["running"] = json!(if kind==ModuleKind::AutoAccount {self.state.registry.enabled_without_world(self.id(kind))} else {self.state.registry.running(self.id(kind))});')
edit(E+'runtime.rs','        self.timer_tick(host);','        self.middle_click_action_tick(host);\n        self.timer_tick(host);')
edit(E+'runtime.rs','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {','    fn client_world_change(&self, host: &dyn Host, event: &mut ClientWorldChangeEvent<\'_>) {\n        self.state().behaviors.borrow_mut().middle_click_action.world_change();')
edit(E+'runtime.rs','            modules.inventory.world_change();','            modules.middle_click_action.world_change();\n            modules.inventory.world_change();',2)
edit(E+'runtime.rs','    fn client_auto_account_message(','''    fn interaction_key(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::InteractionKeyEvent<'_>) {
        if event.button==2 && self.middle_click_action_cancel_pick(host) { *event.canceled=true; }
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_account_message(''')
edit('native/liquidbounce-horizon/middle-click-action-patch.py',"'hook_names':[]", "'hook_names':['interaction_key']")
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'auto-account-patch.py',", "'auto-account-patch.py', 'middle-click-action-patch.py',")
print('Integrated MiddleClickAction')
