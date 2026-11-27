// SPDX-License-Identifier: GPL-3.0-or-later
//! Exact Horizon body replay against owned chunks; never mutates the live body.
use super::*;
use super::{auto_walk_player::NativePlayer,auto_walk_world::{self as world,SnapshotLevel}};
use crate::navigation as nav;
use nav::{geometry::Vec3 as NavVec,walk_control::body::{SimulatedInput,WalkSimulation}};
use std::{cell::RefCell,rc::Rc};
pub(super) struct NativeSimulation{player:NativePlayer,scene:World,entities:Vec<Aabb>,input:SimulatedInput,backed_off:bool}
impl NativeSimulation{
 pub fn new(player:&NativePlayer,input:SimulatedInput,yaw:f32)->Self{
  let mut body=player.read(Clone::clone);body.yaw=yaw;body.auto_jump_enabled=false;
  let region=nav::geometry::BlockRegion::around(player.position(),&[],64.0,64);
  let scene=SnapshotLevel::capture(&player.inventory.level,region).expect("navigation simulation level").world;
  let entities=world::read(|g|g.player_entity_collisions());
  let mut player=player.clone();player.simulated=Some(Rc::new(RefCell::new(body)));
  Self{player,scene,entities,input,backed_off:false}
 }
}
impl WalkSimulation for NativeSimulation{
 type Player=NativePlayer;
 fn player(&self)->&NativePlayer{&self.player}
 fn predicted_position(&self)->NavVec{self.player.position()}
 fn set_input(&mut self,input:SimulatedInput){self.input=input;}
 fn has_backed_off_edge(&self)->bool{self.backed_off}
 fn tick(&mut self){
  let input=self.input;let mut body=self.player.simulated.as_ref().expect("simulated body").borrow_mut();
  let keys=horizon_client::sprint::Keys{forward:input.keys.forward,backward:input.keys.backward,left:input.keys.left,right:input.keys.right,jump:input.keys.jump,shift:input.keys.shift,sprint:input.keys.sprint,analog:None};
  let (_,backed)=horizon_client::navigation_physics::with_prediction(input.force_stay_on_ground_surface,input.ignores_edge_back_off,||world::read(|game|{
   let mut motion=horizon_client::motion::MotionWorld{entities:&self.entities,player:&mut body,scene:&mut self.scene,assets:&game.pack.assets,collision_binding:game.collision,resources:game.motion,health:&game.health_resources};
   motion.replayed_tick(keys,input.keys.sprint).expect("navigation prediction movement");
  }));self.backed_off=backed;
 }
}
