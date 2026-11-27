use crate::{random::ThreadRandom, vomit::*};
use serde_json::{Value, json};
#[test]
fn unchanged_original_module_all_modes_match_plan_order_and_independent_rng_streams() {
    let reference:Value=serde_json::from_str(include_str!("source-vectors.json")).unwrap();
    let mut module=Vomit { block_random:LegacyRandom::seeded(reference["blockSeed"].as_str().unwrap().parse().unwrap()),..Default::default() };
    let mut rng=ThreadRandom::seeded(reference["threadSeed"].as_str().unwrap().parse().unwrap(),reference["thread"].as_u64().unwrap());
    for (index,(line,expected)) in include_str!("commands.txt").lines().zip(reference["rows"].as_array().unwrap()).enumerate() {
        let p=line.split(' ').collect::<Vec<_>>();
        module.constraints.click_delay=[p[1].parse().unwrap(),p[2].parse().unwrap()];
        let mask:u64=p[3].parse().unwrap();
        let mut nonempty=[false;36];for (i,value) in nonempty.iter_mut().enumerate() { *value=mask&(1<<i)!=0; }
        let snapshot=Snapshot { creative:p[0]=="true",player_nonempty:nonempty,container:if p[4]=="-1" { None }else{Some(Container { sync_id:p[4].parse().unwrap(),slot_count:p[5].parse().unwrap(),nonempty_slots:if p[6]=="-" { vec![] }else{p[6].split(',').map(|v|v.parse().unwrap()).collect()} })},block_registry_size:p[7].parse().unwrap() };
        let result=module.plan(&snapshot,&mut rng);
        assert_eq!(result.is_err(),expected["error"],"row {index}: {line}");
        let actions=result.ok().flatten().map(|p|p.actions).unwrap_or_default().into_iter().map(|action|match action {
            Action::CreativeThrow { block_index,count }=>json!([0,block_index,count]),
            Action::CreativeFill { block_index,count,slot }=>{let (kind,index)=slot_value(slot);json!([1,block_index,count,kind,index])},
            Action::Throw { screen,slot }=>{let (kind,index)=slot_value(slot);let (sync,count)=screen.unwrap_or((-1,-1));json!([2,sync,count,kind,index])},
        }).collect::<Vec<_>>();
        assert_eq!(json!(actions),expected["actions"],"row {index}: {line}");
        assert_eq!(module.block_random.seed().to_string(),expected["blockSeed"],"block RNG row {index}");
        let mut copy=rng;let next=copy.next_i32();
        // Exact ThreadRandom state is observed without changing production visibility.
        let seed: u64=expected["threadSeed"].as_str().unwrap().parse().unwrap();
        let mut expected_rng=ThreadRandom::seeded(seed,reference["thread"].as_u64().unwrap());
        assert_eq!(next,expected_rng.next_i32(),"Kotlin RNG row {index}: {line}");
    }
}
fn slot_value(slot:Slot)->(i32,i32) { match slot { Slot::Player(index)=>(0,i32::from(index)),Slot::Container(index)=>(1,index) } }
