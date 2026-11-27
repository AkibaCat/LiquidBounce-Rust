pub mod mod_api{pub trait Host{fn client_target_lock_dead(&self)->Option<bool>;fn client_target_lock_player(&self,id:i32)->String;}}
pub mod lang{pub fn code()->String{"zh_cn".into()}}
pub mod runtime{
 use std::{cell::RefCell,sync::{Mutex,MutexGuard}};
 use crate::target_lock::TargetLock;
 #[derive(Default)]pub struct Modules{pub target_lock:TargetLock}
 impl Modules{pub fn query_running(&self,registry:&bool,name:&str)->bool{assert_eq!(name,"TargetLock");*registry}}
 pub struct State{pub behaviors:RefCell<Modules>,pub registry:bool,pub events:RefCell<Vec<(String,serde_json::Value)>>}
 pub struct LiquidBounceHooks{pub state:Mutex<State>}
 impl Default for LiquidBounceHooks{fn default()->Self{Self{state:Mutex::new(State{behaviors:RefCell::new(Modules::default()),registry:true,events:RefCell::new(Vec::new())})}}}
 impl LiquidBounceHooks{pub fn state(&self)->MutexGuard<'_,State>{self.state.lock().unwrap()}}
}
#[cfg(test)]mod runtime_tests{
 use super::*;use crate::{target_lock_runtime::*,target_lock::*,targeting::*};
 struct Host;impl mod_api::Host for Host{fn client_target_lock_dead(&self)->Option<bool>{Some(false)}fn client_target_lock_player(&self,id:i32)->String{serde_json::json!({"id":id,"name":"Notch","removed":false,"distance_squared_bits":"0"}).to_string()}}
 #[test]fn exact_runtime_attack_tick_notifications_and_native_snapshot(){let hooks=runtime::LiquidBounceHooks::default();let mut host=Host;hooks.target_lock_attack(&mut host,2);hooks.target_lock_tick(&mut host);assert_eq!(hooks.state().behaviors.borrow().target_lock.locks.len(),1);#[cfg(feature="webview")]assert_eq!(hooks.state().events.borrow()[0].1["message"],"锁定Notch，持续30秒。");assert!(player_snapshot("{}").is_none());let p=player_snapshot(r#"{"id":2,"name":"N","removed":false,"distance_squared_bits":"9218868437227405312"}"#).unwrap();assert!(p.distance_squared.is_infinite());let mut info=TargetInfo::default();let mut lock=TargetLock::default();lock.settings.when_no_lock=WhenNoLock::AllowNone;lock.settings.combat_only=true;apply_tag(&lock,true,Some(&p.player),&mut info);assert_eq!(info.classification,Classification::Interesting);}
}
