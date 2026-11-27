// Exact composed MotionWorld jump head and real horizon-world jump consumer.
use horizon_core::geometry::Vec3;
use std::cell::RefCell;
pub mod living_air{pub use horizon_world::living_air::*;}
#[path="living-motion.rs"]mod living_motion;
#[path="__BRIDGE__"]pub mod high_jump_bridge;
thread_local!{static POLICY:RefCell<high_jump::HighJump>=RefCell::new(high_jump::HighJump::default());}
fn jump_callback(motion:&mut f32){POLICY.with(|p|{p.borrow_mut().jump(true,motion);});}
mod health{pub enum HealthEvent{Statistic{name:&'static str,amount:i32}}}
enum MotionEvent{ClientJump{on_ground:bool},Health(health::HealthEvent)}
struct Ground{on_ground:bool}
struct Queries{air_jump:bool,double_jump_mode:bool}
struct Player{velocity:Vec3,client_high_jump_owner:bool,client_side:bool,server_copy:bool,replaying:bool,entity_id:i32,ground:Ground,events:Vec<MotionEvent>,client_airjump_push_queries:Queries,sprinting:bool,exhaustion:f32,sync:bool}
impl Player{fn native_client_queries_allowed(&self)->bool{self.client_side&&!self.server_copy&&!self.replaying}fn cause_food_exhaustion(&mut self,v:f32){self.exhaustion+=v;}}
type Result<T>=std::result::Result<T,std::convert::Infallible>;
struct MotionWorld{player:Player,power_calls:u32}
fn native_living_jump(_id:u32,_kind:&str){}
impl MotionWorld{__JUMP__}
impl living_motion::LivingMotionWorld for MotionWorld{
 type Error=std::convert::Infallible;
 fn velocity(&self)->Vec3{self.player.velocity}
 fn set_velocity(&mut self,v:Vec3)->Result<()>{self.player.velocity=v;Ok(())}
 fn jump_strength(&mut self)->Result<f64>{self.power_calls+=1;Ok(0.42)}
 fn block_jump_factor(&mut self)->Result<f32>{Ok(1.0)}
 fn has_jump_boost(&mut self)->Result<bool>{Ok(false)}
 fn jump_boost_amplifier(&mut self)->Result<i32>{Ok(0)}
 fn sprinting(&mut self)->Result<bool>{Ok(self.player.sprinting)}
 fn yaw(&mut self)->Result<f32>{Ok(0.0)}
 fn mark_needs_sync(&mut self){self.player.sync=true;}
 fn no_gravity(&mut self)->Result<bool>{Ok(false)}
 fn gravity_attribute(&mut self)->Result<f64>{Ok(0.08)}
 fn has_slow_falling(&mut self)->Result<bool>{Ok(false)}
 fn controlling_passenger_is_player(&mut self)->Result<bool>{Ok(false)}
 fn speed(&mut self)->Result<f32>{Ok(0.1)}
 fn omnidirectional_air_mover(&mut self)->Result<bool>{Ok(false)}
 fn air_drag_modifier(&mut self)->Result<f64>{Ok(0.98)}
}
fn world()->MotionWorld{MotionWorld{player:Player{velocity:Vec3::new(0.3,-0.0,-0.4),client_high_jump_owner:true,client_side:true,server_copy:false,replaying:false,entity_id:1,ground:Ground{on_ground:true},events:vec![],client_airjump_push_queries:Queries{air_jump:true,double_jump_mode:false},sprinting:false,exhaustion:0.0,sync:false},power_calls:0}}
#[test]fn synchronous_jump_head_changes_real_threshold_max_and_wait_registration(){
 high_jump_bridge::install(jump_callback);
 for motion in [-2.5,-0.0,1e-30,0.2,0.8,10.0,12.5]{
  POLICY.with(|p|{let mut s=high_jump::HighJump::default();s.set_mode(high_jump::Mode::Vulcan);s.settings.motion=motion;*p.borrow_mut()=s;});
  let mut w=world();w.jump_from_ground().unwrap();
  assert_eq!(w.power_calls,2);assert_eq!(POLICY.with(|p|p.borrow().pending()),1);
  let expected=if motion<=1e-5_f32{-0.0}else{f64::from(motion)};
  assert_eq!(w.player.velocity.y.to_bits(),expected.to_bits());assert_eq!(w.player.sync,motion>1e-5_f32);
  assert_eq!(w.player.exhaustion,0.05);assert_eq!(w.player.events.len(),2);
 }
}
#[test]fn server_copies_and_replay_keep_original_jump_with_no_local_wait(){
 high_jump_bridge::install(jump_callback);
 for denied in 0..4{
  POLICY.with(|p|{let mut s=high_jump::HighJump::default();s.set_mode(high_jump::Mode::Vulcan);*p.borrow_mut()=s;});
  let mut w=world();w.player.client_side=denied!=0;w.player.server_copy=denied==1;w.player.replaying=denied==2;w.player.client_high_jump_owner=denied!=3;
  w.jump_from_ground().unwrap();assert_eq!(w.power_calls,1);assert_eq!(w.player.velocity.y,f64::from(0.42_f32));assert_eq!(POLICY.with(|p|p.borrow().pending()),0);
 }
}
