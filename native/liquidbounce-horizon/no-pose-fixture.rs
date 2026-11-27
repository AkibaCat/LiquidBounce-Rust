#![cfg(test)]
#![allow(dead_code,unused_imports)]
extern crate self as horizon_client;
use horizon_core::geometry::{Vec3,Aabb};
type Result<T> = std::result::Result<T,()>;
mod no_pose;
mod pose;
mod behavior;
use pose::{Pose,Dimensions};
mod motion {
    use super::*;
    #[derive(Clone,Default)]pub struct Fluids {pub was_touching_water:bool,pub was_eye_in_water:bool}
    #[derive(Clone,Default)]pub struct Ground{pub on_ground:bool,pub on_ground_no_blocks:bool,pub supporting_block:Option<()>}
    #[derive(Clone)]pub struct PlayerMotion {
        pub pose:Pose,pub dimensions:Dimensions,pub scale:f32,pub attribute_scale:f32,pub position:Vec3,
        pub client_no_swim_pose:bool,pub client_side:bool,pub server_copy:bool,pub replaying:bool,
        pub sleeping:bool,pub swimming:bool,pub fall_flying:bool,pub flying:bool,pub spectator:bool,
        pub shift_key_down:bool,pub local_crouching:bool,pub auto_spin_attack_ticks:i32,pub auto_spin_attack_dmg:f32,
        pub fall_fly_ticks:i32,pub fluids:Fluids,pub ground:Ground,
    }
    impl Default for PlayerMotion {fn default()->Self{Self{
        pose:Pose::Standing,dimensions:Pose::Standing.dimensions(1.),scale:1.,attribute_scale:1.,position:Vec3::ZERO,
        client_no_swim_pose:false,client_side:true,server_copy:false,replaying:false,sleeping:false,swimming:false,
        fall_flying:false,flying:false,spectator:false,shift_key_down:false,local_crouching:false,
        auto_spin_attack_ticks:0,auto_spin_attack_dmg:0.,fall_fly_ticks:0,fluids:Default::default(),ground:Default::default(),
    }}}
    pub const WATER:u8=0;
    impl PlayerMotion {
        pub fn attribute(&mut self,_:&str)->Result<f64>{Ok(f64::from(self.attribute_scale))}
        pub fn refresh_attributes(&mut self)->Result<()>{if self.scale!=self.attribute_scale{self.refresh_dimensions()?;}Ok(())}
        pub fn is_eye_in_fluid(&self,_:u8)->bool{self.fluids.was_eye_in_water}
        pub fn bounds(&self)->Aabb{self.dimensions.bounds(self.position)}
        __FLUID_THRESHOLD__
    }
    pub struct MotionWorld<'a,'b,'c>{pub player:&'a mut PlayerMotion,pub obstacles:Vec<Aabb>,pub fluid_updates:usize,pub marker:std::marker::PhantomData<(&'b(),&'c())>}
    impl<'a> MotionWorld<'a,'_,'_>{
        pub fn new(player:&'a mut PlayerMotion,roof:f64)->Self{Self{player,obstacles:vec![Aabb::new(Vec3::new(-2.,roof,-2.),Vec3::new(2.,roof+1.,2.))],fluid_updates:0,marker:Default::default()}}
        pub fn no_player_collision(&mut self,bounds:Aabb)->Result<bool>{Ok(self.obstacles.iter().all(|obstacle|!bounds.intersects(*obstacle)))}
        pub fn update_fluids(&mut self)->Result<()>{self.fluid_updates+=1;Ok(())}
        __FLUID_EYE_Y__
    }
}
use motion::PlayerMotion;
mod consumers {
    use super::*;
    pub const DISMOUNT_POSES:[Pose;3]=[Pose::Standing,Pose::Crouching,Pose::Swimming];
    __RIDER_POSES__
    pub struct TrackingBody{pub feet:[f64;3],pub width:f64,pub height:f64}
    __TRACKING_BODY__
    pub struct Guest{pub motion:Option<PlayerMotion>,pub sleeping:bool,pub fall_flying:bool,pub sneaking:bool,pub flying:bool,pub position:[f64;3]}
    impl Guest{__GUEST_POSE__ __GUEST_DIMENSIONS__ __GUEST_BOUNDS__ __GUEST_EYE__}
    pub fn avatar_dims(id:&str,avatar_pose:Pose,scale:f32)->Option<(f64,f64,f64)>{
        struct Kind{id:std::sync::Arc<str>}struct Extra{avatar_pose:Pose}
        struct Mob{kind:Kind,extra:Extra,scale:f32}
        impl Mob{fn model_scale(&self)->f32{self.scale} fn dims(&self)->(f64,f64,f64){
            let k=&self.kind;
            __AVATAR_DIMENSION_BRANCH__
            (-1.,-1.,-1.)
        }}
        let value=Mob{kind:Kind{id:id.into()},extra:Extra{avatar_pose},scale}.dims();
        (value.0>=0.).then_some(value)
    }
    pub struct Extra{pub avatar_pose:Pose}
    pub struct Mob{pub kind:&'static str,pub extra:Extra,pub scale:f32}
    impl Mob{fn model_scale(&self)->f32{self.scale}fn height(&self)->f32{self.extra.avatar_pose.dimensions(self.scale).height}}
    fn avatar(m:&Mob)->bool{matches!(m.kind,"minecraft:player"|"minecraft:mannequin")}
    fn scale(m:&Mob)->f64{f64::from(m.scale)}
    fn base_attachment(v:&Mob)->[f64;3]{[0.,f64::from(v.height())/scale(v),0.]}
    fn unscaled_vehicle_attachment(_:&Mob,_:f32)->f64{f64::from(0.6_f32)}
    __PASSENGER_ATTACHMENT__
    __VEHICLE_ATTACHMENT__
    __GENERIC_PLAYER_SEAT__
    pub fn dismount_dimensions(p:&PlayerMotion)->Vec<(Pose,Dimensions)>{rider_poses(p)}
}
mod equine{pub fn rear_offset(_:&crate::consumers::Mob)->[f64;3]{[0.;3]}}
mod mod_api {
    use std::sync::OnceLock;
    pub trait RenderHooks:Send+Sync{}
    mod host{include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}pub use host::*;
    mod generated{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use generated::*;
    pub fn entered<'a>(mods:&'a[Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(|m|&**m)}
    pub fn mods()->&'static[Box<dyn Hooks>]{static MODS:OnceLock<Vec<Box<dyn Hooks>>>=OnceLock::new();MODS.get_or_init(||vec![Box::new(Runtime)])}
    struct Runtime;impl Hooks for Runtime{
        fn client_no_pose_policy(&self,_:&dyn Host,event:&mut ClientNoPosePolicyEvent){event.no_swim=true;event.sneak_height=2;}
    }
}
mod game{
    use super::*;
    pub struct Game<'a>{pub native:bool,pub player:Option<PlayerMotion>,pub marker:std::marker::PhantomData<&'a()>}
    impl Game<'_>{fn native_window(&self)->bool{self.native}}
    mod native_hooks{use super::*;impl Game<'_>{__NATIVE_HOOKS__}}
    mod mod_api_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}
}

#[test]fn actual_java_dimensions_and_both_attachments_are_bit_exact(){
    for (choice,scale,width,height,eye,passenger,vehicle) in __ORACLE_ROWS__ {
        no_pose::publish(choice);let d=Pose::Crouching.dimensions(scale);
        assert_eq!((d.width.to_bits(),d.height.to_bits(),d.eye_height.to_bits()),(width,height,eye));
        let a=d.no_pose_attachments.unwrap();assert_eq!((a[0].to_bits(),a[1].to_bits()),(passenger,vehicle));
        let m=consumers::Mob{kind:"minecraft:player",extra:consumers::Extra{avatar_pose:Pose::Crouching},scale};
        assert_eq!(consumers::passenger_attachment(&m)[1].to_bits(),passenger);
        assert_eq!(consumers::vehicle_attachment(&m,1.).to_bits(),vehicle);
    }
    no_pose::publish(0);
}
#[test]fn local_swimming_assignment_is_cancelled_without_changing_swimming_gameplay_flag(){
    no_pose::publish(1);let mut p=PlayerMotion{client_no_swim_pose:true,swimming:true,..Default::default()};
    p.set_pose(Pose::Swimming).unwrap();assert_eq!(p.pose,Pose::Standing);assert!(p.swimming);
    for pose in [Pose::Crouching,Pose::FallFlying,Pose::SpinAttack,Pose::Sleeping,Pose::Standing]{p.set_pose(pose).unwrap();assert_eq!(p.pose,pose);}
    for (client_side,server_copy,replaying) in [(false,false,false),(true,true,false),(true,false,true)]{
        let mut other=PlayerMotion{client_side,server_copy,replaying,..p.clone()};other.set_pose(Pose::Swimming).unwrap();assert_eq!(other.pose,Pose::Swimming);
    }
    p.client_no_swim_pose=false;p.set_pose(Pose::Swimming).unwrap();p.client_no_swim_pose=true;
    p.set_pose(Pose::Swimming).unwrap();assert_eq!(p.pose,Pose::Swimming);
    no_pose::publish(0);
}
#[test]fn pose_fit_crawling_and_crouch_use_actual_collision_dimensions(){
    for (choice,roof,crouches) in [(1,1.7,false),(2,1.7,true),(3,1.55,true)]{
        no_pose::publish(choice);let mut p=PlayerMotion{shift_key_down:true,..Default::default()};
        let mut world=motion::MotionWorld::new(&mut p,roof);
        world.update_local_crouching().unwrap();assert_eq!(world.player.local_crouching,crouches);
        world.update_player_pose().unwrap();assert_eq!(world.player.pose,if crouches{Pose::Crouching}else{Pose::Swimming});
        world.player.set_pose(Pose::Standing).unwrap();world.player.client_no_swim_pose=true;
        world.update_player_pose().unwrap();assert_eq!(world.player.pose,if crouches{Pose::Crouching}else{Pose::Standing});
    }
    no_pose::publish(0);
}
#[test]fn global_crouch_dimensions_cover_remote_players_mannequins_server_and_dismounts(){
    no_pose::publish(2);let mut p=PlayerMotion{server_copy:true,..Default::default()};p.set_pose(Pose::Crouching).unwrap();
    assert_eq!(p.dimensions.height,1.65);assert_eq!(p.dimensions.eye_height,1.54);
    for id in ["minecraft:player","minecraft:mannequin"]{assert_eq!(consumers::avatar_dims(id,Pose::Crouching,2.),Some((f64::from(1.2_f32),f64::from(3.3_f32),f64::from(3.08_f32))));}
    assert!(consumers::avatar_dims("minecraft:zombie",Pose::Crouching,1.).is_none());
    let choices=consumers::dismount_dimensions(&p);assert_eq!(choices[1].1.height,1.65);assert_eq!(choices[2].1.height,0.6);
    let guest=consumers::Guest{motion:Some(p.clone()),sleeping:false,fall_flying:false,sneaking:true,flying:false,position:[0.;3]};
    assert_eq!(guest.eye_height(),f64::from(1.54_f32));assert_eq!(guest.bounds().max_position().y,f64::from(1.65_f32));
    let tracking=consumers::player_tracking_body(&p);assert_eq!(tracking.height,f64::from(1.65_f32));
    no_pose::publish(0);
}
#[test]fn cached_dimensions_and_attachments_change_only_on_original_refresh(){
    no_pose::publish(1);let mut p=PlayerMotion{attribute_scale:1.234567,..Default::default()};p.set_pose(Pose::Crouching).unwrap();let cached=p.dimensions;
    no_pose::publish(2);p.set_pose(Pose::Crouching).unwrap();assert_eq!(p.dimensions.height,cached.height);assert_eq!(p.dimensions.no_pose_attachments,cached.no_pose_attachments);
    p.attribute_scale=2.;p.refresh_attributes().unwrap();assert_eq!(p.dimensions.height,3.3);assert_eq!(p.passenger_attachment_y(),f64::from(1.65_f32)*2.);
    no_pose::publish(0);p.set_pose(Pose::Standing).unwrap();p.set_pose(Pose::Crouching).unwrap();assert!(p.dimensions.no_pose_attachments.is_none());assert_eq!(p.dimensions.height,3.);
}
#[test]fn eye_fluid_queries_and_passenger_pose_tick_follow_stored_pose(){
    no_pose::publish(1);let mut p=PlayerMotion{position:Vec3::new(0.,64.,0.),client_no_swim_pose:true,swimming:true,..Default::default()};
    p.set_pose(Pose::Crouching).unwrap();let mut w=motion::MotionWorld::new(&mut p,100.);
    assert_eq!(w.eye_y().unwrap(),64.+f64::from(1.54_f32));assert_eq!(w.player.fluid_jump_threshold(),0.4);
    w.passenger_pose_tick().unwrap();assert!(!w.player.swimming);assert_eq!(w.player.pose,Pose::Standing);assert_eq!(w.fluid_updates,1);
    no_pose::publish(0);
}
#[test]fn actual_generated_host_sets_both_policies_and_server_sync_cannot_clear_global_choice(){
    no_pose::publish(0);let mut g=game::Game{native:true,player:Some(PlayerMotion::default()),marker:Default::default()};
    g.sync_native_no_pose();assert!(g.player.as_ref().unwrap().client_no_swim_pose);assert_eq!(Pose::Crouching.dimensions(1.).height,1.65);
    g.native=false;g.sync_native_no_pose();assert_eq!(Pose::Crouching.dimensions(1.).height,1.65);
    let (mut swim,mut choice)=(false,0);g.native_client_no_pose_policy(&mut swim,&mut choice);assert_eq!((swim,choice),(false,0));
    no_pose::publish(0);
}
