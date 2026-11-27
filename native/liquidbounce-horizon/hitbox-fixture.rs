#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
use horizon_core::geometry::{Aabb,Vec3};
use std::sync::{Mutex,OnceLock};
mod targeting;
mod hitbox;
#[allow(clippy::too_many_arguments)] mod gizmos;
mod pose {#[derive(Clone,Copy,Default,PartialEq,Eq)]pub enum Pose{#[default]Standing,Sleeping}}
mod command {pub mod args {pub fn normalize(id:&str)->String {if id.contains(':'){id.into()}else{format!("minecraft:{id}")}}}}
#[derive(Clone)]struct Kind{id:String}
#[derive(Default)]struct Extra{avatar_pose:pose::Pose}
struct Mob{id:u32,kind:Kind,position:Vec3,removed:bool,health:f32,invisible:bool,effect:bool,extra:Extra,bounds:Aabb}
impl Mob {fn alive(&self)->bool{self.health>0.0}fn has_effect(&self,_:&str)->bool{self.effect}fn bounds(&self)->Aabb{self.bounds}}
struct Stand{id:u32,position:Vec3,health:f32,invisible:bool,marker:bool,bounds:Aabb}
impl Stand{fn bounds(&self)->Aabb{self.bounds}}
struct Dragon{id:u32,position:Vec3,health:f32}
impl Dragon{fn alive(&self)->bool{self.health>0.0}}
struct Health;impl Health{fn health(&self)->f32{20.0}}
struct Player{position:Vec3,health:Health,sleeping:bool}
struct Remote{entity_id:i32}
struct Border{geometry:horizon_core::geometry::border::BorderGeometry}
impl Border{fn geometry(&self)->&horizon_core::geometry::border::BorderGeometry{&self.geometry}}
struct Game<'a>{mobs:Vec<Mob>,armor_stands:Vec<Stand>,dragon:Option<Dragon>,player:Option<Player>,remote:Option<Remote>,
    border:Border,riding:Option<u32>,window:bool,server_copy:bool,replaying:bool,lan:bool,creative:bool,
    entity_reach:f64,block_reach:f64,yaw:f32,pitch:f32,marker:std::marker::PhantomData<&'a ()>}
impl Default for Game<'_>{fn default()->Self{Self{mobs:vec![],armor_stands:vec![],dragon:None,
    player:Some(Player{position:Vec3::ZERO,health:Health,sleeping:false}),remote:Some(Remote{entity_id:7}),
    border:Border{geometry:horizon_core::geometry::border::BorderGeometry::new([0.,0.],100.,29999984).unwrap()},
    riding:None,window:true,server_copy:false,replaying:false,lan:false,creative:false,entity_reach:3.,block_reach:4.5,yaw:0.,pitch:0.,marker:std::marker::PhantomData}}}
impl Game<'_>{
    fn client_reach_player(&self)->bool{self.window&&!self.server_copy&&!self.replaying&&!self.lan&&self.player.is_some()}
    fn native_client_attack_range(&self,min:&mut f32,max:&mut f32,cmin:&mut f32,cmax:&mut f32){
        if self.client_reach_player(){let _=min;let _=cmin;*max+=1.;*cmax+=1.;}}
    fn own_invisible(&self)->bool{false}
    fn remote_player_name(&self,id:u32)->Option<&str>{(id==42).then_some("Alice")}
    fn native_entity(&self,_:u32)->Option<u32>{None}
    fn native_entity_kind(&self,_:u32)->Option<String>{None}
    fn native_entity_position(&self,_:u32)->Option<[f64;3]>{None}
    fn aim(&self)->Option<([f64;3],[f64;3])>{Some(([0.,0.,0.],[0.,0.,1.]))}
    fn remote_camera_id(&self)->Option<u32>{None}
    fn eye_position(&self)->Option<[f64;3]>{Some([0.,0.,0.])}
    fn known_player_movement(&self)->Vec3{Vec3::ZERO}
    fn clip_ray(&self,_:[f64;3],_:[f64;3],_:f64)->Option<Hit>{None}
    fn client_reach_target(&self)->Option<ReachHit>{None}
    __MOB_PICK__
    __STAND_PICK__
    __ATTACK_PICK__
}
struct ReachHit{kind:u8,index:usize,distance:f64,distance_squared:f64,point:[f64;3]}
struct Hit{point:[f64;3]}
fn v3(p:[f64;3])->Vec3{Vec3::new(p[0],p[1],p[2])}
fn interact_location(point:Vec3,feet:Vec3)->[f64;3]{[point.x-feet.x,point.y-feet.y,point.z-feet.z]}
fn view_vector(_:f32,_:f32)->[f64;3]{[0.,0.,1.]}
__RAY_BOX__
__ATTACK_REACH__
mod spear {use super::*;type Vec3d=Vec3;__HITS_ALONG__ __HITS_IN_REACH__}
#[derive(Default)]struct State{hitbox:hitbox::Hitbox,running:bool,targets:targeting::GlobalTargets,friends:targeting::FriendBook,tags:targeting::TagCache}
fn state()->std::sync::MutexGuard<'static,State>{static STATE:OnceLock<Mutex<State>>=OnceLock::new();STATE.get_or_init(||Mutex::new(State::default())).lock().unwrap()}
fn eligible(host:&dyn mod_api::Host,id:i32,m:&mut State)->bool{
    let value:serde_json::Value=serde_json::from_str(&host.client_targeting_entity(id)).unwrap();
    let Some(kind)=value["kind"].as_str()else{return false;};
    let entity=targeting::TargetEntity{kind:targeting::EntityKind::for_type(kind),alive:value["alive"]==true,invisible:value["invisible"]==true,
        sleeping:value["sleeping"]==true,is_local:value["is_local"]==true,has_local_passenger:value["has_local_passenger"]==true,within_border:value["within_border"]==true};
    let info=m.tags.get(id,||m.friends.tag(entity.kind,value["player_name"].as_str()));
    m.targets.combat.should_be_attacked(&entity,info)
}
mod mod_api{
    pub fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(|m|&**m)}
    #[allow(unused_imports)] mod render{include!(concat!(env!("CARGO_MANIFEST_DIR"),"/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"));}pub use render::*;
    #[allow(unused_imports)] mod host{include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}pub use host::*;
    mod hooks{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use hooks::*;
    struct HooksImpl;
    impl Hooks for HooksImpl{
        fn client_entity_margin(&self,host:&dyn Host,event:&mut ClientEntityMarginEvent){let mut m=crate::state();if m.running{event.margin=m.hitbox.pick_margin(event.margin,crate::eligible(host,event.entity,&mut m));}}
        fn client_debug_hitbox_margin(&self,host:&dyn Host,event:&mut ClientDebugHitboxMarginEvent){let mut m=crate::state();if m.running{event.margin=m.hitbox.debug_margin(crate::eligible(host,event.entity,&mut m));}}
        fn client_hitbox_component_margin(&self,_:&dyn Host,event:&mut ClientHitboxComponentMarginEvent){let m=crate::state();if m.running{event.margin=m.hitbox.component_margin(event.margin);}}
    }
    pub fn mods()->&'static[Box<dyn Hooks>]{static MODS:std::sync::OnceLock<Vec<Box<dyn Hooks>>>=std::sync::OnceLock::new();MODS.get_or_init(||vec![Box::new(HooksImpl)])}
}
mod game{pub type Game<'a>=super::Game<'a>;mod native_hooks{use crate::{mod_api,Game};impl Game<'_>{__HOOKS__ __CONTAINS__}}
    mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}}
static SCENE:Mutex<()>=Mutex::new(());
fn reset(){*state()=State{running:true,..State::default()};}
fn add(game:&mut Game<'_>,id:u32,kind:&str,x:f64,z:f64){game.mobs.push(Mob{id,kind:Kind{id:kind.into()},position:Vec3::new(x,-0.5,z),removed:false,health:20.,invisible:false,effect:false,extra:Extra::default(),bounds:Aabb::new(Vec3::new(x-0.25,-0.5,z),Vec3::new(x+0.25,0.5,z+0.5))});}
#[test]fn actual_generated_pick_dispatch_filters_global_targets_and_live_entity_state(){
    let _guard=SCENE.lock().unwrap();reset();let mut game=Game::default();add(&mut game,42,"zombie",0.34,2.);
    assert!(game.mob_hit_within(3.).is_some());game.mobs[0].kind.id="cow".into();state().tags.begin_tick();assert!(game.mob_hit_within(3.).is_none());
    state().targets.combat=targeting::TargetSet::from_targets([targeting::Target::Passive]);assert!(game.mob_hit_within(3.).is_some());
    game.mobs[0].health=0.;assert!(game.mob_hit_within(3.).is_none());state().targets.combat=targeting::TargetSet::ALL;assert!(game.mob_hit_within(3.).is_some());
    game.riding=Some(42);assert!(game.mob_hit_within(3.).is_none());
}
#[test]fn friend_cache_sleeping_and_border_follow_actual_snapshot(){
    let _guard=SCENE.lock().unwrap();reset();let mut game=Game::default();add(&mut game,42,"player",0.,2.);
    state().friends.add("Alice".into(),None);assert_eq!(game.client_pick_margin(42,0.7),f64::from(0.7f32));
    state().friends.remove("Alice");assert_eq!(game.client_pick_margin(42,0.7),f64::from(0.7f32));state().tags.begin_tick();assert_eq!(game.client_pick_margin(42,0.7),f64::from(0.1f32));
    game.mobs[0].extra.avatar_pose=pose::Pose::Sleeping;assert_eq!(game.client_pick_margin(42,0.7),f64::from(0.7f32));
    game.mobs[0].extra.avatar_pose=pose::Pose::Standing;game.mobs[0].position.x=50.;assert_eq!(game.client_pick_margin(42,0.7),f64::from(0.7f32));
}
#[test]fn debug_and_pick_flags_are_independent_and_do_not_expand_local_player(){
    let _guard=SCENE.lock().unwrap();reset();let mut game=Game::default();add(&mut game,42,"zombie",0.,2.);
    assert_eq!(game.client_debug_hitbox_margin(42),0.1);assert_eq!(game.client_pick_margin(42,0.8),f64::from(0.1f32));
    state().hitbox.apply_to_debug_hitbox=false;assert_eq!(game.client_debug_hitbox_margin(42),0.);
    assert_eq!(game.client_debug_hitbox_margin(7),0.);assert_eq!(game.client_pick_margin(7,0.8),f64::from(0.8f32));
    game.dragon=Some(Dragon{id:0,position:Vec3::ZERO,health:200.});state().hitbox.apply_to_debug_hitbox=true;
    assert_ne!(game.client_targeting_dragon_id(),0);assert_eq!(game.client_debug_hitbox_margin(game.client_targeting_dragon_id()),0.1);
}
#[test]fn component_margin_adds_without_target_filter_and_composes_with_reach(){
    let _guard=SCENE.lock().unwrap();reset();let mut game=Game::default();state().targets.combat=targeting::TargetSet::EMPTY;
    let range=serde_json::json!({"min_reach":2.,"max_reach":4.5,"hitbox_margin":0.125});
    assert!(game.client_attack_range_contains(&range,5.7));state().hitbox.apply_to_component=false;assert!(!game.client_attack_range_contains(&range,5.7));
    state().hitbox.apply_to_component=true;add(&mut game,42,"cow",0.47,3.);assert!(game.attack_range_pick(&range).is_some());
    state().hitbox.apply_to_component=false;assert!(game.attack_range_pick(&range).is_none());
}
#[test]fn client_guards_prevent_server_lan_and_replay_mutations(){
    let _guard=SCENE.lock().unwrap();reset();let mut game=Game::default();add(&mut game,42,"zombie",0.,2.);
    for condition in 0..4{game.window=condition!=0;game.server_copy=condition==1;game.replaying=condition==2;game.lan=condition==3;
        assert_eq!(game.client_pick_margin(42,0.7),f64::from(0.7f32));assert_eq!(game.client_debug_hitbox_margin(42),0.);let mut margin=0.125;game.native_client_hitbox_component_margin(&mut margin);assert_eq!(margin,0.125);}
}
#[test]fn actual_aabb_float_widening_and_debug_red_eye_match_java_oracle(){
    __ORACLE_ASSERTIONS__
}
