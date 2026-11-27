#!/usr/bin/env python3
"""One-time actual registry/config to skin policy wiring."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def edit(path,old,new,count=1):
 p=ROOT/path;s=p.read_text(encoding='utf8')
 if s.count(old)!=count:raise ValueError(f'{path}: expected {count} copies of {old[:70]!r}, got {s.count(old)}')
 p.write_text(s.replace(old,new),encoding='utf8')
E='native/liquidbounce-horizon/engine/src/'
edit(E+'behaviors.rs','    pub kick_mode:','    pub yggdrasil_force_slim: bool,\n    pub kick_mode:')
edit(E+'behaviors.rs','            "Kick",','            "Kick",\n            "YggdrasilSignatureFix",')
edit(E+'behaviors.rs','            kick_mode: Default::default(),','            yggdrasil_force_slim: true,\n            kick_mode: Default::default(),')
edit(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        horizon_client::skins::liquidbounce_yggdrasil_policy(self.query_running(registry,"YggdrasilSignatureFix"),self.yggdrasil_force_slim);')
edit(E+'config.rs','mod one_shots;','mod one_shots;\n#[path = "config/yggdrasil_signature.rs"]\nmod yggdrasil_signature;\npub use yggdrasil_signature::serialize_yggdrasil_signature;')
edit(E+'config.rs','    NameCollector => ("NameCollector", "Exploit", "nameCollector"),','    NameCollector => ("NameCollector", "Exploit", "nameCollector"),\n    YggdrasilSignatureFix => ("YggdrasilSignatureFix", "Exploit", "yggdrasilSignatureFix"),')
edit(E+'config.rs','    OneShot(OneShotSetting),','    OneShot(OneShotSetting),\n    YggdrasilSignatureFix(bool),',2)
edit(E+'config.rs','            | ModuleKind::Kick','            | ModuleKind::YggdrasilSignatureFix\n            | ModuleKind::Kick')
edit(E+'config.rs','        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),','        ModuleKind::ResetVl => reset_vl::parse_reset_vl(&mut patch, &root),\n        ModuleKind::YggdrasilSignatureFix => yggdrasil_signature::parse_yggdrasil_signature(&mut patch, &root),')
edit(E+'config.rs','        SettingChange::ResetVl(value) => {','        SettingChange::YggdrasilSignatureFix(value) => { return ApplyResult::Runtime(RuntimeRequest::YggdrasilSignatureFix(*value)); }\n        SettingChange::ResetVl(value) => {')
edit(E+'actor.rs','            ModuleKind::ResetVl => {','            ModuleKind::YggdrasilSignatureFix => config::serialize_yggdrasil_signature(b.yggdrasil_force_slim,&common,format,&describe),\n            ModuleKind::ResetVl => {')
edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::YggdrasilSignatureFix(value)) => {
                    let mut modules = self.state.behaviors.borrow_mut();
                    modules.yggdrasil_force_slim=value;
                    modules.sync_packet_modules(&self.state.registry);
                }
                ApplyResult::Runtime(RuntimeRequest::ResetVl(setting)) => {''')
edit('native/liquidbounce-horizon/make-behavior-patch.py',"'one-shots-patch.py',","'one-shots-patch.py', 'yggdrasil-signature-patch.py',")
print('Integrated YggdrasilSignatureFix actual skins consumer')
