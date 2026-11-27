#!/usr/bin/env python3
"""Transactional PingSpoof integration; --apply writes a fully validated plan."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod spammer;','pub mod spammer;\npub mod ping_spoof;','pure queue export')
 edit(E+'lib.rs','mod spammer_runtime;','mod spammer_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod ping_spoof_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub spammer: crate::spammer_runtime::SpammerRuntime,','    pub spammer: crate::spammer_runtime::SpammerRuntime,\n    pub ping_spoof: std::sync::Arc<crate::ping_spoof_runtime::PingSpoofService>,','field')
 edit(E+'behaviors.rs','            "Spammer",','            "Spammer",\n            "PingSpoof",','registry')
 edit(E+'behaviors.rs','            spammer: Default::default(),','            spammer: Default::default(),\n            ping_spoof: Default::default(),','default')
 edit(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.ping_spoof.set_running(self.query_running(registry,"PingSpoof"));','network running snapshot')
 edit(E+'config.rs','pub use spammer::{SpammerSetting,serialize_spammer};','pub use spammer::{SpammerSetting,serialize_spammer};\n#[path = "config/ping_spoof.rs"]\nmod ping_spoof;\npub use ping_spoof::{PingSpoofSetting,serialize_ping_spoof};','config export')
 edit(E+'config.rs','    Spammer => ("Spammer", "Misc", "spammer"),','    Spammer => ("Spammer", "Misc", "spammer"),\n    PingSpoof => ("PingSpoof", "Exploit", "pingSpoof"),','module kind')
 edit(E+'config.rs','    Spammer(SpammerSetting),','    Spammer(SpammerSetting),\n    PingSpoof(PingSpoofSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::Spammer','            | ModuleKind::Spammer\n            | ModuleKind::PingSpoof','dedicated serializer')
 edit(E+'config.rs','        ModuleKind::Spammer => spammer::parse_spammer(&mut patch,&root),','        ModuleKind::Spammer => spammer::parse_spammer(&mut patch,&root),\n        ModuleKind::PingSpoof => ping_spoof::parse_ping_spoof(&mut patch,&root),','parse dispatch')
 edit(E+'config.rs','        SettingChange::Spammer(value) => {','        SettingChange::PingSpoof(value) => { return ApplyResult::Runtime(RuntimeRequest::PingSpoof(*value)); }\n        SettingChange::Spammer(value) => {','runtime request')
 edit(E+'actor.rs','            ModuleKind::Spammer => config::serialize_spammer(&b.spammer.settings,&common,format,&describe),','            ModuleKind::Spammer => config::serialize_spammer(&b.spammer.settings,&common,format,&describe),\n            ModuleKind::PingSpoof => config::serialize_ping_spoof(&b.ping_spoof,&common,format,&describe),','serializer')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Spammer(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::PingSpoof(setting)) => { setting.apply(&self.state.behaviors.borrow().ping_spoof); }\n                ApplyResult::Runtime(RuntimeRequest::Spammer(setting)) => {','apply setting')
 edit(E+'runtime.rs','            pipeline.add_listener(-1000, modules.inventory_packets.clone());','            pipeline.add_listener(-1000, modules.inventory_packets.clone());\n            pipeline.add_listener(i32::MIN+1, modules.ping_spoof.clone());','final decision packet listener')
 edit(E+'runtime.rs','            modules.fast_use.cancel();','            modules.fast_use.cancel();\n            modules.ping_spoof.world_change();','world cleanup',2)
 edit(E+'runtime.rs','        self.spammer_tick(host);','        self.spammer_tick(host);\n        self.ping_spoof_tick(host);','real tick drain')
 edit(E+'runtime.rs','    fn client_disconnecting(&self, host: &dyn Host, _event: &mut ClientDisconnectingEvent) {','    fn client_disconnecting(&self, host: &dyn Host, _event: &mut ClientDisconnectingEvent) {\n        #[cfg(feature="behavior-hooks")]\n        self.state().behaviors.borrow().ping_spoof.world_change();','disconnect cleanup')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'spammer-patch.py',","'spammer-patch.py', 'ping-spoof-patch.py',",'native game helper')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} PingSpoof edits')
 if args.apply:p.write();print('Applied PingSpoof')
