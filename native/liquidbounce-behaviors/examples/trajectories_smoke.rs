// SPDX-License-Identifier: GPL-3.0-or-later
use serde_json::json;
use liquidbounce_behaviors::trajectories::*;
struct Empty;
impl World for Empty{
    fn min_y(&self)->f64{-64.}
    fn nonempty_fluid(&self,_:V3)->bool{false}
    fn block_hit(&self,_:V3,_:V3,_:u32)->Option<Hit>{None}
    fn entity_hit(&self,_:V3,_:V3,_:u32,_:f64)->Option<Hit>{None}
}
struct Wall;
impl World for Wall{
    fn min_y(&self)->f64{-64.}
    fn nonempty_fluid(&self,_:V3)->bool{true}
    fn block_hit(&self,_:V3,_:V3,_:u32)->Option<Hit>{Some(Hit::Block{position:[1.,2.,3.],block:[1,1,3],normal:[0,1,0]})}
    fn entity_hit(&self,_:V3,_:V3,_:u32,_:f64)->Option<Hit>{panic!("block must have precedence over nearer entities")}
}
fn main(){
    let bow=json!({"id":"minecraft:bow"});assert!(held_shots(&bow,None,false,true).is_empty());assert!(held_shots(&bow,Some(1),false,true).is_empty());let full=held_shots(&bow,Some(20),false,true);assert_eq!(full.len(),1);assert_eq!(full[0].info.initial_velocity,3.);
    let crossbow=json!({"id":"minecraft:crossbow","multishot":1,"charged":[{"id":"minecraft:arrow"},{"id":"minecraft:firework_rocket"},{"id":"minecraft:arrow"}]});let shots=held_shots(&crossbow,None,false,true);assert_eq!(shots.iter().map(|s|s.yaw_offset).collect::<Vec<_>>(),[0.,-10.,10.]);assert!(shots.iter().all(|s|s.kind=="FireworkRocket"));assert!(!shots[0].info.copies_player_velocity);assert_eq!(held_shots(&crossbow,None,false,false).len(),1);
    let persistent=Info::for_type("Arrow",false);let snow=Info::for_type("Snowball",false);assert_eq!(persistent.radius,0.5);assert_eq!(Info::for_type("Arrow",true).radius,0.3);
    let a=simulate(&Empty,[0.,10.,0.],[1.,0.,0.],persistent,"Arrow",1,2);assert_eq!(a.positions.len(),2);assert_eq!(a.positions[0],[1.,10.,0.]);assert_eq!(a.positions[1],[1.99,9.95,0.]);
    let b=simulate(&Empty,[0.,10.,0.],[1.,0.,0.],snow,"Snowball",1,2);assert_eq!(b.positions.len(),1);assert_eq!(b.positions[0],[0.99,9.97,0.]);
    let hit=simulate(&Wall,[0.,10.,0.],[1.,0.,0.],snow,"Snowball",1,240);assert!(matches!(hit.hit,Some(Hit::Block{normal:[0,1,0],..})));assert_eq!(hit.positions,[ [1.,2.,3.] ]);
    let(pos,velocity,offset)=hypothetical([1.,2.,3.],1.62,[0.1,0.2,0.3],false,[0.,0.],[1.,2.,3.],persistent,false);assert!((pos[1]-3.519999998509884).abs()<1e-12);assert_eq!(velocity,[0.1,0.2,3.3]);assert_eq!(offset,[-0.16,0.,0.]);let(_,frozen,_)=hypothetical([1.,2.,3.],1.62,[0.1,0.2,0.3],false,[0.,0.],[1.,2.,3.],persistent,true);assert_eq!(frozen,[-0.,-0.,3.]);
    assert_eq!(clip_aabb([0.,0.,0.],[10.,0.,0.],[[4.,-1.,-1.],[5.,1.,1.]],0.),Some([4.,0.,0.]));
    let mut settings=Trajectories::default();settings.values.insert("ShowDetailedInfo.Enabled".into(),json!(true));settings.values.insert("ShowDetailedInfo.ShowAt".into(),json!("Owner"));settings.values.insert("ShowDetailedInfo.TimeUnit".into(),json!("Seconds"));
    let label=detail(&settings,&hit,1,[0.;3],1,[0.;3],false,None).unwrap();assert_eq!(label.position,[1.,2.,3.]);assert!(!label.crosshair);assert_eq!(label.lines,["0.1s","Dist: 3.7m"]);assert_eq!(label.visible,[true,true]);
    assert!(detail(&settings,&hit,1,[0.;3],1,[0.;3],true,None).unwrap().crosshair);let remote=detail(&settings,&hit,2,[4.,5.,6.],1,[0.;3],false,Some("Alex\n")).unwrap();assert_eq!(remote.position,[4.,5.,6.]);assert_eq!(remote.lines.last().unwrap(),"Owner: Alex");
    settings.values.insert("CullBehindPlayer".into(),json!(true));assert!(settings.culled([0.,0.,-10.],[0.;3],[0.,0.,1.]));assert!(!settings.culled([0.,0.,-1.],[0.;3],[0.,0.,1.]));
    settings.simulation_results.push(snapshot(&hit,1,"Snowball",false));settings.reset();assert!(settings.simulation_results.is_empty());
    println!("Trajectories smoke passed: held shots, native descriptors, collision precedence, physics and detailed indicators");
}
