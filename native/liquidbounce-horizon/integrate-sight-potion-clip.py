#!/usr/bin/env python3
"""One-time wiring for three concrete module adapters."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E='native/liquidbounce-horizon/engine/src/'
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:65]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
for snake,kind,category,key in [('potion_spoof','PotionSpoof','Player','potionSpoof'),('no_clip','NoClip','Movement','noClip'),('true_sight','TrueSight','Render','trueSight')]:
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_leave;',f'pub mod auto_leave;\npub mod {snake};')
 edit(E+'lib.rs','mod auto_break_runtime;',f'mod auto_break_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod {snake}_runtime;')
 edit(E+'behaviors.rs','    pub auto_leave:',f'    pub {snake}: liquidbounce_behaviors::{snake}::{kind},\n    pub auto_leave:')
 edit(E+'behaviors.rs','            "AutoLeave",',f'            "AutoLeave",\n            "{kind}",')
 edit(E+'behaviors.rs','            auto_leave: Default::default(),',f'            {snake}: Default::default(),\n            auto_leave: Default::default(),')
 edit(E+'config.rs','#[path = "config/auto_weapon.rs"]',f'#[path = "config/{snake}.rs"]\nmod {snake};\npub use {snake}::{{{kind}Setting, serialize_{snake}}};\n#[path = "config/auto_weapon.rs"]')
 edit(E+'config.rs','    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),',f'    AutoLeave => ("AutoLeave", "Combat", "autoLeave"),\n    {kind} => ("{kind}", "{category}", "{key}"),')
 edit(E+'config.rs','    OneShot(OneShotSetting),',f'    OneShot(OneShotSetting),\n    {kind}({kind}Setting),',2)
 edit(E+'config.rs','            | ModuleKind::AutoLeave',f'            | ModuleKind::AutoLeave\n            | ModuleKind::{kind}')
 edit(E+'config.rs','        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),',f'        ModuleKind::Damage => damage::parse_damage(&mut patch, &root),\n        ModuleKind::{kind} => {snake}::parse_{snake}(&mut patch, &root),')
 edit(E+'config.rs','        SettingChange::Damage(value) => {',f'        SettingChange::{kind}(value) => {{ return ApplyResult::Runtime(RuntimeRequest::{kind}(*value)); }}\n        SettingChange::Damage(value) => {{')
 edit(E+'actor.rs','            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),',f'            ModuleKind::Damage => config::serialize_damage(&b.damage, &common, format, &describe),\n            ModuleKind::{kind} => config::serialize_{snake}(&b.{snake}, &common, format, &describe),')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {',f'                ApplyResult::Runtime(RuntimeRequest::{kind}(setting)) => {{ setting.apply(&mut self.state.behaviors.borrow_mut().{snake}); }}\n                ApplyResult::Runtime(RuntimeRequest::Damage(setting)) => {{')
edit(E+'behaviors.rs','        match callback {','''        match callback {
            ModuleCallback::OnDisabled(id) if self.query_ids.get("NoClip") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::NoClipDisabled);
            }
            ModuleCallback::OnDisabled(id) if self.query_ids.get("PotionSpoof") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::PotionSpoofDisabled(self.potion_spoof.clone()));
            }''')
edit(E+'one_shots_runtime.rs','#[derive(Clone, Copy, Debug)]','#[derive(Clone, Debug)]')
edit(E+'one_shots_runtime.rs','    CaptureNames,','    CaptureNames,\n    NoClipDisabled,\n    PotionSpoofDisabled(liquidbounce_behaviors::potion_spoof::PotionSpoof),')
edit(E+'one_shots_runtime.rs','        match effect {','''        match effect {
            OneShotEffect::NoClipDisabled => self.no_clip_disabled(host),
            OneShotEffect::PotionSpoofDisabled(settings) => crate::potion_spoof_runtime::disabled(host, &settings),''')
edit(E+'potion_spoof_runtime.rs','    pub(crate) fn potion_spoof_disabled', '    pub(crate) fn potion_spoof_disabled',0) if False else None
edit(E+'potion_spoof_runtime.rs',' pub(crate) fn potion_spoof_disabled(&self,host:&mut dyn Host) {\n  let module=self.state().behaviors.borrow().potion_spoof.clone();module.on_disabled(&mut Adapter(host));\n }','')
edit(E+'potion_spoof_runtime.rs','impl LiquidBounceHooks {','pub(crate) fn disabled(host:&mut dyn Host,module:&liquidbounce_behaviors::potion_spoof::PotionSpoof) {module.on_disabled(&mut Adapter(host));}\nimpl LiquidBounceHooks {')
edit(E+'runtime.rs','        self.high_jump_tick(host);','        self.high_jump_tick(host);\n        self.no_clip_tick(host);')
edit(E+'runtime.rs','    fn client_player_tick_post(&self, host: &mut dyn Host, _event: &mut ClientPlayerTickPostEvent) {','    fn client_player_tick_post(&self, host: &mut dyn Host, _event: &mut ClientPlayerTickPostEvent) {\n        self.potion_spoof_tick(host);')
anchor='    #[cfg(feature = "behavior-hooks")]\n    fn client_world_change('
edit(E+'runtime.rs',anchor,'''    #[cfg(feature = "behavior-hooks")]
    fn client_true_sight_entity(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientTrueSightEntityEvent) {
        self.true_sight_entity(host,event);
    }
    #[cfg(feature = "behavior-hooks")]
    fn client_true_sight_markers(&self, host: &dyn Host, event: &mut horizon_client::mod_api::ClientTrueSightMarkersEvent) {
        self.true_sight_markers(host,event);
    }
'''+anchor)
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'semantic-pointer-patch.py'", "'potion-spoof-patch.py', 'no-clip-patch.py', 'true-sight-patch.py', 'semantic-pointer-patch.py'")
print('Integrated PotionSpoof, NoClip and TrueSight')
