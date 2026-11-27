// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac pathfinder/route/Destination.kt.
use super::geometry::{BlockPos,BlockRegion,Vec3};

pub trait Destination{
    fn is_goal(&self,pos:BlockPos)->bool;
    fn distance(&self,pos:BlockPos)->f64;
    fn ends(&self)->Vec<Vec3>;
    fn region(&self,start:Vec3,max_length:f64,vertical_margin:i32)->BlockRegion{BlockRegion::around(start,&self.ends(),max_length,vertical_margin)}
}
#[derive(Clone,Debug)]
pub struct BlockDestination{positions:Vec<BlockPos>}
impl BlockDestination{
    pub fn new(positions:impl IntoIterator<Item=BlockPos>)->Self{let mut unique=Vec::new();for pos in positions{if !unique.contains(&pos){unique.push(pos);}}Self{positions:unique}}
}
impl PartialEq for BlockDestination{fn eq(&self,other:&Self)->bool{self.positions.len()==other.positions.len()&&self.positions.iter().all(|pos|other.positions.contains(pos))}}
impl Eq for BlockDestination{}
impl Destination for BlockDestination{
    fn is_goal(&self,pos:BlockPos)->bool{self.positions.contains(&pos)}
    fn distance(&self,pos:BlockPos)->f64{(self.positions.iter().map(|goal|goal.distance_sqr_long(pos)).min().expect("empty BlockDestination has no minimum")as f64).sqrt()}
    fn ends(&self)->Vec<Vec3>{self.positions.iter().copied().map(Vec3::bottom_center).collect()}
}
#[derive(Clone,Copy,Debug)]
pub struct NearDestination{pub center:Vec3,pub radius:f64,center_y:i32}
impl NearDestination{
    pub fn new(center:Vec3,radius:f64)->Self{Self{center,radius,center_y:center.y.floor()as i32}}
    fn feet_distance(self,pos:BlockPos)->f64{let dx=f64::from(pos.x)+0.5-self.center.x;let dy=f64::from(pos.y.wrapping_sub(self.center_y));let dz=f64::from(pos.z)+0.5-self.center.z;(dx*dx+dy*dy+dz*dz).sqrt()}
}
impl Destination for NearDestination{
    fn is_goal(&self,pos:BlockPos)->bool{self.feet_distance(pos)<=self.radius}
    fn distance(&self,pos:BlockPos)->f64{let distance=self.feet_distance(pos)-self.radius;if distance<0.0{0.0}else{distance}}
    fn ends(&self)->Vec<Vec3>{vec![self.center]}
}
pub struct StageDestination<'a,D,K>{pub goal:&'a D,start:Vec3,range:f64,known:K,max_stage_distance:f64}
impl<'a,D:Destination,K:Fn(BlockPos)->bool> StageDestination<'a,D,K>{
    pub fn new(goal:&'a D,start:Vec3,range:f64,known:K)->Self{Self{goal,start,range,known,max_stage_distance:goal.distance(BlockPos::containing(start))-2.0}}
    pub fn region(start:Vec3,range:f64,vertical_margin:i32)->BlockRegion{BlockRegion::around(start,&[],range,vertical_margin)}
    pub fn is_needed(start:Vec3,goal:Vec3,range:f64,known:impl FnOnce(BlockPos)->bool)->bool{start.distance_sqr(goal)>=range*range||!known(BlockPos::containing(goal))}
    fn is_stage(&self,pos:BlockPos)->bool{let dx=f64::from(pos.x)+0.5-self.start.x;let dz=f64::from(pos.z)+0.5-self.start.z;dx*dx+dz*dz>=self.range*self.range||!(self.known)(pos.offset(1,0,0))||!(self.known)(pos.offset(-1,0,0))||!(self.known)(pos.offset(0,0,1))||!(self.known)(pos.offset(0,0,-1))}
}
impl<D:Destination,K:Fn(BlockPos)->bool> Destination for StageDestination<'_,D,K>{
    fn is_goal(&self,pos:BlockPos)->bool{self.goal.is_goal(pos)||self.is_stage(pos)&&self.goal.distance(pos)<=self.max_stage_distance}
    fn distance(&self,pos:BlockPos)->f64{self.goal.distance(pos)}
    fn ends(&self)->Vec<Vec3>{self.goal.ends()}
}
