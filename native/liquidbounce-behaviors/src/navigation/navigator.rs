// SPDX-License-Identifier: GPL-3.0-or-later
//! Public owning Agent Navigator: every active tick commits input exactly once.
use super::{
 course::{Course,CourseHost,TravelControl},geometry::{BlockPos,Vec3},input::{Input,Keystrokes,MovementKeys},look_ahead::LookAhead,navigation_context::NavigationContext,
 owned_search::OwnedTerrainSearch,replanner::ReplannerLevel,simulation::player_motion::MotionPlayer,snacking::{SnackHost,Snacking},walk::route::Route,
 walk_control::{block_actions::{PlacedBlock,PlacementOrder},body::{ControlState,WalkPlayer},break_times::BreakState,setbacks::Setbacks,view_rotation::ViewRotation},walking_runtime::WalkingRuntime,
};
use std::sync::Arc;
pub trait NavigatorHost:CourseHost+SnackHost{
 fn navigation_context(&self)->&NavigationContext;
 fn min_press_ticks(&self)->i32;
 fn min_release_ticks(&self)->i32;
 fn set_safe_walk(&mut self,safe:bool);
 /// The only non-deferred Body.press operation in a navigation tick.
 fn commit_navigation_input(&mut self,input:Input,exact:bool);
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Ending{Arrived,Stopped,Unreachable}
pub trait DestinationSource{
 fn goal(&mut self)->Option<Vec3>;
 fn radius(&self)->f64{1.0}
}
struct Fixed{goal:Vec3,radius:f64}
impl DestinationSource for Fixed{fn goal(&mut self)->Option<Vec3>{Some(self.goal)}fn radius(&self)->f64{self.radius}}
struct HeldWalk{source:Box<dyn DestinationSource>,ends_on_arrival:bool,ended:Option<Box<dyn FnMut(Ending)>>,stranded_ticks:i32}
impl HeldWalk{fn end(mut self,ending:Ending){if let Some(ended)=&mut self.ended{ended(ending);}}}
pub struct Navigator<B:ControlState,T,S,O:PlacementOrder,V>{
 pub walking:WalkingRuntime<B,T,S,O>,pub course:Course<B,V>,pub look_ahead:LookAhead,
 snacking:Snacking,last_input:Input,walk:Option<HeldWalk>,keystrokes:Keystrokes,setbacks:Setbacks,ticks:i64,tick_start:Option<Vec3>,view_turned:bool,
 pub stranded_ticks:i32,pub guard:Option<Box<dyn FnMut(Input)->Input>>,
}
impl<B:ControlState+BreakState+'static,T:Send+'static,S:OwnedTerrainSearch<T,Route=Route<B>>+Send+'static,O:PlacementOrder,V> Navigator<B,T,S,O,V>{
 pub const UNREACHABLE_TICKS:i32=40;
 pub fn new(walking:WalkingRuntime<B,T,S,O>,travel:V)->Self{Self{walking,course:Course::new(travel),look_ahead:LookAhead::default(),snacking:Snacking::default(),last_input:Input::EMPTY,walk:None,keystrokes:Keystrokes::default(),setbacks:Setbacks::default(),ticks:0,tick_start:None,view_turned:false,stranded_ticks:0,guard:None}}
 pub fn is_acting(&self)->bool{self.walking.actions.is_active()}
 pub fn is_building(&self)->bool{self.is_acting()||self.walking.follower.path().is_some()&&self.walking.follower.building()}
 pub fn bridge_ahead(&self)->usize{if self.walking.follower.path().is_some(){self.walking.follower.bridge_ahead()}else{0}}
 pub fn places_ahead(&self)->&[BlockPos]{if self.walking.follower.path().is_some(){&self.walking.follower.remaining_places}else{&[]}}
 pub fn bridge_y_rot(&self)->Option<f32>{self.walking.follower.bridge_y_rot()}
 pub fn overshoots(&self)->i32{self.walking.follower.overshoots()}
 pub fn placement(&self)->Option<&Arc<PlacedBlock>>{self.walking.actions.placement.as_ref()}
 pub fn is_stranded(&self)->bool{self.course.is_stranded(&self.walking)}
 pub fn is_in_progress(&self)->bool{self.walk.is_some()}
 pub fn input(&self)->Input{self.last_input}
 pub fn slowdown<H:NavigatorHost>(&self,host:&H)->Option<&'static str>where H::Player:WalkPlayer<State=B>{
  let goal=self.course.last_goal?;if host.player().position().distance_sqr(goal)<4.0*4.0{Some("approach")}else if self.walking.follower.path().is_some()&&self.walking.follower.turn_ahead(host,4.0)>45.0{Some("turn")}else{None}
 }
 pub fn movement_rotation<H:NavigatorHost>(&self,host:&H)->ViewRotation where H::Player:WalkPlayer<State=B>{self.course.heading.rotation(host,&self.walking.follower,&self.walking.actions,self.last_input)}
 pub fn navigate<H:NavigatorHost,L:ReplannerLevel<T>>(&mut self,host:&mut H,level:&mut L,goal:Vec3,radius:f64)->bool where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{
  self.start_tick(host);let mut keys=MovementKeys::default();let plain=self.course.move_to(host,&mut self.walking,level,&mut self.look_ahead,&mut keys,goal,radius);
  self.snacking.tick(host,self.is_building());self.stranded_ticks=if self.is_stranded(){self.stranded_ticks.wrapping_add(1)}else{0};
  let context=host.navigation_context();let pressed=context.take_pressed();let input=if context.should_stand(){Input::EMPTY}else{pressed.unwrap_or_else(||keys.to_input(false))};
  let sprint=self.course.heading.sprint&&input.is_moving();let input=if sprint{input.with_keys(input.jump,input.shift,true)}else{input};
  let exact=!plain||pressed.is_some()&&context.pressed_exact();self.finish_tick(host,input,exact);plain
 }
 pub fn move_to(&mut self,goal:Vec3,radius:f64,ended:Option<Box<dyn FnMut(Ending)>>){self.hold(Some(HeldWalk{source:Box::new(Fixed{goal,radius}),ends_on_arrival:true,ended,stranded_ticks:0}));}
 pub fn follow(&mut self,source:Box<dyn DestinationSource>,ended:Option<Box<dyn FnMut(Ending)>>){self.hold(Some(HeldWalk{source,ends_on_arrival:false,ended,stranded_ticks:0}));}
 fn hold(&mut self,walk:Option<HeldWalk>){let old=std::mem::replace(&mut self.walk,walk);if let Some(old)=old{old.end(Ending::Stopped);}}
 pub fn tick<H:NavigatorHost,L:ReplannerLevel<T>>(&mut self,host:&mut H,level:&mut L)->bool where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{
  let Some(walk)=self.walk.as_mut()else{self.halt(host);return false;};let goal=walk.source.goal();let radius=walk.source.radius();
  let Some(goal)=goal else{self.halt(host);return false;};let plain=self.navigate(host,level,goal,radius);let stranded=self.is_stranded();
  let ending=self.walk.as_mut().and_then(|walk|{walk.stranded_ticks=if stranded{walk.stranded_ticks.wrapping_add(1)}else{0};if !walk.ends_on_arrival{None}else if self.course.arrived{Some(Ending::Arrived)}else if walk.stranded_ticks>=Self::UNREACHABLE_TICKS{Some(Ending::Unreachable)}else{None}});
  if let Some(ending)=ending&&let Some(walk)=self.walk.take(){walk.end(ending);}plain
 }
 pub fn press<H:NavigatorHost>(&mut self,host:&mut H,input:Input,exact:bool)where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{self.idle(host);self.finish_tick(host,input,exact);}
 pub fn halt<H:NavigatorHost>(&mut self,host:&mut H)where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{self.idle(host);self.release_keys();}
 fn idle<H:NavigatorHost>(&mut self,host:&mut H)where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{self.start_tick(host);self.clear_walk(host);self.snacking.stop(host);}
 fn clear_walk<H:NavigatorHost>(&mut self,host:&mut H)where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{self.stranded_ticks=0;self.course.halt(host,&mut self.walking);host.navigation_context().take_pressed();host.set_safe_walk(false);}
 fn release_keys(&mut self){self.keystrokes.reset();self.last_input=Input::EMPTY;}
 pub fn stop<H:NavigatorHost>(&mut self,host:&mut H)where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{self.hold(None);self.halt(host);self.walking.planning.forget_stuck_spots();self.look_ahead.reset();}
 pub fn look_at(&mut self,point:Vec3){self.look_ahead.aim(Some(point));}
 pub fn turn_view<H:NavigatorHost>(&mut self,host:&mut H){self.look_ahead.tick(host);self.view_turned=true;}
 pub fn block_changed<H>(&mut self,pos:BlockPos)where V:TravelControl<H,B>{self.walking.planning.block_changed(pos);self.course.travel.block_changed(pos);}
 pub fn block_confirmed<H:NavigatorHost>(&mut self,host:&mut H,pos:BlockPos,state:&B)where H::Player:WalkPlayer<State=B>{if self.walking.actions.answer(host,pos,state){self.walking.planning.stuck_at(pos);}}
 pub fn level_changed<H:NavigatorHost>(&mut self,host:&mut H)where H::Player:WalkPlayer<State=B>,V:TravelControl<H,B>{self.hold(None);self.clear_walk(host);self.release_keys();self.walking.planning.forget_stuck_spots();self.look_ahead.reset();self.walking.actions.level_changed();self.walking.planning.level_changed();self.course.travel.level_changed();}
 pub fn position_corrected<H:NavigatorHost>(&mut self,host:&H)where H::Player:WalkPlayer<State=B>{let refused=self.tick_start.unwrap_or_else(||host.player().position());self.walking.follower.position_corrected();for pos in self.setbacks.refused(refused,self.ticks){self.walking.planning.stuck_at(pos);}}
 fn start_tick<H:NavigatorHost>(&mut self,host:&mut H)where H::Player:WalkPlayer<State=B>{self.ticks=self.ticks.wrapping_add(1);host.navigation_context().set_should_stand(false);self.tick_start=Some(host.player().position());if !self.view_turned{self.look_ahead.tick(host);}self.view_turned=false;self.walking.start_tick(host);self.course.heading.look_along_path(host,&self.walking.follower,&self.walking.actions,&self.look_ahead);}
 fn finish_tick<H:NavigatorHost>(&mut self,host:&mut H,input:Input,exact:bool){let timed=self.keystrokes.press(input,exact,host.min_press_ticks(),host.min_release_ticks());host.set_safe_walk(self.walking.follower.path().is_some()&&self.walking.follower.movement().safe_walk);let guarded=self.guard.as_mut().map_or(timed,|guard|guard(timed));self.last_input=guarded;host.navigation_context().set_moving(guarded.is_moving());host.commit_navigation_input(guarded,exact||guarded!=timed);}
}
