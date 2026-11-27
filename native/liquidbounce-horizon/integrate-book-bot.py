#!/usr/bin/env python3
"""Transactional BookBot registry, settings, InventoryManager and native wiring."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod clip;','pub mod clip;\npub mod book_bot;','behavior export')
 edit(E+'lib.rs','mod clip_runtime;','mod clip_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod book_bot_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub clip: liquidbounce_behaviors::clip::Clip,','    pub clip: liquidbounce_behaviors::clip::Clip,\n    pub book_bot: crate::book_bot_runtime::BookBotRuntime,','field')
 edit(E+'behaviors.rs','            "Clip",','            "Clip",\n            "BookBot",','registry')
 edit(E+'behaviors.rs','            clip: Default::default(),','            clip: Default::default(),\n            book_bot: Default::default(),','default')
 edit(E+'behaviors.rs','if name == "Freeze" { definition.disable_on_quit = true; }','if matches!(name,"Freeze"|"BookBot") { definition.disable_on_quit = true; }','disable on quit')
 edit(E+'behaviors.rs','            ModuleCallback::OnEnabled(id) if self.query_ids.get("Clip") == Some(&id) => {','''            ModuleCallback::OnEnabled(id) if self.query_ids.get("BookBot") == Some(&id) => {
                self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::BookBotEnabled);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("BookBot") == Some(&id) => {
                self.inventory.manager.cancel_pending(crate::book_bot_runtime::OWNER);
            }
            ModuleCallback::OnEnabled(id) if self.query_ids.get("Clip") == Some(&id) => {''','lifecycle')
 edit(E+'config.rs','pub use clip::{ClipSetting,serialize_clip};','pub use clip::{ClipSetting,serialize_clip};\n#[path = "config/book_bot.rs"]\nmod book_bot;\npub use book_bot::{BookBotSetting,serialize_book_bot};','config export')
 edit(E+'config.rs','    Clip => ("Clip", "Movement", "clip"),','    Clip => ("Clip", "Movement", "clip"),\n    BookBot => ("BookBot", "Exploit", "bookBot"),','module kind')
 edit(E+'config.rs','    Clip(ClipSetting),','    Clip(ClipSetting),\n    BookBot(BookBotSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::Clip','            | ModuleKind::Clip\n            | ModuleKind::BookBot','dedicated serializer')
 edit(E+'config.rs','        ModuleKind::Clip => clip::parse_clip(&mut patch,&root),','        ModuleKind::Clip => clip::parse_clip(&mut patch,&root),\n        ModuleKind::BookBot => book_bot::parse_book_bot(&mut patch,&root),','parser')
 edit(E+'config.rs','        SettingChange::Clip(value) => {','        SettingChange::BookBot(value) => {return ApplyResult::Runtime(RuntimeRequest::BookBot(value.clone()));}\n        SettingChange::Clip(value) => {','runtime settings')
 edit(E+'actor.rs','            ModuleKind::Clip => config::serialize_clip(&b.clip,&common,format,&describe),','            ModuleKind::Clip => config::serialize_clip(&b.clip,&common,format,&describe),\n            ModuleKind::BookBot => config::serialize_book_bot(&b.book_bot.settings,&common,format,&describe),','serializer')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::PingSpoof(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::BookBot(setting)) => {
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().book_bot.settings){eprintln!("[LiquidBounce] BookBot config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::PingSpoof(setting)) => {''','apply settings')
 edit(E+'actor.rs','            ModuleKind::FastUse => Some(self.state.behaviors.borrow().fast_use.settings.mode.tag()),','            ModuleKind::BookBot => Some(if self.state.behaviors.borrow().book_bot.settings.mode==0 {"Random"}else{"File"}),\n            ModuleKind::FastUse => Some(self.state.behaviors.borrow().fast_use.settings.mode.tag()),','source mode tag')
 edit(E+'one_shots_runtime.rs','    OldRegenEnabled,','    OldRegenEnabled,\n    BookBotEnabled,','enabled effect')
 edit(E+'one_shots_runtime.rs','            OneShotEffect::OldRegenEnabled => self.old_regen_enabled(host),','            OneShotEffect::OldRegenEnabled => self.old_regen_enabled(host),\n            OneShotEffect::BookBotEnabled => {let now=host.client_auto_tool_time();self.state().behaviors.borrow_mut().book_bot.enabled(now);},','actual enable clock')
 edit(E+'inventory.rs','    fn id(&mut self) -> u64 {','    pub(crate) fn id(&mut self) -> u64 {','shared plan allocation')
 edit(E+'inventory_runtime.rs','        modules.auto_tool.swaps.cancel_plan();','        modules.auto_tool.swaps.cancel_plan();\n        modules.book_bot.pending=None;','world cleanup')
 edit(E+'inventory_runtime.rs','        if disable_elytra_swap {','        self.book_bot_plan(host,&snapshot.context);\n        if disable_elytra_swap {','real inventory-plan tick')
 edit(E+'inventory_runtime.rs','            let emission = {','            let book_bot_selected=host.client_book_bot_selected();\n            let emission = {','live guard snapshot')
 edit(E+'inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.book_bot.owner(modules.query_running(&state.registry,"BookBot"),book_bot_selected));\n                modules.inventory.observe_context(&context);','owner constraints')
 edit(E+'inventory_runtime.rs','                    if disable {','''                    let book_completed={let state=self.state();let mut b=state.behaviors.borrow_mut();let b=&mut *b;b.book_bot.outcome(&mut b.inventory,outcome)};
                    if book_completed{self.book_bot_write_due(host);}
                    if disable {''','unborrowed completion tail')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'clip-patch.py',","'clip-patch.py', 'book-bot-patch.py',",'native host helpers')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=build_plan();print(f'Validated {len(p.steps)} BookBot edits')
 if args.apply:p.write();print('Applied BookBot')
