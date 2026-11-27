// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{packets::{PacketEvent,PacketListener},runtime::{LiquidBounceHooks,raw_options}};
use horizon_client::mod_api::{Host,ClientboundPlayPacket as Clientbound};
use liquidbounce_behaviors::plugins::{Plugins,Effect,is_anticheat_plugin};
use std::{sync::{Mutex,OnceLock,atomic::{AtomicBool,Ordering}},time::Instant};
fn now()->i64{static START:OnceLock<Instant>=OnceLock::new();START.get_or_init(Instant::now).elapsed().as_millis()as i64}
fn random_id()->i32{liquidbounce_behaviors::random::random_i32_inclusive(0,32766)}
#[derive(Default)]pub struct PluginsService {pub behavior:Mutex<Plugins>,running:AtomicBool,incoming:Mutex<Vec<(i64,Vec<u8>)>>}
impl PluginsService {
    pub fn set_running(&self,running:bool){self.running.store(running,Ordering::Release);}
    pub fn cancel(&self){self.behavior.lock().expect("Plugins poisoned").cancel();self.incoming.lock().expect("Plugins queue poisoned").clear();}
    pub fn disconnected(&self){self.behavior.lock().expect("Plugins poisoned").disconnected();self.incoming.lock().expect("Plugins queue poisoned").clear();}
}
impl PacketListener for PluginsService {
    fn incoming(&self,event:&mut PacketEvent<'_,Clientbound>){
        // Protocol 777's COMMAND_SUGGESTIONS is 15. These are original ordinary
        // PacketEvents even when another listener has already cancelled one.
        if event.context.original()&&self.running.load(Ordering::Acquire)&&let Clientbound::Unknown(15,data)=event.packet {
            self.incoming.lock().expect("Plugins queue poisoned").push((now(),data.clone()));
        }
    }
}
fn language()->u8{match horizon_client::lang::code().to_ascii_lowercase().replace('-',"_").as_str(){"zh_cn"|"zh_sg"|"zh_hans"=>1,"zh_tw"|"zh_hk"|"zh_mo"|"zh_hant"=>2,_=>0}}
impl LiquidBounceHooks {
    fn plugins_service(&self)->std::sync::Arc<PluginsService>{self.state().behaviors.borrow().plugins.clone()}
    fn plugins_running(&self)->bool{let state=self.state();state.behaviors.borrow().query_running(&state.registry,"Plugins")}
    fn plugins_effects(&self,host:&mut dyn Host,effects:Vec<Effect>){
        for effect in effects{match effect{
            Effect::Command(command)=>host.client_plugins_command(command),
            Effect::Suggest{id,input}=>host.client_plugins_suggest(id,input),
            Effect::Found(names)=>{
                let colors=names.iter().map(|name|if is_anticheat_plugin(name){0x55ff55}else{0xff5555}).collect();
                let prefix=match language(){1=>format!("插件（{}）：",names.len()),2=>format!("外掛（{}）：",names.len()),_=>format!("Plugins ({}): ",names.len())};
                host.client_plugins_chat(&prefix,names,colors,false);
            }
            Effect::NoneFound=>host.client_plugins_chat(match language(){1=>"未找到任何插件。",2=>"找不到任何外掛。",_=>"No plugins were found."},Vec::new(),Vec::new(),true),
            Effect::TimeOut=>host.client_plugins_chat(match language(){1=>"插件检查超时。",2=>"外掛檢查逾時。",_=>"Plugins check timed out."},Vec::new(),Vec::new(),false),
            Effect::Disable=>{let options=raw_options(host);let effects={let mut state=self.state();let id=state.behaviors.borrow().query_ids["Plugins"];if let Err(error)=state.with_registry(options,|registry,callbacks|registry.set_enabled_sync(id,false,callbacks)){eprintln!("[LiquidBounce] Plugins: {error:?}");}state.take_effects()};crate::runtime::apply_effects(self,host,effects);}
        }}
    }
    pub(crate) fn plugins_start(&self,host:&mut dyn Host){
        if !self.plugins_running(){return;}
        let service=self.plugins_service();let effects=service.behavior.lock().expect("Plugins poisoned").start(now(),&mut random_id);self.plugins_effects(host,effects);
    }
    pub(crate) fn plugins_tick(&self,host:&mut dyn Host){
        let service=self.plugins_service();
        let incoming=std::mem::take(&mut*service.incoming.lock().expect("Plugins queue poisoned"));
        if !self.plugins_running(){return;}
        for(received,data)in incoming{
            let decoded=host.client_plugins_decode(&data);let Ok(value)=serde_json::from_str::<serde_json::Value>(&decoded)else{continue;};
            let Some(id)=value["id"].as_i64().and_then(|v|i32::try_from(v).ok())else{continue;};
            let Some(values)=value["texts"].as_array()else{continue;};let Some(texts)=values.iter().map(|v|v.as_str().map(str::to_owned)).collect::<Option<Vec<_>>>()else{continue;};
            let effects=service.behavior.lock().expect("Plugins poisoned").suggestions(received,id,texts,now(),&mut random_id);self.plugins_effects(host,effects);
        }
        if self.plugins_running(){let effects=service.behavior.lock().expect("Plugins poisoned").poll(now(),&mut random_id);self.plugins_effects(host,effects);}
    }
    pub(crate) fn plugins_game_message(&self,host:&mut dyn Host,message:&str){if !self.plugins_running(){return;}let service=self.plugins_service();let effects=service.behavior.lock().expect("Plugins poisoned").game_message(now(),message);self.plugins_effects(host,effects);}
    /// Already accepted server roots, before Horizon adds any client commands.
    pub(crate) fn plugins_server_commands(&self,names:&[String]){self.plugins_service().behavior.lock().expect("Plugins poisoned").server_commands=names.to_vec();}
}
