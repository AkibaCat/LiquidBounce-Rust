#!/usr/bin/env python3
"""Transactional ChestCleaner wiring, complete settings and shared inventory executor."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def prepare():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod book_bot;','pub mod book_bot;\npub mod chest_cleaner;','behavior export')
 edit(E+'lib.rs','mod book_bot_runtime;','mod book_bot_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod chest_cleaner_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub book_bot: crate::book_bot_runtime::BookBotRuntime,','    pub book_bot: crate::book_bot_runtime::BookBotRuntime,\n    pub chest_cleaner:crate::chest_cleaner_runtime::ChestCleanerRuntime,','state')
 edit(E+'behaviors.rs','            "BookBot",','            "BookBot",\n            "ChestCleaner",','registry')
 edit(E+'behaviors.rs','            book_bot: Default::default(),','            book_bot: Default::default(),\n            chest_cleaner:Default::default(),','default')
 edit(E+'behaviors.rs','            ModuleCallback::CancelSequences(id) if self.query_ids.get("BookBot") == Some(&id) => {','''            ModuleCallback::CancelSequences(id) if self.query_ids.get("ChestCleaner") == Some(&id) => {
                self.inventory.manager.cancel_pending(crate::chest_cleaner_runtime::OWNER);
            }
            ModuleCallback::CancelSequences(id) if self.query_ids.get("BookBot") == Some(&id) => {''','cancellation')
 edit(E+'config.rs','pub use book_bot::{BookBotSetting,serialize_book_bot};','pub use book_bot::{BookBotSetting,serialize_book_bot};\n#[path="config/chest_cleaner.rs"]\nmod chest_cleaner;\npub use chest_cleaner::{ChestCleanerSetting,serialize_chest_cleaner};','config export')
 edit(E+'config.rs','    BookBot => ("BookBot", "Exploit", "bookBot"),','    BookBot => ("BookBot", "Exploit", "bookBot"),\n    ChestCleaner => ("ChestCleaner","Player","chestCleaner"),','kind')
 edit(E+'config.rs','"ChestSwap" => Some(Self::ElytraSwap),','"ChestSwap" => Some(Self::ElytraSwap), "ContainerCleaner" => Some(Self::ChestCleaner),','source alias')
 edit(E+'config.rs','    BookBot(BookBotSetting),','    BookBot(BookBotSetting),\n    ChestCleaner(ChestCleanerSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::BookBot','            | ModuleKind::BookBot\n            | ModuleKind::ChestCleaner','serializer exemption')
 edit(E+'config.rs','        ModuleKind::BookBot => book_bot::parse_book_bot(&mut patch,&root),','        ModuleKind::BookBot => book_bot::parse_book_bot(&mut patch,&root),\n        ModuleKind::ChestCleaner => chest_cleaner::parse_chest_cleaner(&mut patch,&root),','parser')
 edit(E+'config.rs','        SettingChange::BookBot(value) => {','        SettingChange::ChestCleaner(value) => {return ApplyResult::Runtime(RuntimeRequest::ChestCleaner(value.clone()));}\n        SettingChange::BookBot(value) => {','settings dispatch')
 edit(E+'actor.rs','            ModuleKind::BookBot => config::serialize_book_bot(&b.book_bot.settings,&common,format,&describe),','            ModuleKind::BookBot => config::serialize_book_bot(&b.book_bot.settings,&common,format,&describe),\n            ModuleKind::ChestCleaner => config::serialize_chest_cleaner(&b.chest_cleaner.settings,&common,format,&describe),','tree')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::BookBot(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::ChestCleaner(setting)) => {
                    if let Err(e)=setting.apply(&mut self.state.behaviors.borrow_mut().chest_cleaner.settings,&self.state.item_registry){eprintln!("[LiquidBounce] ChestCleaner config: {e}");}
                }
                ApplyResult::Runtime(RuntimeRequest::BookBot(setting)) => {''','config apply')
 edit(E+'inventory_runtime.rs','        modules.book_bot.pending=None;','        modules.book_bot.pending=None;\n        modules.chest_cleaner.clear();','world cleanup')
 edit(E+'inventory_runtime.rs','        self.book_bot_plan(host,&snapshot.context);','        self.book_bot_plan(host,&snapshot.context);\n        self.chest_cleaner_plan(host,&snapshot.context);','actual GameTick')
 edit(E+'inventory_runtime.rs','                modules.inventory.observe_context(&context);','                owners.push(modules.chest_cleaner.owner(modules.query_running(&state.registry,"ChestCleaner")));\n                modules.inventory.observe_context(&context);','live owner')
 edit(E+'inventory_runtime.rs','                    if book_completed{self.book_bot_write_due(host);}','''                    if book_completed{self.book_bot_write_due(host);}
                    {let state=self.state();let mut b=state.behaviors.borrow_mut();let b=&mut *b;b.chest_cleaner.outcome(&mut b.inventory,outcome,&context);}''','per-perform continuation')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'gui-closer-patch.py',","'gui-closer-patch.py', 'chest-cleaner-patch.py',",'native helper')
 edit(E+'browser.rs','        if request.method == Method::GET\n            && matches!(\n                request.path.as_str(),\n                "/api/v1/client/registry/items"','''        if request.method==Method::GET && request.path=="/api/v1/client/registry/menu" {
            let entries:serde_json::Map<_,_>=liquidbounce_behaviors::chest_cleaner::MENUS.iter().map(|menu|{
                let path=match *menu {"generic_9x6"=>"chestDouble","generic_9x1"|"generic_9x2"|"generic_9x3"|"generic_9x4"|"generic_9x5"=>"chest","shulker_box"=>"shulkerBox","generic_3x3"=>"dispenser","brewing_stand"=>"brewing",other=>other};
                let key=format!("container.{path}");let name=horizon_client::lang::get(&key);
                (format!("minecraft:{menu}"),json!({"name":if name==key{menu.to_string()}else{name},"icon":null}))
            }).collect();
            return ApiResponse::json(Value::Object(entries));
        }
        if request.method == Method::GET
            && matches!(
                request.path.as_str(),
                "/api/v1/client/registry/items"''','actual menu registry endpoint')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--apply',action='store_true');args=a.parse_args();p=prepare();print(f'Validated {len(p.steps)} ChestCleaner edits')
 if args.apply:p.write();print('Applied ChestCleaner')
