// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidFiller decisions from pinned ModuleLiquidFiller.kt. Native adapters
//! supply the ordered 26.3 block scan, real fluid state and collision facts.
use std::collections::{HashSet, VecDeque};
pub type Pos = [i32; 3];
pub const SPONGE_MAX_DEPTH: i32 = 6;
pub const SPONGE_MAX_COUNT: usize = 64;
/// Direction.entries, whose traversal order is observable at the absorb limit.
pub const DIRECTIONS: [Pos; 6] = [[0,-1,0],[0,1,0],[0,0,-1],[0,0,1],[-1,0,0],[1,0,0]];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceOrder { Random, CloserFirst, FurtherFirst, BottomTop, TopBottom }
impl PlaceOrder {
    pub const ALL: [Self; 5] = [Self::Random,Self::CloserFirst,Self::FurtherFirst,Self::BottomTop,Self::TopBottom];
    pub fn tag(self)->&'static str { match self {Self::Random=>"Random",Self::CloserFirst=>"CloserFirst",Self::FurtherFirst=>"FurtherFirst",Self::BottomTop=>"BottomTop",Self::TopBottom=>"TopBottom"} }
    pub fn from_tag(tag:&str)->Option<Self>{Self::ALL.into_iter().find(|v|v.tag().eq_ignore_ascii_case(tag))}
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings { pub place_in: u32, pub order: PlaceOrder, pub use_sponge: bool, pub whitelist: bool, pub blocks: Vec<String> }
impl Default for Settings {
    fn default()->Self {
        let mut blocks=["white","orange","magenta","light_blue","yellow","lime","pink","gray","light_gray","cyan","purple","blue","brown","green","red","black"].into_iter().map(|c|format!("minecraft:{c}_wool")).chain(["dirt","cobblestone","stone","netherrack","diorite","granite","andesite"].into_iter().map(|b|format!("minecraft:{b}"))).collect::<Vec<_>>();
        blocks.sort();
        Self { place_in:3,order:PlaceOrder::FurtherFirst,use_sponge:false,whitelist:true,blocks }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fluid { Empty, Water, Lava, Other }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State { pub replaceable:bool, pub fluid:Fluid, pub source:bool, pub absorb_pass:bool }
impl Default for State {fn default()->Self{Self{replaceable:false,fluid:Fluid::Empty,source:false,absorb_pass:false}}}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {pub index:u8,pub item:String,pub block:Option<String>}
pub fn nearby(index:u8,selected:u8)->i32{if index==selected{i32::MIN}else if index==40{i32::MIN+1}else{(i32::from(index)-i32::from(selected)).abs()}}
pub fn normal_slot(settings:&Settings,slots:&[Slot],selected:u8)->Option<u8>{
    slots.iter().filter(|s|s.block.as_ref().is_some_and(|b|settings.blocks.contains(b)==settings.whitelist)).min_by_key(|s|nearby(s.index,selected)).map(|s|s.index)
}
pub fn sponge_slot(settings:&Settings,slots:&[Slot],selected:u8)->Option<u8>{
    settings.use_sponge.then(||slots.iter().filter(|s|s.item=="minecraft:sponge").min_by_key(|s|nearby(s.index,selected)).map(|s|s.index)).flatten()
}
pub fn slot(settings:&Settings,slots:&[Slot],selected:u8,target:Option<State>)->Option<u8>{
    let sponge=sponge_slot(settings,slots,selected);let normal=normal_slot(settings,slots,selected);
    match target {None=>sponge.or(normal),Some(s) if settings.use_sponge && s.fluid==Fluid::Water=>sponge,Some(_)=>normal}
}
pub fn should_fill(settings:&Settings,state:State,normal:bool,sponge:bool)->bool{
    state.replaceable && state.source && match state.fluid {Fluid::Water if settings.place_in&1!=0=>if settings.use_sponge{sponge}else{normal},Fluid::Lava if settings.place_in&2!=0=>normal,_=>false}
}
pub fn distance(pos:Pos,eye:[f64;3])->f64{(0..3).map(|i|(f64::from(pos[i])+0.5-eye[i]).powi(2)).sum()}
pub trait World {
    /// Section scan and out-of-height air in the exact Level.findBlocksIn order.
    fn scan(&mut self,center:[f64;3],radius:f32)->Vec<(Pos,State)>;
    fn state(&mut self,pos:Pos)->Option<State>;
    fn unobstructed(&mut self,pos:Pos)->bool;
    /// Kotlin Random.nextInt(bound), with a uniform integer in 0..bound.
    fn next_int(&mut self,bound:usize)->usize;
}
fn offset(p:Pos,d:Pos)->Pos{std::array::from_fn(|i|p[i].wrapping_add(d[i]))}
pub fn can_absorb<W:World>(world:&mut W,sponge:Pos,target:Pos)->bool{
    if sponge==target{return true;}
    // BlockPos.breadthFirstTraversal counts ACCEPT nodes, including its origin;
    // skipped cells are visited but are neither counted nor expanded.
    let mut queue=VecDeque::from([(sponge,0)]);let mut visited=HashSet::new();let mut accepted=0;
    while let Some((pos,depth))=queue.pop_front(){
        if !visited.insert(pos){continue;}
        if pos!=sponge {
            let Some(state)=world.state(pos) else{continue;};
            if state.fluid!=Fluid::Water{continue;}
            if pos==target{return true;}
            if !state.absorb_pass{continue;}
        }
        accepted+=1;
        if accepted>=SPONGE_MAX_COUNT+1{return false;}
        if depth<SPONGE_MAX_DEPTH{for d in DIRECTIONS{queue.push_back((offset(pos,d),depth+1));}}
    }
    false
}
pub fn sponge_placement<W:World>(world:&mut W,target:Pos,eye:[f64;3],range:f64)->Option<Pos>{
    let center=target.map(|v|f64::from(v)+0.5);let mut best=None;let mut best_distance=f64::INFINITY;
    for (pos,state) in world.scan(center,SPONGE_MAX_DEPTH as f32){
        let d=distance(pos,eye);
        if d>range*range || d>=best_distance || !state.replaceable{continue;}
        if !DIRECTIONS.into_iter().any(|dir|world.state(offset(pos,dir)).is_some_and(|s|!s.replaceable)){continue;}
        if !world.unobstructed(pos) || !can_absorb(world,pos,target){continue;}
        best=Some(pos);best_distance=d;
    }
    best
}
pub fn targets<W:World>(world:&mut W,settings:&Settings,eye:[f64;3],range:f64,normal:bool,sponge:bool)->Vec<Pos>{
    let mut positions=world.scan(eye,range as f32).into_iter().filter(|(p,s)|distance(*p,eye)<=range*range && should_fill(settings,*s,normal,sponge)).map(|(p,_)|p).collect::<Vec<_>>();
    match settings.order {
        PlaceOrder::Random=>{for i in (2..=positions.len()).rev(){let j=world.next_int(i);positions.swap(i-1,j);}},
        PlaceOrder::CloserFirst=>positions.sort_by(|a,b|distance(*a,eye).total_cmp(&distance(*b,eye))),
        PlaceOrder::FurtherFirst=>positions.sort_by(|a,b|distance(*b,eye).total_cmp(&distance(*a,eye))),
        PlaceOrder::BottomTop=>positions.sort_by_key(|p|p[1]),
        PlaceOrder::TopBottom=>positions.sort_by(|a,b|b[1].cmp(&a[1])),
    }
    positions.into_iter().filter_map(|p|{
        if settings.use_sponge && world.state(p).is_some_and(|s|s.fluid==Fluid::Water){sponge_placement(world,p,eye,range)}else{Some(p)}
    }).collect()
}
