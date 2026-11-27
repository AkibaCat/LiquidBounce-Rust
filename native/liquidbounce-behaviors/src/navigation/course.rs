// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Course.kt. Walking uses the owning Planning/Follower/BlockActions chain.
use super::{
 destination::{Destination,NearDestination},geometry::{BlockPos,Vec3},heading::Heading,input::MovementKeys,
 look_ahead::{LookAhead,ViewHost},owned_search::OwnedTerrainSearch,progress::FollowState,
 replanner::ReplannerLevel,route_report::{IdentityRef,RouteIdentity,RouteListener,RouteReport},
 simulation::player_motion::MotionPlayer,terrain::Terrain,
 walk::route::Route,walk_control::{ability_capture::safe_fall,block_actions::PlacementOrder,body::{ControlState,WalkPlayer},break_times::BreakState,follower::Follower,planning::PlanningHost,shortcut::is_straight_walkable},walking_runtime::WalkingRuntime,
};
use std::sync::Arc;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ProgressOwner{Walking,Straight,Flight,Glide,Ride,Roof}
pub trait CourseHost:PlanningHost+ViewHost+RouteListener<Progress=ProgressOwner>{
 fn root_vehicle_position(&self)->Vec3;
 fn player_width(&self)->f64;
 fn navigation_arrived(&mut self,goal:Vec3);
 fn navigation_failed(&mut self,goal:Vec3);
 fn navigation_stuck(&mut self,pos:BlockPos);
}
#[derive(Clone,Copy,Debug)]
pub struct TravelHeading{pub target:Option<Vec3>,pub sprint:bool}
/// Adapter to the actual owned flying/gliding/riding/boating controllers.
/// `stop_walking` is invoked before a newly selected flight takes its tick.
pub trait TravelControl<H,B:ControlState>{
 fn boat_tick(&mut self,host:&mut H,follower:&Follower<B>)->Option<Option<Vec3>>;
 fn ride(&mut self,host:&mut H,goal:Vec3,radius:f64,max_length:f64)->bool;
 fn fly(&mut self,host:&mut H,goal:Vec3,radius:f64,straight:bool,stop_walking:&mut dyn FnMut(&mut H))->Option<TravelHeading>;
 fn halt(&mut self,host:&mut H);
 fn block_changed(&mut self,pos:BlockPos);
 fn level_changed(&mut self);
}
pub struct Course<B:ControlState,V>{
 pub travel:V,pub heading:Heading,pub report:RouteReport,
 straight:bool,pub straight_goal:Option<Vec3>,failed:bool,pub arrived:bool,pub last_goal:Option<Vec3>,
 serial:u64,path_identity:Option<(Arc<Route<B>>,RouteIdentity)>,block_identity:Option<(RouteIdentity,usize,RouteIdentity,RouteIdentity)>,
}
impl<B:ControlState,V> Course<B,V>{
 pub fn new(travel:V)->Self{Self{travel,heading:Heading::default(),report:RouteReport::default(),straight:false,straight_goal:None,failed:false,arrived:false,last_goal:None,serial:100,path_identity:None,block_identity:None}}
 pub fn is_stranded<T,S,O>(&self,walking:&WalkingRuntime<B,T,S,O>)->bool where B:BreakState+'static,T:Send+'static,S:OwnedTerrainSearch<T,Route=Route<B>>+Send+'static,O:PlacementOrder{
  !self.straight&&walking.follower.path().is_none()&&(walking.planning.failed()||walking.planning.is_settled())
 }
 fn identity(&mut self)->RouteIdentity{self.serial=self.serial.wrapping_add(1);RouteIdentity(self.serial)}
 pub fn move_to<H,T,S,O,L>(&mut self,host:&mut H,walking:&mut WalkingRuntime<B,T,S,O>,level:&mut L,look:&mut LookAhead,keys:&mut MovementKeys,goal:Vec3,radius:f64)->bool
 where B:BreakState+'static,T:Send+'static,S:OwnedTerrainSearch<T,Route=Route<B>>+Send+'static,O:PlacementOrder,H:CourseHost,H::Player:WalkPlayer<State=B>,L:ReplannerLevel<T>,V:TravelControl<H,B>{
  walking.planning.tick(goal);self.last_goal=Some(goal);let position=host.player().position();
  if host.root_vehicle_position().distance_sqr(goal)<=radius*radius{
   self.stop(host,walking,keys);self.heading.forget_look();self.report.done(host);
   if !self.arrived{self.arrived=true;host.navigation_arrived(goal);}return false;
  }
  self.arrived=false;
  let max_length=host.max_length();
  if host.player().on_ground(){self.straight=position.distance_sqr(goal)<=max_length*max_length&&is_straight_walkable(&mut Terrain::player(host.live_level()),position,goal,radius,safe_fall(host));}
  if let Some(target)=self.travel.boat_tick(host,&walking.follower){if let Some(target)=target{self.heading.walk_towards(host,keys,target,false);}else{self.stop(host,walking,keys);}return false;}
  if self.travel.ride(host,goal,radius,host.max_length()){Self::reset_walk(host,walking,&mut self.heading,&mut self.straight,&mut self.straight_goal);return false;}
  let straight=self.straight;
  let mut reset=|host:&mut H|{Self::reset_walk(host,walking,&mut self.heading,&mut self.straight,&mut self.straight_goal);self.report.clear(host);};
  if let Some(travel)=self.travel.fly(host,goal,radius,straight,&mut reset){self.heading.target=travel.target;self.heading.sprint=travel.sprint;look.aim(self.heading.target);return false;}
  if self.straight{
   walking.planning.drop_route();walking.follower.clear();walking.actions.idle(host);self.straight_goal=Some(goal);self.heading.walk_towards(host,keys,goal,true);
   if !keys.jump{keys.jump=walking.hops.starts(host.player(),&mut Terrain::player(host.live_level()),||host.movement_y_rot(),&walking.follower.stride,host.player().position(),goal);}
   self.report.blocks(host,IdentityRef::new(RouteIdentity(0),&[]),IdentityRef::new(RouteIdentity(0),&[]));
   let points=[host.player().position(),goal];let points_id=self.identity();self.report.following(host,IdentityRef::new(points_id,&points),IdentityRef::new(RouteIdentity(2),&ProgressOwner::Straight));
   look.aim(Some(goal));return true;
  }
  self.straight_goal=None;walking.planning.collect(host,&mut walking.follower,level,position);
  let plain=if let Some(path)=walking.follower.path().cloned().filter(|_|!walking.follower.is_finished()){
   let yaw=host.input_y_rot();let state=walking.follower.tick(host,&mut walking.actions,keys,yaw);
   if state==FollowState::Stuck&&let Some(pos)=walking.follower.stuck_at{walking.planning.stuck_at(pos);host.navigation_stuck(pos);}
   let movement=walking.follower.movement();self.heading.target=movement.target;self.heading.sprint=movement.sprint&&(!keys.sneak||movement.dive);
   let same=self.path_identity.as_ref().is_some_and(|(last,_)|Arc::ptr_eq(last,&path));
   let path_id=if same{self.path_identity.as_ref().unwrap().1}else{let id=self.identity();self.path_identity=Some((path.clone(),id));id};
   let index=walking.follower.index();
   if !self.block_identity.is_some_and(|(id,at,_,_)|id==path_id&&at==index){let breaks=self.identity();let places=self.identity();self.block_identity=Some((path_id,index,breaks,places));}
   let (_,_,breaks,places)=self.block_identity.unwrap();
   self.report.blocks(host,IdentityRef::new(breaks,&walking.follower.remaining_breaks),IdentityRef::new(places,&walking.follower.remaining_places));
   self.report.following(host,IdentityRef::new(path_id,path.points()),IdentityRef::new(RouteIdentity(1),&ProgressOwner::Walking));
   let ahead=walking.follower.point_ahead(host.player().position(),f64::from(host.look_settings().distance));look.aim(walking.follower.bridge_target().or(ahead));
   state==FollowState::Walking&&movement.is_plain_walk&&!walking.actions.is_active()
  }else{
   walking.actions.idle(host);let searching=walking.planning.is_searching();
   let target=final_approach(host,goal,radius).map(|point|(point,false)).or_else(||(searching&&!walking.planning.failed()&&has_headway(host,goal)).then_some((goal,true)));
   if let Some((target,sprint))=target{self.heading.walk_towards(host,keys,target,sprint);}else{self.stop(host,walking,keys);}
   if walking.planning.failed(){self.report.invalid(host);}else if searching{let preview_id=self.identity();self.report.searching(host,host.player().position(),goal,radius,walking.planning.preview().map(|p|IdentityRef::new(preview_id,p)));}else{self.report.done(host);}
   look.aim(self.heading.target);false
  };
  walking.planning.plan_if_needed(host,&walking.follower,level,&walking.actions.unbreakable,goal,radius,position);
  if walking.planning.failed()&&!self.failed{host.navigation_failed(goal);}self.failed=walking.planning.failed();plain
 }
 fn stop<H,T,S,O>(&mut self,host:&mut H,walking:&mut WalkingRuntime<B,T,S,O>,keys:&mut MovementKeys)
 where H:PlanningHost,H::Player:WalkPlayer<State=B>,O:PlacementOrder{walking.actions.idle(host);self.heading.stop(host,keys);}
 fn reset_walk<H,T,S,O>(host:&mut H,walking:&mut WalkingRuntime<B,T,S,O>,heading:&mut Heading,straight:&mut bool,straight_goal:&mut Option<Vec3>)
 where B:BreakState+'static,T:Send+'static,S:OwnedTerrainSearch<T,Route=Route<B>>+Send+'static,H:PlanningHost,H::Player:WalkPlayer<State=B>,O:PlacementOrder{walking.reset(host);*straight=false;*straight_goal=None;heading.clear();}
 pub fn halt<H,T,S,O>(&mut self,host:&mut H,walking:&mut WalkingRuntime<B,T,S,O>)
 where B:BreakState+'static,T:Send+'static,S:OwnedTerrainSearch<T,Route=Route<B>>+Send+'static,H:CourseHost,H::Player:WalkPlayer<State=B>,O:PlacementOrder,V:TravelControl<H,B>{self.last_goal=None;self.arrived=false;self.failed=false;self.travel.halt(host);Self::reset_walk(host,walking,&mut self.heading,&mut self.straight,&mut self.straight_goal);self.report.clear(host);}
}
pub fn has_headway<H:PlanningHost>(host:&H,goal:Vec3)->bool{
 if !host.player().on_ground(){return false;}let position=host.player().position();let delta=Vec3::new(goal.x-position.x,0.0,goal.z-position.z);let length=(delta.x*delta.x+delta.z*delta.z).sqrt();
 if length<0.5{return false;}let ahead=position.fma(length.min(2.0)/length,delta);
 is_straight_walkable(&mut Terrain::player(host.live_level()),position,ahead,0.75,safe_fall(host))
}
pub fn final_approach<H:CourseHost>(host:&H,goal:Vec3,radius:f64)->Option<Vec3>{
 let position=host.player().position();let feet=BlockPos::containing(position);if !NearDestination::new(goal,radius).is_goal(feet){return None;}
 let margin=host.player_width()/2.0;Some(Vec3::new(goal.x.clamp(f64::from(feet.x)+margin,f64::from(feet.x)+1.0-margin),position.y,goal.z.clamp(f64::from(feet.z)+margin,f64::from(feet.z)+1.0-margin)))
}
