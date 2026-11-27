use crate::target_lock::*;
use serde_json::{Value,json};
use std::collections::BTreeMap;
#[derive(Default)]struct Fixture{state:TargetLock,now:i64,dead:bool,players:BTreeMap<i32,Option<TrackedPlayer>>,trace:Vec<String>,notice_time:Option<i32>,notice_clock:Option<i64>}
impl Host for Fixture{
 fn with_target_lock<R>(&mut self,operation:impl FnOnce(&mut TargetLock)->R)->R{operation(&mut self.state)}
 fn now_millis(&mut self)->i64{self.trace.push("clock".into());self.now}
 fn dead_or_dying(&mut self)->bool{self.dead}
 fn player_by_id(&mut self,id:i32)->Option<TrackedPlayer>{self.trace.push(format!("entity:{id}"));self.players.get(&id).cloned().flatten()}
 fn closer_than(&mut self,entity:&TrackedPlayer,range:f64)->bool{self.trace.push(format!("range:{}",range.to_bits()));entity.distance_squared<range*range}
 fn notify(&mut self,notification:Notification){let message=match notification{Notification::LockedOn{name,seconds}=>format!("lockedOn:{name}:{seconds}"),Notification::OutOfRange{name}=>format!("outOfRange:{name}"),Notification::TimeUp{name}=>format!("timeUp:{name}")};self.trace.push(format!("notify:{message}:{}",self.state.locks.len()));if let Some(time)=self.notice_time{self.state.settings.maximum_time=time;}if let Some(clock)=self.notice_clock{self.now=clock;}}
}
#[test]fn short_original_source_sequence(){
 let inputs:Value=serde_json::from_str(include_str!("inputs.json")).unwrap();let expected:Value=serde_json::from_str(include_str!("oracle.json")).unwrap();let mut output=Vec::new();
 for row in inputs.as_array().unwrap(){let mut f=Fixture::default();let mut running=true;let mut steps=Vec::new();
  for op in row.as_array().unwrap(){f.trace.clear();let mut decision=None;let mut failed=false;
   let value=&op["value"];let id=op["id"].as_i64().unwrap_or_default() as i32;
   match op["op"].as_str().unwrap(){
    "running"=>running=value.as_bool().unwrap(),"now"=>f.now=value.as_str().unwrap().parse().unwrap(),"dead"=>f.dead=value.as_bool().unwrap(),
    "notice"=>{f.notice_time=op["time"].as_i64().map(|v|v as i32);f.notice_clock=op["clock"].as_str().map(|v|v.parse().unwrap());},
    "player"=>{f.players.insert(id,if op["client"]==false{None}else{Some(TrackedPlayer{player:Player{id,name:op["name"].as_str().unwrap().into()},removed:op["removed"].as_bool().unwrap_or(false),distance_squared:f64::from_bits(op["distance"].as_str().unwrap().parse().unwrap())})});},
    "remove"=>{f.players.remove(&id);},"mode"=>f.state.settings.mode=Mode::parse(value.as_str().unwrap()).unwrap(),
    "combat"=>f.state.settings.combat_only=value.as_bool().unwrap(),"time"=>f.state.settings.maximum_time=value.as_i64().unwrap() as i32,
    "range"=>f.state.settings.maximum_range=f32::from_bits(op["bits"].as_str().unwrap().parse().unwrap()),
    "empty"=>f.state.settings.when_no_lock=WhenNoLock::parse(value.as_str().unwrap()).unwrap(),"filter"=>f.state.settings.filter_type=FilterType::parse(value.as_str().unwrap()).unwrap(),
    "names"=>failed=crate::config::apply_names(&mut f.state,value.clone()),
    "attack"=>{let player=f.players.get(&id).cloned().flatten();TargetLock::attack(running,player.as_ref().map(|p|&p.player),&mut f);},
    "tick"=>TargetLock::tick(running,&mut f),
    "tag"=>{let player=f.players.get(&id).and_then(|p|p.as_ref());decision=Some(format!("{:?}",f.state.tag(running,player.map(|p|&p.player))));},
    other=>panic!("unknown {other}"),
   }
   let locks:Vec<_>=f.state.locks.iter().map(|(id,time)|json!([id,time.to_string()])).collect();
   let step=json!({"locks":locks,"trace":f.trace,"failed":failed,"names":f.state.settings.usernames,"decision":decision});steps.push(step);
  }output.push(steps);
 }
 assert_eq!(json!(output),expected);
}
#[test]fn config_roundtrip_and_tag_cache(){
 use crate::{config::*,targeting::*};
 let mut state=TargetLock::default();state.locks.push((9,4));state.settings.mode=Mode::Filter;state.settings.usernames=vec![None,Some("İ".into())];
 for format in [JsonFormat::File,JsonFormat::Interop]{let common=CommonSettings{enabled:true,hidden:false,key_bind:json!({})};let tree=serialize_target_lock(&state,&common,format,&str::to_owned).unwrap();let mut target=TargetLock::default();let parsed=parse_tree(&tree);assert!(parsed.issues.is_empty(),"{:?}",parsed.issues);for value in parsed.changes{let SettingChange::TargetLock(value)=value;value.apply(&mut target);}assert_eq!(state.settings,target.settings);assert_eq!(state.locks,vec![(9,4)]);if format==JsonFormat::Interop{assert_eq!(tree["value"][2]["choices"]["Filter"]["value"][0]["innerValueType"],"TEXT");}}
 let mut info=TargetInfo::default();info.is_friend=true;let player=Player{id:2,name:"Other".into()};crate::target_lock_runtime::apply_tag(&state,true,Some(&player),&mut info);assert_eq!(info.classification,Classification::Ignored);assert!(info.is_friend);
 let mut cache=TagCache::default();let first=cache.get(2,||info);state.settings.combat_only=true;let second=cache.get(2,TargetInfo::default);assert_eq!(first,second);
}
