#!/usr/bin/env python3
"""One-time canonical integration. Do not rerun after successful integration."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'
def edit(path,old,new,count=1):
    p=ROOT/path;s=p.read_text(encoding='utf8')
    if s.count(old)!=count:raise ValueError(f'{path}: expected {count} anchors of {old[:60]!r}, got {s.count(old)}')
    p.write_text(s.replace(old,new),encoding='utf8')
edit('native/liquidbounce-core/src/module.rs','    pub fn running(&self, id: ModuleId) -> bool {','''    /// Modules with an explicit pre-world running override still honor SelfDestruct.
    pub fn enabled_without_world(&self, id: ModuleId) -> bool {
        !self.destructed && self.module(id).is_some_and(|module| module.enabled())
    }
    pub fn running(&self, id: ModuleId) -> bool {''')
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod auto_account;')
edit(E+'lib.rs','mod auto_disable_runtime;','mod auto_disable_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod auto_account_runtime;')
edit(E+'behaviors.rs','    pub auto_leave:','    pub auto_account: liquidbounce_behaviors::auto_account::AutoAccount,\n    pub auto_leave:')
edit(E+'behaviors.rs','            "AutoLeave",','            "AutoLeave",\n            "AutoAccount",')
edit(E+'behaviors.rs','            auto_leave: Default::default(),','            auto_account: Default::default(),\n            auto_leave: Default::default(),')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoAccount") == Some(&id) => { self.auto_account.cancel(); }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("AutoAccount") == Some(&id) => { self.auto_account.cancel(); }''')
edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/auto_account.rs"]\nmod auto_account;\npub use auto_account::{AutoAccountSetting,serialize_auto_account};\n#[path = "config/auto_weapon.rs"]')
edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    AutoAccount => ("AutoAccount", "Misc", "autoAccount"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    AutoAccount(AutoAccountSetting),',2)
edit(E+'config.rs','            | ModuleKind::AutoLeave','            | ModuleKind::AutoLeave\n            | ModuleKind::AutoAccount')
edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::AutoAccount => auto_account::parse_auto_account(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::AutoAccount(value) => { return ApplyResult::Runtime(RuntimeRequest::AutoAccount(value.clone())); }\n        SettingChange::Damage(value) => {')
edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::AutoAccount => config::serialize_auto_account(&b.auto_account,&common,format,&describe),')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::AutoAccount(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().auto_account); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
edit(E+'actor.rs','ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"],','ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"], ModuleKind::AutoAccount=>vec!["AutoLogin","AutoRegister"],')
edit(E+'coroutine_ticker.rs','    AutoLeave,','    AutoLeave,\n    AutoAccount,')
edit(E+'runtime.rs','                WaitOwner::AutoLeave =>','                WaitOwner::AutoAccount => self.auto_account_callback(host, task.callback),\n                WaitOwner::AutoLeave =>')
edit(E+'runtime.rs','    fn client_plugins_game_message(','''    fn client_auto_account_message(&self,_host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAutoAccountMessageEvent<'_>) {
        self.auto_account_message(event.source,event.message);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_account_command(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientAutoAccountCommandEvent<'_>) {
        event.handled |= self.auto_account_command(host,event.message);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_plugins_game_message(''')
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'plugins-patch.py',", "'plugins-patch.py', 'auto-account-patch.py',")
print('Integrated AutoAccount actual chat/title/command and pre-world sequence')
