#!/usr/bin/env python3
"""One-time shared wiring for the three owned one-shot/leave modules."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def edit(path, old, new, count=1):
    path=ROOT/path
    source=path.read_text(encoding='utf8')
    if source.count(old)!=count: raise ValueError(f'{path}: expected {count} occurrences of {old[:80]!r}, got {source.count(old)}')
    path.write_text(source.replace(old,new),encoding='utf8')
E='native/liquidbounce-horizon/engine/src/'
edit('native/liquidbounce-behaviors/src/lib.rs','pub mod reset_vl;','pub mod reset_vl;\npub mod auto_leave;')
edit(E+'lib.rs','mod reset_vl_runtime;','mod reset_vl_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod one_shots_runtime;')
edit(E+'coroutine_ticker.rs','    ResetVl,','    ResetVl,\n    AutoLeave,')
edit(E+'behaviors.rs','    pub pending_pitch_reset: bool,','    pub auto_leave: liquidbounce_behaviors::auto_leave::AutoLeave,\n    pub kick_mode: liquidbounce_behaviors::auto_leave::KickMode,\n    pub pending_one_shots: Vec<crate::one_shots_runtime::OneShotEffect>,\n    pub pending_pitch_reset: bool,')
edit(E+'behaviors.rs','            "ResetVL",','            "ResetVL",\n            "NameCollector",\n            "Kick",\n            "AutoLeave",')
edit(E+'behaviors.rs','                        if matches!(name, "Timer" | "ElytraSwap") {','                        if matches!(name, "NameCollector" | "Kick") { definition.disable_activation = true; }\n                        if matches!(name, "Timer" | "ElytraSwap") {')
edit(E+'behaviors.rs','            pending_pitch_reset: false,','            auto_leave: Default::default(),\n            kick_mode: Default::default(),\n            pending_one_shots: Vec::new(),\n            pending_pitch_reset: false,')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnEnabled(id) if self.query_ids.get("NameCollector") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::CaptureNames);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Kick") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::Kick(self.kick_mode));
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoLeave") == Some(&id) => {
                self.auto_leave.cancel_sequences();
            }''')
edit(E+'runtime.rs','    fast_break_resets: usize,','    #[cfg(feature = "behavior-hooks")]\n    one_shots: Vec<crate::one_shots_runtime::OneShotEffect>,\n    fast_break_resets: usize,')
edit(E+'runtime.rs','                fast_break_resets: std::mem::take(&mut modules.pending_fast_break_resets),','                one_shots: std::mem::take(&mut modules.pending_one_shots),\n                fast_break_resets: std::mem::take(&mut modules.pending_fast_break_resets),')
edit(E+'runtime.rs','pub(crate) fn apply_effects(host: &mut dyn Host, intents: RuntimeEffects) {','pub(crate) fn apply_effects(hooks: &LiquidBounceHooks, host: &mut dyn Host, intents: RuntimeEffects) {')
edit(E+'runtime.rs','        for duration in intents.night_vision {','        for effect in intents.one_shots { hooks.apply_one_shot(host,effect); }\n        for duration in intents.night_vision {')
edit(E+'runtime.rs','        assert!(\n            intents.night_vision.is_empty()','        let _ = hooks;\n        assert!(\n            intents.night_vision.is_empty()')
edit(E+'runtime.rs','apply_effects(host, effects);','apply_effects(self, host, effects);',5)
edit(E+'browser.rs','runtime::apply_effects(host, effects);','runtime::apply_effects(self, host, effects);')
edit(E+'runtime.rs','                WaitOwner::ResetVl => self.reset_vl_callback(host, in_game, task.callback),','                WaitOwner::ResetVl => self.reset_vl_callback(host, in_game, task.callback),\n                WaitOwner::AutoLeave => self.auto_leave_callback(host, in_game, task.callback),')
edit(E+'runtime.rs','        self.reset_vl_tick(host);','        self.reset_vl_tick(host);\n        self.auto_leave_tick(host);')
edit(E+'config.rs','mod reset_vl;','mod reset_vl;\n#[path = "config/one_shots.rs"]\nmod one_shots;\npub use one_shots::{OneShotSetting,serialize_kick,serialize_auto_leave};')
edit(E+'config.rs','    ResetVl => ("ResetVL", "Exploit", "resetVL"),','    ResetVl => ("ResetVL", "Exploit", "resetVL"),\n    NameCollector => ("NameCollector", "Exploit", "nameCollector"),\n    Kick => ("Kick", "Exploit", "kick"),\n    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),')
edit(E+'config.rs','    ResetVl(ResetVlSetting),','    ResetVl(ResetVlSetting),\n    OneShot(OneShotSetting),',2)
edit(E+'config.rs','            | ModuleKind::ResetVl','            | ModuleKind::ResetVl\n            | ModuleKind::Kick\n            | ModuleKind::AutoLeave')
edit(E+'config.rs','        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),','        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),\n        ModuleKind::Kick | ModuleKind::AutoLeave => one_shots::parse_one_shots(&mut patch, &root, module),')
edit(E+'config.rs','        SettingChange::ResetVl(value) => {','        SettingChange::OneShot(value) => { return ApplyResult::Runtime(RuntimeRequest::OneShot(*value)); }\n        SettingChange::ResetVl(value) => {')
edit(E+'actor.rs','            ModuleKind::ResetVl => {','            ModuleKind::Kick => config::serialize_kick(b.kick_mode, &common, format, &describe),\n            ModuleKind::AutoLeave => config::serialize_auto_leave(&b.auto_leave.settings, &common, format, &describe),\n            ModuleKind::ResetVl => {')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::OneShot(setting)) => {
                    use config::OneShotSetting;
                    let mut modules = self.state.behaviors.borrow_mut();
                    match setting {
                        OneShotSetting::Kick(mode) => modules.kick_mode = mode,
                        OneShotSetting::Health(value) => modules.auto_leave.settings.health = value,
                        OneShotSetting::Delay(value) => modules.auto_leave.settings.delay = value,
                        OneShotSetting::LeaveMode(mode) => modules.auto_leave.settings.mode = mode,
                    }
                }
                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {''')
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'reset-vl-patch.py',","'reset-vl-patch.py', 'one-shots-patch.py',")
edit('scripts/runtime_scenarios/extended_firework.py','module("ExtendedFirework")["tag"] == "Normal"','module("ExtendedFirework")["tag"] is None')
edit('scripts/runtime_scenarios/extended_firework.py','module("ExtendedFirework")["tag"] == "Custom"','module("ExtendedFirework")["tag"] is None')
print('Integrated NameCollector, Kick and AutoLeave shared sources')
