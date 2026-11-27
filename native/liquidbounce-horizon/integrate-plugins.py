#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:65]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod plugins;')
edit(E+'lib.rs','mod auto_break_runtime;','mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod plugins_runtime;')
edit(E+'behaviors.rs','    pub auto_leave:','    pub plugins: std::sync::Arc<crate::plugins_runtime::PluginsService>,\n    pub auto_leave:')
edit(E+'behaviors.rs','            "AutoLeave",','            "AutoLeave",\n            "Plugins",')
edit(E+'behaviors.rs','matches!(name, "Timer" | "ElytraSwap")','matches!(name, "Timer" | "ElytraSwap" | "Plugins")')
edit(E+'behaviors.rs','            auto_leave: Default::default(),','            plugins: Default::default(),\n            auto_leave: Default::default(),')
edit(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.plugins.set_running(self.query_running(registry,"Plugins"));')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::EnabledEffect(id) if self.query_ids.get("Plugins") == Some(&id) => {
                if registry.module(id).is_some_and(|module|module.enabled()) {
                    self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::PluginsStart);
                }
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("Plugins") == Some(&id) => self.plugins.cancel(),''')
edit(E+'one_shots_runtime.rs','    CaptureNames,','    CaptureNames,\n    PluginsStart,')
edit(E+'one_shots_runtime.rs','        match effect {','        match effect {\n            OneShotEffect::PluginsStart => self.plugins_start(host),')
edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/plugins.rs"]\nmod plugins;\npub use plugins::serialize_plugins;\n#[path = "config/auto_weapon.rs"]')
edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    Plugins => ("Plugins", "Exploit", "plugins"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    Plugins(u32),',2)
edit(E+'config.rs','            | ModuleKind::AutoLeave','            | ModuleKind::AutoLeave\n            | ModuleKind::Plugins')
edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::Plugins => plugins::parse_plugins(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::Plugins(value) => { return ApplyResult::Runtime(RuntimeRequest::Plugins(*value)); }\n        SettingChange::Damage(value) => {')
edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::Plugins => config::serialize_plugins(&b.plugins.behavior.lock().expect("Plugins poisoned"), &common, format, &describe),')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Plugins(value)) => { self.state.behaviors.borrow().plugins.behavior.lock().expect("Plugins poisoned").try_methods=value; }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
edit(E+'runtime.rs','            pipeline.add_listener(0, modules.anti_hunger.clone());','            pipeline.add_listener(i32::MAX, modules.plugins.clone());\n            pipeline.add_listener(0, modules.anti_hunger.clone());')
edit(E+'runtime.rs','        self.auto_disable_drain(host);\n        use liquidbounce_behaviors::rotations','        self.auto_disable_drain(host);\n        self.plugins_tick(host);\n        use liquidbounce_behaviors::rotations')
edit(E+'runtime.rs','            self.auto_disable_event(host, reason);','            self.auto_disable_event(host, reason);') if False else None
edit(E+'runtime.rs','        self.auto_disable_event(host, reason);','        self.auto_disable_event(host, reason);\n        if event.reason == 3 { self.state().behaviors.borrow().plugins.disconnected(); }')
anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
edit(E+'runtime.rs',anchor,'''    #[cfg(feature = "behavior-hooks")]
    fn client_plugins_game_message(&self, host: &mut dyn Host, event: &mut horizon_client::mod_api::ClientPluginsGameMessageEvent<'_>) {
        self.plugins_game_message(host,event.message);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_plugins_server_commands(&self, _host: &dyn Host, event: &mut horizon_client::mod_api::ClientPluginsServerCommandsEvent<'_>) {
        self.plugins_server_commands(event.names);
    }
'''+anchor)
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'one-shots-patch.py',", "'one-shots-patch.py', 'plugins-patch.py',")
print('Integrated Plugins')
