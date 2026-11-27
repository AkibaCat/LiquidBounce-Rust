#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
extern crate self as liquidbounce_behaviors;
mod timer;
mod clock;
mod oracle;
mod java_text;
mod timer_runtime;
mod combat_window;
mod coroutine_ticker;
mod no_swing;
pub mod packets;
mod packet_bridge;
mod config;
use std::{marker::PhantomData,sync::{Mutex,OnceLock},time::{Duration,Instant}};

pub mod social { pub fn set_presence_status(_:u8){} pub fn tick(){} }
pub mod spans { pub fn time<T>(_:&str,callback:impl FnOnce()->T)->T{callback()} }
pub mod mod_api {
    use std::sync::OnceLock;
    pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket,Serverbound as ServerboundPlayPacket};
    pub trait RenderHooks:Send+Sync{}
    #[allow(unused_imports)]
    mod generated_host {include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}
    pub use generated_host::*;
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}
    pub use generated::*;
    pub fn mods()->&'static [Box<dyn Hooks>] {
        static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Cancel),Box::new(TimerHook)])
    }
    fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a Box<dyn Hooks>>{mods.iter()}
    struct Cancel;
    impl Hooks for Cancel {fn client_player_tick(&self,_:&dyn Host,event:&mut ClientPlayerTickEvent){event.cancelled=super::env().lock().unwrap().cancelled;}}
    struct TimerHook;
    impl Hooks for TimerHook {
        fn client_timer_scale(&self,_:&dyn Host,event:&mut ClientTimerScaleEvent){let mut state=super::env().lock().unwrap();event.delta=state.timer.scale_frame(event.delta,true,&mut |_|true);}
        fn client_timer_frame(&self,_:&dyn Host,event:&mut ClientTimerFrameEvent){let mut state=super::env().lock().unwrap();state.conditions.push(event.index);event.end=state.timer.frame_ticks(event.ticks,event.index,true,&mut |_|true);event.held=state.timer.is_held();}
        fn client_player_tick_final(&self,_:&dyn Host,event:&mut ClientPlayerTickFinalEvent){let mut state=super::env().lock().unwrap();state.player_events+=1;state.timer.player_tick_final(event.cancelled);}
    }
}
#[derive(Default)]
struct Environment {timer:timer::Timer,cancelled:bool,conditions:Vec<i32>,player_events:u32}
fn env()->&'static Mutex<Environment>{static STATE:OnceLock<Mutex<Environment>>=OnceLock::new();STATE.get_or_init(Default::default)}
struct Script;
impl Script{fn armed(&self)->bool{false}}
struct Ambience;
impl Ambience{fn reseed(&self,_:i64){}}
#[derive(Clone,Copy,PartialEq,Eq)]enum Screen{Inventory,Creative,Playing}
struct Position{x:f64,z:f64}
struct Player{move_vector:[f32;2],allowed:bool,position:Position}
struct Streaming{lit:bool}
impl Streaming{fn is_lit(&self,_:i32,_:i32)->bool{self.lit}}
#[derive(Default)]struct Cooldowns{ticks:u32}
impl Cooldowns{fn tick(&mut self){self.ticks+=1;}}
struct Remote{loaded_sent:bool,last_input:u8}
/* SLEEP_TIMER */
impl Player{fn native_client_queries_allowed(&self)->bool{self.allowed}}
struct Game<'a>{clock:clock::FixedStep,title:Option<()>,pause:bool,tick_script:Script,ambience:Ambience,pre:u32,post:u32,simulation:u32,marker:PhantomData<&'a()>,window:bool,lan_acting:Option<()>,lan_acting_uuid:Option<()>,player:Option<Player>,streaming:Streaming,screen:Screen,menu:i32,container:Option<()>,station:Option<()>,mounted:bool,craft_size:i32,chest:Option<usize>,cooldowns:Cooldowns,hotbar_pop:[u32;2],ambient:u32,effects:u32,sleep_timer:u32,sleeping:Option<()>,remote:Option<Remote>,sent:Vec<u8>}
mod native_hooks { use super::*; impl Game<'_>{/* HELPERS */ /* SNAPSHOT */pub(super) fn production_player_consumers(&mut self,hold:bool)->Result<(),()>{/* PLAYER_CONSUMERS */self.simulation+=1;Ok(())}} }
mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
impl Game<'_> {
    fn native_window(&self)->bool{self.window}
    fn remote_menu(&self)->(i32,){(self.menu,)}
    fn mount_active(&self)->bool{self.mounted}
    fn chest_rows(&self)->Option<usize>{self.chest}
    fn tick_client_ambient(&mut self){self.ambient+=1;}
    fn tick_client_effects(&mut self){self.effects+=1;}
    fn remote_send_state(&mut self,input:u8){self.sent.push(input);}
    fn presence_status(&self)->u8{0}
    fn native_client_tick_pre(&mut self){self.pre+=1;env().lock().unwrap().timer.game_tick();}
    fn native_client_tick_post(&mut self){self.post+=1;}
    fn tick_script_step(&self,_:&()){}
    fn tick(&mut self)->Result<(),()>{self.clock.world_runs();if !self.client_timer_player_tick(){self.simulation+=1;}Ok(())}
    fn tick_ambience(&self){}
    fn visual_ticks(&self)->f64{0.0}
    fn tick_fov(&self){}
    fn tick_win_screen(&self)->Result<(),()>{Ok(())}
    fn native_configs_tick_client(&self){}
    fn tick_chat_queue(&self){}
    fn tick_music(&self){}
    fn tick_sound_engine(&self){}
    fn sprint_report(&self,_:&clock::SprintReport){}
    fn frame(&mut self,elapsed:Duration,event_loop:&())->Result<(),()>{
        /* FRAME_LOOP */
        Ok(())
    }
}
fn game()->Game<'static>{*env().lock().unwrap()=Default::default();Game{clock:clock::FixedStep::client(),title:None,pause:false,tick_script:Script,ambience:Ambience,pre:0,post:0,simulation:0,marker:PhantomData,window:true,lan_acting:None,lan_acting_uuid:None,player:Some(Player{move_vector:[0.0;2],allowed:true,position:Position{x:3.0,z:-1.0}}),streaming:Streaming{lit:true},screen:Screen::Playing,menu:0,container:None,station:None,mounted:false,craft_size:2,chest:None,cooldowns:Default::default(),hotbar_pop:[3,1],ambient:0,effects:0,sleep_timer:5,sleeping:Some(()),remote:Some(Remote{loaded_sent:true,last_input:9}),sent:Vec::new()}}
mod behaviors {
    pub struct BehaviorModules{pub timer:std::sync::Arc<crate::timer_runtime::TimerService>,pub coroutine_ticker:crate::coroutine_ticker::CoroutineTicker,pub running:bool}
    impl BehaviorModules{pub fn query_running(&self,_:&(),_:&str)->bool{self.running}}
}
mod runtime {
    use std::sync::{Mutex,MutexGuard};
    pub struct State{pub behaviors:std::cell::RefCell<crate::behaviors::BehaviorModules>,pub registry:(),pub combat:crate::combat_window::CombatWindow}
    pub struct LiquidBounceHooks(pub Mutex<State>);
    impl LiquidBounceHooks{pub fn state(&self)->MutexGuard<'_,State>{self.0.lock().unwrap()}}
}
#[test]fn kotlin_service_raw_bits(){oracle::compare();}
#[test]fn real_clock_scales_before_residual_and_dynamic_loop_includes_terminating_condition(){
    let mut game=game();env().lock().unwrap().timer.request_speed(2.0,20,1,100);
    game.frame(Duration::from_millis(37),&()).unwrap();assert_eq!(game.pre,1);
    assert_eq!(game.clock.fraction(),f64::from(1.48f32-1.0));
    assert_eq!(env().lock().unwrap().conditions,[0,1]);
    assert_eq!(env().lock().unwrap().timer.balance,-0.5);
    game.frame(Duration::from_millis(13),&()).unwrap();assert_eq!(game.pre,2);assert_eq!(game.clock.fraction(),0.0);
}
#[test]fn held_ticks_produce_no_tick_events_and_force_both_partial_tick_consumers(){
    let mut game=game();env().lock().unwrap().timer.hold(2,3);
    game.frame(Duration::from_millis(100),&()).unwrap();assert_eq!((game.pre,game.post,game.simulation),(0,0,0));
    assert_eq!(game.clock.fraction(),1.0);assert_eq!(game.clock.camera_fraction(),1.0);
    assert_eq!(env().lock().unwrap().timer.balance,2.0);
    game.frame(Duration::from_millis(100),&()).unwrap();assert_eq!((game.pre,game.post,game.simulation),(1,1,1));
    assert_eq!(env().lock().unwrap().timer.balance,3.0);
}
#[test]fn burst_adds_ticks_after_original_bound_and_cancelled_player_preserves_final_balance(){
    let mut game=game();{let mut state=env().lock().unwrap();state.timer.balance=8.0;assert_eq!(state.timer.burst(true,3,[0,20]),Ok(3));state.cancelled=true;}
    game.frame(Duration::from_millis(50),&()).unwrap();assert_eq!((game.pre,game.post,game.simulation),(4,4,0));
    let state=env().lock().unwrap();assert_eq!(state.conditions,[0,1,2,3,4]);assert_eq!(state.player_events,4);assert_eq!(state.timer.balance,9.0);assert_eq!(state.timer.real_ticks,1.0);
}
#[test]fn unchanged_ten_tick_cap_applies_before_bursts_and_client_rate_does_not_change_server(){
    let mut game=game();env().lock().unwrap().timer.request_speed(20.0,20,1,100);
    let server=std::thread::spawn(||{let mut clock=clock::FixedStep::default();clock.advance(Duration::from_millis(50))});
    game.frame(Duration::from_millis(100),&()).unwrap();assert_eq!(game.pre,10);assert_eq!(server.join().unwrap(),1);
    assert_eq!(game.clock.rate(),20.0);
}
#[test]fn native_pause_and_freeze_bookkeeping_survive_scaled_delta(){
    let mut clock=clock::FixedStep::client();let first=clock.begin_advance(Duration::from_millis(25));assert_eq!(clock.finish_advance(first,first.delta_ticks*2.0),1);
    clock.end_frame(true);let frame=clock.begin_advance(Duration::from_millis(20));clock.finish_advance(frame,frame.delta_ticks*2.0);
    assert_eq!(clock.fraction(),0.0);clock.end_frame(false);assert_eq!(clock.fraction(),0.0);
    clock.set_frozen(true);clock.world_runs();clock.end_frame(false);assert_eq!(clock.fraction(),1.0);assert_eq!(clock.camera_fraction(),0.0);
}
#[test]fn actual_context_host_distinguishes_inventory_chest_other_containers_and_player_ownership(){
    use mod_api::Host;let mut game=game();assert_eq!(game.client_timer_context(),Some([false,false]));
    game.player.as_mut().unwrap().move_vector=[0.0,f32::NAN];game.screen=Screen::Inventory;
    assert_eq!(game.client_timer_context(),Some([true,true]));game.menu=7;game.container=Some(());assert_eq!(game.client_timer_context(),Some([true,false]));
    game.chest=Some(3);assert_eq!(game.client_timer_context(),Some([true,true]));game.screen=Screen::Creative;assert_eq!(game.client_timer_context(),Some([true,false]));
    game.lan_acting=Some(());assert_eq!(game.client_timer_context(),None);game.lan_acting=None;game.player.as_mut().unwrap().allowed=false;assert_eq!(game.client_timer_context(),None);
}
#[test]fn production_service_and_real_pipeline_reset_cancelled_corrections_on_render_executor(){
    use std::sync::Arc;use packets::{PacketEvent,PacketListener,PacketInterceptor};use horizon_protocol::packets::play::{Clientbound,PlayerPosition};
    struct CancelCorrection;impl PacketListener for CancelCorrection{fn incoming(&self,event:&mut PacketEvent<'_,Clientbound>){event.cancel();}}
    let service=Arc::new(timer_runtime::TimerService::default());service.update_settings(|settings|settings.mode=timer::Mode::Boost);service.set_running(true);service.scale(1.0);
    let pipeline=Arc::new(packets::PacketPipeline::default());pipeline.add_listener(100,Arc::new(CancelCorrection));pipeline.add_listener(0,service.clone());
    let packet=||Clientbound::PlayerPosition(PlayerPosition{teleport:1,position:[0.0;3],delta:[0.0;3],yaw:0.0,pitch:0.0,relatives:0});
    let context=packets::PacketContext{session:5,origin:packets::TransferOrigin::Incoming,delivery:packets::Delivery::Original};
    service.state.lock().unwrap().timer.balance=5.0;let mut data=packet();let mut event=PacketEvent::new(context,&mut data);pipeline.incoming(&mut event);assert!(event.is_cancelled());assert_eq!(service.state.lock().unwrap().timer.balance,0.0);
    service.state.lock().unwrap().timer.balance=7.0;let worker_pipeline=pipeline.clone();std::thread::spawn(move||{let mut data=packet();worker_pipeline.incoming(&mut PacketEvent::new(context,&mut data));}).join().unwrap();
    assert_eq!(service.state.lock().unwrap().timer.balance,7.0);service.frame(0,0);assert_eq!(service.state.lock().unwrap().timer.balance,0.0);
    service.update_settings(|settings|settings.reset_balance_on_flag=false);service.state.lock().unwrap().timer.balance=9.0;let mut data=packet();pipeline.incoming(&mut PacketEvent::new(context,&mut data));assert_eq!(service.state.lock().unwrap().timer.balance,9.0);
}
#[test]fn actual_player_consumers_skip_cancelled_simulation_but_preserve_external_send_changes(){
    let mut game=game();env().lock().unwrap().cancelled=true;
    game.production_player_consumers(false).unwrap();
    assert_eq!((game.cooldowns.ticks,game.hotbar_pop,game.effects,game.sleep_timer,game.simulation),(0,[3,1],0,5,0));
    assert_eq!(game.ambient,1);assert_eq!(game.sent,[9]);assert_eq!(env().lock().unwrap().player_events,1);
    game.remote.as_mut().unwrap().loaded_sent=false;game.production_player_consumers(true).unwrap();assert_eq!(game.sent,[9]);
    env().lock().unwrap().cancelled=false;game.production_player_consumers(false).unwrap();
    assert_eq!((game.cooldowns.ticks,game.hotbar_pop,game.effects,game.sleep_timer,game.simulation),(1,[2,0],1,6,1));
}
#[test]fn actual_player_event_waits_for_player_and_loaded_chunk(){
    let mut game=game();env().lock().unwrap().cancelled=true;
    game.streaming.lit=false;assert!(!game.client_timer_player_tick_ready());assert_eq!(env().lock().unwrap().player_events,0);
    game.player=None;game.streaming.lit=true;assert!(!game.client_timer_player_tick_ready());assert_eq!(env().lock().unwrap().player_events,0);
}
