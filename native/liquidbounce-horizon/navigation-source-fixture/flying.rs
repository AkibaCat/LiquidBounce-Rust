use super::*;
use crate::navigation::{aabb::Aabb,flight_body::{FlightActor,PlayerFlightAbilities}};
use std::{collections::{HashMap,VecDeque},sync::{Arc,Mutex}};
type Events=Arc<Mutex<Vec<String>>>;
struct Abilities {flying:bool,may_fly:bool,speed:f32,events:Events}
impl PlayerFlightAbilities for Abilities {
 fn flying(&self)->bool {self.events.lock().unwrap().push("af".into());self.flying}
 fn may_fly(&self)->bool {self.events.lock().unwrap().push("am".into());self.may_fly}
 fn flying_speed(&self)->f32 {self.events.lock().unwrap().push("as".into());self.speed}
}
struct Player {position:Vec3,delta:Vec3,tick:i32,yaw:f32,sprint:bool,horizontal:bool,vertical:bool,ground:bool,abilities:Abilities,events:Events}
impl Player {fn event(&self,value:&str){self.events.lock().unwrap().push(value.into());}}
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
struct Body {player:Player,host:bool,present:bool,collision:bool,events:Events}
impl FlightBody for Body {
 type Player=Player;
 fn player(&self)->&Player {self.events.lock().unwrap().push("P".into());&self.player}
 fn is_flying(&self)->bool {self.events.lock().unwrap().push("H".into());self.host}
 fn press(&mut self,input:Input,exact:bool){self.events.lock().unwrap().push(format!("K:{}:{exact}",input.java_hash()));}
 fn no_block_collision(&mut self,b:Aabb)->bool{if self.present {self.events.lock().unwrap().push(format!("C:{}",[b.min.x,b.min.y,b.min.z,b.max.x,b.max.y,b.max.z].map(raw).join(",")));!self.collision} else {false}}
}

use crate::navigation::{aabb::Direction,background::{BackgroundLevel,Executor},destination::Destination,geometry::BlockRegion,owned_search::OwnedTerrainSearch,replanner::ReplannerLevel,route_storage::shared_destination,search::{Failure,Outcome},staged_search::SharedLevel,terrain::{BlockKind,Terrain as RealTerrain,TerrainLevel,TerrainState},voxel_shape::VoxelShape};
fn raw(v:f64)->String{(if v.is_nan(){0x7ff8_0000_0000_0000}else{v.to_bits()}).to_string()}
fn vector(v:Vec3)->String{[v.x,v.y,v.z].map(raw).join(",")}
fn vecs(v:&[Vec3])->String{v.iter().copied().map(vector).collect::<Vec<_>>().join(";")}
fn pos(p:BlockPos)->String{format!("{},{},{}",p.x,p.y,p.z)}
fn p(s:&str)->BlockPos{let q:Vec<i32>=s.split(',').map(|s|s.parse().unwrap()).collect();BlockPos::new(q[0],q[1],q[2])}
type WorkQueue=Arc<Mutex<VecDeque<Box<dyn FnOnce()+Send>>>>;
#[derive(Clone,Default)]struct Queue(WorkQueue);
impl Executor for Queue{fn execute(&self,t:Box<dyn FnOnce()+Send>){self.0.lock().unwrap().push_back(t);}}
impl Queue{fn run(&self){let task=self.0.lock().unwrap().pop_front();if let Some(task)=task{task();}}fn len(&self)->usize{self.0.lock().unwrap().len()}}
#[derive(Clone)]struct Config{program:Vec<String>,points:Vec<Vec3>,reachable:bool}
#[derive(Clone)]struct State(i32);
impl TerrainState for State{
 fn same_instance(&self,other:&Self)->bool{self.0==other.0}
 fn same_block(&self,other:&Self)->bool{self.0==other.0}
 fn same_fluid(&self,_:&Self)->bool{true}
 fn is_hazard(&self)->bool{false}
 fn is_water(&self)->bool{false}
 fn has_liquid(&self)->bool{false}
 fn is_climbable(&self)->bool{false}
 fn kind(&self,_:BlockKind)->bool{false}
 fn open(&self)->Option<bool>{None}
 fn with_open(&self,_:bool)->Self{self.clone()}
 fn is_upper_half(&self)->bool{false}
 fn horizontal_facing(&self)->Option<Direction>{None}
 fn bounce_restitution(&self)->f32{0.0}
 fn collision_shape<L:TerrainLevel<Self>>(&self,_:&L,_:BlockPos)->VoxelShape{panic!("unused controlled-search shape")}
}
struct Level{states:HashMap<BlockPos,i32>,known:bool,events:Events}
impl TerrainLevel<State> for Level{
 fn block_state(&self,p:BlockPos)->State{self.events.lock().unwrap().push(format!("B:{}",pos(p)));State(*self.states.get(&p).unwrap_or(&0))}
 fn void_air(&self)->State{State(0)}
 fn is_known(&self,p:BlockPos)->bool{self.events.lock().unwrap().push(format!("L:{}",pos(p)));self.known&&(-64..=319).contains(&p.y)}
 fn is_inside_border(&self,x:f64,z:f64)->bool{self.events.lock().unwrap().push(format!("D:{},{}",raw(x),raw(z)));true}
 fn min_y(&self)->i32{-64}
 fn height(&self)->i32{384}
}
type Terrain=RealTerrain<State,SharedLevel<Level>>;
struct World{present:bool,known:bool,states:HashMap<BlockPos,i32>,events:Events}
impl World{fn new(events:Events)->Self{Self{present:true,known:true,states:HashMap::new(),events}}}
impl BackgroundLevel<Terrain> for World{
 fn snapshot(&mut self,r:BlockRegion)->Option<Terrain>{if !self.present{return None;}self.events.lock().unwrap().push(format!("N:{},{},{},{},{},{}",r.min.x,r.min.y,r.min.z,r.max.x,r.max.y,r.max.z));Some(Terrain::player(SharedLevel::new(Level{states:self.states.clone(),known:self.known,events:self.events.clone()})))}
 fn has_level(&mut self)->bool{self.present}
 fn invalidate(&mut self,t:&mut Terrain,p:BlockPos)->bool{self.events.lock().unwrap().push(format!("I:{}",pos(p)));t.invalidate(p,&State(*self.states.get(&p).unwrap_or(&0)))}
}
impl ReplannerLevel<Terrain> for World{fn is_known(&mut self,p:BlockPos)->bool{(-64..=319).contains(&p.y)&&self.known}}
struct Search{terrain:Terrain,config:Config,start:Vec3,step:usize,events:Events}
impl Search{
 #[allow(clippy::too_many_arguments)]
 fn new(mut terrain:Terrain,start:Vec3,goal:SearchGoal,abilities:FlyAbilities,limits:Limits,config:Config,events:Events)->Self{
  let staged=goal.stage_range.is_some();let destination=shared_destination::<State,Level>(goal,terrain.level().clone());let checked=[BlockPos::new(0,0,0),BlockPos::new(4,0,0),BlockPos::new(100,0,0)].into_iter().map(|p|format!("{},{}",destination.is_goal(p),raw(destination.distance(p)))).collect::<Vec<_>>().join(";");
  events.lock().unwrap().push(format!("F:{}:{staged}:{}:{}:{}:{}:{}:{}:{checked}",vector(start),if abilities.control()==Acceleration::Direct{"DIRECT"}else{"MOMENTUM"},raw(abilities.horizontal_speed()),raw(abilities.vertical_speed()),abilities.is_flying(),raw(limits.max_length),limits.max_visited_nodes));
  terrain.state(BlockPos::new(0,0,0));Self{terrain,config,start,step:0,events}
 }
}
impl OwnedTerrainSearch<Terrain> for Search{
 type Route=FlyRoute;
 fn terrain_mut(&mut self)->&mut Terrain{&mut self.terrain}
 fn advance(&mut self)->Outcome<FlyRoute>{let token=&self.config.program[self.step.min(self.config.program.len()-1)];self.events.lock().unwrap().push(format!("A:{}",self.step));self.step+=1;match token.as_bytes()[0]{b'S'=>Outcome::Searching,b'F'=>Outcome::Found(FlyRoute{start:self.start,points:self.config.points.clone(),can_reach:self.config.reachable}),b'A'=>Outcome::AlreadyThere,b'E'=>Outcome::Failed(match &token[1..]{"NO_PATH"=>Failure::NoPath,"TOO_FAR"=>Failure::TooFar,"TIMED_OUT"=>Failure::TimedOut,_=>panic!("token")}),_=>panic!("fixture search exception")}}
 fn preview(&mut self)->Option<Vec<Vec3>>{self.events.lock().unwrap().push(format!("Q:{}",self.step));(self.config.program[self.step.saturating_sub(1).min(self.config.program.len()-1)]=="S1").then(||vec![Vec3::new(self.step as f64,0.0,0.0)])}
}
struct Report(Events);
impl FlightReport for Report{
 fn following(&mut self,points:&[Vec3],f:&Follower){self.0.lock().unwrap().push(format!("R:F:{}:{}:{}",vecs(points),f.is_moving(),vecs(f.ahead())));}
 fn invalid(&mut self){self.0.lock().unwrap().push("R:I".into());}
 fn searching(&mut self,start:Vec3,goal:Vec3,radius:f64,preview:Option<&[Vec3]>){self.0.lock().unwrap().push(format!("R:S:{}:{}:{}:{}",vector(start),vector(goal),raw(radius),preview.map_or_else(||"-".into(),vecs)));}
 fn done(&mut self){self.0.lock().unwrap().push("R:D".into());}
}
fn create(queue:&Queue,config:Arc<Mutex<Config>>,events:Events)->Flying<Terrain,Search>{Flying::new(Arc::new(queue.clone()),Arc::new(move|t,start,goal,a,l|Search::new(t,start,goal,a,l,config.lock().unwrap().clone(),events.clone())))}
#[test]
fn unchanged_flying_background_and_owned_search_lifecycle(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
 let events=Events::default();let mut body=Body{player:Player{position:Vec3::ZERO,delta:Vec3::ZERO,tick:0,yaw:0.0,sprint:false,horizontal:false,vertical:false,ground:false,abilities:Abilities{flying:false,may_fly:false,speed:0.05,events:events.clone()},events:events.clone()},host:false,present:true,collision:false,events:events.clone()};
 let config=Arc::new(Mutex::new(Config{program:vec!["S1".into(),"F".into()],points:vec![Vec3::ZERO,Vec3::new(8.0,0.0,0.0)],reachable:true}));let mut queue=Queue::default();let mut world=World::new(events.clone());let mut flying=create(&queue,config.clone(),events.clone());let mut report=Report(events.clone());let mut current=FlyAbilities::new(Acceleration::Direct,1.0,0.4,true);let mut actual=Vec::new();
 for command in commands.lines(){let a:Vec<_>=command.split(' ').collect();let d=|s:&str|f64::from_bits(s.parse().unwrap());let v=|s:&str|{let q:Vec<_>=s.split(',').map(d).collect();Vec3::new(q[0],q[1],q[2])};match a[0]{
  "new"=>{world=World::new(events.clone());body.present=true;body.collision=false;queue=Queue::default();flying=create(&queue,config.clone(),events.clone());},
  "base"=>current=FlyAbilities::new(if a[1]=="DIRECT"{Acceleration::Direct}else{Acceleration::Momentum},d(a[2]),d(a[3]),a[4].parse().unwrap()),
  "player"=>{let p=&mut body.player;p.tick=a[1].parse().unwrap();p.position=v(a[2]);p.delta=v(a[3]);p.yaw=f32::from_bits(a[4].parse().unwrap());p.sprint=a[5].parse().unwrap();p.horizontal=a[6].parse().unwrap();p.vertical=a[7].parse().unwrap();p.ground=a[8].parse().unwrap();p.abilities.flying=a[9].parse().unwrap();p.abilities.may_fly=a[10].parse().unwrap();p.abilities.speed=f32::from_bits(a[11].parse().unwrap());},
  "program"=>*config.lock().unwrap()=Config{program:a[1].split(',').map(str::to_owned).collect(),points:a[2].split(';').map(v).collect(),reachable:a[3].parse().unwrap()},
  "world"=>{world.present=a[1].parse().unwrap();body.present=world.present;},"known"=>world.known=a[1].parse().unwrap(),"state"=>{world.states.insert(p(a[1]),a[2].parse().unwrap());},"collision"=>body.collision=a[1].parse().unwrap(),
  "run"=>queue.run(),"nav"=>flying.navigate(&mut body,&mut world,&mut report,v(a[1]),d(a[2]),d(a[3]),current,f32::from_bits(a[4].parse().unwrap())),"halt"=>flying.halt(),"level"=>flying.level_changed(),"change"=>flying.block_changed(p(a[1])),_=>panic!("{command}")
 }
 let r=&flying.routes;let status=format!("{},{},{},{},{},{},{},{},{},{}",flying.is_active(),flying.stuck,r.ticks(),r.failed(),r.ends_at_stage(),r.is_searching(),r.is_due(false),r.is_due(true),flying.target().map_or_else(||"-".into(),vector),flying.sprint());
 actual.push(format!("{status}|{}|{}|{}|{}",crate::navigation::flight_follower::fixture_state(&flying.follower,&flying.handling),r.preview().map_or_else(||"-".into(),vecs),queue.len(),events.lock().unwrap().join("/")));events.lock().unwrap().clear();
 }
 std::fs::write(root.join("actual.txt"),actual.join("\n")+"\n").unwrap();let failures:Vec<_>=commands.lines().zip(expected.lines()).zip(&actual).enumerate().filter(|(_,((_,expected),actual))|expected!=actual).map(|(i,((command,expected),actual))|format!("case {i}: {command}\nexpected {expected}\nactual   {actual}")).collect();assert!(failures.is_empty(),"{} mismatches\n{}",failures.len(),failures.iter().take(8).cloned().collect::<Vec<_>>().join("\n"));
}
