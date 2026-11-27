#![allow(dead_code)]
#[path="../../liquidbounce-behaviors/src/liquid_filler.rs"] mod model;
#[path="../../liquidbounce-behaviors/src/block_placer.rs"] mod placer;
use model::*;
use std::collections::{HashMap,HashSet,VecDeque};
#[derive(Default)]
struct Fixture{scan:Vec<(Pos,State)>,states:HashMap<Pos,State>,blocked:HashSet<Pos>,random:Vec<usize>}
impl World for Fixture{
    fn scan(&mut self,_:[f64;3],_:f32)->Vec<(Pos,State)>{self.scan.clone()}
    fn state(&mut self,p:Pos)->Option<State>{Some(self.states.get(&p).copied().unwrap_or_default())}
    fn unobstructed(&mut self,p:Pos)->bool{!self.blocked.contains(&p)}
    fn next_int(&mut self,bound:usize)->usize{let value=self.random.remove(0);assert!(value<bound);value}
}
fn water()->State{State{replaceable:true,fluid:Fluid::Water,source:true,absorb_pass:true}}
fn air()->State{State{replaceable:true,..Default::default()}}
fn slot_at(index:u8,item:&str,block:Option<&str>)->Slot{Slot{index,item:format!("minecraft:{item}"),block:block.map(|s|format!("minecraft:{s}"))}}
fn traversal()->Vec<Pos>{
    let mut queue=VecDeque::from([([0,0,0],0)]);let mut seen=HashSet::new();let mut out=vec![];
    while let Some((p,depth))=queue.pop_front(){if !seen.insert(p){continue;}out.push(p);if depth<6{for direction in DIRECTIONS{queue.push_back((std::array::from_fn(|i|p[i]+direction[i]),depth+1));}}}
    out
}
#[test]
fn liquid_filler_and_block_placer_source_smoke(){
    let mut s=Settings::default();assert_eq!(s.blocks.len(),23);
    let slots=vec![slot_at(40,"sponge",Some("sponge")),slot_at(2,"stone",Some("stone")),slot_at(6,"dirt",Some("dirt")),slot_at(3,"stick",None)];
    assert_eq!(normal_slot(&s,&slots,6),Some(6));
    assert_eq!(normal_slot(&s,&slots,4),Some(2)); // Stable tie keeps the earlier slot.
    s.use_sponge=true;assert_eq!(slot(&s,&slots,6,None),Some(40));
    assert_eq!(slot(&s,&slots,6,Some(water())),Some(40));
    assert_eq!(slot(&s,&slots,6,Some(air())),Some(6)); // Air sponge targets use the normal filler.
    assert_eq!(slot(&s,&slots,6,Some(State{fluid:Fluid::Lava,..water()})),Some(6));
    assert!(should_fill(&s,water(),false,true));assert!(!should_fill(&s,water(),true,false));
    assert!(!should_fill(&s,State{source:false,..water()},true,true));
    assert!(!should_fill(&s,State{replaceable:false,..water()},true,true));
    s.whitelist=false;assert_eq!(normal_slot(&s,&slots,6),Some(40));s.whitelist=true;s.use_sponge=false;
    let source=[([1,0,0],water()),([-1,0,0],water()),([0,2,0],water()),([0,-2,0],water())];
    let mut w=Fixture{scan:source.to_vec(),..Default::default()};
    s.order=PlaceOrder::CloserFirst;assert_eq!(targets(&mut w,&s,[0.5;3],6.,true,false),vec![[1,0,0],[-1,0,0],[0,2,0],[0,-2,0]]);
    s.order=PlaceOrder::FurtherFirst;assert_eq!(targets(&mut w,&s,[0.5;3],6.,true,false),vec![[0,2,0],[0,-2,0],[1,0,0],[-1,0,0]]);
    s.order=PlaceOrder::BottomTop;assert_eq!(targets(&mut w,&s,[0.5;3],6.,true,false),vec![[0,-2,0],[1,0,0],[-1,0,0],[0,2,0]]);
    s.order=PlaceOrder::TopBottom;assert_eq!(targets(&mut w,&s,[0.5;3],6.,true,false),vec![[0,2,0],[1,0,0],[-1,0,0],[0,-2,0]]);
    s.order=PlaceOrder::Random;w.random=vec![0,1,0];assert_eq!(targets(&mut w,&s,[0.5;3],6.,true,false),vec![[0,2,0],[0,-2,0],[-1,0,0],[1,0,0]]);
    let mut w=Fixture::default();for x in 1..=7{w.states.insert([x,0,0],water());}
    assert!(can_absorb(&mut w,[0,0,0],[6,0,0]));assert!(!can_absorb(&mut w,[0,0,0],[7,0,0]));
    w.states.insert([3,0,0],State{absorb_pass:false,..water()});assert!(!can_absorb(&mut w,[0,0,0],[4,0,0]));assert!(can_absorb(&mut w,[0,0,0],[3,0,0]));
    let order=traversal();assert_eq!(&order[1..7],&DIRECTIONS);
    let mut w=Fixture::default();for p in &order{w.states.insert(*p,water());}
    assert!(can_absorb(&mut w,[0,0,0],order[64]));assert!(!can_absorb(&mut w,[0,0,0],order[65]));
    let mut w=Fixture{scan:vec![([0,0,0],air()),([1,0,0],air())],..Default::default()};w.states.insert([0,1,0],water());w.states.insert([1,1,0],water());
    assert_eq!(sponge_placement(&mut w,[0,1,0],[1.,0.5,0.5],6.),Some([0,0,0]));w.blocked.insert([0,0,0]);assert_eq!(sponge_placement(&mut w,[0,1,0],[1.,0.5,0.5],6.),Some([1,0,0]));
    let mut q=placer::Queue::default();q.update(&[[1,0,0],[2,0,0],[1,0,0]]);q.update(&[[2,0,0],[1,0,0],[3,0,0]]);assert_eq!(q.blocks,vec![[1,0,0],[2,0,0],[3,0,0]]);
    q.ran_action=true;q.ticks_to_wait=1;q.tick_start(||panic!("cooldown must wait for the next tick"));assert_eq!(q.ticks_to_wait,0);assert!(q.ran_action);q.tick_start(||2);assert_eq!(q.ticks_to_wait,2);assert!(!q.ran_action);
    q.ran_action=true;q.sneak_times=2;assert!(q.movement());q.reset();assert_eq!(q.ticks_to_wait,2);assert!(q.ran_action);assert_eq!(q.sneak_times,0);
    let mut e=placer::PostRotationExecutor::default();e.add(1,false,true);e.add(2,true,false);e.add(3,false,false);assert!(e.after_move(false).is_empty());assert_eq!(e.first_tick(),vec![1,2,3]);
    e.add(4,true,true);e.add(5,true,false);e.add(6,false,false);assert_eq!(e.first_tick(),vec![5,6]);assert_eq!(e.after_move(true),vec![4]);e.add(7,true,false);assert!(e.after_move(true).is_empty());assert_eq!(e.first_tick(),vec![7]);
    let mut c=placer::CrystalDestroyer::default();c.suggest(placer::Crystal{id:1,distance_squared:9.},true);c.suggest(placer::Crystal{id:2,distance_squared:4.},false);assert_eq!(c.target.unwrap().id,1);c.suggest(placer::Crystal{id:3,distance_squared:9.},true);assert_eq!(c.target.unwrap().id,1);c.suggest(placer::Crystal{id:4,distance_squared:1.},true);c.removed(1);assert_eq!(c.target.unwrap().id,4);c.attacked(100);assert!(!c.elapsed(105,6));assert!(c.elapsed(106,6));
    println!("LiquidFiller/BlockPlacer: source fluids, filter/slot, five orders, sponge traversal limits, queue cooldown, executor phases and crystal selection passed");
}
