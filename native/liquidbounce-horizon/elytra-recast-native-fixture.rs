// SPDX-License-Identifier: GPL-3.0-or-later
use std::{cell::RefCell,sync::{Arc,Mutex,LazyLock,atomic::{AtomicBool,Ordering}}};
use horizon_assets::item_model::{ItemModels,StackView};
use horizon_core::{geometry::Vec3,position::BlockPos};
use horizon_items::items::ItemStack;
use horizon_protocol::{connection::{Connection,ConnWriter,Pipe},packets::play};
use play::Serverbound as S;
type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
pub mod items {pub use horizon_items::items::*;}
pub mod item_data {pub use horizon_items::item_data::*;}
pub mod item_render {use super::*;use horizon_world::nbt::Tag;use serde_json::Value;__STACK_VIEW__}
pub mod slot_bridge {pub fn outgoing_sent(_:u64,_:&horizon_protocol::packets::play::Serverbound,_:bool) {}}
#[path="packet-bridge.rs"]pub mod packet_bridge;
#[path="__POLICY__"]pub mod elytra_recast;
#[path="__CHEST__"]pub mod elytra_recast_chest;
#[path="__MOTION_ADAPTER__"]pub mod elytra_recast_motion;
#[path="__TESTS__"]#[cfg(test)]mod policy_tests;
static RUNNING:AtomicBool=AtomicBool::new(false);
static LOCK:Mutex<()>=Mutex::new(());
#[allow(dead_code)]pub mod mod_api {
    use super::*;pub trait RenderHooks:Send+Sync{}
    #[allow(unused_imports)]mod host {include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}pub use host::*;
    mod generated {include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use generated::*;
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(Box::as_ref)}
    pub fn mods()->&'static [Box<dyn Hooks>]{static MODS:LazyLock<Vec<Box<dyn Hooks>>>=LazyLock::new(||vec![Box::new(Adapter)]);&MODS}
    struct Adapter;impl Hooks for Adapter {fn client_elytra_recast_running(&self,_:&dyn Host,e:&mut ClientElytraRecastRunningEvent){e.running=RUNNING.load(Ordering::Relaxed);}}
}
#[allow(dead_code)]struct Session {writer:Arc<Mutex<Option<ConnWriter>>>,configuring:Arc<AtomicBool>,serial:u64,debug:bool}
impl Session {__SESSION__}
struct Remote {session:Session,entity_id:i32}
#[derive(Default)]struct Status {levitation:Option<i32>}
#[derive(Default)]struct Fluids {was_touching_water:bool}
#[derive(Default)]struct Health {tick_count:i32}
impl Health {fn dead_or_dying(&self)->bool {false}}
#[derive(Clone)]pub struct Player {
    client_elytra_recast:elytra_recast::ElytraRecast,
    flying:bool,is_passenger:bool,fall_flying:bool,was_auto_jump:bool,no_jump_delay:i32,
    status:Status,fluids:Fluids,server_copy:bool,replaying:bool,
    health:Health,jumping:bool,in_powder_snow:bool,ticks_frozen:i32,
    block_effects:bool,auto_spin_attack_ticks:i32,fall_fly_ticks:i32,
}
impl Clone for Status {fn clone(&self)->Self{Self{levitation:self.levitation}}}
impl Clone for Fluids {fn clone(&self)->Self{Self{was_touching_water:self.was_touching_water}}}
impl Clone for Health {fn clone(&self)->Self{Self{tick_count:self.tick_count}}}
impl Player {
    __IDENTITY__
    fn client_movement_queries(&self)->MovementQueries{MovementQueries{reset_jump_delay:false}}
    fn drop_tiny_velocity(&mut self){}
    fn bounds(&self)->i32{0}
    fn can_freeze(&self)->bool{false}
    fn remove_frost(&mut self)->Result<()>{Ok(())}
    fn try_add_frost(&mut self,_:bool)->Result<()>{Ok(())}
    fn is_fully_frozen(&self)->bool{false}
    fn new()->Self{Self{client_elytra_recast:Default::default(),flying:false,is_passenger:false,fall_flying:false,was_auto_jump:false,no_jump_delay:7,status:Default::default(),fluids:Default::default(),server_copy:false,replaying:false,health:Default::default(),jumping:false,in_powder_snow:false,ticks_frozen:0,block_effects:false,auto_spin_attack_ticks:0,fall_fly_ticks:0}}
}
struct MovementQueries{reset_jump_delay:bool}
struct State {name:String}struct Registry;impl Registry{fn state(&self,_:u32)->Option<State>{None}}
struct Assets{registry:Registry}
pub mod motion {
    use super::*;
    pub struct MotionWorld<'p,'w,'r>{pub player:&'p mut Player,pub(super) assets:&'r Assets,pub(super) trace:&'w RefCell<Vec<&'static str>>,pub(super) climbable:bool,pub(super) fail:bool}
    __TICK_INPUT__
    impl MotionWorld<'_,'_,'_>{
        pub fn climbable(&mut self)->Result<bool>{self.trace.borrow_mut().push("climbable");if self.fail {Err("provider".into())}else{Ok(self.climbable)}}
        fn jump_phase(&mut self)->Result<()>{self.trace.borrow_mut().push("jump");Ok(())}
        fn player_travel(&mut self,_:Vec3)->Result<()>{self.trace.borrow_mut().push("travel");Ok(())}
        fn apply_effects_from_blocks(&mut self)->Result<()>{self.trace.borrow_mut().push("blocks");Ok(())}
        fn support(&mut self,_:f64)->Result<BlockPos>{Ok(BlockPos::new(0,0,0))}
        fn block(&self,_:BlockPos)->u32{0}
        fn update_soul_speed_block(&mut self)->Result<()>{Ok(())}
        fn freeze_damage(&mut self)->Result<()>{Ok(())}
        fn check_auto_spin_attack(&mut self,_:i32){}
        __DRY_TICK__
        // Driver shell: the extracted real dry tick invokes the exact tail.
        pub(super) fn controlled_tick_with_ai_tail(&mut self,jump:bool,tail:&mut dyn FnMut(&mut Self)->Result<()>)->Result<()>{
            self.dry_tick_with_ai_tail(TickInput::PreparedLocal{direction:[0.,0.],jumping:jump},tail)?;
            if self.player.fall_flying{self.player.fall_fly_ticks+=1;}else{self.player.fall_fly_ticks=0;}
            self.trace.borrow_mut().push("tick-bookkeeping");Ok(())
        }
    }
    mod movement_support {pub const LEGACY_SUPPORT_OFFSET:f64=0.;}
}
struct ItemRenderer {models:ItemModels}struct Pack{item_renderer:ItemRenderer}
#[derive(Clone,Copy)]enum Action{Jump}
struct Keys{jump:bool}impl Keys{fn held(&self,_:Action)->bool{self.jump}}
struct Audio{stops:Vec<(String,usize)>}
impl Audio{fn stop_sounds(&mut self,s:Option<&str>,source:Option<usize>){self.stops.push((s.unwrap().into(),source.unwrap()));}}
pub struct Game<'a>{player:Option<Player>,armor:[Option<ItemStack>;4],remote:Option<Remote>,audio:Audio,pack:Pack,keys:Keys,lan_acting_uuid:Option<u128>,window:bool,trace:RefCell<Vec<&'static str>>,marker:std::marker::PhantomData<&'a ()>}
impl Game<'_>{
    fn native_window(&self)->bool{self.window}
}
// Match game/remote.rs's sibling boundary: item_render is deliberately absent
// from this module's imports, so an unqualified capture path cannot compile.
mod remote_scope {
use crate::{Action,Game,Result};
use horizon_client::motion::MotionWorld;
use horizon_protocol::packets::play::{self,Serverbound as S};
impl Game<'_>{
    pub(super) fn tick_ai(&mut self,input:u8,climbable:bool)->Result<bool>{
        __CAPTURE__
        let player=self.player.as_mut().unwrap();
        let was_gliding=player.fall_flying;
        let assets=crate::Assets{registry:crate::Registry};
        let mut world=MotionWorld{player,assets:&assets,trace:&self.trace,climbable,fail:false};
        world.controlled_tick_with_ai_tail(input&16!=0,&mut elytra_ai_tail)?;
        let started_gliding=world.player.fall_flying&&!was_gliding&&!elytra_recast_sent;
        Ok(started_gliding)
    }
}
}
mod native_hooks{use super::*;impl Game<'_>{__NATIVE__}}
mod generated_host {include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}

#[cfg(test)]mod native_tests {
use super::*;
fn setup()->(Game<'static>,Connection){
    let (client,server)=Pipe::pair();let (_,writer)=Connection::local(client).split();
    let models=ItemModels{prototypes:serde_json::from_str(include_str!("__ITEMS__")).unwrap(),..Default::default()};
    (Game{player:Some(Player::new()),armor:[None,Some(ItemStack::new("minecraft:elytra",1)),None,None],remote:Some(Remote{session:Session{writer:Arc::new(Mutex::new(Some(writer))),configuring:Arc::new(AtomicBool::new(false)),serial:419,debug:false},entity_id:71}),audio:Audio{stops:vec![]},pack:Pack{item_renderer:ItemRenderer{models}},keys:Keys{jump:false},lan_acting_uuid:None,window:true,trace:RefCell::new(vec![]),marker:std::marker::PhantomData},Connection::local(server))
}
fn next(c:&mut Connection)->S{let(id,data)=c.recv().unwrap();S::decode(id,&data).unwrap()}
fn prime(g:&mut Game<'_>){g.player.as_mut().unwrap().client_elytra_recast.previous_elytra=true;}

#[test]fn generated_native_query_actual_dry_tail_and_ordinary_transport_are_synchronous(){
    let _lock=LOCK.lock().unwrap();RUNNING.store(true,Ordering::Relaxed);
    let(mut g,mut peer)=setup();prime(&mut g);assert!(g.native_client_recasts_elytra());
    assert!(!g.tick_ai(16,false).unwrap(),"recast packet already sent; ordinary start detection must suppress duplicate");
    assert_eq!(next(&mut peer),S::PlayerCommand(play::PlayerCommand{id:71,action:6,data:0}));
    let p=g.player.as_ref().unwrap();assert!(p.fall_flying);assert!(!p.client_elytra_recast.previous_elytra);assert_eq!(p.no_jump_delay,0);assert_eq!(p.fall_fly_ticks,1);
    assert_eq!(g.audio.stops,[("minecraft:item.elytra.flying".into(),7)]);
    assert_eq!(g.trace.borrow().as_slice(),["jump","travel","climbable","tick-bookkeeping"]);
    g.tick_ai(16,false).unwrap();assert!(g.player.as_ref().unwrap().client_elytra_recast.previous_elytra);
}
#[test]fn cancellation_and_packet_mutation_use_real_composed_session_path(){
    struct Interceptor{cancel:Arc<AtomicBool>,seen:Arc<Mutex<Vec<String>>>}
    impl packet_bridge::PacketInterceptor for Interceptor {
        fn outgoing(&self,event:&mut packet_bridge::PacketEvent<'_,S>){
            if let S::PlayerCommand(p)=event.packet {assert_eq!(p.action,6);assert_eq!(p.data,0);assert_eq!(p.id,71);assert!(event.context.original());self.seen.lock().unwrap().push("ordinary".into());if self.cancel.load(Ordering::Relaxed){event.cancel();}else{p.action=9;}}
        }
    }
    let _lock=LOCK.lock().unwrap();RUNNING.store(true,Ordering::Relaxed);let cancel=Arc::new(AtomicBool::new(false));let seen=Arc::new(Mutex::new(vec![]));
    let previous=packet_bridge::set_interceptor(Some(Arc::new(Interceptor{cancel:cancel.clone(),seen:seen.clone()})));
    let(mut g,mut peer)=setup();prime(&mut g);g.tick_ai(16,false).unwrap();
    assert_eq!(next(&mut peer),S::PlayerCommand(play::PlayerCommand{id:71,action:9,data:0}));
    g.player.as_mut().unwrap().fall_flying=false;prime(&mut g);cancel.store(true,Ordering::Relaxed);g.tick_ai(16,false).unwrap();
    let p=g.player.as_ref().unwrap();assert!(p.fall_flying);assert_eq!(p.no_jump_delay,0,"cancellation never rolls back local flag/cooldown");
    g.remote.as_ref().unwrap().session.send(S::ClientTickEnd(Default::default()));assert!(matches!(next(&mut peer),S::ClientTickEnd(_)));
    assert_eq!(seen.lock().unwrap().len(),2);packet_bridge::set_interceptor(previous);
}
#[test]fn local_identity_and_component_patch_are_respected_without_screen_guard(){
    let _lock=LOCK.lock().unwrap();RUNNING.store(true,Ordering::Relaxed);
    for denied in 0..4{let(mut g,_)=setup();prime(&mut g);match denied{0=>g.window=false,1=>g.lan_acting_uuid=Some(4),2=>g.player.as_mut().unwrap().server_copy=true,_=>g.player.as_mut().unwrap().replaying=true};g.tick_ai(16,false).unwrap();assert!(g.audio.stops.is_empty());assert!(g.player.as_ref().unwrap().client_elytra_recast.previous_elytra);}
    let(mut g,_)=setup();prime(&mut g);g.armor[1]=Some(ItemStack::new("minecraft:stick",1));g.armor[1].as_mut().unwrap().data_mut().set_other("minecraft:glider",Some(horizon_world::nbt::Tag::compound([] as [(&str,horizon_world::nbt::Tag);0])));g.keys.jump=true;
    g.tick_ai(0,false).unwrap();assert!(g.player.as_ref().unwrap().fall_flying,"equippable is not required; physical key alone works");
    let(mut g,_)=setup();prime(&mut g);g.armor[1].as_mut().unwrap().data_mut().set_other("!minecraft:glider",Some(horizon_world::nbt::Tag::compound([] as [(&str,horizon_world::nbt::Tag);0])));g.tick_ai(16,false).unwrap();assert!(!g.player.as_ref().unwrap().fall_flying);assert_eq!(g.player.as_ref().unwrap().no_jump_delay,0);assert_eq!(g.audio.stops.len(),1);
}
#[test]fn disabled_tail_updates_entity_history_without_side_effects(){
    let _lock=LOCK.lock().unwrap();RUNNING.store(false,Ordering::Relaxed);let(mut g,_)=setup();g.player.as_mut().unwrap().fall_flying=true;g.tick_ai(16,false).unwrap();assert!(g.player.as_ref().unwrap().client_elytra_recast.previous_elytra);g.player.as_mut().unwrap().fall_flying=false;g.tick_ai(16,false).unwrap();assert!(!g.player.as_ref().unwrap().client_elytra_recast.previous_elytra);assert!(g.audio.stops.is_empty());
}
}
