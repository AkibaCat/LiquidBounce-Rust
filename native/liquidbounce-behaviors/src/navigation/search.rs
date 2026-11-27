// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac route/AStarSearch, Limits, Outcome and Failure.
use super::astar::{AStar,InvalidEdgeCost,WeightedEdge};
use std::{cell::RefCell,hash::Hash,time::Instant};

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Limits{pub max_length:f64,pub max_waypoints:i32,pub max_visited_nodes:i32,pub visited_nodes_per_slice:i32,pub max_compute_nanos:i64}
impl Limits{pub const fn new(max_length:f64)->Self{Self{max_length,max_waypoints:24,max_visited_nodes:8000,visited_nodes_per_slice:400,max_compute_nanos:150_000_000}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Failure{InvalidDestination,TooFar,NoPath,TooComplex,TimedOut}
#[derive(Clone,Debug,PartialEq)]
pub enum Outcome<T>{Searching,AlreadyThere,Found(T),Failed(Failure)}
pub enum Begin<N,T>{Outcome(Outcome<T>),SearchFrom{start:N,max_cost:f64}}
impl<N,T> Begin<N,T>{pub fn from_start(start:N)->Self{Self::SearchFrom{start,max_cost:f64::INFINITY}}}
pub trait SearchProblem<N:Clone+Eq+Hash,T>{
    fn begin(&mut self)->Begin<N,T>;
    fn is_goal(&mut self,node:&N)->bool;
    fn neighbors(&mut self,node:&N)->Vec<WeightedEdge<N>>;
    fn neighbors_with_context(&mut self,node:&N,_search:&AStar<N>)->Vec<WeightedEdge<N>>{self.neighbors(node)}
    fn heuristic(&mut self,node:&N)->f64;
    fn route(&mut self,nodes:Vec<N>)->Outcome<T>;
    fn closest_or(&mut self,failure:Failure,_search:Option<&AStar<N>>)->Outcome<T>{Outcome::Failed(failure)}
}
#[derive(Clone,Debug)]
pub struct AStarSearch<N,T>{pub limits:Limits,pub search:Option<AStar<N>>,compute_nanos:i64,result:Outcome<T>}
impl<N:Clone+Eq+Hash,T:Clone> AStarSearch<N,T>{
    pub fn new(limits:Limits)->Self{Self{limits,search:None,compute_nanos:0,result:Outcome::Searching}}
    pub fn compute_nanos(&self)->i64{self.compute_nanos}
    pub fn advance(&mut self,problem:&mut impl SearchProblem<N,T>)->Result<Outcome<T>,InvalidEdgeCost>{let began=Instant::now();self.advance_with_clock(problem,||began.elapsed().as_nanos()as i64)}
    pub fn advance_with_clock(&mut self,problem:&mut impl SearchProblem<N,T>,mut nano_time:impl FnMut()->i64)->Result<Outcome<T>,InvalidEdgeCost>{
        if !matches!(self.result,Outcome::Searching){return Ok(self.result.clone());}
        let began=nano_time();
        self.result=if let Some(search)=self.search.as_mut(){
            let problem_cell=RefCell::new(&mut *problem);
            let cap=self.limits.visited_nodes_per_slice.min(self.limits.max_visited_nodes.wrapping_sub(search.iterations));
            let path=search.advance_with_context(cap,|node|problem_cell.borrow_mut().is_goal(node),|search,node|problem_cell.borrow_mut().neighbors_with_context(node,search),|node|problem_cell.borrow_mut().heuristic(node))?;
            if let Some(path)=path{problem.route(path.nodes)}
            else if search.is_finished(){problem.closest_or(Failure::NoPath,Some(search))}
            else if search.iterations>=self.limits.max_visited_nodes{problem.closest_or(Failure::TooComplex,Some(search))}
            else{Outcome::Searching}
        }else{
            match problem.begin(){
                Begin::Outcome(outcome)=>outcome,
                Begin::SearchFrom{start,max_cost}=>{let heuristic=problem.heuristic(&start);self.search=Some(AStar::new(start,heuristic,max_cost));Outcome::Searching}
            }
        };
        self.compute_nanos=self.compute_nanos.wrapping_add(nano_time().wrapping_sub(began));
        if matches!(self.result,Outcome::Searching)&&self.compute_nanos>self.limits.max_compute_nanos{self.result=problem.closest_or(Failure::TimedOut,self.search.as_ref());}
        Ok(self.result.clone())
    }
}
pub fn closest_nodes<N:Clone+Eq+Hash>(search:Option<&AStar<N>>,mut distance:impl FnMut(&N)->f64)->Option<Vec<N>>{
    let nodes=search?.closest_path()?.nodes;
    if nodes.len()<2||distance(nodes.first()?)-distance(nodes.last()?)<1.0{None}else{Some(nodes)}
}
