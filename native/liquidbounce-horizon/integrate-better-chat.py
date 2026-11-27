#!/usr/bin/env python3
"""Idempotent transactional BetterChat module/global-translation wiring; --check is read-only."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-block-esp.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('../../../liquidbounce-behaviors/src/lib.rs','pub mod nametags;','pub mod nametags;\npub mod better_chat;')
 p.edit('lib.rs','mod nametags_runtime;','mod nametags_runtime;\n#[cfg(feature="behavior-hooks")]\nmod better_chat_runtime;')
 p.edit('behaviors.rs','    pub nametags:liquidbounce_behaviors::nametags::Nametags,','    pub nametags:liquidbounce_behaviors::nametags::Nametags,\n    pub better_chat:crate::better_chat_runtime::BetterChatRuntime,')
 p.edit('behaviors.rs','            nametags:Default::default(),','            nametags:Default::default(),\n            better_chat:Default::default(),')
 p.edit('behaviors.rs','            "Nametags",','            "Nametags",\n            "BetterChat",')
 p.edit('behaviors.rs','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Nametags")==Some(&id)=>self.nametags.clear(),','            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("Nametags")==Some(&id)=>self.nametags.clear(),\n            ModuleCallback::OnDisabled(id)|ModuleCallback::CancelSequences(id) if self.query_ids.get("BetterChat")==Some(&id)=>self.better_chat.cancel(),')
 p.edit('config.rs','pub use nametags::{NametagsSetting,serialize_nametags};','pub use nametags::{NametagsSetting,serialize_nametags};\n#[path="config/better_chat.rs"]\nmod better_chat;\npub use better_chat::{BetterChatSetting,serialize_better_chat,serialize_global_translate,apply_global_translate,global_translate_changes};')
 p.edit('config.rs','    Nametags=>("Nametags","Render","nametags"),','    Nametags=>("Nametags","Render","nametags"),\n    BetterChat=>("BetterChat","Render","betterChat"),')
 p.edit('config.rs','"ContainerStealer"=>Some(Self::ChestStealer),','"ContainerStealer"=>Some(Self::ChestStealer), "AntiSpam"=>Some(Self::BetterChat),')
 p.edit('config.rs','    Nametags(NametagsSetting),','    Nametags(NametagsSetting),\n    BetterChat(BetterChatSetting),',2)
 p.edit('config.rs','            | ModuleKind::Nametags','            | ModuleKind::Nametags\n            | ModuleKind::BetterChat')
 p.edit('config.rs','        ModuleKind::Nametags=>nametags::parse_nametags(&mut patch,&root),','        ModuleKind::Nametags=>nametags::parse_nametags(&mut patch,&root),\n        ModuleKind::BetterChat=>better_chat::parse_better_chat(&mut patch,&root),')
 p.edit('config.rs','        SettingChange::Nametags(v)=>return ApplyResult::Runtime(RuntimeRequest::Nametags(v.clone())),','        SettingChange::Nametags(v)=>return ApplyResult::Runtime(RuntimeRequest::Nametags(v.clone())),\n        SettingChange::BetterChat(v)=>return ApplyResult::Runtime(RuntimeRequest::BetterChat(v.clone())),')
 p.edit('actor.rs','            ModuleKind::Nametags=>config::serialize_nametags(&b.nametags.settings,&common,format,&describe),','            ModuleKind::Nametags=>config::serialize_nametags(&b.nametags.settings,&common,format,&describe),\n            ModuleKind::BetterChat=>config::serialize_better_chat(&b.better_chat.settings,&common,format,&describe),')
 p.edit('actor.rs','                ApplyResult::Runtime(RuntimeRequest::Nametags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().nametags.settings,&[]){eprintln!("[LiquidBounce] Nametags config: {e}");}},','                ApplyResult::Runtime(RuntimeRequest::Nametags(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().nametags.settings,&[]){eprintln!("[LiquidBounce] Nametags config: {e}");}},\n                ApplyResult::Runtime(RuntimeRequest::BetterChat(v))=>{if let Err(e)=v.apply(&mut self.state.behaviors.borrow_mut().better_chat.settings){eprintln!("[LiquidBounce] BetterChat config: {e}");}},')
 p.edit('runtime.rs','    #[cfg(feature="behavior-hooks")]\n    fn client_nametags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientNametagsEvent<\'_>){self.nametags(event);}','    #[cfg(feature="behavior-hooks")]\n    fn client_nametags(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientNametagsEvent<\'_>){self.nametags(event);}\n    #[cfg(feature="behavior-hooks")]\n    fn client_better_chat(&self,_host:&dyn Host,event:&mut horizon_client::mod_api::ClientBetterChatEvent<\'_>){self.better_chat(event);}')
 # The global provider tree is persisted separately from module settings and
 # respects the same shared settings API, notifications and language switch.
 for mode,describe in [('Interop','&describe'),('File','&str::to_owned')]:
  anchor=f'        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_client_commands(&self.state.behaviors.borrow().client_commands,JsonFormat::{mode},{describe}));'
  p.edit('actor.rs',anchor,anchor+f'\n        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::{mode},{describe}));')
 anchor='        let before = self.global_settings();'
 addition='''        let before = self.global_settings();
        {
            let old=config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::Interop,&str::to_owned);
            for issue in config::apply_global_translate(&mut self.state.behaviors.borrow_mut().better_chat.translation,value){eprintln!("[LiquidBounce] global translator setting skipped: {issue}");}
            self.state.behaviors.borrow_mut().better_chat.provider_changed();
            let language=horizon_client::lang::code();
            let new=config::serialize_global_translate(&self.state.behaviors.borrow().better_chat.translation,JsonFormat::Interop,&|k|liquidbounce_interop::localization::describe(&language,k));
            for leaf in config::global_translate_changes(&old,&new){self.state.events.borrow_mut().push(("valueChanged".into(),json!({"value":leaf})));}
        }'''
 p.edit('actor.rs',anchor,addition)
 p.edit('actor.rs','| "CommandManager" | "Commands")','| "CommandManager" | "Commands" | "AutoTranslate")',2)
 p.edit('../../make-behavior-patch.py',"'step-patch.py'):","'better-chat-patch.py', 'step-patch.py'):")
 return p
if __name__=='__main__':
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--check',action='store_true');args=ap.parse_args();p=prepare()
 if not args.check:p.commit()
 print(f"BetterChat {'validated'if args.check else'integrated'}: {len(p.changed)} shared files")
