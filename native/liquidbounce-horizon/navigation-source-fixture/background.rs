use crate::navigation::{background::{BackgroundLevel,BackgroundSearch,Executor,SliceSearch},destination::{Destination,NearDestination,StageDestination},geometry::{BlockPos,BlockRegion,Vec3},replanner::{Replanner,ReplannerLevel,RouteFactory},search::{Failure,Outcome}};
use std::{collections::{HashMap,VecDeque},sync::{Arc,Mutex}};
type Task=Box<dyn FnOnce()+Send>;
type Events=Arc<Mutex<Vec<String>>>;
#[derive(Clone,Default)]struct Queue(Arc<Mutex<VecDeque<Task>>>);
impl Executor for Queue{fn execute(&self,task:Task){self.0.lock().unwrap().push_back(task);}}
impl Queue{fn run(&self){let task=self.0.lock().unwrap().pop_front();if let Some(task)=task{task();}}fn len(&self)->usize{self.0.lock().unwrap().len()}}
fn p(raw:&str)->BlockPos{let a:Vec<i32>=raw.split(',').map(|v|v.parse().unwrap()).collect();BlockPos::new(a[0],a[1],a[2])}
fn pos(pos:BlockPos)->String{format!("{},{},{}",pos.x,pos.y,pos.z)}
fn v(raw:&str)->Vec3{let a:Vec<f64>=raw.split(',').map(|v|f64::from_bits(v.parse().unwrap())).collect();Vec3::new(a[0],a[1],a[2])}
fn vec(v:Vec3)->String{format!("{},{},{}",v.x.to_bits(),v.y.to_bits(),v.z.to_bits())}
fn region(r:BlockRegion)->String{format!("{},{},{},{},{},{}",r.min.x,r.min.y,r.min.z,r.max.x,r.max.y,r.max.z)}
struct Terrain{states:HashMap<BlockPos,i32>,cache:HashMap<BlockPos,i32>,known:bool,events:Events}
impl Terrain{fn state(&mut self,p:BlockPos)->i32{let events=&self.events;*self.cache.entry(p).or_insert_with(||{if self.known{events.lock().unwrap().push(format!("B:{}",pos(p)));*self.states.get(&p).unwrap_or(&0)}else{0}})}}
struct World{present:bool,known:bool,states:HashMap<BlockPos,i32>,events:Events}
impl World{fn new(events:Events)->Self{Self{present:true,known:true,states:HashMap::new(),events}}}
impl BackgroundLevel<Terrain> for World{
 fn snapshot(&mut self,r:BlockRegion)->Option<Terrain>{if !self.present{return None;}self.events.lock().unwrap().push(format!("N:{}",region(r)));Some(Terrain{states:self.states.clone(),cache:HashMap::new(),known:self.known,events:self.events.clone()})}
 fn has_level(&mut self)->bool{self.present}
 fn invalidate(&mut self,t:&mut Terrain,p:BlockPos)->bool{self.events.lock().unwrap().push(format!("I:{}",pos(p)));let now=*self.states.get(&p).unwrap_or(&0);if t.cache.get(&p).is_some_and(|old|*old==now){false}else{t.cache.remove(&p).is_some()}}
}
impl ReplannerLevel<Terrain> for World{fn is_known(&mut self,p:BlockPos)->bool{(-64..=319).contains(&p.y)&&self.known}}
struct Search{program:Vec<String>,step:usize,events:Events}
impl Search{fn new(t:&mut Terrain,program:Vec<String>)->Self{t.state(BlockPos::new(0,0,0));Self{program,step:0,events:t.events.clone()}}}
impl SliceSearch<Terrain,i32> for Search{
 fn advance(&mut self,_t:&mut Terrain)->Outcome<i32>{let token=self.program.get(self.step).unwrap_or_else(||self.program.last().unwrap());self.events.lock().unwrap().push(format!("A:{}",self.step));self.step+=1;match token.as_bytes()[0]{b'S'=>Outcome::Searching,b'F'=>Outcome::Found(token[1..].parse().unwrap()),b'E'=>Outcome::Failed(failure(&token[1..])),b'A'=>Outcome::AlreadyThere,_=>panic!("fixture advance failure")}}
 fn preview(&self,_t:&Terrain)->Option<Vec<Vec3>>{self.events.lock().unwrap().push(format!("P:{}",self.step));(self.program[self.step.saturating_sub(1).min(self.program.len()-1)]=="S1").then(||vec![Vec3::new(self.step as f64,0.0,0.0)])}
}
fn failure(v:&str)->Failure{match v{"INVALID_DESTINATION"=>Failure::InvalidDestination,"TOO_FAR"=>Failure::TooFar,"NO_PATH"=>Failure::NoPath,"TOO_COMPLEX"=>Failure::TooComplex,"TIMED_OUT"=>Failure::TimedOut,_=>panic!("{v}")}}
fn outcome(v:Outcome<i32>)->String{match v{Outcome::Searching=>"S".into(),Outcome::AlreadyThere=>"A".into(),Outcome::Found(v)=>format!("F:{v}"),Outcome::Failed(f)=>format!("E:{}",match f{Failure::InvalidDestination=>"INVALID_DESTINATION",Failure::TooFar=>"TOO_FAR",Failure::NoPath=>"NO_PATH",Failure::TooComplex=>"TOO_COMPLEX",Failure::TimedOut=>"TIMED_OUT"})}}
fn meta(d:&impl Destination)->String{[BlockPos::new(0,0,0),BlockPos::new(4,0,0),BlockPos::new(100,0,0)].into_iter().map(|p|format!("{},{}",d.is_goal(p),d.distance(p).to_bits())).collect::<Vec<_>>().join(";")}
#[test]fn unchanged_background_search_and_replanner_lifecycle(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
 let events=Arc::new(Mutex::new(Vec::new()));let mut world=World::new(events.clone());let mut queue=Queue::default();let mut bg:BackgroundSearch<i32,Terrain,Search,i32>=BackgroundSearch::new(Arc::new(queue.clone()));let mut rp:Replanner<String,Terrain,Search,i32>=Replanner::new(8,Arc::new(queue.clone()));let mut mode="bg";
 for(index,(command,expected))in commands.lines().zip(expected.lines()).enumerate(){let a:Vec<_>=command.split(' ').collect();let d=|s:&str|f64::from_bits(s.parse().unwrap());let mut returned="-".to_owned();match a[0]{
  "new"=>{world=World::new(events.clone());queue=Queue::default();mode=a[1];bg=BackgroundSearch::new(Arc::new(queue.clone()));rp=Replanner::new(a[2].parse().unwrap(),Arc::new(queue.clone()));},
  "world"=>world.present=a[1].parse().unwrap(),"known"=>world.known=a[1].parse().unwrap(),"state"=>{world.states.insert(p(a[1]),a[2].parse().unwrap());},
  "request"=>{let key=a[1].parse::<i32>().unwrap();let b:Vec<i32>=a[2].split(',').map(|v|v.parse().unwrap()).collect();let r=BlockRegion{min:BlockPos::new(b[0],b[1],b[2]),max:BlockPos::new(b[3],b[4],b[5])};let program:Vec<String>=a[3].split(',').map(str::to_owned).collect();let e=events.clone();bg.request(&mut world,key,r,Arc::new(move|t|{e.lock().unwrap().push(format!("F:{key}"));Search::new(t,program.clone())}));},
  "req"=>{let start=v(a[1]);let goal=NearDestination::new(v(a[2]),d(a[3]));let program:Vec<String>=a[7].split(',').map(str::to_owned).collect();let e=events.clone();let factory:RouteFactory<Terrain,Search>=Arc::new(move|t,descriptor|{let metadata=if let Some(range)=descriptor.stage_range{meta(&StageDestination::new(&descriptor.goal,descriptor.start,range,|_|t.known))}else{meta(&descriptor.goal)};e.lock().unwrap().push(format!("F:{}:{metadata}",if descriptor.stage_range.is_some(){"StageDestination"}else{"NearDestination"}));Search::new(t,program.clone())});rp.request(&mut world,start,goal,d(a[4]),a[6].to_owned(),d(a[5]),factory);},
  "poll"=>returned=outcome(if mode=="bg"{bg.poll(&mut world)}else{rp.poll(&mut world)}),"run"=>queue.run(),
  "tick"=>{for _ in 0..a[1].parse::<usize>().unwrap(){rp.tick();}},"change"=>if mode=="bg"{bg.block_changed(p(a[1]));}else{rp.block_changed(p(a[1]));},
  "reset"=>if mode=="bg"{bg.cancel();}else{rp.reset();},"level"=>if mode=="bg"{bg.level_changed();}else{rp.level_changed();},
  "end"=>rp.route_ends(p(a[1])),"forget"=>rp.forget(),"query"=>{let g=v(a[1]);returned=format!("{},{}",rp.has_goal_moved(g),rp.is_new_goal(g));},
  "seed"=>{rp.ticks=a[1].parse().unwrap();rp.last_request=a[2].parse().unwrap();rp.retry_at=a[3].parse().unwrap();},_=>panic!("{command}")
 }
 let preview=(if mode=="bg"{bg.preview()}else{rp.preview()}).map_or_else(||"-".into(),|p|p.iter().copied().map(vec).collect::<Vec<_>>().join(";"));let status=if mode=="bg"{bg.is_idle().to_string()}else{format!("{},{},{},{},{},{}",rp.ticks(),rp.failed(),rp.ends_at_stage(),rp.is_searching(),rp.is_due(false),rp.is_due(true))};let mut e=events.lock().unwrap();let result=format!("{returned}|{status}|{preview}|{}|{}",queue.len(),e.join("/"));e.clear();assert_eq!(result,expected,"case {index}: {command}");
 }
}
#[test]fn running_rust_slice_owns_its_snapshot_after_nonblocking_cancel(){
 use crate::navigation::background::ThreadExecutor;
 use std::{sync::mpsc,time::Duration};
 struct Level;
 impl BackgroundLevel<()> for Level{fn snapshot(&mut self,_:BlockRegion)->Option<()>{Some(())}fn has_level(&mut self)->bool{true}fn invalidate(&mut self,_:&mut (),_:BlockPos)->bool{false}}
 struct Blocking{started:mpsc::SyncSender<()>,release:mpsc::Receiver<()>,finished:mpsc::SyncSender<()>}
 impl SliceSearch<(),i32> for Blocking{fn advance(&mut self,_:&mut ())->Outcome<i32>{self.started.send(()).unwrap();self.release.recv().unwrap();self.finished.send(()).unwrap();Outcome::Found(7)}}
 let(started_tx,started_rx)=mpsc::sync_channel(1);let(release_tx,release_rx)=mpsc::sync_channel(1);let(finished_tx,finished_rx)=mpsc::sync_channel(1);
 let channels=Mutex::new(Some((started_tx,release_rx,finished_tx)));let mut bg=BackgroundSearch::<i32,(),Blocking,i32>::new(Arc::new(ThreadExecutor));let mut level=Level;
 bg.request(&mut level,1,BlockRegion{min:BlockPos::new(0,0,0),max:BlockPos::new(1,1,1)},Arc::new(move|_|{let(started,release,finished)=channels.lock().unwrap().take().unwrap();Blocking{started,release,finished}}));
 assert_eq!(bg.poll(&mut level),Outcome::Searching);started_rx.recv_timeout(Duration::from_secs(5)).unwrap();bg.cancel();assert!(bg.is_idle());assert_eq!(bg.poll(&mut level),Outcome::Searching);
 release_tx.send(()).unwrap();finished_rx.recv_timeout(Duration::from_secs(5)).unwrap();assert!(bg.is_idle());
}
