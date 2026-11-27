#![cfg(test)]
#![allow(dead_code)]
#[path="behavior.rs"]mod behavior;
#[path="bridge.rs"]mod bridge;
#[path="owners.rs"]mod owners;
mod packets {
 #[derive(Debug,PartialEq)]pub struct MovePlayerPosRot{pub x:f64,pub y:f64,pub z:f64,pub yaw:f32,pub pitch:f32,pub flags:u8}
 #[derive(Debug,PartialEq)]pub struct MovePlayerPos{pub x:f64,pub y:f64,pub z:f64,pub flags:u8}
 #[derive(Debug,PartialEq)]pub struct MovePlayerRot{pub yaw:f32,pub pitch:f32,pub flags:u8}
 #[derive(Debug,PartialEq)]pub struct MovePlayerStatus{pub flags:u8}
 #[derive(Debug,PartialEq)]pub enum Serverbound{MovePlayerPosRot(MovePlayerPosRot),MovePlayerPos(MovePlayerPos),MovePlayerRot(MovePlayerRot),MovePlayerStatus(MovePlayerStatus)}
}
#[path="position_sender.rs"]mod position_sender;
mod camera {
 #[derive(Clone,Copy,PartialEq)]pub enum CameraType{FirstPerson,ThirdPersonBack,ThirdPersonFront}
 impl CameraType{pub fn is_first_person(self)->bool{self==Self::FirstPerson}pub fn is_mirrored(self)->bool{self==Self::ThirdPersonFront}}
 #[derive(Clone,Copy)]pub struct LivingCamera{pub scale:f32,pub camera_distance:f64,pub sleeping:Option<Option<f32>>}
 pub struct CameraEntity{pub position:[f64;3],pub previous:[f64;3],pub y_rot:f32,pub x_rot:f32,pub living:Option<LivingCamera>,pub mount:Option<(f32,f64)>}
 impl CameraEntity{fn view_y_rot(&self,_:f32)->f32{self.y_rot}fn view_x_rot(&self,_:f32)->f32{self.x_rot}}
 pub struct Perspective{pub distance:f32,pub no_clip:bool}
 impl Perspective{fn render_distance(&self,_:f32)->f32{self.distance}}
 pub struct Camera{pub position:[f64;3],pub y_rot:f32,pub x_rot:f32,pub eye_height_old:f32,pub eye_height:f32,pub eye:[f64;3],pub eye_rotation:[f32;2],pub detached:bool,pub perspective_event:Perspective,pub forwards:[f32;3]}
 impl Default for Camera{fn default()->Self{Self{position:[0.;3],y_rot:0.,x_rot:0.,eye_height_old:1.6,eye_height:1.6,eye:[0.;3],eye_rotation:[0.;2],detached:false,perspective_event:Perspective{distance:4.,no_clip:true},forwards:[0.,0.,1.]}}}
 fn lerp(p:f32,a:f32,b:f32)->f32{a+p*(b-a)}
 fn lerp_d(p:f64,a:f64,b:f64)->f64{a+p*(b-a)}
 fn java_max(a:f32,b:f32)->f32{a.max(b)}
 impl Camera{
  fn set_rotation(&mut self,y:f32,x:f32){self.y_rot=y;self.x_rot=x;self.forwards=[-(y.to_radians()).sin(),0.,(y.to_radians()).cos()];}
  fn move_by(&mut self,f:f32,_u:f32,_r:f32){for i in 0..3{self.position[i]+=f64::from(f*self.forwards[i]);}}
  __CAMERA_METHODS__
 }
 pub fn entity()->CameraEntity{CameraEntity{position:[1.,60.,3.],previous:[1.,60.,3.],y_rot:9.,x_rot:8.,living:None,mount:None}}
}
#[test]fn actual_camera_consumer_overrides_smoothed_position_and_preserves_unwrapped_yaw(){
 let mut camera=camera::Camera::default();let entity=camera::entity();
 camera.align_with_entity_free_cam(&entity,0.5,camera::CameraType::FirstPerson,|_|panic!("first person"),|_|[999.;3],None,Some(([20.,80.,30.],[366.,-12.])),|_,_|None);
 assert_eq!(camera.position,[20.,80.,30.]);assert_eq!(camera.eye,camera.position);
 assert_eq!(camera.eye_rotation,[366.,-12.]);assert!(camera.detached);
}
#[test]fn free_look_backs_off_from_free_cam_eye_and_owns_final_rotation(){
 let mut camera=camera::Camera::default();let entity=camera::entity();
 camera.align_with_entity_free_cam(&entity,1.,camera::CameraType::FirstPerson,|d|d,|_|panic!("FreeLook skips smoothing"),Some([0.,0.]),Some(([20.,80.,30.],[90.,0.])),|_,_|None);
 assert_eq!(camera.eye,[20.,80.,30.]);assert_eq!(camera.eye_rotation,[90.,0.]);
 assert_eq!(camera.position,[20.,80.,26.]);assert_eq!(camera.y_rot,0.);assert!(camera.detached);
}
#[test]fn suppressed_rotation_keeps_baseline_until_a_managed_rotation_is_allowed(){
 use packets::Serverbound::*;
 let mut sender=position_sender::PositionSender::default();
 assert_eq!(sender.update_free_cam([0.;3],(20.,15.),false,false,false),None);
 assert!(matches!(sender.update_free_cam([1.,0.,0.],(20.,15.),false,false,false),Some(MovePlayerPos(_))));
 assert!(matches!(sender.update_free_cam([1.,0.,0.],(20.,15.),false,false,true),Some(MovePlayerRot(_))));
 assert_eq!(sender.update([1.,0.,0.],(20.,15.),false,false),None);
}
#[test]fn third_navigation_owner_cannot_release_another_owners_physical_actions(){
 use owners::{Owner,Holds,Slots};
 let mut holds=Holds::empty();holds.set(Owner::AutoWalk,true);holds.set(Owner::FreeCam,true);
 assert!(!holds.releasing_last(Owner::FreeCam));holds.set(Owner::FreeCam,false);assert!(holds.owned(Owner::AutoWalk));
 let mut slots=Slots::default();slots.claim(Owner::AutoWalk,2,0);slots.claim(Owner::FightBot,3,2);slots.claim(Owner::FreeCam,4,3);
 assert_eq!(slots.release(Owner::FreeCam),Some(3));assert_eq!(slots.release(Owner::AutoWalk),Some(3));assert_eq!(slots.release(Owner::FightBot),Some(0));
}
#[test]fn actual_physics_bridge_distinguishes_camera_activation_from_rotation_permission(){
 use std::sync::atomic::{AtomicU32,Ordering};static FLAGS:AtomicU32=AtomicU32::new(1);
 bridge::install(|kind,x,z,flags|{if kind==1&&(x!=0.||z!=0.){FLAGS.store(0,Ordering::Relaxed);}*flags|=FLAGS.load(Ordering::Relaxed);});
 assert!(bridge::active());assert!(!bridge::allow_rotation());FLAGS.store(5,Ordering::Relaxed);assert!(bridge::allow_rotation());
 bridge::moved(0.,0.);assert!(bridge::active());bridge::moved(0.1,0.);assert!(!bridge::active());
}
