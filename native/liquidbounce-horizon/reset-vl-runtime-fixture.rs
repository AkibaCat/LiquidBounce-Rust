//! Exact runtime methods with actual core registry, TimerService and FIFO;
//! explicit State/Host/registration providers keep this a focused consumer test.
extern crate self as horizon_client;
extern crate self as liquidbounce_behaviors;
use std::{cell::RefCell,collections::BTreeMap,sync::{Arc,Mutex,MutexGuard}};
use liquidbounce_core::{ModuleCallback,ModuleCallbacks,ModuleDefinition,ModuleId,ModuleRegistry,WorldState};
pub mod mod_api{pub trait Host{fn client_reset_vl_ground(&self)->Option<bool>;fn client_reset_vl_velocity(&mut self,y:f64);}}
pub mod lang{pub fn code()->String{"en_us".into()}}
#[path="__RESET_VL__"]pub mod reset_vl;
#[path="__TIMER__"]pub mod timer;
#[path="coroutine-ticker.rs"]mod coroutine_ticker;
#[path="timer-service.rs"]mod timer_runtime;
#[path="runtime-source.rs"]mod reset_vl_runtime;

struct Modules{
    reset_vl:reset_vl::ResetVl,
    timer:Arc<timer_runtime::TimerService>,
    coroutine_ticker:coroutine_ticker::CoroutineTicker,
    query_ids:BTreeMap<&'static str,ModuleId>,
}
impl Modules{
    fn query_running(&self,registry:&ModuleRegistry,name:&str)->bool{registry.running(self.query_ids[name])}
    fn sync(&self,registry:&ModuleRegistry){self.timer.set_provider_running(reset_vl_runtime::RESET_VL_TIMER_PROVIDER,self.query_running(registry,"ResetVL"));}
}
mod runtime{
    use super::*;
    pub(crate) struct RawOptions;
    pub(crate) struct State{
        pub registry:ModuleRegistry,
        pub behaviors:RefCell<Modules>,
        pub events:RefCell<Vec<(String,serde_json::Value)>>,
        pub callbacks:RefCell<Vec<(ModuleCallback,i32)>>,
    }
    impl State{
        pub(crate) fn with_registry<R>(&mut self,_options:RawOptions,operation:impl FnOnce(&mut ModuleRegistry,&dyn ModuleCallbacks)->R)->R{
            let callbacks=|registry:&mut ModuleRegistry,callback:ModuleCallback|{
                self.callbacks.borrow_mut().push((callback,self.behaviors.borrow().reset_vl.count));
                if let ModuleCallback::CancelSequences(id)=callback{
                    assert_eq!(id,self.behaviors.borrow().query_ids["ResetVL"]);
                    self.behaviors.borrow_mut().reset_vl.cancel_sequences();
                }
                self.behaviors.borrow().sync(registry);Ok(())
            };
            let result=operation(&mut self.registry,&callbacks);self.behaviors.borrow().sync(&self.registry);result
        }
    }
    pub(crate) struct LiquidBounceHooks(pub Mutex<State>);
    impl LiquidBounceHooks{
        pub(crate) fn state(&self)->MutexGuard<'_,State>{self.0.lock().unwrap()}
    }
    pub(crate) fn raw_options(_host:&dyn mod_api::Host)->RawOptions{RawOptions}
}
fn hooks(times:i32)->runtime::LiquidBounceHooks{
    let mut registry=ModuleRegistry::new();let mut definition=ModuleDefinition::new("ResetVL");definition.enabled=true;
    let id=registry.register(definition).unwrap();registry.update_world(WorldState{world_present:true,player_present:true});
    let mut reset_vl=reset_vl::ResetVl::default();reset_vl.settings.times=times;
    runtime::LiquidBounceHooks(Mutex::new(runtime::State{registry,behaviors:RefCell::new(Modules{reset_vl,timer:Default::default(),coroutine_ticker:Default::default(),query_ids:BTreeMap::from([("ResetVL",id)])}),events:Default::default(),callbacks:Default::default()}))
}
struct Host<'a>{hooks:&'a runtime::LiquidBounceHooks,ground:bool,velocity:Vec<f64>}
impl mod_api::Host for Host<'_>{
    fn client_reset_vl_ground(&self)->Option<bool>{
        let state=self.hooks.0.try_lock().expect("Game Host query must release engine state");
        assert!(state.behaviors.try_borrow_mut().is_ok());Some(self.ground)
    }
    fn client_reset_vl_velocity(&mut self,y:f64){
        let state=self.hooks.0.try_lock().expect("Game Host effect must release engine state");
        assert!(state.behaviors.try_borrow_mut().is_ok());self.velocity.push(y);
    }
}
fn callbacks(hooks:&runtime::LiquidBounceHooks,host:&mut dyn mod_api::Host,in_game:bool){
    let ticker=hooks.state().behaviors.borrow().coroutine_ticker.clone();let mut batch=ticker.begin_tick().unwrap();
    while let Some(task)=batch.next_task(){batch.finish_current(hooks.reset_vl_callback(host,in_game,task.callback));}
}
#[test]
fn exact_runtime_releases_borrows_disables_synchronously_and_publishes_success(){
    let hooks=hooks(1);let mut host=Host{hooks:&hooks,ground:true,velocity:vec![]};
    hooks.reset_vl_tick(&mut host);hooks.reset_vl_tick(&mut host);
    assert_eq!(host.velocity,vec![0.1]);callbacks(&hooks,&mut host,true);
    assert_eq!(host.velocity,vec![0.1,-0.1]);
    let state=hooks.state();let modules=state.behaviors.borrow();
    assert!(!modules.query_running(&state.registry,"ResetVL"));assert_eq!(modules.reset_vl.count,0);
    assert!(state.callbacks.borrow().iter().any(|(event,count)|matches!(event,ModuleCallback::CancelSequences(_))&&*count==1));
    #[cfg(feature="webview")]{assert_eq!(*state.events.borrow(),vec![("notification".into(),serde_json::json!({"title":"ResetVL","message":"Successfully reset violations","severity":"SUCCESS"}))]);}
    assert_eq!(modules.timer.scale(1.0),1.0);
}
#[test]
fn exact_runtime_active_provider_airborne_requests_expiry_and_reenable_progress(){
    let hooks=hooks(10);let mut host=Host{hooks:&hooks,ground:false,velocity:vec![]};
    hooks.reset_vl_tick(&mut host);
    let timer=hooks.state().behaviors.borrow().timer.clone();assert_eq!(timer.scale(1.0),2.0);
    timer.first_game_tick();assert_eq!(timer.scale(1.0),2.0);timer.first_game_tick();assert_eq!(timer.scale(1.0),1.0);
    host.ground=true;hooks.reset_vl_tick(&mut host);callbacks(&hooks,&mut host,true);
    {let state=hooks.state();assert_eq!(state.behaviors.borrow().reset_vl.count,1);}
    host.ground=false;hooks.reset_vl_tick(&mut host);
    {let mut state=hooks.state();let id=state.behaviors.borrow().query_ids["ResetVL"];
        state.with_registry(runtime::RawOptions,|registry,callbacks|registry.set_enabled_sync(id,false,callbacks)).unwrap();
        assert_eq!(state.behaviors.borrow().reset_vl.count,1);
        state.with_registry(runtime::RawOptions,|registry,callbacks|registry.set_enabled_sync(id,true,callbacks)).unwrap();}
    assert_eq!(timer.scale(1.0),2.0);
}
#[test]
fn exact_runtime_guard_failure_cancels_wait_without_velocity_or_count_change(){
    let hooks=hooks(10);let mut host=Host{hooks:&hooks,ground:true,velocity:vec![]};
    hooks.reset_vl_tick(&mut host);callbacks(&hooks,&mut host,false);
    assert_eq!(host.velocity,vec![0.1]);let state=hooks.state();let modules=state.behaviors.borrow();
    assert_eq!(modules.reset_vl.count,0);assert!(!modules.reset_vl.busy());
}
