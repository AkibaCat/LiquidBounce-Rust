use super::{abilities::{GlideAbilities,GlideStack},cruise::{Cruise,ColumnHeights,cruise_altitude,rockets_needed},elytra_motion,landing::Landing,rockets::Rockets,Controls,Steering};
use crate::navigation::geometry::Vec3;
use std::{cell::{Cell,RefCell},collections::HashMap,rc::Rc};
type Events=Rc<RefCell<Vec<String>>>;
fn raw(v:f64)->String{(if v.is_nan(){0x7ff8_0000_0000_0000}else{v.to_bits()}).to_string()}
fn vector(v:Vec3)->String{[v.x,v.y,v.z].map(raw).join(",")}
fn rocket(r:&Rockets)->String{format!("{},{},{}",r.left(),r.boost_ticks(),r.can_fire())}
fn controls(c:Controls)->String{format!("{},{},{},{}",c.y_rot.to_bits(),c.x_rot.to_bits(),c.should_fire_rocket,c.is_boosted)}
struct Stack{id:i32,glide:bool,damageable:bool,maximum:i32,damaged:i32,rocket:bool,amount:i32,duration:Option<i32>,events:Events}
impl Stack{fn log(&self,s:&str){self.events.borrow_mut().push(format!("{}:{s}",self.id));}}
impl GlideStack for Stack{
 fn can_glide_chest(&self)->bool{self.log("glide:CHEST");self.glide}
 fn is_damageable(&self)->bool{self.log("damageable");self.damageable}
 fn max_damage(&self)->i32{self.log("max");self.maximum}
 fn damage(&self)->i32{self.log("damage");self.damaged}
 fn is_firework_rocket(&self)->bool{self.log("item");self.rocket}
 fn count(&self)->i32{self.log("count");self.amount}
 fn firework_flight_duration(&self)->Option<i32>{self.log("flight");self.duration}
}
struct Heights{mode:Rc<Cell<i32>>,events:Events}
impl ColumnHeights for Heights{fn top(&mut self,x:i32,z:i32)->Option<i32>{self.events.borrow_mut().push(format!("H:{x},{z}"));match self.mode.get(){0=>Some(0),1=>None,2=>if x.wrapping_add(z)%3==0{None}else{Some(x.wrapping_mul(x).wrapping_add(z.wrapping_mul(z))%80)},_=>Some(if x<0{120}else{5})}}}
type Fallback=Box<dyn FnMut(Vec3)->Option<Vec3>>;
type FixtureCruise=Cruise<Heights,Fallback>;
fn create(goal:Vec3,rockets:Rockets,ceiling:i32,mode:Rc<Cell<i32>>,fallback:Rc<Cell<Option<Vec3>>>,events:Events)->FixtureCruise{
 let heights=Heights{mode,events:events.clone()};Cruise::new(goal,heights,rockets,ceiling,Box::new(move|at|{events.borrow_mut().push(format!("F:{}",vector(at)));fallback.get()}))
}
#[test]
fn unchanged_glide_capture_rockets_motion_landing_and_cruise(){
 let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let commands=std::fs::read_to_string(root.join("commands.txt")).unwrap();let expected=std::fs::read_to_string(root.join("results.txt")).unwrap();assert_eq!(commands.lines().count(),expected.lines().count());
 let events=Events::default();let mode=Rc::new(Cell::new(0));let fallback=Rc::new(Cell::new(None));let mut stacks=HashMap::new();let mut r=Rockets::new(0,0);let mut l=Landing::new(Vec3::ZERO,r,0.0);let mut c=create(Vec3::ZERO,r,320,mode.clone(),fallback.clone(),events.clone());let mut position=Vec3::ZERO;let mut motion=Vec3::ZERO;let mut actual=Vec::new();
 for command in commands.lines(){let a:Vec<_>=command.split(' ').collect();let d=|s:&str|f64::from_bits(s.parse().unwrap());let v=|s:&str|{let q:Vec<_>=s.split(',').map(d).collect();Vec3::new(q[0],q[1],q[2])};let i=|s:&str|s.parse::<i32>().unwrap();let b=|s:&str|s.parse::<bool>().unwrap();let result=match a[0]{
  "stack"=>{let id=i(a[1]);stacks.insert(id,Stack{id,glide:b(a[2]),damageable:b(a[3]),maximum:i(a[4]),damaged:i(a[5]),rocket:b(a[6]),amount:i(a[7]),duration:(a[8]!="-").then(||i(a[8])),events:events.clone()});"-".into()},
  "abilities"=>{let selected:Vec<_>=if a[3]=="-"{Vec::new()}else{a[3].split(',').map(|id|stacks.get(&i(id)).unwrap()).collect()};GlideAbilities::of(stacks.get(&i(a[1])).unwrap(),selected,b(a[2])).map_or_else(||"-".into(),|a|format!("{},{},{}",a.rockets,a.rocket_ticks,a.glide_ticks))},
  "rocket"=>{r=Rockets::new(i(a[1]),i(a[2]));rocket(&r)},"fire"=>{r.fire();rocket(&r)},"burn"=>format!("{}:{}",r.tick(),rocket(&r)),
  "motion"=>vector(elytra_motion::update(v(a[1]),f32::from_bits(a[2].parse().unwrap()),f32::from_bits(a[3].parse().unwrap()),b(a[4]))),
  "angles"=>format!("{},{}",elytra_motion::y_rot_towards(d(a[1]),d(a[2])).to_bits(),elytra_motion::x_rot_towards(d(a[3]),d(a[4])).to_bits()),
  "landing"=>{l=Landing::new(v(a[1]),Rockets::new(i(a[2]),i(a[3])),d(a[4]));l.fixture_state()},"land"=>format!("{}:{}",controls(l.steer(v(a[1]),v(a[2]))),l.fixture_state()),
  "height"=>{mode.set(i(a[1]));"-".into()},"altitude"=>raw(cruise_altitude(v(a[1]),v(a[2]),&mut Heights{mode:mode.clone(),events:events.clone()})),"needed"=>rockets_needed(d(a[1]),d(a[2]),i(a[3])).to_string(),
  "fallback"=>{fallback.set((a[1]!="-").then(||v(a[1])));"-".into()},"cruise"=>{c=create(v(a[1]),Rockets::new(i(a[2]),i(a[3])),i(a[4]),mode.clone(),fallback.clone(),events.clone());c.fixture_state()},"sail"=>format!("{}:{}",controls(c.steer(v(a[1]),v(a[2]))),c.fixture_state()),
  "position"=>{position=v(a[1]);motion=v(a[2]);"-".into()},"step"=>{let input=if a[1]=="L"{l.steer(position,motion)}else{c.steer(position,motion)};motion=elytra_motion::update(motion,input.y_rot,input.x_rot,input.is_boosted);position=Vec3::new(position.x+motion.x,position.y+motion.y,position.z+motion.z);format!("{}:{}:{}:{}",controls(input),vector(position),vector(motion),if a[1]=="L"{l.fixture_state()}else{c.fixture_state()})},_=>panic!("{command}")};actual.push(format!("{result}|{}",events.borrow().join("/")));events.borrow_mut().clear();
 }
 std::fs::write(root.join("actual.txt"),actual.join("\n")+"\n").unwrap();let failures:Vec<_>=commands.lines().zip(expected.lines()).zip(&actual).enumerate().filter(|(_,((_,expected),actual))|expected!=actual).map(|(i,((command,expected),actual))|format!("case {i}: {command}\nexpected {expected}\nactual   {actual}")).collect();assert!(failures.is_empty(),"{} mismatches\n{}",failures.len(),failures.iter().take(10).cloned().collect::<Vec<_>>().join("\n"));
}
