use crate::terrain_speed_order::{self,EntityTicks,Order};
use serde_json::Value;
use std::collections::{HashMap,HashSet};
struct Entities {
    order:Order,members:HashSet<u32>,removed:HashSet<u32>,parents:HashMap<u32,u32>,
    actions:HashMap<u32,Value>,trace:Vec<u32>,history:Vec<String>,counts:Vec<u32>,velocity:Vec<f64>,
}
impl EntityTicks for Entities {
    type Error=std::convert::Infallible;
    fn roots(&self)->Vec<u32>{self.order.roots(0)}
    fn removed(&self,id:u32)->bool{self.removed.contains(&id)}
    fn vehicle(&self,id:u32)->Option<u32>{self.parents.get(&id).copied()}
    fn ticking(&self,id:u32)->bool{self.members.contains(&id)}
    fn player(&self,id:u32)->bool{id==0}
    fn passengers(&self,id:u32)->Vec<u32>{self.order.passengers(id)}
    fn detach(&mut self,id:u32){self.parents.remove(&id);}
    fn tick(&mut self,id:u32,passenger:bool)->Result<(),Self::Error>{
        if passenger{self.velocity[id as usize]=0.0;}
        self.counts[id as usize]+=1;
        self.trace.push(id);self.history.push(format!("{id}:{}:{}",self.counts[id as usize],self.velocity[id as usize].to_bits()));
        if let Some(a)=self.actions.remove(&id){
            let target=a["target"].as_u64().unwrap()as u32;
            match a["kind"].as_str().unwrap(){
                "remove"=>{self.removed.insert(target);self.members.remove(&target);},
                "detach"=>{self.parents.remove(&target);},
                "add"=>{self.members.insert(target);self.order.added(target);},
                _=>unreachable!(),
            }
        }
        if passenger&&let Some(parent)=self.parents.get(&id)
            &&self.order.passengers(*parent).contains(&id){self.history.push(format!("seat:{parent}:{id}"));}
        Ok(())
    }
}
#[test]
fn actual_mc_client_level_walk_matches_insertion_passengers_frozen_and_live_mutations(){
    let oracle:Value=serde_json::from_str(include_str!("scheduler/oracle.json")).unwrap();
    for (case,row) in oracle.as_array().unwrap().iter().enumerate(){
        let input=&row["input"];let count=input["count"].as_u64().unwrap()as usize;
        let mut e=Entities{order:Order::default(),members:HashSet::new(),removed:HashSet::new(),parents:HashMap::new(),actions:HashMap::new(),trace:Vec::new(),history:Vec::new(),counts:vec![0;count],velocity:(0..count).map(|n|n as f64+0.25).collect()};
        for id in input["insertion"].as_array().unwrap(){let id=id.as_u64().unwrap()as u32;e.members.insert(id);if id==0{e.order.player_added();}else{e.order.added(id);}}
        for edge in input["passengers"].as_array().unwrap(){let edge=edge.as_array().unwrap();let vehicle=edge[0].as_u64().unwrap()as u32;let children:Vec<i32>=edge[1..].iter().map(|c|c.as_i64().unwrap()as i32).collect();e.order.set_passengers(vehicle,&children);for child in children{e.parents.insert(child as u32,vehicle);}}
        for id in input["removed"].as_array().unwrap(){e.removed.insert(id.as_u64().unwrap()as u32);}
        for action in input["actions"].as_array().unwrap(){e.actions.insert(action["turn"].as_u64().unwrap()as u32,action.clone());}
        terrain_speed_order::tick_entities(&mut e,!input["frozen"].as_bool().unwrap()).unwrap();
        assert_eq!(serde_json::json!(e.trace),row["trace"],"MC scheduler trace case {case}: {input}");
        assert_eq!(serde_json::json!(e.history),row["history"],"MC rideTick ZERO/commonTick/seat case {case}: {input}");
    }
    assert_eq!(oracle.as_array().unwrap().len(),305);
}
#[test]
fn actual_entity_push_preserves_individual_impulses_and_signed_zero(){
    let oracle:Value=serde_json::from_str(include_str!("scheduler/pushes.json")).unwrap();
    for row in oracle.as_array().unwrap(){
        let axis=|i:usize|f64::from_bits(row["offset"][i].as_str().unwrap().parse().unwrap());
        let impulse=crate::push_impulse(crate::Vec3::ZERO,crate::Vec3::new(axis(0),axis(1),axis(2)));
        let initial=crate::Vec3::new(0.3,-0.0,-0.4);
        for (first,key) in [(true,"first"),(false,"second")]{
            let result=impulse.map_or(initial,|v|initial.add_vector(if first{crate::Vec3::new(-v.x,0.0,-v.z)}else{v}));
            assert_eq!(serde_json::json!([result.x,result.y,result.z].map(|v|v.to_bits().to_string())),row[key],"original Entity.push {row}");
        }
    }
    assert_eq!(oracle.as_array().unwrap().len(),144);
}
