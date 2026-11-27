use crate::navigation::{
    walk::{WalkState,abilities::WalkAbilities,moves::Moves,route::{Move,Route,Step},route_search::WalkRouteSearch},
    walk_jump::jump::Jump,simulation::{Mechanics,motion_traits::MotionTraits,block_grid::SimulationState},
    destination::{Destination,BlockDestination,NearDestination},search::{Limits,Outcome,Failure},
};
use std::sync::Arc;
impl WalkState for S {
    fn is_air(&self)->bool{self.air()}
    fn can_be_replaced(&self)->bool{self.registry.borrow().walk_facts[&self.def().block].3}
    fn is_falling(&self)->bool{self.facts().0&64!=0}
    fn is_slime(&self)->bool{self.facts().0&128!=0}
    fn friction(&self)->f32{self.registry.borrow().walk_facts[&self.def().block].0}
    fn speed_factor(&self)->f32{self.registry.borrow().walk_facts[&self.def().block].1}
    fn jump_factor(&self)->f32{self.registry.borrow().walk_facts[&self.def().block].2}
}
impl SimulationState for S {
    fn is_fence(&self)->bool{self.facts().0&256!=0}
    fn is_wall(&self)->bool{self.facts().0&512!=0}
    fn is_fence_gate(&self)->bool{self.facts().0&4!=0}
    fn is_cobweb(&self)->bool{self.def().block=="cobweb"}
    fn is_step_air(&self)->bool{self.air()}
    fn fall_distance_reduction(&self)->f32{self.registry.borrow().walk_fall[&self.def().block]}
}
enum Goal{Blocks(BlockDestination),Near(NearDestination),NoEnds(BlockPos)}
impl Destination for Goal{
    fn is_goal(&self,p:BlockPos)->bool{match self{Self::Blocks(g)=>g.is_goal(p),Self::Near(g)=>g.is_goal(p),Self::NoEnds(t)=>p==*t}}
    fn distance(&self,p:BlockPos)->f64{match self{Self::Blocks(g)=>g.distance(p),Self::Near(g)=>g.distance(p),Self::NoEnds(t)=>(p.distance_sqr_long(*t) as f64).sqrt()}}
    fn ends(&self)->Vec<Vec3>{match self{Self::Blocks(g)=>g.ends(),Self::Near(g)=>g.ends(),Self::NoEnds(_)=>Vec::new()}}
}
fn goal(kind:&str,raw:&str,radius:&str)->Goal{match kind{"B"=>Goal::Blocks(BlockDestination::new(raw.split(';').map(p))),"N"=>Goal::Near(NearDestination::new(v(raw),d(radius))),"G"=>Goal::NoEnds(p(raw)),_=>panic!("goal kind")}}
type WalkSearch=WalkRouteSearch<S,World,Goal>;
#[derive(Default)]
struct WalkFixture{abilities:Option<WalkAbilities<S>>,moves:Option<Moves<S>>,moves_terrain:Option<Terrain<S,World>>,search:Option<WalkSearch>}
fn long_set(raw:&str)->Arc<HashSet<i64>>{Arc::new(if raw=="-"{HashSet::new()}else{raw.split(';').map(|q|p(q).packed()).collect()})}
fn jump_raw(j:&Jump)->String{format!("{}^{}^{}^{}^{}^{}^{}",vector(j.run_up),j.takeoff,j.jumps,j.air_ticks,j.margin.to_bits(),j.hops,vector(j.aim))}
fn move_name(m:Move)->&'static str{match m{Move::Traverse=>"TRAVERSE",Move::Diagonal=>"DIAGONAL",Move::Ascend=>"ASCEND",Move::Fall=>"FALL",Move::Parkour=>"PARKOUR",Move::Pillar=>"PILLAR",Move::Downward=>"DOWNWARD",Move::Swim=>"SWIM",Move::ClimbUp=>"CLIMB_UP",Move::ClimbDown=>"CLIMB_DOWN"}}
fn positions(v:&[BlockPos])->String{v.iter().copied().map(position).collect::<Vec<_>>().join(";")}
fn offer_raw(to:BlockPos,m:Move,cost:f64,breaks:&[BlockPos],places:&[BlockPos],opens:&[BlockPos],jump:Option<&Jump>)->String{
    format!("{}:{}:{}:{}:{}:{}:{}",position(to),move_name(m),cost.to_bits(),positions(breaks),positions(places),positions(opens),jump.map_or_else(||"-".into(),jump_raw))
}
fn step_raw(s:&Step)->String{format!("{}#{}#{}#{}#{}#{}#{}",position(s.from),offer_raw(s.to,s.movement,s.cost,&s.breaks,&s.places,&s.opens,s.jump.as_ref()),vector(s.feet),s.shift,s.edge,s.is_under_water,s.is_bridge())}
fn outcome(o:Outcome<Route<S>>)->String{match o{
    Outcome::Searching=>"S".into(),Outcome::AlreadyThere=>"A".into(),Outcome::Failed(f)=>format!("E:{}",match f{Failure::InvalidDestination=>"INVALID_DESTINATION",Failure::TooFar=>"TOO_FAR",Failure::NoPath=>"NO_PATH",Failure::TooComplex=>"TOO_COMPLEX",Failure::TimedOut=>"TIMED_OUT"}),
    Outcome::Found(r)=>format!("F${}${}${}${}${}${}${}",vector(r.start()),r.steps().iter().map(step_raw).collect::<Vec<_>>().join("@"),r.can_reach(),vector(r.end()),r.points().iter().copied().map(vector).collect::<Vec<_>>().join(";"),r.cost().to_bits(),std::ptr::eq(r.points(),r.points())),
}}
fn extra(a:&[&str],t:&mut Terrain<S,World>,w:&mut WalkFixture,expected:&str)->String{
    match a[0]{
        "ability"=>{
            let mut ab=WalkAbilities::new(a[1].parse().unwrap(),a[2].parse().unwrap(),a[3].parse().unwrap(),a[4].parse().unwrap(),a[6].parse().unwrap(),a[7].parse().unwrap(),a[8].parse().unwrap(),Arc::new(|state:&S|{state.events.borrow_mut().push(format!("T:{}",state.id));state.registry.borrow().walk_ticks.get(&state.id).copied().unwrap_or(1.0)}),long_set(a[9]));
            ab.risky_jumps=a[5].parse().unwrap();ab.air=a[10].parse().unwrap();ab.motion=MotionTraits::new(f32::from_bits(a[11].parse().unwrap()),f32::from_bits(a[12].parse().unwrap()),f32::from_bits(a[13].parse().unwrap()),if t.level().data.borrow().legacy{Mechanics::V1_8}else{Mechanics::Modern});ab.jump_climbs=a[14].parse().unwrap();ab.bridge_ticks=d(a[15]);ab.unbreakable=long_set(a[16]);ab.health=f32::from_bits(a[17].parse().unwrap());ab.reserved=long_set(a[18]);
            w.moves=Some(Moves::new(t.width(),ab.clone()));w.moves_terrain=Some(Terrain::new(t.level().clone(),t.width(),t.height(),t.can_open_doors()));w.abilities=Some(ab);"-".into()
        }
        "ticks"=>{t.level().registry.borrow_mut().walk_ticks.insert(a[1].parse().unwrap(),d(a[2]));"-".into()}
        "moves"=>{let mut offers=Vec::new();let filter=if a[2]=="-"{None}else{Some(p(a[2]))};w.moves.as_mut().unwrap().for_each(w.moves_terrain.as_mut().unwrap(),p(a[1]),&mut |_,_,offer|offers.push(offer_raw(offer.to,offer.movement,offer.cost,offer.breaks,offer.places,offer.opens,offer.jump)),&mut |to|filter.is_none_or(|f|f==to));offers.join("|")}
        "step"=>w.moves.as_mut().unwrap().step(w.moves_terrain.as_mut().unwrap(),p(a[1]),p(a[2])).as_ref().map_or_else(||"-".into(),step_raw),
        "path"=>{let terrain=Terrain::new(t.level().clone(),t.width(),t.height(),t.can_open_doors());let mut limits=Limits::new(d(a[5]));limits.max_visited_nodes=a[6].parse().unwrap();limits.visited_nodes_per_slice=a[7].parse().unwrap();limits.max_compute_nanos=a[8].parse().unwrap();w.search=Some(WalkSearch::new(terrain,v(a[1]),goal(a[2],a[3],a[4]),w.abilities.as_ref().unwrap().clone(),limits));"-".into()}
        "advance"=>{
            let s=w.search.as_mut().unwrap();let nanos:i64=expected.split('~').nth(1).unwrap().parse().unwrap();let elapsed=nanos.wrapping_sub(s.engine.compute_nanos());let mut second=false;
            let returned=s.advance_with_clock(||if second{elapsed}else{second=true;0}).map_or_else(|_|"invalid".into(),outcome);
            let iterations=s.engine.search.as_ref().map_or_else(||"-".into(),|s|s.iterations.to_string());let finished=s.engine.search.as_ref().map_or_else(||"-".into(),|s|s.is_finished().to_string());let preview=s.preview().map_or_else(||"-".into(),|points|points.into_iter().map(vector).collect::<Vec<_>>().join(";"));
            format!("{returned}~{}~{iterations}~{finished}~{preview}",s.engine.compute_nanos())
        }
        "preview"=>w.search.as_ref().unwrap().preview().map_or_else(||"-".into(),|points|points.into_iter().map(vector).collect::<Vec<_>>().join(";")),
        _=>panic!("{}",a[0]),
    }
}
