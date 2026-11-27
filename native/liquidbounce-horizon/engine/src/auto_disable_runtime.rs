// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{packets::{PacketEvent,PacketListener},runtime::{LiquidBounceHooks,raw_options}};
use horizon_client::mod_api::{Host,ClientboundPlayPacket as Clientbound};
use liquidbounce_behaviors::auto_disable::Reason;
use std::sync::{Mutex,atomic::{AtomicBool,Ordering}};
/// Reader callbacks only append; registry and native effects stay on the actor.
#[derive(Default)] pub struct AutoDisablePackets {running:AtomicBool,pending:Mutex<Vec<Reason>>}
impl AutoDisablePackets {
    pub fn set_running(&self,value:bool){self.running.store(value,Ordering::Release);}
    pub fn take(&self)->Vec<Reason>{std::mem::take(&mut*self.pending.lock().expect("AutoDisable queue poisoned"))}
}
impl PacketListener for AutoDisablePackets {
    fn incoming(&self,event:&mut PacketEvent<'_,Clientbound>){
        if event.context.original()&&self.running.load(Ordering::Acquire)&&matches!(event.packet,Clientbound::PlayerPosition(_)) {
            self.pending.lock().expect("AutoDisable queue poisoned").push(Reason::Flag);
        }
    }
}
fn language()->u8 {match horizon_client::lang::code().to_ascii_lowercase().replace('-',"_").as_str(){"zh_cn"|"zh_sg"|"zh_hans"=>1,"zh_tw"|"zh_hk"|"zh_mo"|"zh_hant"=>2,_=>0}}
impl LiquidBounceHooks {
    pub(crate) fn auto_disable_event(&self,host:&mut dyn Host,reason:Reason){
        let options=raw_options(host);
        let selected={let state=self.state();let modules=state.behaviors.borrow();if !modules.auto_disable.handles(modules.query_running(&state.registry,"AutoDisable"),reason){return;}modules.auto_disable.selected().to_vec()};
        let mut any=false;
        for name in selected {
            let effects={
                let mut state=self.state();
                let id=state.registry.modules().find(|module|module.definition().name==name&&module.enabled()).map(|module|module.id());
                if let Some(id)=id {
                    match state.with_registry(options,|registry,callbacks|registry.set_enabled_sync(id,false,callbacks)) {
                        Ok(_)=>any=true,
                        Err(error)=>eprintln!("[LiquidBounce] AutoDisable: {error:?}"),
                    }
                    Some(state.take_effects())
                }else{None}
            };
            // onDisabled's native cleanup completes before the next module.
            if let Some(effects)=effects{crate::runtime::apply_effects(self,host,effects);}
        }
        if any {
            let reason=match (language(),reason){(1,Reason::Flag)=>"服务器回退",(1,Reason::Death)=>"玩家死亡",(1,Reason::WorldChange)=>"世界切换",(1,Reason::Disconnect)=>"断开连接",(2,Reason::Flag)=>"伺服器回退",(2,Reason::Death)=>"玩家死亡",(2,Reason::WorldChange)=>"世界切換",(2,Reason::Disconnect)=>"中斷連線",(_,reason)=>reason.description()};
            let message=match language(){1=>format!("由于{reason}，已关闭指定模块"),2=>format!("由於{reason}，已關閉指定模組"),_=>format!("Disabled modules due to {reason}")};
            #[cfg(feature="webview")]self.state().events.borrow_mut().push(("notification".into(),serde_json::json!({"title":"Notifier","message":message,"severity":"INFO"})));
            #[cfg(not(feature="webview"))]eprintln!("[LiquidBounce] Notifier: {message}");
        }
    }
    /// Drain before normal GameTick work and immediately before accepted flags.
    pub(crate) fn auto_disable_drain(&self,host:&mut dyn Host){let pending=self.state().behaviors.borrow().auto_disable_packets.take();for reason in pending{self.auto_disable_event(host,reason);}}
    /// The source command's add/remove/clear/list operations. Returns false for
    /// unrelated input, so ordinary chat and commands continue unchanged.
    pub(crate) fn auto_disable_command(&self,host:&mut dyn Host,message:&str)->bool {
        let mut words=message.split_whitespace();
        if !words.next().is_some_and(|v|v.eq_ignore_ascii_case(".autodisable")){return false;}
        let operation=words.next().unwrap_or("list").to_ascii_lowercase();
        let requested:Vec<_>=words.flat_map(|word|word.split(',')).filter(|name|!name.is_empty()).collect();
        let responses={
            let state=self.state();let available:Vec<_>=state.registry.modules().map(|m|m.definition().name.clone()).collect();
            let mut modules=state.behaviors.borrow_mut();let module=&mut modules.auto_disable;let mut responses=Vec::new();
            match operation.as_str(){
                "clear"=>{module.clear();responses.push(match language(){1=>"已清空自动关闭列表".into(),2=>"已清空自動關閉清單".into(),_=>"Cleared AutoDisable modules".into()});}
                "list"=>responses.push(format!("AutoDisable: {}",module.selected().join(", "))),
                "add"|"remove"=>for requested in requested {
                    if let Some(name)=available.iter().find(|name|name.eq_ignore_ascii_case(requested)) {
                        let changed=if operation=="add"{module.add(name)}else{module.remove(name)};
                        responses.push(match(language(),operation.as_str(),changed){(1,"add",true)=>format!("已添加 {name}"),(1,"remove",true)=>format!("已移除 {name}"),(2,"add",true)=>format!("已新增 {name}"),(2,"remove",true)=>format!("已移除 {name}"),(1,_,false)=>format!("{name} 的列表状态未改变"),(2,_,false)=>format!("{name} 的清單狀態未改變"),(_,"add",true)=>format!("Added {name}"),(_,"remove",true)=>format!("Removed {name}"),_=>format!("{name} membership unchanged")});
                    }else{responses.push(match language(){1=>format!("未知模块：{requested}"),2=>format!("未知模組：{requested}"),_=>format!("Unknown module: {requested}")});}
                },
                _=>responses.push(".autodisable list | clear | add <modules> | remove <modules>".into()),
            }
            responses
        };
        for response in responses{host.chat(&response);}true
    }
}
