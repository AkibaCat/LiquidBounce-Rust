#!/usr/bin/env python3
"""Transactional Clip wiring. --apply commits the validated changes."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod ping_spoof;','pub mod ping_spoof;\npub mod clip;','behavior export')
 edit(E+'lib.rs','mod ping_spoof_runtime;','mod ping_spoof_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod clip_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub spammer: crate::spammer_runtime::SpammerRuntime,','    pub spammer: crate::spammer_runtime::SpammerRuntime,\n    pub clip: liquidbounce_behaviors::clip::Clip,','field')
 edit(E+'behaviors.rs','            "Spammer",','            "Spammer",\n            "Clip",','registry')
 edit(E+'behaviors.rs','            spammer: Default::default(),','            spammer: Default::default(),\n            clip: Default::default(),','default')
 edit(E+'behaviors.rs','            ModuleCallback::OnEnabled(id) if self.query_ids.get("Damage") == Some(&id) => {','            ModuleCallback::OnEnabled(id) if self.query_ids.get("Clip") == Some(&id) => {\n                self.clip.cancel();\n                if self.clip.mode==1 {self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::Clip(self.clip));}\n            }\n            ModuleCallback::CancelSequences(id) | ModuleCallback::OnDisabled(id) if self.query_ids.get("Clip") == Some(&id) => {self.clip.cancel();}\n            ModuleCallback::OnEnabled(id) if self.query_ids.get("Damage") == Some(&id) => {','mode lifecycle')
 edit(E+'config.rs','pub use ping_spoof::{PingSpoofSetting,serialize_ping_spoof};','pub use ping_spoof::{PingSpoofSetting,serialize_ping_spoof};\n#[path = "config/clip.rs"]\nmod clip;\npub use clip::{ClipSetting,serialize_clip};','config export')
 edit(E+'config.rs','    PingSpoof => ("PingSpoof", "Exploit", "pingSpoof"),','    PingSpoof => ("PingSpoof", "Exploit", "pingSpoof"),\n    Clip => ("Clip", "Movement", "clip"),','module kind')
 edit(E+'config.rs','    PingSpoof(PingSpoofSetting),','    PingSpoof(PingSpoofSetting),\n    Clip(ClipSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::PingSpoof','            | ModuleKind::PingSpoof\n            | ModuleKind::Clip','dedicated serializer')
 edit(E+'config.rs','        ModuleKind::PingSpoof => ping_spoof::parse_ping_spoof(&mut patch,&root),','        ModuleKind::PingSpoof => ping_spoof::parse_ping_spoof(&mut patch,&root),\n        ModuleKind::Clip => clip::parse_clip(&mut patch,&root),','parser')
 edit(E+'config.rs','        SettingChange::PingSpoof(value) => {','        SettingChange::Clip(value) => {return ApplyResult::Runtime(RuntimeRequest::Clip(*value));}\n        SettingChange::PingSpoof(value) => {','runtime settings')
 edit(E+'actor.rs','            ModuleKind::PingSpoof => config::serialize_ping_spoof(&b.ping_spoof,&common,format,&describe),','            ModuleKind::PingSpoof => config::serialize_ping_spoof(&b.ping_spoof,&common,format,&describe),\n            ModuleKind::Clip => config::serialize_clip(&b.clip,&common,format,&describe),','serializer')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::PingSpoof(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::Clip(setting)) => {let mut b=self.state.behaviors.borrow_mut();let old=b.clip.mode;setting.apply(&mut b.clip);if old!=b.clip.mode&&b.clip.mode==1&&b.query_running(&self.state.registry,"Clip"){let settings=b.clip;b.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::Clip(settings));}}\n                ApplyResult::Runtime(RuntimeRequest::PingSpoof(setting)) => {','mode enable callback')
 edit(E+'runtime.rs','        self.ping_spoof_tick(host);','        self.ping_spoof_tick(host);\n        self.clip_tick(host);','real tick')
 edit(E+'runtime.rs','                WaitOwner::FastUse => self.fast_use_callback(host, in_game, task.callback),','                WaitOwner::FastUse => self.fast_use_callback(host, in_game, task.callback),\n                WaitOwner::Clip => self.clip_callback(host, in_game, task.callback),','FIFO coroutine')
 edit(E+'runtime.rs','            self.smart_eat_render(host, event.batch);','            self.smart_eat_render(host, event.batch);\n            self.clip_render(host,event.batch);','actual hotbar overlay')
 edit(E+'runtime.rs','            modules.ping_spoof.world_change();','            modules.ping_spoof.world_change();\n            modules.clip.cancel();','world cleanup',2)
 edit(E+'coroutine_ticker.rs','    FastUse,','    FastUse,\n    Clip,','wait owner')
 edit(E+'one_shots_runtime.rs','    Damage(liquidbounce_behaviors::damage::Damage),','    Damage(liquidbounce_behaviors::damage::Damage),\n    Clip(liquidbounce_behaviors::clip::Clip),','oneshot effect')
 edit(E+'one_shots_runtime.rs','            OneShotEffect::Damage(settings) => crate::damage_runtime::run(host, settings),','            OneShotEffect::Damage(settings) => crate::damage_runtime::run(host, settings),\n            OneShotEffect::Clip(settings) => self.clip_old(host,settings),','immediate old effect')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'ping-spoof-patch.py',","'ping-spoof-patch.py', 'clip-patch.py',",'native host helpers')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Clip edits')
 if args.apply:p.write();print('Applied Clip')
