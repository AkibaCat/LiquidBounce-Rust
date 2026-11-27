use super::*;
use crate::navigation::{aabb::Aabb,flight_body::{capture_flight,PlayerFlightAbilities},progress::FollowState};
use std::{cell::RefCell,rc::Rc};
type Events=Rc<RefCell<Vec<String>>>;
struct Abilities {flying:bool,may_fly:bool,speed:f32,events:Events}
impl PlayerFlightAbilities for Abilities {
 fn flying(&self)->bool {self.events.borrow_mut().push("af".into());self.flying}
 fn may_fly(&self)->bool {self.events.borrow_mut().push("am".into());self.may_fly}
 fn flying_speed(&self)->f32 {self.events.borrow_mut().push("as".into());self.speed}
}
struct Player {position:Vec3,delta:Vec3,tick:i32,yaw:f32,sprint:bool,horizontal:bool,vertical:bool,ground:bool,abilities:Abilities,events:Events}
impl Player {fn event(&self,value:&str){self.events.borrow_mut().push(value.into());}}
impl FlightPlayer for Player {
 fn position(&self)->Vec3 {self.event("p");self.position}
 fn delta_movement(&self)->Vec3 {self.event("d");self.delta}
 fn tick_count(&self)->i32 {self.event("t");self.tick}
 fn horizontal_collision(&self)->bool {self.event("h");self.horizontal}
 fn vertical_collision(&self)->bool {self.event("v");self.vertical}
 fn y_rot(&self)->f32 {self.event("y");self.yaw}
 fn is_sprinting(&self)->bool {self.event("s");self.sprint}
}
impl FlightActor for Player {
 type Abilities=Abilities;
 fn abilities(&self)->&Abilities {self.event("a");&self.abilities}
 fn y(&self)->f64 {self.event("Y");self.position.y}
 fn on_ground(&self)->bool {self.event("g");self.ground}
 fn bounding_box(&self)->Aabb {self.event("B");Aabb::new(Vec3::new(self.position.x-0.3,self.position.y,self.position.z-0.3),Vec3::new(self.position.x+0.3,self.position.y+1.8,self.position.z+0.3))}
 fn interpolated_position(&self,partial_ticks:f32)->Vec3 {self.event(&format!("i:{}",partial_ticks.to_bits()));Vec3::new(self.position.x+self.delta.x*f64::from(partial_ticks),self.position.y+self.delta.y*f64::from(partial_ticks),self.position.z+self.delta.z*f64::from(partial_ticks))}
}
struct Body {player:Player,host:bool,can_fly:bool,collision:bool,events:Events}
impl FlightBody for Body {
 type Player=Player;
 fn player(&self)->&Player {self.events.borrow_mut().push("P".into());&self.player}
 fn is_flying(&self)->bool {self.events.borrow_mut().push("H".into());self.host}
 fn press(&mut self,input:Input,exact:bool){self.events.borrow_mut().push(format!("K:{}:{exact}",input.java_hash()));}
 fn no_block_collision(&mut self,b:Aabb)->bool{self.events.borrow_mut().push(format!("C:{}",[b.min.x,b.min.y,b.min.z,b.max.x,b.max.y,b.max.z].map(raw).join(",")));!self.collision}
}
fn raw(v:f64)->String {if v.is_nan(){0x7ff8_0000_0000_0000_u64}else{v.to_bits()}.to_string()}
fn vector(v:Vec3)->String {[v.x,v.y,v.z].map(raw).join(",")}
fn vecs(v:&[Vec3])->String {v.iter().copied().map(vector).collect::<Vec<_>>().join(";")}
fn acceleration(a:Acceleration)->&'static str {match a{Acceleration::Momentum=>"MOMENTUM",Acceleration::Direct=>"DIRECT"}}
fn state(f:&Follower,h:&Handling)->String {
 let route=f.route.as_ref().map_or_else(||"-".into(),|r|format!("{}^{}^{}^{}^{}^{}^{}",r.corner,r.watch.idle_ticks(),raw(r.nearest),r.missed,vector(r.path.start),vecs(&r.path.points),r.path.can_reach));
 format!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{route}|{}|{}|{}|{}|{}|{}",f.take_off_ticks,f.is_landing,f.has_landed,f.pressed.map_or_else(||"-".into(),|p|p.java_hash().to_string()),f.target().map_or_else(||"-".into(),vector),f.sprint(),f.is_finished(),f.is_moving(),vecs(f.ahead()),raw(h.drag()),raw(h.vertical_drag()),raw(h.speed()),raw(h.vertical_speed()),raw(h.step()),h.last_move().map_or_else(||"-".into(),vector))
}
#[test]
fn unchanged_flight_follower_capture_and_progress(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
 let events=Events::default();let mut body=Body{player:Player{position:Vec3::ZERO,delta:Vec3::ZERO,tick:0,yaw:0.0,sprint:false,horizontal:false,vertical:false,ground:false,abilities:Abilities{flying:false,may_fly:false,speed:0.05,events:events.clone()},events:events.clone()},host:false,can_fly:true,collision:true,events:events.clone()};let mut h=Handling::default();let mut f=Follower::default();let mut current=FlyAbilities::new(Acceleration::Direct,1.0,0.4,true);let mut watch=ProgressWatch::new(0.1,60);let mut actual=Vec::new();
 for command in commands.lines(){
  let a:Vec<_>=command.split(' ').collect();let d=|s:&str|f64::from_bits(s.parse().unwrap());let v=|s:&str|{let q:Vec<_>=s.split(',').map(d).collect();Vec3::new(q[0],q[1],q[2])};let mut returned="-".to_owned();
  match a[0]{
   "new"=>{h=Handling::default();f=Follower::default();},
   "base"=>current=h.abilities(FlyAbilities::new(if a[1]=="DIRECT"{Acceleration::Direct}else{Acceleration::Momentum},d(a[2]),d(a[3]),a[4].parse().unwrap())),
   "player"=>{let p=&mut body.player;p.tick=a[1].parse().unwrap();p.position=v(a[2]);p.delta=v(a[3]);p.yaw=f32::from_bits(a[4].parse().unwrap());p.sprint=a[5].parse().unwrap();p.horizontal=a[6].parse().unwrap();p.vertical=a[7].parse().unwrap();p.ground=a[8].parse().unwrap();p.abilities.flying=a[9].parse().unwrap();p.abilities.may_fly=a[10].parse().unwrap();p.abilities.speed=f32::from_bits(a[11].parse().unwrap());},
   "host"=>{body.host=a[1].parse().unwrap();body.can_fly=a[2].parse().unwrap();body.collision=a[3].parse().unwrap();},
   "capture"=>{events.borrow_mut().push("o".into());returned=if let Some(captured)=capture_flight(&body,body.can_fly,&mut h){current=captured;format!("{},{},{},{}",acceleration(captured.control()),raw(captured.horizontal_speed()),raw(captured.vertical_speed()),captured.is_flying())}else{"null".into()};},
   "follow"=>f.follow(FlyRoute{start:v(a[1]),points:if a[2]=="-"{Vec::new()}else{a[2].split(';').map(v).collect()},can_reach:a[3].parse().unwrap()}),
   "clear"=>f.clear(),
   "tick"=>returned=match f.tick(&mut body,&mut h,f32::from_bits(a[1].parse().unwrap()),current){FollowState::Walking=>"WALKING",FollowState::Acting=>"ACTING",FollowState::Finished=>"FINISHED",FollowState::Stuck=>"STUCK"}.into(),
   "off"=>returned=f.is_off_path(&body,&h).to_string(),
   "head"=>returned=vector(f.head(&body,f32::from_bits(a[1].parse().unwrap()))),
   "direction"=>returned=directional_input(v(a[1]),f32::from_bits(a[2].parse().unwrap())).java_hash().to_string(),
   "segment"=>returned=raw(distance_segment(v(a[1]),v(a[2]),v(a[3]))),
   "sharp"=>returned=is_sharp(&a[1].split(';').map(v).collect::<Vec<_>>(),a[2].parse().unwrap()).to_string(),
   "seed"=>f.take_off_ticks=a[1].parse().unwrap(),
   "watch"=>watch=ProgressWatch::new(d(a[1]),a[2].parse().unwrap()),
   "stalls"=>returned=watch.stalls(d(a[1])).to_string(),
   "excuse"=>watch.excuse(),"reset"=>watch.reset(),
   "watchseed"=>crate::navigation::progress::fixture_seed_idle(&mut watch,a[1].parse().unwrap()),
   _=>panic!("{command}"),
  }
  actual.push(format!("{returned}|{}|{}|{}",state(&f,&h),watch.idle_ticks(),events.borrow().join("/")));events.borrow_mut().clear();
 }
 std::fs::write(root.join("actual.txt"),actual.join("\n")+"\n").unwrap();let failures:Vec<_>=commands.lines().zip(expected.lines()).zip(&actual).enumerate().filter(|(_,((_,expected),actual))|expected!=actual).map(|(i,((command,expected),actual))|format!("case {i}: {command}\nexpected {expected}\nactual   {actual}")).collect();assert!(failures.is_empty(),"{} mismatches\n{}",failures.len(),failures.iter().take(8).cloned().collect::<Vec<_>>().join("\n"));
}
