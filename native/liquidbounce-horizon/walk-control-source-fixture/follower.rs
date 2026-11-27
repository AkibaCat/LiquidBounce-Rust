use crate::navigation::{walk_control::follower::{Follower,FollowerHost},progress::FollowState};
impl WalkBody for ActionBody<'_>{type Player=Player;type LiveLevel=LiveWorld;type Simulation=Sim;
 fn player(&self)->&Player{self.inner.player}fn live_level(&self)->LiveWorld{LiveWorld(self.inner.player.level.clone())}fn exact_simulation(&self)->bool{Body{player:self.inner.player,script:self.script}.exact_simulation()}fn input_y_rot(&self)->f32{Body{player:self.inner.player,script:self.script}.input_y_rot()}fn movement_y_rot(&self)->f32{Body{player:self.inner.player,script:self.script}.movement_y_rot()}fn simulate(&self,input:SimulatedInput,yaw:f32)->Sim{Body{player:self.inner.player,script:self.script}.simulate(input,yaw)}
}
impl FollowerHost for ActionBody<'_>{
 fn places_bridge_floors(&self)->bool{self.inner.player.event("o");self.meta.external_floors}
 fn bridging(&self)->Bridging{self.inner.player.event("o");self.method}fn risky_bridges(&self)->bool{self.inner.player.event("o");self.risky}fn sneaking_speed(&self)->f64{self.inner.player.event("n");self.sneak_speed}
 fn step_reached(&mut self,index:usize,to:BlockPos){self.inner.player.event(&format!("reached:{index}:{}",position(to)))}
 fn fell_off_bridge(&mut self,to:BlockPos){self.inner.player.event(&format!("fell:{}",position(to)))}
 fn interpolated_position(&self,partial:f32)->Vec3{self.inner.player.event(&format!("head:{}",partial.to_bits()));self.inner.player.facts.pos}
 fn crossing_started(&self,length:usize,method:Bridging){self.inner.player.event(&format!("cross-start:{length}:{}",bridge_name(method)))}
 fn crossing_fallback(&self,method:Bridging,fallback:Option<Bridging>){self.inner.player.event(&format!("cross-fallback:{}:{}",bridge_name(method),fallback.map_or("the edge",bridge_name)))}
 fn crossing_hopped(&self,method:Option<Bridging>,placed:i32){self.inner.player.event(&format!("cross-hop:{}:{placed}",method.map_or("null",bridge_name)))}
}
fn positions(v:&[BlockPos])->String{v.iter().map(|&p|position(p)).collect::<Vec<_>>().join(";")}
fn follower_snapshot(follower:&Follower<S>)->String{let m=follower.movement();format!("{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}",follower.index(),follower.is_finished(),follower.is_moving(),follower.building(),m.target.map_or_else(||"-".into(),vector),m.sprint,m.dive,m.is_plain_walk,m.safe_walk,follower.view().map_or_else(||"-".into(),rotation_raw),follower.stuck_at.map_or_else(||"-".into(),position),follower.bridge_y_rot().map_or_else(||"-".into(),|r|r.to_bits().to_string()),follower.bridge_target().map_or_else(||"-".into(),vector),positions(&follower.remaining_breaks),positions(&follower.remaining_places),follower.overshoots(),follower.ahead().iter().map(|&p|vector(p)).collect::<Vec<_>>().join(";"))}
fn follower_extra(a:&[&str],w:&mut WalkFixture)->String{
 let mut host=ActionBody{inner:TargetBody{player:&w.player,targets:&w.targets,sent:w.bridge.sent},meta:&mut w.action_meta,script:&w.script,method:w.bridge.method,risky:w.bridge.risky,sneak_speed:w.bridge.sneak_speed};
 let result=match a[0]{
 "fnew"=>{host.meta.external_floors=false;w.follower=Follower::default();"-".into()},
 "ffollow"=>{let route=w.follower_path.get_or_insert_with(||Arc::new(w.route.as_ref().unwrap().clone())).clone();w.follower.follow(route,a[1].parse().unwrap());"-".into()},
 "fclear"=>{w.follower.clear();"-".into()},"fcorrect"=>{w.follower.position_corrected();"-".into()},"fexternal"=>{host.meta.external_floors=a[1].parse().unwrap();"-".into()},
 "fwalk"=>{let mut keys=mask_keys(a[2]);let state=w.follower.tick(&mut host,&mut w.block_actions,&mut keys,f(a[1]));format!("{}:{}",match state{FollowState::Walking=>"WALKING",FollowState::Acting=>"ACTING",FollowState::Finished=>"FINISHED",FollowState::Stuck=>"STUCK"},keys.to_input(false).java_hash())},
 "foff"=>w.follower.is_off_path(&host).to_string(),"fnext"=>w.follower.next_block(&host).map_or_else(||"-".into(),position),
 "fruns"=>format!("{}:{}",w.follower.bridge_ahead(),w.follower.swim_ahead().iter().map(|s|position(s.to)).collect::<Vec<_>>().join(";")),
 "fpoint"=>w.follower.point_ahead(v(a[1]),d(a[2])).map_or_else(||"-".into(),vector),"flook"=>w.follower.look_ahead(v(a[1]),v(a[2]),d(a[3])).map_or_else(||"-".into(),vector),"fturn"=>w.follower.turn_ahead(&host,d(a[1])).to_bits().to_string(),"fhead"=>vector(w.follower.head(&host,f(a[1]))),
 _=>panic!("follower command {a:?}")
 };
 result+"@"+&follower_snapshot(&w.follower)+"@"+&action_snapshot(&w.block_actions,&mut w.action_meta)
}
