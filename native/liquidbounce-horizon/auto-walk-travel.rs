// SPDX-License-Identifier: GPL-3.0-or-later
//! The complete native Travel selection order over owning Agent controllers.
use super::{auto_walk_host::{NativeHost,NativeRoute},auto_walk_vehicle::NativeEntity,auto_walk_world::{NativeLevel,NativeState,SearchLevel,SearchTerrain,SnapshotLevel}};
use crate::navigation as nav;
use nav::{background::Executor,course::{ProgressOwner,TravelControl,TravelHeading},geometry::{BlockPos,Vec3},ride::{body::RideEntity,vehicle::Vehicle},route_storage::{SharedDestination,shared_destination},staged_search::SharedLevel,terrain::Terrain};
use std::{cell::RefCell,rc::Rc,sync::Arc};

type Level=SharedLevel<SnapshotLevel>;
pub(super) type WalkSearch=nav::walk::route_search::WalkRouteSearch<NativeState,Level,SharedDestination>;
type FlySearch=nav::flight::FlyRouteSearch<NativeState,Level,SharedDestination>;
type GlideSearch=nav::glide::route_search::RouteSearch<NativeState,Level>;
type AirwaySearch=nav::glide::airway_search::AirwaySearch<NativeState,Level,SharedDestination>;
type RideSearch=nav::ride::route_search::RouteSearch<NativeState,Level,SharedDestination>;

struct Report{route:Rc<RefCell<NativeRoute>>,owner:ProgressOwner}
impl Report{
 fn following(&mut self,points:&[Vec3]){let mut route=self.route.borrow_mut();route.points=points.to_vec();route.progress=Some(self.owner);route.preview=false;}
 fn searching(&mut self,preview:Option<&[Vec3]>){let mut route=self.route.borrow_mut();route.points=preview.unwrap_or_default().to_vec();route.progress=None;route.preview=true;}
 fn blocks_empty(&mut self){let mut route=self.route.borrow_mut();route.breaks.clear();route.places.clear();}
 fn done(&mut self){*self.route.borrow_mut()=NativeRoute::default();}
}
impl nav::flying::FlightReport for Report{fn following(&mut self,p:&[Vec3],_:&nav::flight_follower::Follower){self.following(p)}fn invalid(&mut self){self.done()}fn searching(&mut self,_:Vec3,_:Vec3,_:f64,p:Option<&[Vec3]>){self.searching(p)}fn done(&mut self){self.done()}}
impl nav::ride::riding::RideReport for Report{fn following(&mut self,p:&[Vec3],_:usize){self.following(p)}fn searching(&mut self,_:Vec3,_:Vec3,_:f64,p:Option<&[Vec3]>){self.searching(p)}fn done(&mut self){self.done()}}
impl nav::glide::contacts::ContactListener for Report{fn touched(&mut self,pos:BlockPos,face:nav::aabb::Direction){self.route.borrow_mut().contacts.push((pos,face));}}
impl nav::glide::gliding::GlideReport for Report{fn blocks_empty(&mut self){self.blocks_empty()}fn following(&mut self,p:&[Vec3],_:Vec3){self.following(p)}}
impl nav::glide::roof_bounce::RoofReport for Report{fn blocks_empty(&mut self){self.blocks_empty()}fn following(&mut self,p:&[Vec3],_:&nav::glide::roof_bounce::RoofBounce){self.following(p)}}

pub(super) struct NativeTravel{
 pub flight:nav::flying::Flying<SearchTerrain,FlySearch>,
 pub glide:nav::glide::gliding::Gliding<SearchTerrain,GlideSearch,AirwaySearch,NativeState,NativeLevel>,
 pub ride:nav::ride::riding::Riding<SearchTerrain,RideSearch,NativeEntity>,
 pub roof:nav::glide::roof_bounce::RoofBounce,
 boat:nav::ride::boating::Boating<NativeEntity>,
 level:SearchLevel,ride_level:SearchLevel,
 report:Report,
}
impl NativeTravel{
 pub fn new(executor:Arc<dyn Executor>,level:NativeLevel,route:Rc<RefCell<NativeRoute>>)->Self{
  let flight=nav::flying::Flying::new(executor.clone(),Arc::new(|terrain:SearchTerrain,start,descriptor,abilities,limits|{let goal=shared_destination::<NativeState,SnapshotLevel>(descriptor,terrain.level().clone());nav::flight::FlyRouteSearch::new(terrain,start,goal,abilities,limits)}));
  let live=level.clone();let glide=nav::glide::gliding::Gliding::new(executor.clone(),Arc::new(nav::glide::route_search::RouteSearch::new),Arc::new(nav::glide::search_factory::airway_search::<NativeState,SnapshotLevel>),Arc::new(move||live.clone()));
  let ride=nav::ride::riding::Riding::new(executor,Arc::new(|terrain:SearchTerrain,start,descriptor,vehicle,limits|{let goal=shared_destination::<NativeState,SnapshotLevel>(descriptor,terrain.level().clone());nav::ride::route_search::RouteSearch::new(terrain,start,goal,vehicle,limits)}));
  Self{flight,glide,ride,roof:nav::glide::roof_bounce::RoofBounce::default(),boat:Default::default(),level:SearchLevel{live:level.clone(),width:Terrain::<NativeState,NativeLevel>::PLAYER_WIDTH,height:Terrain::<NativeState,NativeLevel>::PLAYER_HEIGHT,can_open_doors:false},ride_level:SearchLevel{live:level,width:0.6_f32 as f64,height:1.8_f32 as f64,can_open_doors:false},report:Report{route,owner:ProgressOwner::Flight}}
 }
 fn halt_flying(&mut self,host:&mut NativeHost){self.flight.halt();self.glide.halt(host);self.roof.halt();}
 fn roof(&mut self,host:&mut NativeHost,goal:Vec3,stop:&mut dyn FnMut(&mut NativeHost))->Option<TravelHeading>{
  let before=self.roof.is_active();let ability=nav::glide::capture::capture_glide(host.options.can_elytra,host.options.elytra_rockets,||host.player.clone());let yaw=host.input_yaw;self.report.owner=ProgressOwner::Roof;
  if !self.roof.navigate(host,&mut self.report,goal,ability,yaw){return None;}if !before{stop(host);}Some(TravelHeading{target:None,sprint:true})
 }
 pub fn progress(&self,owner:ProgressOwner,host:&NativeHost,partial:f32)->(Vec3,Vec<Vec3>){match owner{
  ProgressOwner::Flight=>(self.flight.follower().head(host,partial),self.flight.follower().ahead().to_vec()),
  ProgressOwner::Ride=>(self.ride.head(host,partial),self.ride.ahead().to_vec()),
  ProgressOwner::Glide=>(host.player.interpolated(partial),self.glide.target().into_iter().collect()),
  ProgressOwner::Roof=>(host.player.interpolated(partial),self.roof.target().into_iter().collect()),
  _=>(host.player.interpolated(partial),Vec::new()),
 }}
}
struct Swim(Vec<BlockPos>);impl nav::ride::boating::SwimAhead for Swim{fn swim_ahead(&mut self)->Vec<BlockPos>{self.0.clone()}}
impl TravelControl<NativeHost,NativeState> for NativeTravel{
 fn boat_tick(&mut self,host:&mut NativeHost,follower:&nav::walk_control::follower::Follower<NativeState>)->Option<Option<Vec3>>{let mut swim=Swim(follower.swim_ahead().iter().map(|step|step.to).collect());self.boat.tick(host,&mut swim).then(||self.boat.walk_target())}
 fn ride(&mut self,host:&mut NativeHost,goal:Vec3,radius:f64,max_length:f64)->bool{
  let player=NativeEntity::player(host.level());if let Some(mount)=player.controlled_vehicle()&&let Some(vehicle)=Vehicle::of(&player,&mount){self.ride_level.width=vehicle.width;self.ride_level.height=vehicle.height;}
  self.report.owner=ProgressOwner::Ride;if !self.ride.navigate(host,&mut self.ride_level,&mut self.report,goal,radius,||max_length){return false;}self.halt_flying(host);true
 }
 fn fly(&mut self,host:&mut NativeHost,goal:Vec3,radius:f64,straight:bool,stop:&mut dyn FnMut(&mut NativeHost))->Option<TravelHeading>{
  let abilities=nav::flight_body::capture_flight(host,host.options.can_fly,&mut self.flight.handling).filter(|a|!straight||a.is_flying());
  if let Some(abilities)=abilities{
   if !self.flight.is_active(){self.glide.halt(host);self.roof.halt();stop(host);}
   let yaw=host.input_yaw;let max_length=host.options.max_length();self.report.owner=ProgressOwner::Flight;self.flight.navigate(host,&mut self.level,&mut self.report,goal,radius,max_length,abilities,yaw);
   return Some(TravelHeading{target:self.flight.target(),sprint:self.flight.sprint()});
  }
  self.flight.halt();
  if self.roof.is_active()&&let Some(result)=self.roof(host,goal,stop){return Some(result);}
  let before=self.glide.is_active();let ability=nav::glide::capture::capture_glide(host.options.can_elytra,host.options.elytra_rockets,||host.player.clone());let yaw=host.input_yaw;self.report.owner=ProgressOwner::Glide;
  if self.glide.navigate(host,&mut self.level,&mut self.report,goal,radius,ability,straight,yaw){if !before{stop(host);}return Some(TravelHeading{target:None,sprint:self.glide.sprint()});}
  self.roof(host,goal,stop)
 }
 fn halt(&mut self,host:&mut NativeHost){self.ride.halt();self.halt_flying(host);self.boat.halt(host);}
 fn block_changed(&mut self,pos:BlockPos){self.flight.block_changed(pos);self.glide.block_changed(pos);self.ride.block_changed(pos);}
 fn level_changed(&mut self){self.flight.level_changed();self.glide.level_changed();self.ride.level_changed();self.boat=Default::default();}
}
