#!/usr/bin/env python3
"""Validate Macros and the shared command seam; integration owner applies once."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('macros_shared',HERE/'integrate-super-knockback.py');shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
E='native/liquidbounce-horizon/engine/src/'
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod auto_queue;\n','pub mod auto_queue;\npub mod macros;\npub mod client_commands;\n','behavior and lexer exports')
    p.replace(E+'lib.rs','mod auto_queue_runtime;\n','mod auto_queue_runtime;\n#[cfg(feature="behavior-hooks")]\nmod macros_runtime;\n#[cfg(feature="behavior-hooks")]\nmod client_commands_runtime;\n','runtime exports')
    b=E+'behaviors.rs'
    p.replace(b,'    pub auto_queue:liquidbounce_behaviors::auto_queue::State,\n','    pub auto_queue:liquidbounce_behaviors::auto_queue::State,\n    pub macros:liquidbounce_behaviors::macros::State,\n    pub client_commands:liquidbounce_behaviors::client_commands::Settings,\n','module and shared command state')
    p.replace(b,'            "AutoQueue",\n','            "AutoQueue",\n            "Macros",\n','registration')
    p.replace(b,'            auto_queue:Default::default(),\n','            auto_queue:Default::default(),\n            macros:Default::default(),\n            client_commands:Default::default(),\n','defaults')
    anchor='            ModuleCallback::CancelSequences(id) if self.query_ids.get("AutoQueue")==Some(&id)=>{self.auto_queue.cancel();}\n'
    p.replace(b,anchor,anchor+'            ModuleCallback::OnDisabled(id) if self.query_ids.get("Macros")==Some(&id)=>{self.pending_slot_resets.push(crate::macros_runtime::REQUESTER);}\n            ModuleCallback::CancelSequences(id) if self.query_ids.get("Macros")==Some(&id)=>{self.macros.cancel();}\n','source requester reset and child cancellation')
    c=E+'config.rs'
    anchor='pub use auto_queue::{AutoQueueSetting,serialize_auto_queue};\n'
    p.replace(c,anchor,anchor+'#[path="config/macros.rs"]\nmod macros;\npub use macros::{MacrosSetting,serialize_macros};\n#[path="config/client_commands.rs"]\nmod client_commands;\npub use client_commands::{serialize_client_commands,apply_client_commands};\n','module and global config exports')
    p.replace(c,'    AutoQueue => ("AutoQueue", "Player", "autoQueue"),\n','    AutoQueue => ("AutoQueue", "Player", "autoQueue"),\n    Macros => ("Macros", "Misc", "macros"),\n','source category')
    p.replace(c,'    AutoQueue(AutoQueueSetting),\n','    AutoQueue(AutoQueueSetting),\n    Macros(MacrosSetting),\n','setting and runtime variants',count=2)
    p.replace(c,'            | ModuleKind::AutoQueue\n','            | ModuleKind::AutoQueue\n            | ModuleKind::Macros\n','dedicated tree')
    p.replace(c,'        ModuleKind::AutoQueue=>auto_queue::parse_auto_queue(&mut patch,&root),\n','        ModuleKind::AutoQueue=>auto_queue::parse_auto_queue(&mut patch,&root),\n        ModuleKind::Macros=>macros::parse_macros(&mut patch,&root),\n','recursive settings parse')
    anchor='        SettingChange::AutoQueue(value)=>return ApplyResult::Runtime(RuntimeRequest::AutoQueue(value.clone())),\n'
    p.replace(c,anchor,anchor+'        SettingChange::Macros(value)=>return ApplyResult::Runtime(RuntimeRequest::Macros(value.clone())),\n','runtime request')
    a=E+'actor.rs'
    anchor='            ModuleKind::AutoQueue=>config::serialize_auto_queue(&b.auto_queue,&common,format,&describe),\n'
    p.replace(a,anchor,anchor+'            ModuleKind::Macros=>config::serialize_macros(&b.macros,&common,format,&describe),\n','full source setting tree')
    anchor='                ApplyResult::Runtime(RuntimeRequest::AutoQueue(setting))=>{\n'
    p.replace(a,anchor,'                ApplyResult::Runtime(RuntimeRequest::Macros(setting))=>{\n                    if let Err(error)=setting.clone().apply(&mut self.state.behaviors.borrow_mut().macros,&self.state.item_registry){eprintln!("[LiquidBounce] Macros config leaf skipped: {error}");}\n                }\n'+anchor,'live settings apply')
    p.section(a,'    pub fn global_settings(&self) -> Value {','    pub fn set_global_settings(', '        tree\n','        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_client_commands(&self.state.behaviors.borrow().client_commands,JsonFormat::Interop,&describe));\n        tree\n','global interop tree')
    p.section(a,'    pub fn global_file(&self) -> Value {','    pub fn save_global(', '        tree\n','        tree["value"].as_array_mut().expect("Settings array").push(config::serialize_client_commands(&self.state.behaviors.borrow().client_commands,JsonFormat::File,&str::to_owned));\n        tree\n','global file tree')
    anchor='        let before = self.global_settings();\n'
    p.section(a,'    pub fn set_global_settings(','    pub fn global_file(',anchor,anchor+'''        for issue in config::apply_client_commands(&mut self.state.behaviors.borrow_mut().client_commands,value){eprintln!("[LiquidBounce] global config leaf skipped: {issue}");}
        let after=self.global_settings();
        for name in ["Prefix","HintCount"]{
            let find=|tree:&Value|tree["value"].as_array().and_then(|groups|groups.iter().find(|group|group["name"]=="CommandManager")).and_then(|group|group["value"].as_array()).and_then(|values|values.iter().find(|leaf|leaf["name"]==name)).cloned();
            if find(&before)!=find(&after){if let Some(leaf)=find(&after){self.state.events.borrow_mut().push(("valueChanged".into(),json!({"value":leaf})));}}
        }
''','global live prefix and independent leaf events')
    p.replace(a,'matches!(name, "Targets" | "Enemies" | "Inventory" | "BlinkManager")','matches!(name, "Targets" | "Enemies" | "Inventory" | "BlinkManager" | "CommandManager" | "Commands")','exclude preserved live command settings')
    p.replace(a,'Some("Targets" | "Enemies" | "Inventory" | "BlinkManager")','Some("Targets" | "Enemies" | "Inventory" | "BlinkManager" | "CommandManager" | "Commands")','exclude preserved file duplicates')
    r=E+'runtime.rs'
    p.section(r,'    fn feed_keyboard(','/// The old launcher', '        apply_effects(self, host, effects);\n','        apply_effects(self, host, effects);\n        #[cfg(feature="behavior-hooks")]\n        self.macros_key(host,code);\n','real keyboard KeyEvent after module binds, all actions')
    anchor='    #[cfg(feature="behavior-hooks")]\n    fn client_auto_queue_wake('
    p.replace(r,anchor,'''    #[cfg(feature="behavior-hooks")]
    fn client_macros_wake(&self,host:&mut dyn Host,_event:&mut horizon_client::mod_api::ClientMacrosWakeEvent){self.macros_wake(host);}
    #[cfg(feature="behavior-hooks")]
    fn client_commands_chat(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::ClientCommandsChatEvent<'_>){event.handled|=self.client_command_message(host,event.message);}
'''+anchor,'Main scheduler and shared chat hook')
    p.replace(r,'            modules.auto_queue.cancel();\n','            modules.auto_queue.cancel();\n            modules.macros.cancel();\n','disconnect child cancellation')
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'auto-queue-patch.py',","'auto-queue-patch.py', 'macros-patch.py',",'native extension after AutoQueue')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);g=parser.add_mutually_exclusive_group();g.add_argument('--apply',action='store_true');g.add_argument('--check',action='store_true');args=parser.parse_args();p=build_plan();print(f'Validated {len(p.steps)} Macros/CommandManager changes across {len(p.text)} shared files')
    if args.apply:p.write();print('Applied Macros integration')
    else:print('Check only; no shared writes')
if __name__=='__main__':main()
