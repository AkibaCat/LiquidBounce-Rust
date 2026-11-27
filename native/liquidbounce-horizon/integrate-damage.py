#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:70]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
E='native/liquidbounce-horizon/engine/src/'
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;','pub mod auto_leave;\npub mod damage;')
edit(E+'lib.rs','mod one_shots_runtime;','mod one_shots_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod damage_runtime;')
edit(E+'behaviors.rs','    pub kick_mode:','    pub damage: liquidbounce_behaviors::damage::Damage,\n    pub kick_mode:')
edit(E+'behaviors.rs','            "Kick",','            "Kick",\n            "Damage",')
edit(E+'behaviors.rs','matches!(name, "NameCollector" | "Kick")','matches!(name, "NameCollector" | "Kick" | "Damage")')
edit(E+'behaviors.rs','            kick_mode: Default::default(),','            damage: Default::default(),\n            kick_mode: Default::default(),')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Damage") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::Damage(self.damage));
            }''')
edit(E+'one_shots_runtime.rs','    CaptureNames,','    CaptureNames,\n    Damage(liquidbounce_behaviors::damage::Damage),')
edit(E+'one_shots_runtime.rs','        match effect {','        match effect {\n            OneShotEffect::Damage(settings) => crate::damage_runtime::run(host,settings),')
edit(E+'config.rs','mod one_shots;','mod one_shots;\n#[path = "config/damage.rs"]\nmod damage;\npub use damage::{DamageSetting,serialize_damage};')
edit(E+'config.rs','    Kick => ("Kick", "Exploit", "kick"),','    Kick => ("Kick", "Exploit", "kick"),\n    Damage => ("Damage", "Exploit", "damage"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    Damage(DamageSetting),',2)
edit(E+'config.rs','            | ModuleKind::Kick','            | ModuleKind::Damage\n            | ModuleKind::Kick')
edit(E+'config.rs','        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),','        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),\n        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::ResetVl(value) => {','        SettingChange::Damage(value) => { return ApplyResult::Runtime(RuntimeRequest::Damage(*value)); }\n        SettingChange::ResetVl(value) => {')
edit(E+'actor.rs','            ModuleKind::ResetVl => {','            ModuleKind::Damage => config::serialize_damage(&b.damage,&common,format,&describe),\n            ModuleKind::ResetVl => {')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => { setting.apply(&mut self.state.behaviors.borrow_mut().damage); }\n                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {')
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'one-shots-patch.py',","'one-shots-patch.py', 'damage-patch.py',")
print('Integrated Damage shared source')
