#![cfg(test)]
extern crate self as horizon_client;
pub use horizon_items::cooldowns;
pub mod items {
    pub use horizon_items::items::{ItemStack, bucket_contents};
    use std::sync::Arc;
    #[derive(Clone)]
    pub struct ItemInfo {
        pub id: Arc<str>, pub use_cooldown: Option<super::cooldowns::UseCooldown>,
        pub equip_slot: Option<()>, pub food: Option<()>, pub consumable: Option<()>,
        pub swappable: bool, pub blocks_attacks: Option<()>,
    }
}
// Resource-only branches outside the rods exercised here are explicit stubs.
pub mod spear { pub struct Kinetic; impl Kinetic {pub fn of(_: &str)->Option<Self>{None}} }
pub mod mod_api {pub use horizon_protocol::packets::play::{Clientbound as ClientboundPlayPacket,Serverbound as ServerboundPlayPacket};}
#[path="PACKET_BRIDGE_PATH"] pub mod packet_bridge;
#[path="FISH_BRIDGE_PATH"] pub mod fish_bridge;
#[path="SLOT_BRIDGE_PATH"] pub mod slot_bridge;
#[path="PACKETS_PATH"] pub mod packets;
#[path="SLOTS_PATH"] pub mod slots;
#[path="FISHING_PACKETS_PATH"] pub mod fishing_packets;
#[path="BEHAVIOR_TESTS_PATH"] mod behavior_tests;
#[path="SOUNDS_PATH"] mod auto_fish_sounds;
mod remote {
    pub fn builtin_registries(_: &std::path::Path)->Option<std::sync::Arc<horizon_protocol::registry::Builtin>> {
        let mut builtin=horizon_protocol::registry::Builtin::default();
        builtin.registries.insert("minecraft:sound_event".into(),horizon_protocol::registry::Ids::from_names(vec!["minecraft:entity.fishing_bobber.splash".into()]));
        Some(std::sync::Arc::new(builtin))
    }
}

#[allow(dead_code)]
mod fixture {
use super::*;
use horizon_protocol::{connection::{Connection,ConnWriter,Pipe},packets::play,registry::{Builtin,Ids}};
use horizon_world::nbt::Tag;
use items::{ItemInfo,ItemStack};
use play::Serverbound as S;
use std::{cell::Cell,collections::HashMap,sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}}};

#[derive(Clone,Copy,PartialEq,Eq,Debug)] enum Hand {Main,Off}
mod shield {pub const OFFHAND_SLOT:usize=40;}
#[derive(Clone,Copy,Default,Debug,PartialEq)] struct Vec3 {x:f64,y:f64,z:f64}
impl Vec3 {fn new(x:f64,y:f64,z:f64)->Self{Self{x,y,z}} fn subtract(self,rhs:Self)->Self{Self::new(self.x-rhs.x,self.y-rhs.y,self.z-rhs.z)}}
#[derive(Default)] struct Player {server_copy:bool,replaying:bool,fall_flying:bool,yaw:f32,pitch:f32,position:Vec3}
impl Player { /* IDENTITY */ }
#[derive(Clone,Default)] struct Lerp;
impl Lerp {fn step(&self,position:Vec3)->Vec3{position} fn moved(&mut self,_:Vec3){}}
#[derive(Clone)] struct Hook {id:u32,owner:u32,position:Vec3,previous:Vec3,velocity:Vec3,state:u8,on_ground:bool,random:(),lerp:Lerp}
impl Hook {fn remote(id:u32,position:Vec3)->Self{Self{id,owner:0,position,previous:position,velocity:Vec3::default(),state:0,on_ground:false,random:(),lerp:Lerp}} fn id(&self)->u32{self.id}}
fn push_remote_hook(hooks:&mut Vec<Hook>,id:u32,owner:u32,at:Vec3,velocity:Vec3){let mut h=Hook::remote(id,at);h.owner=owner;h.velocity=velocity;hooks.push(h);}
fn remove_remote_hook_from(hooks:&mut Vec<Hook>,id:u32){hooks.retain(|h|h.id!=id);}
struct Mob {id:u32,position:Vec3}
struct NativeInput;
impl NativeInput {fn claimant(&self,_:&str)->Option<()>{None}}
struct Audio;
impl Audio {fn play(&mut self,_:&str,_:Option<[f64;3]>,_:f32,_:f32){}}
#[derive(Clone)] struct Pos {x:i32,y:i32,z:i32}
type BucketResult=(bool,Option<Vec<()>>,Vec<(Pos,&'static str,f32,f32)>);
struct Consumable;
impl Consumable {fn ticks(&self)->i32{0}}
#[derive(Default)] struct Heights {used:Vec<Hand>}
impl Heights {fn item_used(&mut self,hand:Hand){self.used.push(hand);}}
struct Session {writer:Arc<Mutex<Option<ConnWriter>>>,configuring:Arc<AtomicBool>,serial:u64,debug:bool}
impl Session {fn serial(&self)->u64{self.serial} /* SESSION */ }
struct Remote {session:Session,joined:bool,reconfiguring:Option<()>,builtin:Arc<Builtin>,entity_id:i32,carried:Cell<Option<usize>>,sequence:i32}
struct Options {resources:std::path::PathBuf}
struct Game {
    player:Option<Player>,lan_acting_uuid:Option<u128>,window:bool,remote:Option<Remote>,
    selected:usize,inventory:Vec<Option<ItemStack>>,offhand:Option<ItemStack>,hand:Hand,
    hook:Option<Hook>,remote_hooks:Vec<Hook>,mobs:Vec<Mob>,native_input:NativeInput,
    items:HashMap<Arc<str>,ItemInfo>,cooldowns:cooldowns::Cooldowns,audio:Audio,heights:Heights,
    yaw:f32,pitch:f32,use_failed:bool,spectator:bool,swings:Vec<Hand>,options:Options,
    prediction:Arc<AtomicBool>,predicted_angles:Vec<(f32,f32,f32,f32)>,local_uses:usize,drop_local_hook:bool,
}
impl Game {
    fn native_window(&self)->bool{self.window}
    fn spectator(&self)->bool{self.spectator}
    fn stack_in(&self,slot:usize)->Option<&ItemStack>{if slot==40 {self.offhand.as_ref()} else {self.inventory[slot].as_ref()}}
    fn items(&self)->&HashMap<Arc<str>,ItemInfo>{&self.items}
    fn start_mod_use(&mut self,_:&ItemStack)->bool{false}
    fn equip_from_hand(&mut self,_:&ItemStack)->bool{false}
    fn swing_arm(&mut self){if !packet_bridge::cancel_local_swing(){self.swings.push(self.hand);}}
    fn remote_use_bucket(&mut self,_:&ItemStack)->BucketResult{(false,None,vec![])}
    fn predict_blocks(&mut self,_:&[()],_:i32){}
    fn spawn_egg_allowed(&self,_:&str)->bool{false}
    fn bucket_hit(&self)->Option<()>{None}
    fn consume_held(&mut self){}
    fn apply_use_cooldown(&mut self,_:&ItemStack){}
    fn start_eating(&mut self,_:&ItemInfo)->bool{false}
    fn use_finds_no_target(&mut self,_:&ItemStack)->bool{false}
    fn consumable(&self,_:&ItemStack)->Option<Consumable>{None}
    fn remote_use_book(&mut self,_:&ItemStack)->bool{false}
    fn remote_start_held_use(&mut self,_:&ItemStack)->bool{
        let p=self.player.as_ref().unwrap();
        self.predicted_angles.push((self.yaw,self.pitch,p.yaw,p.pitch));
        self.prediction.store(true,Ordering::Release);
        true
    }
    fn blocks_attacks(&self,_:&ItemStack)->Option<()>{None}
    fn patched_held_use(&self,_:&ItemStack)->bool{false}
    fn use_hand(&mut self,hit:Option<()>)->Result<bool,()>{
        assert!(hit.is_none());self.local_uses+=1;
        if self.held().is_none(){return Ok(false);}
        let p=self.player.as_ref().unwrap();
        self.predicted_angles.push((self.yaw,self.pitch,p.yaw,p.pitch));
        self.hook=Some(Hook::remote(72,Vec3::new(3.0,4.0,5.0)));
        self.native_publish_fishing_state();
        self.swing_arm();Ok(true)
    }
    fn client_hook_step(&mut self,_:u32,_:Option<Vec3>,(mut position,velocity,state,ground):(Vec3,Vec3,u8,bool),_:&mut ())->(Vec3,Vec3,u8,bool){position.x+=0.25;(position,velocity,state,ground)}
    fn tick_fishing_committed(&mut self){if self.drop_local_hook {self.hook.take();return;}self.local_uses+=1;}
    /* HELPERS */
    /* HOTBAR */
    /* HELD */
    /* REMOTE_USE */
    /* HOOK_METHODS */
}
/* REMOTE_FACTS */

#[test]
fn exact_native_snapshot_packet_listener_and_item_prediction_use_actual_pipe_and_slots() {
    use packets::{PacketEvent,PacketListener,PacketPipeline};
    struct ChangeSound;
    impl PacketListener for ChangeSound {
        fn incoming(&self,event:&mut PacketEvent<'_,play::Clientbound>){if let play::Clientbound::Sound(p)=event.packet{p.sound=(1,None);event.cancel();}}
    }
    struct UseObserver {prediction:Arc<AtomicBool>,cancel:AtomicBool}
    impl PacketListener for UseObserver {
        fn outgoing(&self,event:&mut PacketEvent<'_,S>){
            if let S::UseItem(p)=event.packet {
                assert!(self.prediction.load(Ordering::Acquire),"prediction completes before ordinary send");
                if self.cancel.load(Ordering::Relaxed){event.cancel();} else {p.yaw+=1.0;}
            }
        }
    }
    let pipeline=Arc::new(PacketPipeline::default());
    let prior=packet_bridge::set_interceptor(Some(pipeline.clone()));
    let slot_service=Arc::new(slots::SlotService::default());
    let prior_slots=slot_bridge::set_policy(Some(slot_service.clone()));
    slot_service.select_slot_silently(liquidbounce_behaviors::silent_hotbar::Requester::allocate(),7,100,||Some(2)).unwrap();
    let fish=Arc::new(fishing_packets::AutoFishService::default());
    fish.set_running(true);
    let sound_mutator=pipeline.add_listener(100,Arc::new(ChangeSound));
    pipeline.add_listener(0,fish.clone());
    let prediction=Arc::new(AtomicBool::new(false));
    let observer=Arc::new(UseObserver{prediction:prediction.clone(),cancel:AtomicBool::new(false)});
    pipeline.add_listener(0,observer.clone());
    let (client,server)=Pipe::pair();
    let (mut client_reader,client_writer)=Connection::local(client).split();
    let mut server=Connection::local(server);
    let mut builtin=Builtin::default();
    builtin.registries.insert("minecraft:sound_event".into(),Ids::from_names(vec!["minecraft:entity.pig.ambient".into(),"minecraft:entity.fishing_bobber.splash".into()]));
    let mut inventory=vec![None;36];
    inventory[2]=Some(ItemStack::new("minecraft:stone",1));
    inventory[7]=Some(ItemStack::new("minecraft:fishing_rod",1));
    let infos=["minecraft:fishing_rod","minecraft:air"].map(|id|{let info=ItemInfo{id:Arc::from(id),use_cooldown:None,equip_slot:None,food:None,consumable:None,swappable:false,blocks_attacks:None};(Arc::from(id),info)}).into_iter().collect();
    let mut game=Game{player:Some(Player{yaw:10.0,pitch:20.0,..Player::default()}),lan_acting_uuid:None,window:true,
        remote:Some(Remote{session:Session{writer:Arc::new(Mutex::new(Some(client_writer))),configuring:Arc::new(AtomicBool::new(false)),serial:91,debug:false},joined:true,reconfiguring:None,builtin:Arc::new(builtin),entity_id:3,carried:Cell::new(None),sequence:0}),
        selected:2,inventory,offhand:Some(ItemStack::new("minecraft:fishing_rod",1)),hand:Hand::Main,hook:None,remote_hooks:vec![],mobs:vec![],native_input:NativeInput,items:infos,cooldowns:cooldowns::Cooldowns::default(),audio:Audio,heights:Heights::default(),yaw:10.0,pitch:20.0,use_failed:false,spectator:false,swings:vec![],options:Options{resources:Default::default()},prediction:prediction.clone(),predicted_angles:vec![],local_uses:0,drop_local_hook:false};
    assert_eq!(game.native_client_sound_event_names(),["minecraft:entity.fishing_bobber.splash"]);
    assert_eq!(game.native_client_fishing_rod_hand(),Some(0));
    game.inventory[7]=None;
    assert_eq!(game.native_client_fishing_rod_hand(),Some(1));
    game.inventory[7]=Some(ItemStack::new("minecraft:fishing_rod",1));
    game.spawn_remote_hook(44,Vec3::new(8.0,2.0,3.0),Vec3::default(),3);
    assert_eq!(fish_bridge::snapshot().session,91);
    assert_eq!(fish_bridge::snapshot().hook,Some([8.0,2.0,3.0]));
    game.tick_remote_hooks();
    assert_eq!(fish_bridge::snapshot().hook,Some([8.25,2.0,3.0]));
    let sound=play::Clientbound::Sound(play::Sound{sound:(0,None),source:0,x:66,y:16,z:24,volume:1.0,pitch:1.0,seed:5});
    server.send(&sound.encode()).unwrap();
    let (id,bytes)=client_reader.recv().unwrap();
    let mut sound=play::Clientbound::decode(id,&bytes).unwrap();
    assert!(packet_bridge::incoming(91,&mut sound));
    assert!(fish.behavior.lock().unwrap().caught_fish(),"a cancelled ordinary event still reaches AutoFish and uses prior listener mutations");
    fish.behavior.lock().unwrap().on_disabled();
    pipeline.remove_listener(sound_mutator);
    assert!(!packet_bridge::incoming(92,&mut sound));
    assert!(!fish.behavior.lock().unwrap().caught_fish());
    game.remove_remote_hook(44);
    assert_eq!(fish_bridge::snapshot().hook,None);
    packet_bridge::incoming(91,&mut sound);
    assert!(!fish.behavior.lock().unwrap().caught_fish());
    game.spawn_remote_hook(45,Vec3::new(8.25,2.0,3.0),Vec3::default(),3);
    if let play::Clientbound::Sound(p)=&mut sound {p.sound=(-1,Some(("minecraft:entity.fishing_bobber.splash".into(),None)));}
    packet_bridge::incoming(91,&mut sound);
    assert!(!fish.behavior.lock().unwrap().caught_fish(),"inline event is not the built-in registry object");
    fish.behavior.lock().unwrap().settings.sounds.insert(None);
    packet_bridge::incoming(91,&mut sound);
    assert!(fish.behavior.lock().unwrap().caught_fish());
    fish.behavior.lock().unwrap().on_disabled();
    let mut replay_event=PacketEvent::new(packet_bridge::PacketContext{session:91,origin:packet_bridge::TransferOrigin::Incoming,delivery:packet_bridge::Delivery::IncomingReplay},&mut sound);
    packets::PacketInterceptor::incoming(&*pipeline,&mut replay_event);
    assert!(!fish.behavior.lock().unwrap().caught_fish());
    assert!(game.native_client_use_item(1,720.0,-190.0));
    assert_eq!(game.predicted_angles,[(720.0,-90.0,720.0,-90.0)]);
    assert_eq!((game.yaw,game.pitch),(10.0,20.0));
    assert_eq!((game.player.as_ref().unwrap().yaw,game.player.as_ref().unwrap().pitch),(10.0,20.0));
    assert_eq!(game.heights.used,[Hand::Off]);
    assert_eq!(game.swings,[Hand::Off]);
    let next=|server:&mut Connection|{let (id,body)=server.recv().unwrap();S::decode(id,&body).unwrap()};
    assert_eq!(next(&mut server),S::SetCarriedItem(play::SetCarriedItem{slot:7}));
    assert_eq!(next(&mut server),S::UseItem(play::UseItem{hand:1,sequence:1,yaw:721.0,pitch:-190.0}));
    observer.cancel.store(true,Ordering::Relaxed);
    prediction.store(false,Ordering::Relaxed);
    assert!(game.native_client_use_item(0,-3.0,4.0)); // local prediction still happens.
    observer.cancel.store(false,Ordering::Relaxed);
    assert!(game.native_client_use_item(0,5.0,6.0));
    assert_eq!(next(&mut server),S::UseItem(play::UseItem{hand:0,sequence:3,yaw:6.0,pitch:6.0}));
    game.cooldowns.add(Arc::from("minecraft:fishing_rod"),10);
    assert!(!game.native_client_use_item(0,7.0,8.0));
    assert_eq!(next(&mut server),S::UseItem(play::UseItem{hand:0,sequence:4,yaw:8.0,pitch:8.0}),"cooldown PASS still sends the use packet");
    let before=fish_bridge::snapshot().hook;
    for identity in 0..3 {
        game.player.as_mut().unwrap().server_copy=identity==0;
        game.player.as_mut().unwrap().replaying=identity==1;
        game.lan_acting_uuid=(identity==2).then_some(5);
        assert_eq!(game.native_client_fishing_rod_hand(),None);
        assert!(!game.native_client_use_item(0,0.0,0.0));
        game.native_publish_fishing_state();
        assert_eq!(fish_bridge::snapshot().hook,before,"temporary guest/replay/fake state must not replace the real local snapshot");
    }
    game.player.as_mut().unwrap().server_copy=false;
    game.player.as_mut().unwrap().replaying=false;
    game.lan_acting_uuid=None;
    game.clear_remote_hooks();
    assert_eq!(fish_bridge::snapshot().hook,None);
    game.remote=None;
    game.cooldowns=Default::default();
    assert!(game.native_client_use_item(0,15.0,16.0));
    assert_eq!(game.local_uses,1);
    assert_eq!(game.predicted_angles.last(),Some(&(15.0,16.0,15.0,16.0)));
    assert_eq!((game.yaw,game.pitch),(10.0,20.0));
    game.drop_local_hook=true;
    game.tick_fishing(); // Exact wrapper commits an early-return removal.
    assert_eq!(fish_bridge::snapshot().hook,None);
    game.player=None;
    game.native_publish_fishing_state();
    assert_eq!(fish_bridge::snapshot().session,0);
    fish_bridge::clear();
    packet_bridge::set_interceptor(prior);
    slot_bridge::set_policy(prior_slots);
}
}
