use crate::navigation::{
    walk::{WalkState,abilities::WalkAbilities,changes::Changes,costs::Costs,course::{Course,CourseMotion},nodes::{Landing,Nodes},water::Water},
    simulation::{Mechanics,motion_traits::MotionTraits},
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
#[derive(Default)]
struct WalkFixture{abilities:Option<WalkAbilities<S>>,nodes:Option<Nodes<S>>,changes:Changes,course:Option<Course>}
fn long_set(raw:&str)->Arc<HashSet<i64>>{Arc::new(if raw=="-"{HashSet::new()}else{raw.split(';').map(|q|p(q).packed()).collect()})}
fn landing_raw(l:Option<Landing>)->String{l.map_or_else(||"-".into(),|l|format!("{},{},{}",l.y,l.drop.to_bits(),l.damage))}
fn course_raw(c:&Course)->String{format!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",c.from_x.to_bits(),c.from_y.to_bits(),c.from_z.to_bits(),position(c.target),c.target_x.to_bits(),c.target_z.to_bits(),c.target_height.to_bits(),c.catches,c.dir_x.to_bits(),c.dir_z.to_bits(),c.distance.to_bits(),c.aim_x.to_bits(),c.aim_z.to_bits(),c.edge.to_bits(),c.obstructed)}
fn extra(a:&[&str],t:&mut Terrain<S,World>,w:&mut WalkFixture)->String{
    match a[0]{
        "walk"=>{
            let mut ab=WalkAbilities::new(a[1].parse().unwrap(),a[2].parse().unwrap(),a[3].parse().unwrap(),true,a[4].parse().unwrap(),a[5].parse().unwrap(),a[6].parse().unwrap(),Arc::new(|state:&S|{
                state.events.borrow_mut().push(format!("T:{}",state.id));state.registry.borrow().walk_ticks.get(&state.id).copied().unwrap_or(1.0)
            }),long_set(a[10]));
            ab.health=f32::from_bits(a[7].parse().unwrap());ab.air=a[8].parse().unwrap();ab.reserved=long_set(a[9]);ab.unbreakable=long_set(a[11]);
            w.nodes=Some(Nodes::new(t.width(),ab.clone()));w.changes=Changes::default();w.course=Some(Course::new(ab.safe_fall));w.abilities=Some(ab);"-".into()
        }
        "abilities"=>{
            let ab=w.abilities.as_ref().unwrap();let q=ab.with_motion(MotionTraits::new(f32::from_bits(a[1].parse().unwrap()),f32::from_bits(a[2].parse().unwrap()),f32::from_bits(a[3].parse().unwrap()),if a[4]=="true"{Mechanics::V1_8}else{Mechanics::Modern}));
            format!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",q.can_break,q.can_place,q.placeable_blocks,q.can_parkour,q.risky_jumps,q.can_sprint,q.safe_fall,q.water_fall,q.air,q.jump_climbs,q.bridge_ticks.to_bits(),q.health.to_bits(),q.motion.walking_speed.to_bits(),q.motion.jump_strength.to_bits(),q.motion.jump_boost_power.to_bits(),q.motion.jump_power.to_bits(),q.motion.jump_height.to_bits(),Arc::ptr_eq(&q.avoid,&ab.avoid),Arc::ptr_eq(&q.unbreakable,&ab.unbreakable),Arc::ptr_eq(&q.reserved,&ab.reserved),Arc::ptr_eq(&q.break_ticks,&ab.break_ticks),ab.motion.walking_speed.to_bits())
        }
        "ticks"=>{t.level().registry.borrow_mut().walk_ticks.insert(a[1].parse().unwrap(),d(a[2]));"-".into()}
        "node"=>{let n=w.nodes.as_ref().unwrap();let q=p(a[1]);[
            n.is_node(t,q).to_string(),n.is_underwater(t,q).to_string(),n.stands(t,q).to_string(),n.swims(t,q).to_string(),n.climbs(t,q).to_string(),n.crouches(t,q).to_string(),n.height(t,q).to_bits().to_string(),vector(n.feet(t,q)),n.floor_top(t,q).to_bits().to_string(),n.is_harmless(t,q).to_string(),n.is_solid(t,q).to_string(),n.is_placeable(t,q).to_string(),n.is_dry(t,q).to_string(),n.water.is_low(t,q).to_string(),n.water.is_submerged(t,q).to_string()].join("|")}
        "landing"=>{let q=p(a[1]);landing_raw(w.nodes.as_ref().unwrap().landing(t,q.x,q.y,q.z,d(a[2])))}
        "drop"=>w.nodes.as_ref().unwrap().is_near_drop(t,p(a[1]),d(a[2])).to_string(),
        "floorBlock"=>{let q=p(a[1]);format!("Block{{minecraft:{}}}",w.nodes.as_ref().unwrap().floor_block(t,q.x,q.z,d(a[2])).def().block)}
        "cost"=>{let s=t.level().state_id(a[1].parse().unwrap());format!("{}|{}|{}|{}|{}",Costs::floor_factor(&s).to_bits(),Costs::bounce(&s,a[2].parse().unwrap()).to_bits(),Costs::fall(a[2].parse().unwrap()).to_bits(),Costs::jump().to_bits(),Water::moves_underwater(a[3].parse().unwrap()))}
        "from"=>{w.changes.from=p(a[1]);"-".into()}
        "break"=>{let q=p(a[1]);let n=w.nodes.as_ref().unwrap();let ab=w.abilities.as_ref().unwrap();let result=if a[2]=="pass"{w.changes.pass_or_break(t,n,ab,q).to_string()}else{w.changes.break_ticks(t,n,ab,q,a[2].parse().unwrap()).to_bits().to_string()};format!("{}|{}|{}|{}",result,w.changes.break_cost.to_bits(),w.changes.breaks.iter().copied().map(position).collect::<Vec<_>>().join(";"),w.changes.places.iter().copied().map(position).collect::<Vec<_>>().join(";"))}
        "placeAgainst"=>w.changes.can_place_against(t,w.nodes.as_ref().unwrap(),p(a[1]),a[2].parse().unwrap()).to_string(),
        "clearChanges"=>{w.changes.clear();"-".into()}
        "course"=>{let c=w.course.as_mut().unwrap();c.set(t,v(a[1]),p(a[2]),d(a[3]));course_raw(c)}
        "aim"=>{let c=w.course.as_mut().unwrap();c.aim_beside(d(a[1]));course_raw(c)}
        "behind"=>vector(w.course.as_ref().unwrap().behind(d(a[1]))),
        "way"=>w.course.as_ref().unwrap().is_way_clear(t,v(a[1])).to_string(),
        "runUp"=>w.course.as_ref().unwrap().run_up_length(t,d(a[1])).to_bits().to_string(),
        "canStand"=>w.course.as_ref().unwrap().can_stand_at(t,v(a[1])).to_string(),
        "edge"=>w.course.as_ref().unwrap().edge_length(t).to_bits().to_string(),
        "safe"=>w.course.as_ref().unwrap().is_safe(t).to_string(),
        "over"=>{let q=v(a[1]);let c=w.course.as_ref().unwrap();format!("{}|{}|{}",c.is_over(q.x,q.z),c.is_in_target(q.x,q.y,q.z),c.margin(q.x,q.z).to_bits())}
        "overshoots"=>w.course.as_ref().unwrap().overshoots(d(a[1]),d(a[2]),d(a[3]),d(a[4])).to_string(),
        "estimate"=>w.course.as_ref().unwrap().estimate(CourseMotion{position:v(a[1]),delta:v(a[2]),is_sprinting:a[3].parse().unwrap()}).to_bits().to_string(),
        _=>panic!("{}",a[0]),
    }
}
