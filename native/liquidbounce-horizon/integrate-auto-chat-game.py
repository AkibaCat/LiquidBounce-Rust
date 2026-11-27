#!/usr/bin/env python3
"""Transactionally publish AutoChatGame, using the real GAME_MESSAGE event."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod spammer;','pub mod spammer;\npub mod auto_chat_game;\npub mod auto_chat_game_wire;','model exports')
 edit(E+'lib.rs','mod spammer_runtime;','mod spammer_runtime;\n#[cfg(feature="behavior-hooks")]\nmod auto_chat_game_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub spammer: crate::spammer_runtime::SpammerRuntime,','    pub spammer: crate::spammer_runtime::SpammerRuntime,\n    pub auto_chat_game:crate::auto_chat_game_runtime::AutoChatGameRuntime,','field')
 edit(E+'behaviors.rs','            "Spammer",','            "Spammer",\n            "AutoChatGame",','registry')
 edit(E+'behaviors.rs','            spammer: Default::default(),','            spammer: Default::default(),\n            auto_chat_game:Default::default(),','default')
 edit(E+'behaviors.rs','        match callback {','        match callback {\n            ModuleCallback::OnEnabled(id) if self.query_ids.get("AutoChatGame")==Some(&id)=>{self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::AutoChatGameEnabled);}\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoChatGame")==Some(&id)=>self.auto_chat_game.cancel(),','lifecycle')
 edit(E+'config.rs','pub use spammer::{SpammerSetting,serialize_spammer};','pub use spammer::{SpammerSetting,serialize_spammer};\n#[path="config/auto_chat_game.rs"]\nmod auto_chat_game;\npub use auto_chat_game::{AutoChatGameSetting,serialize_auto_chat_game};','config export')
 edit(E+'config.rs','    Spammer => ("Spammer", "Misc", "spammer"),','    Spammer => ("Spammer", "Misc", "spammer"),\n    AutoChatGame => ("AutoChatGame", "Misc", "autoChatGame"),','module kind')
 edit(E+'config.rs','    Spammer(SpammerSetting),','    Spammer(SpammerSetting),\n    AutoChatGame(AutoChatGameSetting),','setting variants',2)
 edit(E+'config.rs','            | ModuleKind::Spammer','            | ModuleKind::Spammer\n            | ModuleKind::AutoChatGame','serializer kind')
 edit(E+'config.rs','        ModuleKind::Spammer => spammer::parse_spammer(&mut patch,&root),','        ModuleKind::Spammer => spammer::parse_spammer(&mut patch,&root),\n        ModuleKind::AutoChatGame=>auto_chat_game::parse_auto_chat_game(&mut patch,&root),','parse')
 edit(E+'config.rs','        SettingChange::Spammer(value) => {','        SettingChange::AutoChatGame(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoChatGame(value.clone())),\n        SettingChange::Spammer(value) => {','apply')
 edit(E+'actor.rs','            ModuleKind::Spammer => config::serialize_spammer(&b.spammer.settings,&common,format,&describe),','            ModuleKind::Spammer => config::serialize_spammer(&b.spammer.settings,&common,format,&describe),\n            ModuleKind::AutoChatGame=>config::serialize_auto_chat_game(&b.auto_chat_game.state.settings,&common,format,&describe),','serialize')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::Spammer(setting)) => {','                ApplyResult::Runtime(RuntimeRequest::AutoChatGame(setting))=>{if let Err(error)=setting.apply(&mut self.state.behaviors.borrow_mut().auto_chat_game.state.settings){eprintln!("[LiquidBounce] AutoChatGame setting: {error}");}}\n                ApplyResult::Runtime(RuntimeRequest::Spammer(setting)) => {','actor settings')
 edit(E+'runtime.rs','        self.spammer_tick(host);','        self.spammer_tick(host);\n        self.auto_chat_game_tick(host);','tick')
 edit(E+'runtime.rs','        self.plugins_game_message(host, event.message);','        self.auto_chat_game_message(event.message);\n        self.auto_chat_game_schedule(host);\n        self.plugins_game_message(host, event.message);','game message only')
 edit(E+'one_shots_runtime.rs','    AutoQueueEnabled,','    AutoQueueEnabled,\n    AutoChatGameEnabled,','enable effect')
 edit(E+'one_shots_runtime.rs','            OneShotEffect::AutoQueueEnabled=>self.auto_queue_enabled_info(host),','            OneShotEffect::AutoQueueEnabled=>self.auto_queue_enabled_info(host),\n            OneShotEffect::AutoChatGameEnabled=>self.auto_chat_game_enabled(host),','enable dispatch')
 edit('native/liquidbounce-horizon/engine/Cargo.toml','[dependencies]','[dependencies]\nureq = { version = "2", default-features = false, features = ["tls"] }\nurl = "2"','existing HTTP stack')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} AutoChatGame edits in {len(p.text)} files')
 if args.apply:p.write();print('Applied AutoChatGame')
