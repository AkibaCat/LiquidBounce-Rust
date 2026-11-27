#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:65]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod old_regen;')
edit(E+'lib.rs','mod auto_break_runtime;','mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod old_regen_runtime;')
edit(E+'behaviors.rs','    pub auto_leave:','    pub old_regen: liquidbounce_behaviors::old_regen::OldRegen,\n    pub auto_leave:')
edit(E+'behaviors.rs','            "AutoLeave",','            "AutoLeave",\n            "OldRegen",')
edit(E+'behaviors.rs','matches!(name, "Timer" | "ElytraSwap" | "Plugins")','matches!(name, "Timer" | "ElytraSwap" | "Plugins" | "OldRegen")')
edit(E+'behaviors.rs','            auto_leave: Default::default(),','            old_regen: Default::default(),\n            auto_leave: Default::default(),')
edit(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.timer.set_provider_running(crate::old_regen_runtime::TIMER_PROVIDER,self.query_running(registry,"OldRegen"));')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnEnabled(id) if self.query_ids.get("OldRegen") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::OldRegenEnabled);
            }''')
edit(E+'one_shots_runtime.rs','    CaptureNames,','    CaptureNames,\n    OldRegenEnabled,')
edit(E+'one_shots_runtime.rs','        match effect {','        match effect {\n            OneShotEffect::OldRegenEnabled => self.old_regen_enabled(host),')
edit(E+'config.rs','#[path = "config/auto_weapon.rs"]','#[path = "config/old_regen.rs"]\nmod old_regen;\npub use old_regen::{OldRegenSetting,serialize_old_regen};\n#[path = "config/auto_weapon.rs"]')
edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    OldRegen => ("OldRegen", "Exploit", "oldRegen"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    OldRegen(OldRegenSetting),',2)
edit(E+'config.rs','            | ModuleKind::AutoLeave','            | ModuleKind::AutoLeave\n            | ModuleKind::OldRegen')
edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::OldRegen => old_regen::parse_old_regen(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::Damage(value) => {','        SettingChange::OldRegen(value) => { return ApplyResult::Runtime(RuntimeRequest::OldRegen(*value)); }\n        SettingChange::Damage(value) => {')
edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::OldRegen => config::serialize_old_regen(&b.old_regen,&common,format,&describe),')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::OldRegen(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().old_regen); }\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {')
edit(E+'actor.rs','ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"],','ModuleKind::GuiCloser=>vec!["AutoClose","ContainerCloser"], ModuleKind::OldRegen=>vec!["Regen","Zoot"],')
edit(E+'runtime.rs','        self.reset_vl_tick(host);','        self.old_regen_tick(host);\n        self.reset_vl_tick(host);')
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'one-shots-patch.py',", "'one-shots-patch.py', 'old-regen-patch.py',")
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'semantic-pointer-patch.py'", "'auto-walk-patch.py', 'semantic-pointer-patch.py'")
print('Integrated OldRegen and real navigation dependency patch')
