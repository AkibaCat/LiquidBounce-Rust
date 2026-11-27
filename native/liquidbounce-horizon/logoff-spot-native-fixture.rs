// Declaration-only Host/world/model providers for the production native bridge.
pub mod mobs {
    use std::sync::Arc;
    #[derive(Clone,Debug,Default,PartialEq)]pub struct Stack(pub &'static str);
    #[derive(Clone,Default)]pub struct Extra {pub off_hand:Option<Arc<str>>,pub head:Option<Arc<str>>,pub chest:Option<Arc<str>>,pub legs:Option<Arc<str>>,pub feet:Option<Arc<str>>,pub armor:[Option<Stack>;4],pub main:Option<Stack>}
    impl Extra{pub fn set_main_hand_stack(&mut self,v:Option<Stack>){self.main=v;}}
    #[derive(Clone)]pub struct Kind{pub id:Arc<str>}
    #[derive(Clone,Copy,Default)]pub struct Join(pub u64);impl Join{pub fn get(self)->u64{self.0}}
    #[derive(Clone,Copy,Default)]pub struct Position{pub x:f64,pub y:f64,pub z:f64}
    #[derive(Clone)]pub struct Mob{pub id:u32,pub kind:Arc<Kind>,pub join:Join,pub removed:bool,pub position:Position,pub previous:Position,pub yaw:f32,pub head_yaw:f32,pub previous_head_yaw:f32,pub head_pitch:f32,pub previous_head_pitch:f32,pub health:f32,pub skin:Option<String>,pub slim:bool,pub skin_3d:Option<Arc<str>>,pub cape:Option<String>,pub extra:Extra}
    impl Mob{pub fn new(id:u32,kind:Arc<Kind>,position:Position,yaw:f32)->Self{Self{id,kind,join:Join(id as u64),removed:false,position,previous:position,yaw,head_yaw:yaw,previous_head_yaw:yaw,head_pitch:0.,previous_head_pitch:0.,health:20.,skin:None,slim:false,skin_3d:None,cape:None,extra:Extra::default()}}pub fn equipment_stacks(&self)->[Option<Stack>;8]{std::array::from_fn(|i|if i==0{self.extra.main.clone()}else{None})}}
}
pub mod lan {pub mod playerdata{pub fn ints_uuid(v:[i32;4])->u128{v.into_iter().fold(0u128,|a,v|(a<<32)|u128::from(v as u32))}}}
pub mod save{pub fn random_uuid()->[i32;4]{use std::sync::atomic::{AtomicI32,Ordering};static NEXT:AtomicI32=AtomicI32::new(5);[12,34,56,NEXT.fetch_add(1,Ordering::Relaxed)]}}
pub mod game_fixture {
    use crate::mobs::*;use std::{collections::HashMap,sync::Arc};
    #[derive(Default)]pub struct Remote{pub level_players:HashMap<i32,u128>,pub level_player_names:HashMap<i32,String>,pub level_player_textures:HashMap<i32,String>}
    #[derive(Default)]pub struct Persist{pub uuids:HashMap<u32,[i32;4]>}
    pub struct Resources(pub Option<Arc<Kind>>);impl Resources{pub fn player_type(&self)->&Option<Arc<Kind>>{&self.0}}
    pub struct World{pub loaded:bool}impl World{pub fn loaded(&self,_:i32,_:i32)->bool{self.loaded}}
    pub struct Score{pub board:Board}pub struct Board{pub display:String,pub name:String,pub score:Option<ScoreValue>}
    pub struct Text<'a>(&'a str);impl Text<'_>{pub fn plain(&self)->&str{self.0}}
    pub struct Objective<'a>{pub name:String,pub display:&'a str}impl Objective<'_>{pub fn display_text(&self)->Text<'_>{Text(self.display)}}
    pub struct ScoreValue{pub value:i32}
    impl Board{pub fn display_objective(&self,place:&str)->Option<Objective<'_>>{assert_eq!(place,"below_name");Some(Objective{name:self.name.clone(),display:&self.display})}pub fn score(&self,holder:&str,_:&str)->Option<&ScoreValue>{if holder=="Alice"{self.score.as_ref()}else{None}}}
    pub struct Game<'a>{pub mobs:Vec<Mob>,pub remote:Option<Remote>,pub persist:Persist,pub res:Resources,pub world:World,pub score:Score,pub lan_acting_uuid:Option<u128>,pub client_logoff_spots:HashMap<u128,logoff_spot::LoggedOffPlayer>,pub window:bool,pub blink:Option<i32>,pub hooks:crate::runtime::LiquidBounceHooks,pub chat:Vec<String>,pub lifetime:std::marker::PhantomData<&'a ()>}
    impl Default for Game<'_>{fn default()->Self{Self{mobs:vec![],remote:Some(Remote::default()),persist:Persist::default(),res:Resources(Some(Arc::new(Kind{id:"minecraft:player".into()}))),world:World{loaded:true},score:Score{board:Board{display:"Health ❤".into(),name:"hp".into(),score:Some(ScoreValue{value:7})}},lan_acting_uuid:None,client_logoff_spots:HashMap::new(),window:true,blink:None,hooks:Default::default(),chat:vec![],lifetime:std::marker::PhantomData}}}
    pub mod remote_mob_data{pub fn client_defaults(_: &mut crate::mobs::Mob){}}
    impl Game<'_>{fn native_window(&self)->bool{self.window}fn native_client_blink_dummy(&self)->Option<i32>{self.blink}fn native_entity(&self,id:u32)->Option<()>{self.mobs.iter().any(|m|m.id==id).then_some(())}fn remote_player_uuid(&self,id:u32)->Option<u128>{self.remote.as_ref()?.level_players.get(&(id as i32)).copied()}fn remote_player_name(&self,id:u32)->Option<&str>{self.remote.as_ref()?.level_player_names.get(&(id as i32)).map(String::as_str)}
    // @@HOST@@
    // @@REMOTE@@
    }
    #[path="@@GAME@@"]pub mod logoff_spot;
    pub mod mod_api{pub fn mods(){}pub fn dispatch_client_logoff_spot(game:&mut super::Game<'_>,_:(),input:&str,output:&mut String){let hooks=game.hooks.clone();hooks.logoff_spot(game,&mut crate::mod_api::ClientLogoffSpotEvent{input,output});}}
    impl crate::mod_api::Host for Game<'_>{fn chat(&mut self,s:&str){self.chat.push(s.into());}fn client_logoff_spot_clear(&mut self){self.client_logoff_clear();}}
}
