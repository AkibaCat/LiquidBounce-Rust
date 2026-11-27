//! Exact production push consumer with storage, team and climb-query providers.
//! Only namespace imports are adapted; actual impulse/boat predicates are used.
use super::Vec3;
use horizon_core::geometry::Aabb;
use std::{cell::Cell,collections::HashMap};
mod mobs{#[derive(PartialEq)]pub enum Species{Bat,Pig}pub enum Who{Mob(u32)}}
mod scoreboard{pub fn pushable_by(_:Option<()>,_:Option<()>,_:bool)->bool{true}}
mod boat_passengers{use super::*;pub(super) fn push_impulse(a:Vec3,b:Vec3)->Option<Vec3>{crate::push_impulse(a,b)} __BOAT_PUSHES__}
struct Health;impl Health{fn dead_or_dying(&self)->bool{false}}
struct Player{position:Vec3,velocity:Vec3,health:Health,spectator:bool,sleeping:bool}
impl Player{fn bounds(&self)->Aabb{Aabb::new(self.position,self.position.add_vector(Vec3::new(1.,2.,1.)))}}
struct Body{id:u32,position:Vec3}impl Body{fn alive(&self)->bool{true}fn species(&self)->mobs::Species{mobs::Species::Pig}fn bounds(&self)->Aabb{Aabb::new(self.position,self.position.add_vector(Vec3::new(1.,1.,1.)))}}
struct Listed{name:String,game_mode:i32}
struct Remote{entity_id:i32,passenger_vehicles:HashMap<u32,u32>,level_players:HashMap<i32,u128>,tab_players:HashMap<u128,Listed>,profile_name:String}
struct Teams;impl Teams{fn of(&self,w:mobs::Who)->Option<()>{let mobs::Who::Mob(id)=w;let _=id;None}}
struct Board;impl Board{fn players_team(&self,_:&str)->Option<()>{None}}
struct Score{board:Board}struct Pack{assets:()}
struct MotionWorld<'a>{entities:&'a[()],player:&'a mut Player,scene:&'a mut (),assets:&'a (),collision_binding:(),resources:(),health:&'a ()}
impl MotionWorld<'_>{fn climbable(&self)->Result<bool,()>{Ok(false)}}
struct Game{player:Option<Player>,remote:Option<Remote>,mobs:Vec<Body>,boats:Vec<Body>,world:(),pack:Pack,score:Score,collision:(),motion:(),health_resources:(),entities_allowed:bool,queries:Cell<u32>}
const SPECTATOR:i32=3;
mod consumer{use super::*;impl Game{
    fn player_entity_collisions(&self)->Vec<()>{vec![]}
    fn mob_teams(&self)->Teams{Teams}
    fn native_client_airjump_push_policy(&self)->[bool;7]{self.queries.set(self.queries.get()+1);[false,false,false,self.entities_allowed,true,true,true]}
    __PUSH__
}pub(super) fn push(g:&mut Game,id:u32){g.native_terrain_entity_push(id)}}
fn game(allowed:bool)->Game{Game{player:Some(Player{position:Vec3::ZERO,velocity:Vec3::new(0.3,-0.0,-0.4),health:Health,spectator:false,sleeping:false}),remote:Some(Remote{entity_id:0,passenger_vehicles:HashMap::new(),level_players:HashMap::new(),tab_players:HashMap::new(),profile_name:String::new()}),mobs:vec![Body{id:1,position:Vec3::new(0.25,0.,0.4)}],boats:vec![Body{id:2,position:Vec3::new(0.25,0.,0.4)}],world:(),pack:Pack{assets:()},score:Score{board:Board},collision:(),motion:(),health_resources:(),entities_allowed:allowed,queries:Cell::new(0)}}
#[test]fn actual_push_consumer_applies_nopush_only_to_living_entity_queries(){
    let initial=Vec3::new(0.3,-0.0,-0.4);
    let impulse=crate::push_impulse(Vec3::ZERO,Vec3::new(0.25,0.,0.4)).unwrap();
    for allowed in [false,true]{for id in [1,2]{
        let mut g=game(allowed);consumer::push(&mut g,id);
        let expected=if id==1&&!allowed{initial}else{initial.add_vector(Vec3::new(-impulse.x,0.,-impulse.z))};
        let actual=g.player.unwrap().velocity;
        assert_eq!([actual.x,actual.y,actual.z].map(f64::to_bits),[expected.x,expected.y,expected.z].map(f64::to_bits),"allowed={allowed},entity={id}");
        assert_eq!(g.queries.get(),u32::from(id==1));
    }}
}
