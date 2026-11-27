// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent pathfinder/route/AStar.kt, commit 0abf69ac, GPL-3.0-or-later.
//! The open list uses the JVM PriorityQueue's tie order and Double comparator.
use std::{cmp::Ordering,collections::HashMap,hash::Hash};

#[derive(Clone,Debug,PartialEq)]
pub struct WeightedEdge<N> {pub node:N,pub cost:f64}
#[derive(Clone,Debug,PartialEq)]
pub struct ShortestPath<N> {pub nodes:Vec<N>,pub total_cost:f64}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct InvalidEdgeCost(pub f64);
#[derive(Clone,Debug)]
struct Open<N> {node:N,cost:f64,estimate:f64}

fn double_compare(left:f64,right:f64)->Ordering {
    if left<right{return Ordering::Less;}
    if left>right{return Ordering::Greater;}
    let bits=|value:f64|if value.is_nan(){0x7ff8_0000_0000_0000i64}else{value.to_bits() as i64};
    bits(left).cmp(&bits(right))
}

#[derive(Clone,Debug)]
pub struct AStar<N> {
    start:N,
    best_cost:HashMap<N,f64>,
    came_from:HashMap<N,N>,
    open:Vec<Open<N>>,
    nearest:Option<Open<N>>,
    nearest_heuristic:f64,
    found:Option<ShortestPath<N>>,
    max_cost:f64,
    pub iterations:i32,
}
impl<N:Clone+Eq+Hash> AStar<N> {
    pub fn new(start:N,heuristic:f64,max_cost:f64)->Self {
        Self{best_cost:HashMap::from([(start.clone(),0.0)]),came_from:HashMap::new(),
            open:vec![Open{node:start.clone(),cost:0.0,estimate:heuristic}],start,
            nearest:None,nearest_heuristic:f64::INFINITY,found:None,max_cost,iterations:0}
    }
    pub fn is_finished(&self)->bool {self.found.is_some()||self.open.is_empty()}
    pub fn cost(&self,node:&N)->f64 {self.best_cost.get(node).copied().unwrap_or(f64::INFINITY)}
    pub fn previous(&self,node:&N)->Option<&N>{self.came_from.get(node)}
    fn push(&mut self,entry:Open<N>) {
        let mut index=self.open.len();self.open.push(entry.clone());
        while index>0 {
            let parent=(index-1)>>1;
            if double_compare(entry.estimate,self.open[parent].estimate)!=Ordering::Less{break;}
            self.open[index]=self.open[parent].clone();index=parent;
        }
        self.open[index]=entry;
    }
    fn poll(&mut self)->Option<Open<N>> {
        let result=self.open.first()?.clone();let entry=self.open.pop().expect("nonempty heap");
        if !self.open.is_empty() {
            let mut index=0;let half=self.open.len()>>1;
            while index<half {
                let mut child=(index<<1)+1;let right=child+1;
                if right<self.open.len()&&double_compare(self.open[child].estimate,self.open[right].estimate)==Ordering::Greater{child=right;}
                if double_compare(entry.estimate,self.open[child].estimate)!=Ordering::Greater{break;}
                self.open[index]=self.open[child].clone();index=child;
            }
            self.open[index]=entry;
        }
        Some(result)
    }
    pub fn advance(&mut self,max_iterations:i32,mut is_goal:impl FnMut(&N)->bool,
        mut neighbors:impl FnMut(&N)->Vec<WeightedEdge<N>>,mut heuristic:impl FnMut(&N)->f64,
    )->Result<Option<ShortestPath<N>>,InvalidEdgeCost> {
        self.advance_with_context(max_iterations,&mut is_goal,|_,node|neighbors(node),&mut heuristic)
    }
    /// The original neighbor callback can read the owning search's current
    /// costs and predecessors before any of its returned edges are relaxed.
    pub fn advance_with_context(&mut self,max_iterations:i32,mut is_goal:impl FnMut(&N)->bool,
        mut neighbors:impl FnMut(&Self,&N)->Vec<WeightedEdge<N>>,mut heuristic:impl FnMut(&N)->f64,
    )->Result<Option<ShortestPath<N>>,InvalidEdgeCost> {
        let mut left=max_iterations;
        while self.found.is_none()&&left>0 {
            let Some(entry)=self.poll()else{break;};left=left.wrapping_sub(1);self.iterations=self.iterations.wrapping_add(1);
            if entry.cost>self.max_cost||entry.cost>self.cost(&entry.node){continue;}
            let remaining=entry.estimate-entry.cost;
            if remaining<self.nearest_heuristic{self.nearest_heuristic=remaining;self.nearest=Some(entry.clone());}
            if is_goal(&entry.node){self.found=Some(ShortestPath{nodes:self.trace(&entry.node),total_cost:entry.cost});continue;}
            for edge in neighbors(self,&entry.node) {
                if edge.cost.is_nan()||edge.cost<0.0{return Err(InvalidEdgeCost(edge.cost));}
                let cost=entry.cost+edge.cost;
                if cost>self.max_cost||cost>=self.cost(&edge.node){continue;}
                self.best_cost.insert(edge.node.clone(),cost);self.came_from.insert(edge.node.clone(),entry.node.clone());
                let estimate=cost+heuristic(&edge.node);self.push(Open{node:edge.node,cost,estimate});
            }
        }
        Ok(self.found.clone())
    }
    pub fn closest_path(&self)->Option<ShortestPath<N>> {
        let entry=self.nearest.as_ref()?;
        if entry.node==self.start{return None;}
        Some(ShortestPath{nodes:self.trace(&entry.node),total_cost:entry.cost})
    }
    fn trace(&self,end:&N)->Vec<N> {
        let mut reversed=vec![end.clone()];let mut node=end;
        while *node!=self.start {
            let Some(previous)=self.came_from.get(node)else{return Vec::new();};
            reversed.push(previous.clone());node=previous;
        }
        reversed.reverse();reversed
    }
}
