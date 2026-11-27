pub mod command{pub mod args{pub fn normalize(value:&str)->String{if value.contains(':'){value.into()}else{format!("minecraft:{value}")}}}}
mod native_fixture{
 #[derive(Clone,Copy,Default)]struct Position{x:f64,y:f64,z:f64}
 struct Health(bool);impl Health{fn dead_or_dying(&self)->bool{self.0}}
 struct Local{position:Position,health:Health,allowed:bool}impl Local{fn native_client_queries_allowed(&self)->bool{self.allowed}}
 struct Remote{entity_id:i32,profile_name:String}
 struct Kind{id:String}struct Mob{id:u32,kind:Kind,position:Position,removed:bool}
 struct Guest{placed:bool,entity_id:i32,dimension:String,name:String,position:[f64;3]}
 struct Hub{guests:Vec<Guest>}struct Lan{hub:Hub}struct Dim;impl Dim{fn id(&self)->&'static str{"minecraft:overworld"}}
 struct Game{native:bool,lan_acting:Option<u32>,lan_acting_uuid:Option<u128>,player:Option<Local>,remote:Option<Remote>,mobs:Vec<Mob>,lan:Vec<Lan>,dim:Dim,names:std::collections::HashMap<u32,String>}
 impl Default for Game{fn default()->Self{Self{native:true,lan_acting:None,lan_acting_uuid:None,player:Some(Local{position:Position::default(),health:Health(false),allowed:true}),remote:Some(Remote{entity_id:1,profile_name:"LocalProfile".into()}),mobs:Vec::new(),lan:Vec::new(),dim:Dim,names:Default::default()}}}
 impl Game{fn native_window(&self)->bool{self.native}fn acting_player_name(&self)->&str{"ResidentProfile"}fn remote_player_name(&self,id:u32)->Option<&str>{self.names.get(&id).map(String::as_str)}
 __OWNED_HELPERS__
 }
 #[test]fn native_profile_identity_distance_and_guards(){
  let mut game=Game::default();let local=crate::target_lock_runtime::player_snapshot(&game.native_client_target_lock_player(1)).unwrap();assert_eq!(local.player.name,"LocalProfile");
  game.mobs.push(Mob{id:2,kind:Kind{id:"player".into()},position:Position{x:3.,y:4.,z:0.},removed:true});assert_eq!(game.native_client_target_lock_player(2),"null");game.names.insert(2,"LevelProfile".into());let player=crate::target_lock_runtime::player_snapshot(&game.native_client_target_lock_player(2)).unwrap();assert_eq!(player.distance_squared,25.);assert!(player.removed);
  game.lan.push(Lan{hub:Hub{guests:vec![Guest{placed:true,entity_id:3,dimension:"minecraft:overworld".into(),name:"GuestProfile".into(),position:[0.,0.,6.]}]}});assert_eq!(crate::target_lock_runtime::player_snapshot(&game.native_client_target_lock_player(3)).unwrap().player.name,"GuestProfile");
  game.player.as_mut().unwrap().health.0=true;assert_eq!(game.native_client_target_lock_dead(),Some(true));game.lan_acting=Some(1);assert_eq!(game.native_client_target_lock_dead(),None);assert_eq!(game.native_client_target_lock_player(1),"null");
 }
}
