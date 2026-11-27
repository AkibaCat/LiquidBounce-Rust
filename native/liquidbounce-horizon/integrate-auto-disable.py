#!/usr/bin/env python3
"""One-time shared wiring for the concrete AutoDisable implementation."""
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
E = 'native/liquidbounce-horizon/engine/src/'
def edit(path, old, new, count=1):
    p = ROOT / path
    source = p.read_text(encoding='utf8')
    if source.count(old) != count:
        raise ValueError(f'{path}: expected {count} copies of {old[:80]!r}, got {source.count(old)}')
    p.write_text(source.replace(old, new), encoding='utf8')
edit('native/liquidbounce-behaviors/src/lib.rs', 'pub mod auto_leave;', 'pub mod auto_leave;\npub mod auto_disable;')
edit(E+'lib.rs', 'mod auto_break_runtime;', 'mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod auto_disable_runtime;')
edit(E+'behaviors.rs', '    pub auto_leave:', '    pub auto_disable: liquidbounce_behaviors::auto_disable::AutoDisable,\n    pub auto_disable_packets: std::sync::Arc<crate::auto_disable_runtime::AutoDisablePackets>,\n    pub auto_leave:')
edit(E+'behaviors.rs', '            "AutoLeave",', '            "AutoLeave",\n            "AutoDisable",')
edit(E+'behaviors.rs', '            auto_leave: Default::default(),', '            auto_disable: Default::default(),\n            auto_disable_packets: Default::default(),\n            auto_leave: Default::default(),')
edit(E+'behaviors.rs', '    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {', '    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.auto_disable_packets.set_running(self.query_running(registry,"AutoDisable"));')
edit(E+'config.rs', '#[path = "config/auto_weapon.rs"]', '#[path = "config/auto_disable.rs"]\nmod auto_disable;\npub use auto_disable::{AutoDisableSetting, serialize_auto_disable};\n#[path = "config/auto_weapon.rs"]')
edit(E+'config.rs', '    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),', '    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    AutoDisable => ("AutoDisable", "World", "autoDisable"),')
edit(E+'config.rs', '    OneShot(OneShotSetting),', '    OneShot(OneShotSetting),\n    AutoDisable(AutoDisableSetting),', 2)
edit(E+'config.rs', '            | ModuleKind::AutoLeave', '            | ModuleKind::AutoLeave\n            | ModuleKind::AutoDisable')
edit(E+'config.rs', '        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),', '        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::AutoDisable => auto_disable::parse_auto_disable(&mut patch, &root),')
edit(E+'config.rs', '        SettingChange::Damage(value) => {', '        SettingChange::AutoDisable(value) => { return ApplyResult::Runtime(RuntimeRequest::AutoDisable(value.clone())); }\n        SettingChange::Damage(value) => {')
edit(E+'actor.rs', '            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),', '            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::AutoDisable => config::serialize_auto_disable(&b.auto_disable, &common, format, &describe),')
edit(E+'actor.rs', '                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {', '                ApplyResult::Runtime(RuntimeRequest::AutoDisable(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().auto_disable); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
edit(E+'runtime.rs', '            pipeline.add_listener(0, modules.anti_hunger.clone());', '            pipeline.add_listener(0, modules.anti_hunger.clone());\n            pipeline.add_listener(0, modules.auto_disable_packets.clone());')
edit(E+'runtime.rs', '    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {', '    fn client_tick_pre(&self, host: &mut dyn Host, _event: &mut ClientTickPreEvent) {\n        self.auto_disable_drain(host);')
anchor = '    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
edit(E+'runtime.rs', anchor, '''    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_flag(&self, host: &mut dyn Host, _event: &mut horizon_client::mod_api::ClientAutoDisableFlagEvent) {
        self.auto_disable_drain(host);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_death(&self, host: &mut dyn Host, _event: &mut horizon_client::mod_api::ClientAutoDisableDeathEvent) {
        self.auto_disable_drain(host);
        self.auto_disable_event(host, liquidbounce_behaviors::auto_disable::Reason::Death);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_lifecycle(&self, host: &mut dyn Host, event: &mut horizon_client::mod_api::ClientAutoDisableLifecycleEvent) {
        self.auto_disable_drain(host);
        let reason = if event.reason == 2 { liquidbounce_behaviors::auto_disable::Reason::WorldChange } else { liquidbounce_behaviors::auto_disable::Reason::Disconnect };
        self.auto_disable_event(host, reason);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_auto_disable_command(&self, host: &mut dyn Host, event: &mut horizon_client::mod_api::ClientAutoDisableCommandEvent<'_>) {
        event.handled |= self.auto_disable_command(host, event.message);
    }
'''+anchor)
edit('native/liquidbounce-horizon/make-behavior-patch.py', "'one-shots-patch.py',", "'one-shots-patch.py', 'auto-disable-patch.py',")
edit('native/liquidbounce-behaviors/src/auto_leave.rs', '//! ModuleAutoLeave / tickConditional, including its threshold-tick precedence.', '//! ModuleAutoLeave / tickConditional, including its threshold-tick precedence.\n#[path = "navigation/java_case.rs"]\nmod java_case;')
edit('native/liquidbounce-behaviors/src/auto_leave.rs', 'find(|mode| mode.tag() == tag)', 'find(|mode| java_case::equals_ignore_case(mode.tag(), tag))')
print('Integrated AutoDisable and enum case folding')
