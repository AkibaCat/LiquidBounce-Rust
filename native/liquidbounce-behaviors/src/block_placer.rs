// SPDX-License-Identifier: GPL-3.0-or-later
//! BlockPlacer queue/cooldown and PostRotationExecutor state from LiquidBounce.
use std::collections::VecDeque;
pub type Pos=[i32;3];
#[derive(Clone,Debug,PartialEq)]
pub struct Settings {
    pub range:f32,pub wall_range:f32,pub cooldown:[i32;2],pub swing:String,
    pub construct_fail_result:bool,pub sneak:[i32;2],pub ignore:u32,pub slot_reset_delay:[i32;2],
    pub rotation_mode:String,pub normal_post_move:bool,pub none_post_move:bool,
    pub send_rotation_packet:bool,pub placements:i32,
}
impl Default for Settings {fn default()->Self{Self{range:4.5,wall_range:4.5,cooldown:[1,2],swing:"DoNotHide".into(),construct_fail_result:true,sneak:[1,1],ignore:3,slot_reset_delay:[4,6],rotation_mode:"Normal".into(),normal_post_move:false,none_post_move:false,send_rotation_packet:false,placements:1}}}
impl Settings {pub fn can_act(&self,inventory:bool,using:bool)->bool{!(inventory&&self.ignore&1==0 || using&&self.ignore&2==0)}}
#[derive(Clone,Debug,PartialEq)]
pub struct CrystalSettings {
    pub enabled:bool,pub range:f32,pub wall_range:f32,pub delay:i32,pub swing_mode:String,
    pub rotation_mode:String,pub normal_post_move:bool,pub normal_instant:bool,pub ignore_open_inventory:bool,
    pub none_post_move:bool,pub none_instant:bool,pub send_rotation_packet:bool,
}
impl Default for CrystalSettings {fn default()->Self{Self{enabled:true,range:4.5,wall_range:4.5,delay:0,swing_mode:"DoNotHide".into(),rotation_mode:"Normal".into(),normal_post_move:false,normal_instant:false,ignore_open_inventory:true,none_post_move:false,none_instant:false,send_rotation_packet:false}}}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Crystal {pub id:i32,pub distance_squared:f64}
#[derive(Clone,Debug,Default,PartialEq)]
pub struct CrystalDestroyer {pub target:Option<Crystal>,pub last_attack:i64}
impl CrystalDestroyer {
    pub fn suggest(&mut self,candidate:Crystal,reachable:bool){
        if self.target.is_some_and(|old|old.id==candidate.id || old.distance_squared<=candidate.distance_squared){return;}
        if reachable{self.target=Some(candidate);}
    }
    pub fn elapsed(&self,now:i64,delay:i32)->bool{now.wrapping_sub(self.last_attack)>=i64::from(delay)}
    pub fn attacked(&mut self,now:i64){self.last_attack=now;self.target=None;}
    pub fn removed(&mut self,id:i32){if self.target.is_some_and(|c|c.id==id){self.target=None;}}
}
#[derive(Clone,Debug,PartialEq)]
pub struct Queue {pub blocks:Vec<Pos>,pub inaccessible:Vec<Pos>,pub ticks_to_wait:i32,pub ran_action:bool,pub sneak_times:i32}
impl Default for Queue {fn default()->Self{Self{blocks:vec![],inaccessible:vec![],ticks_to_wait:0,ran_action:false,sneak_times:0}}}
impl Queue {
    /// Retained entries keep their order; sorting only determines newly appended
    /// entries. This matches Long2BooleanLinkedOpenHashMap.computeIfAbsent.
    pub fn update(&mut self,positions:&[Pos])->(Vec<Pos>,Vec<Pos>){
        let removed=self.blocks.iter().copied().filter(|p|!positions.contains(p)).collect::<Vec<_>>();
        self.blocks.retain(|p|positions.contains(p));let mut added=vec![];
        for p in positions {if !self.blocks.contains(p){self.blocks.push(*p);added.push(*p);}}
        (removed,added)
    }
    pub fn clear(&mut self)->Vec<Pos>{std::mem::take(&mut self.blocks)}
    pub fn reset(&mut self){self.sneak_times=0;self.blocks.clear();self.inaccessible.clear();}
    pub fn tick_start(&mut self,cooldown:impl FnOnce()->i32){
        if self.ticks_to_wait>0{self.ticks_to_wait-=1;}else if self.ran_action{self.ran_action=false;self.ticks_to_wait=cooldown();}
    }
    pub fn movement(&mut self)->bool{if self.sneak_times>0{self.sneak_times-=1;true}else{false}}
    pub fn success(&mut self,pos:Pos){self.blocks.retain(|p|*p!=pos);}
}
/// Typed payloads retain the original global priority/post-move queues; callers
/// check their owning module immediately before executing a drained action.
#[derive(Clone,Debug,PartialEq)]
pub struct PostRotationExecutor<T> {pub priority:Option<T>,pub priority_post_move:bool,pub post_move:VecDeque<T>,pub normal:VecDeque<T>}
impl<T> Default for PostRotationExecutor<T>{fn default()->Self{Self{priority:None,priority_post_move:false,post_move:VecDeque::new(),normal:VecDeque::new()}}}
impl<T> PostRotationExecutor<T>{
    pub fn add(&mut self,task:T,post_move:bool,priority:bool){if priority{self.priority=Some(task);self.priority_post_move=post_move;}else if post_move{self.post_move.push_back(task);}else{self.normal.push_back(task);}}
    pub fn first_tick(&mut self)->Vec<T>{let mut tasks=vec![];if !self.priority_post_move{tasks.extend(self.priority.take());}tasks.extend(self.post_move.drain(..));tasks.extend(self.normal.drain(..));tasks}
    pub fn after_move(&mut self,current_rotation:bool)->Vec<T>{
        if (!self.priority_post_move&&self.priority.is_some()) || current_rotation&&self.priority.is_none(){return vec![];}
        let mut tasks=vec![];tasks.extend(self.priority.take());tasks.extend(self.post_move.drain(..));tasks
    }
    pub fn clear(&mut self){self.priority=None;self.priority_post_move=false;self.post_move.clear();self.normal.clear();}
}
