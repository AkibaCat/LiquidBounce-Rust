from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
E='native/liquidbounce-horizon/engine/src/'
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
 p=Plan()
 def edit(f,a,b,label,count=1):p.replace(f,a,b,label,count)
 edit('native/liquidbounce-behaviors/src/lib.rs','pub mod book_bot;','pub mod book_bot;\npub mod sneak;','behavior export')
 edit(E+'lib.rs','mod book_bot_runtime;','mod book_bot_runtime;\n#[cfg(feature = "behavior-hooks")]\nmod sneak_runtime;','runtime export')
 edit(E+'behaviors.rs','    pub book_bot: crate::book_bot_runtime::BookBotRuntime,','    pub book_bot: crate::book_bot_runtime::BookBotRuntime,\n    pub sneak: std::sync::Arc<crate::sneak_runtime::SneakService>,','field')
 edit(E+'behaviors.rs','            "BookBot",','            "BookBot",\n            "Sneak",','registry')
 edit(E+'behaviors.rs','            book_bot: Default::default(),','            book_bot: Default::default(),\n            sneak: Default::default(),','default')
 edit(E+'behaviors.rs','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {','    pub fn sync_packet_modules(&self, registry: &ModuleRegistry) {\n        self.sneak.set_running(self.query_running(registry,"Sneak"));','running publication')
 edit(E+'behaviors.rs','            ModuleCallback::OnEnabled(id) if self.query_ids.get("BookBot") == Some(&id) => {','            ModuleCallback::OnEnabled(id) if self.query_ids.get("Sneak") == Some(&id) => {if self.sneak.settings().protocol_warning(){self.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::SneakSwitchWarn);}}\n            ModuleCallback::OnEnabled(id) if self.query_ids.get("BookBot") == Some(&id) => {','switch enable')
 edit(E+'config.rs','pub use book_bot::{BookBotSetting,serialize_book_bot};','pub use book_bot::{BookBotSetting,serialize_book_bot};\n#[path="config/sneak.rs"]\nmod sneak;\npub use sneak::{SneakSetting,serialize_sneak};','config export')
 edit(E+'config.rs','    BookBot => ("BookBot", "Exploit", "bookBot"),','    BookBot => ("BookBot", "Exploit", "bookBot"),\n    Sneak => ("Sneak", "Movement", "sneak"),','module kind')
 edit(E+'config.rs','    BookBot(BookBotSetting),','    BookBot(BookBotSetting),\n    Sneak(SneakSetting),','variants',2)
 edit(E+'config.rs','            | ModuleKind::BookBot','            | ModuleKind::BookBot\n            | ModuleKind::Sneak','serializer routing')
 edit(E+'config.rs','        ModuleKind::BookBot => book_bot::parse_book_bot(&mut patch,&root),','        ModuleKind::BookBot => book_bot::parse_book_bot(&mut patch,&root),\n        ModuleKind::Sneak => sneak::parse_sneak(&mut patch,&root),','parser')
 edit(E+'config.rs','        SettingChange::BookBot(value) => {','        SettingChange::Sneak(value) => {return ApplyResult::Runtime(RuntimeRequest::Sneak(*value));}\n        SettingChange::BookBot(value) => {','apply dispatch')
 edit(E+'actor.rs','            ModuleKind::BookBot => config::serialize_book_bot(&b.book_bot.settings,&common,format,&describe),','            ModuleKind::BookBot => config::serialize_book_bot(&b.book_bot.settings,&common,format,&describe),\n            ModuleKind::Sneak => config::serialize_sneak(&b.sneak.settings(),&common,format,&describe),','serialization')
 edit(E+'actor.rs','                ApplyResult::Runtime(RuntimeRequest::BookBot(setting)) => {','''                ApplyResult::Runtime(RuntimeRequest::Sneak(setting)) => {
                    let mut b=self.state.behaviors.borrow_mut();let old=b.sneak.settings().mode;
                    setting.apply(&mut b.sneak.settings.lock().expect("Sneak settings"));
                    if old!=b.sneak.settings().mode&&b.sneak.settings().protocol_warning()&&b.query_running(&self.state.registry,"Sneak"){b.pending_one_shots.push(crate::one_shots_runtime::OneShotEffect::SneakSwitchWarn);}
                }
                ApplyResult::Runtime(RuntimeRequest::BookBot(setting)) => {''','mode lifecycle')
 edit(E+'actor.rs','            ModuleKind::BookBot => Some(if self.state.behaviors.borrow().book_bot.settings.mode==0 {"Random"}else{"File"}),','            ModuleKind::BookBot => Some(if self.state.behaviors.borrow().book_bot.settings.mode==0 {"Random"}else{"File"}),\n            ModuleKind::Sneak => Some(self.state.behaviors.borrow().sneak.settings().tag()),','mode tag')
 edit(E+'runtime.rs','            pipeline.add_listener(0, modules.anti_hunger.clone());','            pipeline.add_listener(0, modules.anti_hunger.clone());\n            pipeline.add_listener(0,modules.sneak.clone());','real ordinary packets')
 edit(E+'runtime.rs','    fn client_directional_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {','    fn client_directional_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {\n        self.sneak_input(host,event);','legit event')
 edit(E+'runtime.rs','    fn client_sprint_queries(&self, host: &dyn Host, event: &mut ClientSprintQueriesEvent) {','    fn client_sprint_queries(&self, host: &dyn Host, event: &mut ClientSprintQueriesEvent) {\n        self.state().behaviors.borrow().sneak.moving.store(event.sideways!=0.0||event.forward!=0.0,std::sync::atomic::Ordering::Release);','network movement context')
 edit(E+'one_shots_runtime.rs','    BookBotEnabled,','    BookBotEnabled,\n    SneakSwitchWarn,','deferred notification')
 edit(E+'one_shots_runtime.rs','            OneShotEffect::OldRegenEnabled => self.old_regen_enabled(host),','            OneShotEffect::OldRegenEnabled => self.old_regen_enabled(host),\n            OneShotEffect::SneakSwitchWarn => self.sneak_protocol_warning(),','notification outside registry borrow')
 edit('native/liquidbounce-horizon/make-behavior-patch.py',"'book-bot-patch.py',","'book-bot-patch.py', 'sneak-patch.py',",'native helpers')
 return p
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--apply',action='store_true');a=parser.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Sneak edits')
 if a.apply:p.write();print('Applied Sneak')
