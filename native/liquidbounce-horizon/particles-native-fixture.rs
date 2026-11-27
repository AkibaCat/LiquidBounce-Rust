// Only native world/entity/actor declarations supplied; host/game functions included verbatim.
pub mod gfx{pub mod particles{pub fn begin_frame(input:&str){crate::render::begin_frame(input);}}}
pub mod geometry{
    #[path="@@TRIG@@"]pub mod trig;
    #[derive(Clone,Copy,Default)]pub struct Vec3{pub x:f64,pub y:f64,pub z:f64}
    impl Vec3{pub fn new(x:f64,y:f64,z:f64)->Self{Self{x,y,z}}
// @@DIRECTION@@
    }
}
pub mod game{
    use crate::{mod_api,runtime::LiquidBounceHooks};use std::{cell::RefCell,collections::BTreeMap};
    #[derive(Default)]pub struct Dimensions{pub eye_height:f32}
    #[derive(Default)]pub struct Player{pub position:crate::geometry::Vec3,pub dimensions:Dimensions}
    impl Player{pub fn native_client_queries_allowed(&self)->bool{true}}
    pub struct Remote{pub entity_id:i32}
    pub struct Guest{pub placed:bool,pub entity_id:i32,pub dimension:String,pub position:[f64;3]}
    pub struct Hub{pub guests:Vec<Guest>}
    pub struct Lan{pub hub:Hub}
    pub struct Dimension;impl Dimension{pub fn id(&self)->&'static str{"minecraft:overworld"}}
    pub struct Camera{pub position:[f64;3]}
    pub struct Perspective;impl Perspective{pub fn first_person(&self)->bool{true}}
    pub struct Game<'a>{pub player:Option<Player>,pub remote:Option<Remote>,pub lan:Vec<Lan>,pub dim:Dimension,pub yaw:f32,pub pitch:f32,pub hooks:LiquidBounceHooks,pub entities:BTreeMap<u32,[f64;3]>,pub player_name:Option<String>,pub geometry:RefCell<serde_json::Value>,pub marker:std::marker::PhantomData<&'a()>}
    impl Default for Game<'_>{fn default()->Self{Self{player:Some(Player{position:Default::default(),dimensions:Dimensions{eye_height:1.62}}),remote:None,lan:vec![],dim:Dimension,yaw:0.,pitch:0.,hooks:Default::default(),entities:BTreeMap::new(),player_name:None,geometry:RefCell::new(serde_json::Value::Null),marker:Default::default()}}}
    impl mod_api::Host for Game<'_>{
        fn block_collision(&self,p:[i32;3])->Vec<[f64;6]>{if p[1]<0{vec![[0.,0.,0.,1.,1.,1.]]}else{vec![]}}
        fn client_targeting_entity(&self,_:i32)->String{serde_json::json!({"kind":"minecraft:player","alive":true,"invisible":false,"sleeping":false,"is_local":false,"has_local_passenger":false,"within_border":true,"player_name":self.player_name}).to_string()}
        fn client_particles_context(&self,id:i32)->String{self.native_client_particles_context(id)}
        fn client_particles_direction(&self,yaw:f32,pitch:f32)->[f64;3]{self.native_client_particles_direction(yaw,pitch)}
    }
    impl Game<'_>{
        pub fn native_window(&self)->bool{true}
        pub fn native_entity(&self,id:u32)->Option<u32>{self.entities.contains_key(&id).then_some(id)}
        pub fn native_entity_position(&self,id:u32)->Option<[f64;3]>{self.entities.get(&id).copied()}
        pub fn native_camera(&self)->Option<Camera>{Some(Camera{position:[0.;3]})}
        pub fn camera_orientation(&self)->glam::Quat{glam::Quat::IDENTITY}
        pub fn effective_camera_type(&self)->Perspective{Perspective}
        pub fn tick(&self){self.client_particles_tick()}
        pub fn frame(&self,partial:f64){self.client_particles_frame(partial);let state=self.hooks.state();*self.geometry.borrow_mut()=state.behaviors.borrow().particles.geometry(super::particles_runtime::wall(),partial as f32,[1.,0.,0.],[0.,1.,0.]);}
// @@HOST@@
    }
    #[path="@@GAME@@"]mod particles;
}
