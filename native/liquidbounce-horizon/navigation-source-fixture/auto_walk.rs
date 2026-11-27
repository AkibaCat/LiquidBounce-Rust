use crate::navigation::{auto_walk::{AutoWalk,AutoWalkHost,FollowPlayer,Mode},geometry::{BlockPos,Vec3},input::Input,java_case::equals_ignore_case};
use std::collections::HashMap;
fn p(raw:&str)->BlockPos{let a:Vec<i32>=raw.split(',').map(|s|s.parse().unwrap()).collect();BlockPos::new(a[0],a[1],a[2])}
fn v(raw:&str)->Vec3{let a:Vec<f64>=raw.split(',').map(|s|f64::from_bits(s.parse().unwrap())).collect();Vec3::new(a[0],a[1],a[2])}
fn vector(v:Vec3)->String{format!("{},{},{}",v.x.to_bits(),v.y.to_bits(),v.z.to_bits())}
fn point(p:BlockPos)->String{format!("{},{},{}",p.x,p.y,p.z)}
fn string(raw:&str)->String{if raw=="-"{String::new()}else{String::from_utf8(raw.as_bytes().as_chunks::<2>().0.iter().map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect()).unwrap()}}
fn hex(raw:&str)->String{if raw.is_empty(){"-".into()}else{raw.as_bytes().iter().map(|b|format!("{b:02x}")).collect()}}
fn mode(raw:&str)->Mode{match raw{"Forward"=>Mode::Forward,"Direction"=>Mode::Direction,"Goto"=>Mode::Goto,"Follow"=>Mode::Follow,_=>panic!("{raw}")}}
struct Host{position:Vec3,vehicle:Option<Vec3>,yaw:f32,direction:Vec3,chunk:bool,stands:HashMap<BlockPos,bool>,players:Vec<FollowPlayer>,events:Vec<String>}
impl AutoWalkHost for Host{
 fn position(&self)->Vec3{self.position}fn yaw(&self)->f32{self.yaw}fn root_vehicle_position(&self)->Vec3{self.vehicle.unwrap_or(self.position)}fn direction_from_rotation(&self,_yaw:f32)->Vec3{self.direction}
 fn is_standable(&mut self,p:BlockPos)->bool{self.events.push(format!("T:{}",point(p)));*self.stands.get(&p).unwrap_or(&false)}fn has_chunk_at(&self,_:BlockPos)->bool{self.chunk}fn players(&self)->Vec<FollowPlayer>{let mut players=vec![FollowPlayer{is_local_player:true,profile_name:"local".into(),position:self.position}];players.extend(self.players.clone());players}
 fn navigate(&mut self,input:&mut Input,goal:Vec3,radius:f64){self.events.push(format!("N:{}:{}",vector(goal),radius.to_bits()));*input=Input::from_mask(86);}
 fn halt(&mut self){self.events.push("H".into());}fn reset(&mut self){self.events.push("R".into());}fn arrived(&mut self,destination:BlockPos){self.events.push(format!("C:arrived:{}",point(destination)));}
}
#[test]fn unchanged_autowalk_modes_caches_arrival_lifecycle_and_unicode_names(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();let directions=std::fs::read_to_string(root.join("directions.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());assert_eq!(commands.lines().count(),directions.lines().count());
 let mut state=AutoWalk::default();let mut host=Host{position:Vec3::ZERO,vehicle:None,yaw:0.0,direction:Vec3::ZERO,chunk:true,stands:HashMap::new(),players:Vec::new(),events:Vec::new()};let mut in_game=true;let mut parent_running=true;
 for(index,((command,expected),direction))in commands.lines().zip(expected.lines()).zip(directions.lines()).enumerate(){host.direction=v(direction);let a:Vec<_>=command.split(' ').collect();let running=state.enabled()&&in_game&&parent_running;let mut returned="-".to_owned();match a[0]{
  "enabled"=>state.set_enabled(a[1].parse().unwrap(),in_game,&mut host),"mode"=>state.select_mode(mode(a[1]),running,&mut host),
  "input"=>{let mut input=Input::from_mask(a[1].parse().unwrap());if running{state.movement_input(&mut input,&mut host);}returned=input.java_hash().to_string();},
  "position"=>host.position=v(a[1]),"vehicle"=>host.vehicle=(a[1]!="self").then(||v(a[1])),"yaw"=>host.yaw=f32::from_bits(a[1].parse().unwrap()),
  "dest"=>state.destination=p(a[1]),"stand"=>{host.stands.insert(p(a[1]),a[2].parse().unwrap());},"chunk"=>host.chunk=a[1].parse().unwrap(),"target"=>state.target=string(a[1]),"distance"=>state.follow_distance=f32::from_bits(a[1].parse().unwrap()),
  "players"=>{host.players.clear();if a[1]!="-"{for raw in a[1].split(';'){let mut fields=raw.split(':');let profile_name=string(fields.next().unwrap());let position=v(fields.next().unwrap());host.players.push(FollowPlayer{is_local_player:false,profile_name,position});}}},
  "goto"=>state.go_to(v(a[1]),running,in_game,&mut host),"follow"=>state.follow(string(a[1]),running,in_game,&mut host),"ingame"=>in_game=a[1].parse().unwrap(),"parent"=>parent_running=a[1].parse().unwrap(),"disable"=>state.disable_active(&mut host),"case"=>returned=equals_ignore_case(&string(a[1]),&string(a[2])).to_string(),_=>panic!("{command}")
 }
 let result=format!("{returned}|{}|{}|{}|{}|{}|{}",state.enabled(),state.mode().tag(),point(state.destination),hex(&state.target),state.follow_distance.to_bits(),host.events.join("/"));host.events.clear();assert_eq!(result,expected,"case {index}: {command}");
 }
}
