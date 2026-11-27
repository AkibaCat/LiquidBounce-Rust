#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
mod timer;
type Result<T,E=()>=std::result::Result<T,E>;
use std::{cell::RefCell,marker::PhantomData,sync::OnceLock};
mod execution_role;
use execution_role::ExecutionRole;

pub mod mod_api {
    pub trait RenderHooks:Send+Sync{}
    #[allow(unused_imports)]
    mod generated_host {include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}
    pub use generated_host::*;
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a Box<dyn Hooks>>{mods.iter()}
    pub fn mods()->&'static [Box<dyn Hooks>]{static MODS:super::OnceLock<Vec<Box<dyn Hooks>>>=super::OnceLock::new();MODS.get_or_init(||vec![Box::new(Observer)])}
    struct Observer;
    impl Hooks for Observer {
        fn client_world_change(&self,_:&dyn Host,event:&mut ClientWorldChangeEvent<'_>){
            super::STATE.with_borrow_mut(|state|{
                state.trace.push(format!("world:{}",event.dimension.unwrap_or("null")));
                state.timer.world_change();
            });
        }
    }
}
#[derive(Default)]struct State{timer:timer::Timer,trace:Vec<String>}
thread_local!{static STATE:RefCell<State>=RefCell::new(State::default());}
struct Options{execution_role:ExecutionRole}
struct Audio;impl Audio{fn stop_all(&mut self){trace("sounds");}}
struct Music;impl Music{fn stop_playing(&mut self,_:Option<&str>,_:&mut Audio){trace("music");}}
struct Ambience;impl Ambience{fn clear(&mut self,_:&mut Audio){trace("ambience");}}
struct Remote{heard_level:Option<String>}
struct Spawn{dimension:String}
struct Integrated{stopped:bool}
impl Integrated{fn stopped(&self)->bool{self.stopped}}
struct Game<'a>{audio:Audio,music:Music,ambience:Ambience,options:Options,remote:Option<Remote>,lan_acting:Option<()>,lan_acting_uuid:Option<()>,integrated:Option<Integrated>,world_icon:Option<()>,marker:PhantomData<&'a()>}
mod native_hooks{use super::*;impl Game<'_>{/* WORLD_HELPER */ /* WINDOW_HELPER */}}
mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
impl Game<'_>{
    fn enter(&mut self,spawn:&Spawn,keep_world:bool){
        let r=self.remote.as_ref().unwrap();
        /* SAME_LEVEL */
        if !same {
            /* ENTER_CONSUMER */
            trace("level-load");
            self.remote.as_mut().unwrap().heard_level=Some(spawn.dimension.clone());
        }
    }
    fn leave_server(&mut self,reason:Option<String>)->Result<(),()>{
        /* LEAVE_EARLY */
        /* LEAVE_CONSUMER */
        self.remote=None;
        Ok(())
    }
    /* FINISH_CONSUMER */
        Ok(())
    }
    fn begin_closing_save(&mut self,_:bool)->Result<(),()>{trace("save-pending");Ok(())}
    fn stop_sounds_for_new_level(&self){trace("stop-new-level");}
}
fn trace(message:&str){STATE.with_borrow_mut(|state|state.trace.push(message.to_owned()));}
fn game(role:ExecutionRole,dimension:Option<&str>)->Game<'static>{
    STATE.with_borrow_mut(|state|{*state=State::default();state.timer.balance=13.0;state.timer.real_ticks=17.0;state.timer.request_speed(1.5,20,1,40);});
    Game{audio:Audio,music:Music,ambience:Ambience,options:Options{execution_role:role},remote:dimension.map(|d|Remote{heard_level:Some(d.to_owned())}),lan_acting:None,lan_acting_uuid:None,integrated:None,world_icon:Some(()),marker:PhantomData}
}
fn events()->Vec<String>{STATE.with_borrow(|state|state.trace.clone())}
fn assert_reset(){STATE.with_borrow_mut(|state|{assert_eq!(state.timer.balance,0.0);assert_eq!(state.timer.real_ticks,17.0);assert_eq!(state.timer.scale_frame(2.0,true,&mut |_|true),3.0);});}

#[test]fn actual_login_dimension_and_same_key_reload_reset_before_engine_consumers(){
    for (old,keep,new)in[(None,false,"minecraft:overworld"),(Some("minecraft:overworld"),true,"minecraft:the_nether"),(Some("minecraft:overworld"),false,"minecraft:overworld")]{
        let mut client=game(ExecutionRole::Client,old);
        if client.remote.is_none(){client.remote=Some(Remote{heard_level:None});}
        client.enter(&Spawn{dimension:new.into()},keep);
        assert_eq!(events(),[format!("world:{new}"),"stop-new-level".into(),"level-load".into()]);assert_reset();
    }
}
#[test]fn same_level_respawn_keeps_client_level_and_timer_balance(){
    let mut client=game(ExecutionRole::Client,Some("minecraft:overworld"));
    client.enter(&Spawn{dimension:"minecraft:overworld".into()},true);
    assert!(events().is_empty());STATE.with_borrow(|state|assert_eq!(state.timer.balance,13.0));
}
#[test]fn disconnect_and_save_completion_emit_exactly_one_null_update(){
    let mut client=game(ExecutionRole::Client,Some("minecraft:overworld"));
    client.leave_server(Some("closed".into())).unwrap();assert_eq!(events(),["world:null","ambience","music","sounds"]);assert_reset();
    let mut client=game(ExecutionRole::Client,Some("minecraft:overworld"));
    client.finish_close_world().unwrap();assert_eq!(events(),["world:null","ambience","music","sounds"]);assert_reset();
    let mut client=game(ExecutionRole::Client,Some("minecraft:overworld"));client.integrated=Some(Integrated{stopped:false});
    client.leave_server(None).unwrap();assert_eq!(events(),["save-pending"]);STATE.with_borrow(|state|assert_eq!(state.timer.balance,13.0));
    client.integrated.as_mut().unwrap().stopped=true;client.finish_close_world().unwrap();
    assert_eq!(events(),["save-pending","world:null","ambience","music","sounds"]);assert_reset();
}
#[test]fn actual_execution_role_and_actor_gates_exclude_server_and_remote_actor_worlds(){
    for role in [ExecutionRole::IntegratedServer,ExecutionRole::DedicatedServer]{
        let mut server=game(role,Some("minecraft:overworld"));server.enter(&Spawn{dimension:"minecraft:the_nether".into()},true);server.leave_server(Some("closed".into())).unwrap();server.finish_close_world().unwrap();
        assert!(events().iter().all(|event|!event.starts_with("world:")));STATE.with_borrow(|state|assert_eq!(state.timer.balance,13.0));
    }
    for actor in [0,1]{let mut client=game(ExecutionRole::Client,Some("minecraft:overworld"));if actor==0{client.lan_acting=Some(());}else{client.lan_acting_uuid=Some(());}client.native_client_world_change(None);assert!(events().is_empty());}
}
#[test]fn independent_integrated_server_thread_cannot_reset_window_timer(){
    let client=game(ExecutionRole::Client,Some("minecraft:overworld"));
    std::thread::spawn(||{let mut server=game(ExecutionRole::IntegratedServer,Some("minecraft:overworld"));server.leave_server(Some("closed".into())).unwrap();assert!(events().iter().all(|event|!event.starts_with("world:")));}).join().unwrap();
    STATE.with_borrow(|state|assert_eq!(state.timer.balance,13.0));client.native_client_world_change(None);assert_reset();
}
