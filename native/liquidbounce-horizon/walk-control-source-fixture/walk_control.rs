use crate::navigation::{input::MovementKeys,simulation::{Mechanics,block_grid::SimulationState,motion_traits::MotionTraits,player_motion::MotionPlayer},walk::{WalkState,abilities::WalkAbilities,route::{Move,Route,Step}},walk_jump::jump::Jump,walk_control::{self as ctrl,body::{ControlState,WalkPlayer},path_points::{self,RouteControl,StepControl},step_progress::{StepProgress,Movement,BridgeLine},setbacks::Setbacks,stuck_spots::StuckSpots,stride::Stride,overshoot::Overshoot,swimming::Swimming,hops::Hops,step_check,prediction,bridge::Bridge}};
use std::sync::Arc;
impl WalkState for S{
 fn is_air(&self)->bool{self.air()}fn can_be_replaced(&self)->bool{self.registry.borrow().walk_facts[&self.def().block].3}
 fn is_falling(&self)->bool{false}fn is_slime(&self)->bool{false}
 fn friction(&self)->f32{self.registry.borrow().walk_facts[&self.def().block].0}fn speed_factor(&self)->f32{1.0}fn jump_factor(&self)->f32{1.0}
}
impl SimulationState for S{
 fn is_fence(&self)->bool{false}fn is_wall(&self)->bool{false}fn is_fence_gate(&self)->bool{self.facts().0&4!=0}fn is_cobweb(&self)->bool{self.def().block=="cobweb"}fn is_step_air(&self)->bool{self.air()}fn fall_distance_reduction(&self)->f32{0.0}
}
impl ControlState for S{fn is_farmland(&self)->bool{self.def().block=="farmland"}fn is_bottom_stair(&self)->bool{self.def().block=="oak_stairs"&&self.def().upper==0}}
#[derive(Clone)]struct Facts{tick:i32,pos:Vec3,delta:Vec3,speed:f64,strength:f64,boost:f32,flags:i32,food:i32,air:i32,max_air:i32,eye:Vec3,support:Option<BlockPos>,below:BlockPos,delay:i32,yaw:f32}
impl Default for Facts{fn default()->Self{Self{tick:0,pos:Vec3::ZERO,delta:Vec3::ZERO,speed:0.1,strength:0.42,boost:0.0,flags:1,food:20,air:300,max_air:300,eye:Vec3::ZERO,support:None,below:BlockPos::new(0,-1,0),delay:0,yaw:0.0}}}
struct Player{facts:Facts,level:World}
#[derive(Default)]struct Script{exact:bool,moving:Vec<(Vec3,Vec3,i32)>,still:Vec<(Vec3,Vec3,i32)>}
struct Sim{player:Player,frames:Vec<(Vec3,Vec3,i32)>,index:usize}
use crate::navigation::walk_control::body::{WalkBody,WalkSimulation,SimulatedInput};
fn simulation_raw(input:SimulatedInput)->String{format!("{}:{}:{}",input.keys.java_hash(),input.force_stay_on_ground_surface,input.ignores_edge_back_off)}
impl WalkSimulation for Sim{
 type Player=Player;fn player(&self)->&Player{&self.player}fn predicted_position(&self)->Vec3{self.player.event("Q");self.player.facts.pos}
 fn set_input(&mut self,input:SimulatedInput){self.player.event(&format!("I:{}",simulation_raw(input)));}
 fn tick(&mut self){self.player.event("T");if !self.frames.is_empty(){let (pos,delta,flags)=self.frames[self.index.min(self.frames.len()-1)];self.player.facts.pos=pos;self.player.facts.delta=delta;self.player.facts.flags=flags;self.index+=1;}}
 fn has_backed_off_edge(&self)->bool{self.player.facts.flags&256!=0}
}
struct Body<'a>{player:&'a Player,script:&'a Script}
use crate::navigation::walk_control::{bridge_host::{BridgeHost,PlacementIdentity},bridging::Bridging,view_rotation::ViewRotation,turning::{Turning,LookPurpose},telly::Telly,crossing::Crossing};
fn bridging(raw:&str)->Bridging{match raw{"AUTO"=>Bridging::Auto,"NINJA"=>Bridging::Ninja,"GODBRIDGE"=>Bridging::Godbridge,"MOONWALK"=>Bridging::Moonwalk,"BREEZILY"=>Bridging::Breezily,"ONE_STACK"=>Bridging::OneStack,"SWIFT_SNEAK"=>Bridging::SwiftSneak,"TELLY"=>Bridging::Telly,_=>panic!("bridge method")}}
fn bridge_name(method:Bridging)->&'static str{match method{Bridging::Auto=>"AUTO",Bridging::Ninja=>"NINJA",Bridging::Godbridge=>"GODBRIDGE",Bridging::Moonwalk=>"MOONWALK",Bridging::Breezily=>"BREEZILY",Bridging::OneStack=>"ONE_STACK",Bridging::SwiftSneak=>"SWIFT_SNEAK",Bridging::Telly=>"TELLY"}}
fn rotation_raw(rotation:ViewRotation)->String{format!("{},{}",rotation.y_rot.to_bits(),rotation.x_rot.to_bits())}
struct BridgeMeta{sent:ViewRotation,method:Bridging,risky:bool,sneak_speed:f64,placement:Option<PlacementIdentity>,next_id:u64,rejections:i32,action:Option<(BlockPos,bool,ViewRotation)>}
impl Default for BridgeMeta{fn default()->Self{Self{sent:ViewRotation::ZERO,method:Bridging::Auto,risky:false,sneak_speed:0.3,placement:None,next_id:0,rejections:0,action:None}}}
impl BridgeMeta{fn action_raw(&self)->String{self.action.map_or_else(||"-".into(),|(pos,ready,held)|format!("{}:{}:{}",position(pos),ready,rotation_raw(held)))}}
struct BridgeBody<'a>{inner:Body<'a>,meta:&'a mut BridgeMeta,actions:&'a Confirmation,stride:&'a Stride}
impl WalkBody for BridgeBody<'_>{type Player=Player;type LiveLevel=LiveWorld;type Simulation=Sim;
 fn player(&self)->&Player{self.inner.player()}fn live_level(&self)->LiveWorld{self.inner.live_level()}fn exact_simulation(&self)->bool{self.inner.exact_simulation()}fn input_y_rot(&self)->f32{self.inner.input_y_rot()}fn movement_y_rot(&self)->f32{self.inner.movement_y_rot()}fn simulate(&self,input:SimulatedInput,y:f32)->Sim{self.inner.simulate(input,y)}
}
impl BridgeHost for BridgeBody<'_>{
 fn sent_rotation(&self)->ViewRotation{self.player().event("R");self.meta.sent}
 fn look(&mut self,rotation:ViewRotation,purpose:LookPurpose){self.player().event(&format!("L:{}:{}",rotation_raw(rotation),match purpose{LookPurpose::Movement=>"MOVEMENT",LookPurpose::Aim=>"AIM",LookPurpose::View=>"VIEW",LookPurpose::Steer=>"STEER"}));}
 fn is_placed(&self,pos:BlockPos)->bool{step_check::is_placed(&self.player().level,pos)}
 fn place(&mut self,pos:BlockPos,ready:bool,held:ViewRotation){self.meta.action=Some((pos,ready,held));}
 fn placement(&self)->Option<PlacementIdentity>{self.meta.placement}fn rejections(&self)->i32{self.meta.rejections}fn action_rotation(&self)->Option<ViewRotation>{None}
 fn is_unconfirmed(&self,pos:BlockPos)->bool{self.actions.unconfirmed.contains(&pos)}
 fn bridging(&self)->Bridging{self.player().event("o");self.meta.method}fn risky_bridges(&self)->bool{self.player().event("o");self.meta.risky}fn sneaking_speed(&self)->f64{self.player().event("n");self.meta.sneak_speed}fn stride_moved(&self)->Vec3{self.stride.moved()}
 fn clicks_from(&self,_eye:Vec3,_rotation:ViewRotation,_floor:BlockPos)->bool{panic!("clicks fixture not configured")}
 fn placeable_slot(&self)->Option<i32>{panic!("placement fixture not configured")}
 fn placement_rotation(&self,_pos:BlockPos,_slot:i32,_nearest:bool)->Option<ViewRotation>{panic!("placement fixture not configured")}
 fn crossing_started(&self,length:usize,method:Bridging){self.player().event(&format!("cross-start:{length}:{}",bridge_name(method)))}
 fn crossing_fallback(&self,method:Bridging,fallback:Option<Bridging>){self.player().event(&format!("cross-fallback:{}:{}",bridge_name(method),fallback.map_or("the edge",bridge_name)))}
 fn crossing_hopped(&self,method:Option<Bridging>,placed:i32){self.player().event(&format!("cross-hop:{}:{placed}",method.map_or("null",bridge_name)))}
}
fn crossing_raw(crossing:&Crossing)->String{let(r,p,h,d,i,f)=crossing.timing();format!("{}/{}/{}/{}/{}/{}/{}/{}/{}",crossing.method.map_or("-",bridge_name),crossing.view.map_or_else(||"-".into(),rotation_raw),crossing.acting.map_or_else(||"-".into(),position),r,p,h,d,i,f)}
impl WalkBody for Body<'_>{
 type Player=Player;type LiveLevel=LiveWorld;type Simulation=Sim;
 fn player(&self)->&Player{self.player}fn live_level(&self)->LiveWorld{LiveWorld(self.player.level.clone())}
 fn exact_simulation(&self)->bool{self.player.event("X");self.script.exact}
 fn input_y_rot(&self)->f32{self.player.event("i");self.player.facts.yaw}
 fn movement_y_rot(&self)->f32{self.player.event("y");self.player.facts.yaw}
 fn simulate(&self,input:SimulatedInput,y_rot:f32)->Sim{self.player.event(&format!("sim:{}:{}",simulation_raw(input),y_rot.to_bits()));Sim{player:Player{facts:self.player.facts.clone(),level:self.player.level.clone()},frames:if input.keys.is_moving(){self.script.moving.clone()}else{self.script.still.clone()},index:0}}
}
fn frames(raw:&str)->Vec<(Vec3,Vec3,i32)>{if raw=="-"{Vec::new()}else{raw.split(';').map(|s|{let q:Vec<_>=s.split('^').collect();(v(q[0]),v(q[1]),q[2].parse().unwrap())}).collect()}}
#[derive(Clone)]struct LiveWorld(World);
impl TerrainLevel<S> for LiveWorld{
 fn block_state(&self,pos:BlockPos)->S{self.0.block_state(pos)}fn void_air(&self)->S{self.0.void_air()}
 fn is_known(&self,pos:BlockPos)->bool{if pos.y<self.min_y()||pos.y>self.max_y(){return false;}self.0.events.borrow_mut().push(format!("H:{},{}",pos.x,pos.z));true}
 fn is_inside_border(&self,x:f64,z:f64)->bool{self.0.is_inside_border(x,z)}fn min_y(&self)->i32{self.0.min_y()}fn height(&self)->i32{self.0.height()}
 fn axis_step_order(&self,movement:Vec3)->[Axis;3]{self.0.axis_step_order(movement)}
}
impl Player{fn event(&self,e:&str){self.level.events.borrow_mut().push(e.into());}}
impl MotionPlayer for Player{
 fn position(&self)->Vec3{self.event("p");self.facts.pos}fn delta_movement(&self)->Vec3{self.event("d");self.facts.delta}fn on_ground(&self)->bool{self.event("g");self.facts.flags&1!=0}
 fn is_sprinting(&self)->bool{self.event("s");self.facts.flags&2!=0}fn jump_delay(&self)->i32{self.event("J");self.facts.delay}fn is_crouching(&self)->bool{self.event("c");self.facts.flags&8!=0}fn supporting_block(&self)->Option<BlockPos>{self.event("S");self.facts.support}
}
impl WalkPlayer for Player{
 type State=S;type Level=World;
 fn y(&self)->f64{self.event("Y");self.facts.pos.y}
 fn level(&self)->&World{self.event("l");&self.level}
 fn movement_speed(&self)->f64{self.event("a");self.facts.speed}fn jump_strength(&self)->f64{self.event("j");self.facts.strength}fn jump_boost_power(&self)->f32{self.event("b");self.facts.boost}
 fn mechanics(&self)->Mechanics{self.event("l");self.event("m");if self.level.data.borrow().legacy{Mechanics::V1_8}else{Mechanics::Modern}}
 fn tick_count(&self)->i32{self.event("t");self.facts.tick}fn horizontal_collision(&self)->bool{self.event("h");self.facts.flags&4!=0}fn is_in_water(&self)->bool{self.event("w");self.facts.flags&16!=0}fn on_climbable(&self)->bool{self.event("k");self.facts.flags&32!=0}fn is_using_item(&self)->bool{self.event("u");self.facts.flags&64!=0}
 fn block_below_movement(&self)->BlockPos{self.event("B");self.facts.below}fn food_level(&self)->i32{self.event("f");self.event("F");self.facts.food}fn eye_position(&self)->Vec3{self.event("E");self.facts.eye}fn is_eye_in_water(&self)->bool{self.event("e");self.facts.flags&128!=0}fn air_supply(&self)->i32{self.event("A");self.facts.air}fn max_air_supply(&self)->i32{self.event("M");self.facts.max_air}
}
#[derive(Default)]struct Confirmation{unconfirmed:HashSet<BlockPos>}
impl step_check::ActionConfirmation for Confirmation{fn is_unconfirmed(&self,pos:BlockPos)->bool{self.unconfirmed.contains(&pos)}}
struct WalkFixture{player:Player,stride:Stride,overshoot:Overshoot,swim:Swimming,progress:StepProgress,setbacks:Setbacks,stuck:StuckSpots,hops:Hops,actions:Confirmation,steps:HashMap<i32,Step>,route:Option<Route<S>>,script:Script,bridge:BridgeMeta,telly:Telly,crossing:Crossing,targets:TargetFixture,block_actions:BlockActions<ActionOrder>,action_meta:ActionMeta,follower:Follower<S>,follower_path:Option<Arc<Route<S>>>,capture:CaptureFixture}
impl WalkFixture{
 fn new(world:World)->Self{let targets=TargetFixture::new(world.events.clone());let action_meta=ActionMeta::default();let block_actions=BlockActions::new(action_meta.order.clone());Self{player:Player{facts:Facts::default(),level:world},stride:Stride::default(),overshoot:Overshoot::default(),swim:Swimming::default(),progress:StepProgress::default(),setbacks:Setbacks::default(),stuck:StuckSpots::default(),hops:Hops::default(),actions:Confirmation::default(),steps:HashMap::new(),route:None,script:Script::default(),bridge:BridgeMeta::default(),telly:Telly::default(),crossing:Crossing::default(),targets,block_actions,action_meta,follower:Follower::default(),follower_path:None,capture:CaptureFixture::default()}}
 fn snapshot(&self)->String{let mut blocks:Vec<_>=self.stuck.blocks.iter().copied().collect();blocks.sort();format!("{}^{}^{}^{}^{}^{}^{}^{}^{}^{}^{}",self.stride.factor().to_bits(),vector(self.stride.moved()),self.overshoot.count(),self.progress.watch.idle_ticks(),self.progress.acting.map_or_else(||"-".into(),position),self.progress.jump_ticks,self.progress.is_due_to_predict(),self.hops.aim().map_or_else(||"-".into(),vector),self.swim.is_breathing(),self.stuck.version(),blocks.iter().map(ToString::to_string).collect::<Vec<_>>().join(","))}
}
fn f(raw:&str)->f32{f32::from_bits(raw.parse().unwrap())}
fn ps(raw:&str)->Vec<BlockPos>{if raw=="-"{Vec::new()}else{raw.split(';').map(p).collect()}}
fn ab()->WalkAbilities<S>{WalkAbilities::new(false,false,0,true,true,3,16,Arc::new(|_:&S|1.0),Arc::new(HashSet::new()))}
fn movement_raw(keys:MovementKeys,m:Movement)->String{format!("{}~{}~{}~{}~{}~{}",keys.to_input(false).java_hash(),m.target.map_or_else(||"-".into(),vector),m.sprint,m.dive,m.is_plain_walk,m.safe_walk)}
fn extra(a:&[&str],t:&mut Terrain<S,World>,fixture:&mut Option<WalkFixture>)->String{
 if a[0]=="cnew"||fixture.is_none(){*fixture=Some(WalkFixture::new(t.level().clone()));}
 let w=fixture.as_mut().unwrap();w.player.level=t.level().clone();let player=&w.player;
 let result=match a[0]{
 "cnew"=>"-".into(),
 "player"=>{w.player.facts=Facts{tick:a[1].parse().unwrap(),pos:v(a[2]),delta:v(a[3]),speed:d(a[4]),strength:d(a[5]),boost:f(a[6]),flags:a[7].parse().unwrap(),food:a[8].parse().unwrap(),air:a[9].parse().unwrap(),max_air:a[10].parse().unwrap(),eye:v(a[11]),support:if a[12]=="-"{None}else{Some(p(a[12]))},below:p(a[13]),delay:a[14].parse().unwrap(),yaw:f(a[15])};"-".into()},
 "step"=>{let flags:i32=a[8].parse().unwrap();let movement=match a[4]{"TRAVERSE"=>Move::Traverse,"DIAGONAL"=>Move::Diagonal,"ASCEND"=>Move::Ascend,"FALL"=>Move::Fall,"PARKOUR"=>Move::Parkour,"PILLAR"=>Move::Pillar,"DOWNWARD"=>Move::Downward,"SWIM"=>Move::Swim,"CLIMB_UP"=>Move::ClimbUp,"CLIMB_DOWN"=>Move::ClimbDown,_=>panic!("move")};w.steps.insert(a[1].parse().unwrap(),Step{from:p(a[2]),to:p(a[3]),movement,feet:v(a[5]),breaks:ps(a[6]),places:ps(a[7]),opens:ps(a[9]),cost:0.0,shift:flags&1!=0,edge:flags&2!=0,is_under_water:flags&4!=0,jump:(flags&8!=0).then(||Jump{run_up:v(a[10]),takeoff:0,jumps:true,air_ticks:1,margin:0.0,hops:0,aim:Vec3::ZERO})});"-".into()},
 "route"=>{w.follower_path=None;w.route=Some(Route::new(v(a[1]),if a[2]=="-"{Vec::new()}else{a[2].split(',').map(|i|w.steps[&i.parse::<i32>().unwrap()].clone()).collect()},true,ab()));"-".into()},
 "stride"=>{w.stride.tick(player);"-".into()},
 "abilities"=>{let chosen=ab().with_motion(MotionTraits::new(f(a[1]),f(a[2]),f(a[3]),if t.level().data.borrow().legacy{Mechanics::V1_8}else{Mechanics::Modern}));let actual=w.stride.abilities(&chosen,player).motion;format!("{},{},{},{},{}",actual.walking_speed.to_bits(),actual.jump_strength.to_bits(),actual.jump_boost_power.to_bits(),if actual.mechanics==Mechanics::V1_8{"V1_8"}else{"MODERN"},actual.jump_power.to_bits())},
 "center"=>ctrl::center_input(v(a[1]),v(a[2]),f(a[3])).java_hash().to_string(),
 "stepfacts"=>{let s=&w.steps[&a[1].parse::<i32>().unwrap()];format!("{},{},{},{},{},{},{},{}",s.builds(),s.is_level_walk(),s.is_rising_bridge(),s.is_plain(),s.past_edge(v(a[2])).to_bits(),s.hold_back(v(a[2]),v(a[3]),f(a[4])).java_hash(),path_points::has_passed(s,v(a[2])),step_check::wants_jump_at(s,v(a[2]),a[5].parse().unwrap(),a[6].parse().unwrap()))},
 "straight"=>path_points::continues_straight(&w.steps[&a[1].parse::<i32>().unwrap()],&w.steps[&a[2].parse::<i32>().unwrap()]).to_string(),
 "run"=>path_points::run_end(w.route.as_ref().unwrap().steps(),a[1].parse().unwrap()).to_string(),
 "off"=>w.route.as_ref().unwrap().is_off_path(a[1].parse().unwrap(),v(a[2])).to_string(),
 "join"=>w.route.as_ref().unwrap().join_index(v(a[1])).map_or_else(||"-".into(),|i|i.to_string()),
 "look"=>w.route.as_ref().unwrap().look_ahead(a[1].parse().unwrap(),v(a[2]),v(a[3]),d(a[4])).map_or_else(||"-".into(),vector),
 "point"=>vector(w.route.as_ref().unwrap().point_ahead(a[1].parse().unwrap(),v(a[2]),d(a[3]))),
 "turn"=>w.route.as_ref().unwrap().turn_ahead(a[1].parse().unwrap(),v(a[2]),d(a[3])).to_bits().to_string(),
 "bridge"=>Bridge::at(w.route.as_ref().unwrap().steps(),a[1].parse().unwrap()).map_or_else(||"-".into(),|b|format!("{},{},{},{},{},{},{},{}",b.start,b.end,position(b.to),b.rises,b.is_straight,b.y_rot.to_bits(),b.turn_after,b.length())),
 "line"=>{let line=BridgeLine::new(v(a[1]),v(a[2]));format!("{},{}",line.heading().to_bits(),vector(line.aim_from(v(a[3]))))},
 "overshoot"=>{let s=&w.steps[&a[1].parse::<i32>().unwrap()];vector(w.overshoot.forward_aim(player,v(a[2]),v(a[3]),s,|pos,dist|{player.event(&format!("P:{}:{}",vector(pos),dist.to_bits()));if a[4]=="-"{None}else{Some(v(a[4]))}}))},
 "breathes"=>w.swim.breathes(player,t.level()).to_string(),"shortair"=>w.swim.is_short_of_air(player).to_string(),
 "swim"=>{let mut keys=MovementKeys::default();let m=w.swim.swim(player,&mut keys,f(a[2]),&w.steps[&a[1].parse::<i32>().unwrap()]);movement_raw(keys,m)},
 "climb"=>{let mut live=Terrain::player(LiveWorld(t.level().clone()));let mut keys=MovementKeys::default();let m=ctrl::climbing::climb_up(player,&mut live,&mut keys,f(a[2]),&w.steps[&a[1].parse::<i32>().unwrap()]);movement_raw(keys,m)},
 "hop"=>{let mut live=Terrain::player(LiveWorld(t.level().clone()));w.hops.starts(player,&mut live,||{player.event("y");player.facts.yaw},&w.stride,v(a[1]),v(a[2])).to_string()},
 "clearhop"=>{w.hops.clear();"-".into()},
 "reached"=>step_check::is_reached(player,t.level(),&w.actions,&w.steps[&a[1].parse::<i32>().unwrap()],if a[2]=="-"{None}else{Some(&w.steps[&a[2].parse::<i32>().unwrap()])}).to_string(),
 "skip"=>step_check::skip_ahead(player,w.route.as_ref().unwrap(),a[1].parse().unwrap()).to_string(),
 "confirm"=>{if a[2]=="true"{w.actions.unconfirmed.insert(p(a[1]));}else{w.actions.unconfirmed.remove(&p(a[1]));}"-".into()},
 "wants"=>prediction::wants_jump(&w.steps[&a[1].parse::<i32>().unwrap()],player).to_string(),"stairs"=>prediction::leads_onto_stairs(&w.steps[&a[1].parse::<i32>().unwrap()],t.level()).to_string(),
 "act"=>w.progress.acts_too_long(p(a[1])).to_string(),"unact"=>{w.progress.acting=None;"-".into()},"watch"=>w.progress.watch.stalls(d(a[1])).to_string(),"excuse"=>{w.progress.watch.excuse();"-".into()},
 "refuse"=>w.setbacks.refused(v(a[1]),a[2].parse().unwrap()).into_iter().map(position).collect::<Vec<_>>().join(";"),"stuck"=>w.stuck.add(p(a[1])).to_string(),"forget"=>{w.stuck.clear();"-".into()},"goal"=>{w.stuck.tick(v(a[1]));"-".into()},
 "script"=>{w.script=Script{exact:a[1].parse().unwrap(),moving:frames(a[2]),still:frames(a[3])};"-".into()},
 "predict"=>{let input=crate::navigation::input::Input::from_mask(a[2].parse().unwrap());let keys=MovementKeys{directional_input:input,jump:input.jump,sneak:input.shift};let flags:i32=a[5].parse().unwrap();prediction::gets_ahead(&Body{player,script:&w.script},&keys,f(a[3]),&w.steps[&a[1].parse::<i32>().unwrap()],Movement{target:if a[4]=="-"{None}else{Some(v(a[4]))},sprint:flags&1!=0,dive:flags&2!=0,is_plain_walk:flags&4!=0,safe_walk:flags&8!=0}).to_string()},
 "floor"=>{let input=crate::navigation::input::Input::from_mask(a[2].parse().unwrap());let mut keys=MovementKeys{directional_input:input,jump:input.jump,sneak:input.shift};let result=ctrl::floor_prediction::can_click_floor(&w.steps[&a[1].parse::<i32>().unwrap()],&Body{player,script:&w.script},&mut keys,f(a[3]),a[4].parse().unwrap());format!("{},{}",result,keys.to_input(false).java_hash())},
 "correct"=>ctrl::directional_input_for_y_rot(a[1].parse().unwrap(),a[2].parse().unwrap(),f(a[3]),if a[4]=="-"{None}else{Some(f(a[4]))}).java_hash().to_string(),
 "leaping"=>{let mut keys=MovementKeys::default();let s=&w.steps[&a[1].parse::<i32>().unwrap()];let measured=ab().with_motion(MotionTraits::new(0.1,0.42,0.0,if t.level().data.borrow().legacy{Mechanics::V1_8}else{Mechanics::Modern}));let m=ctrl::leaping::leap(&Body{player,script:&w.script},&mut keys,f(a[2]),s,s.jump.as_ref().unwrap(),&measured,&mut w.progress.leap,&w.stride);format!("{}~{}",movement_raw(keys,m),w.progress.leap.hopeless)},
 "rotation"=>rotation_raw(ViewRotation::from_vector(v(a[1]))),
 "viewvector"=>vector(ViewRotation::new(f(a[1]),f(a[2])).view_vector()),
 "turning"=>{let from=ViewRotation::new(f(a[1]),f(a[2]));let to=ViewRotation::new(f(a[3]),f(a[4]));let base=match a[5]{"instant"=>Turning::Instant,"linear"=>Turning::linear(f(a[6]),f(a[7])),_=>Turning::eased(f(a[6]),f(a[7]),f(a[8]),f(a[9]))};let purpose=match a[11]{"MOVEMENT"=>LookPurpose::Movement,"AIM"=>LookPurpose::Aim,"VIEW"=>LookPurpose::View,"STEER"=>LookPurpose::Steer,_=>panic!("purpose")};rotation_raw(if a[10]=="true"{base.human_paced()}else{base}.turn(from,to,purpose))},
 "bridgecadence"=>{let method=bridging(a[1]);format!("{},{},{},{},{},{},{}",method.x_rot().to_bits(),method.y_rot_offset().to_bits(),method.clicks_past_edge(),method.sneaks_throughout(),method.hops_on_reset(),method.resets(a[2].parse().unwrap()),method.tap(a[2].parse().unwrap(),d(a[3]),a[4].parse().unwrap()))},
 "bridgepick"=>{let method=bridging(a[1]);let bridge=Bridge::at(w.route.as_ref().unwrap().steps(),a[2].parse().unwrap()).unwrap();format!("{},{}",method.can_cross(&bridge),bridge_name(method.pick(&bridge,a[3].parse().unwrap(),d(a[4]),a[5].parse().unwrap())))},
 "bridgenew"=>{w.bridge.sent=ViewRotation::new(f(a[1]),f(a[2]));w.bridge.method=bridging(a[3]);w.bridge.risky=a[4].parse().unwrap();w.bridge.sneak_speed=d(a[5]);w.telly=Telly::default();w.crossing=Crossing::default();"-".into()},
 "bridgesent"=>{w.bridge.sent=ViewRotation::new(f(a[1]),f(a[2]));"-".into()},
 "telly"=>{let bridge=Bridge::at(w.route.as_ref().unwrap().steps(),a[1].parse().unwrap()).unwrap();let mut keys=mask_keys(a[3]);let m=w.telly.tick(&mut BridgeBody{inner:Body{player,script:&w.script},meta:&mut w.bridge,actions:&w.actions,stride:&w.stride},&mut keys,f(a[2]),w.route.as_ref().unwrap().steps(),&bridge,f(a[4]));format!("{}~{}~{}~{}",movement_raw(keys,m),w.telly.view.map_or_else(||"-".into(),rotation_raw),w.telly.acting.map_or_else(||"-".into(),position),w.bridge.action_raw())},
 "takes"=>{let result=w.crossing.takes(&BridgeBody{inner:Body{player,script:&w.script},meta:&mut w.bridge,actions:&w.actions,stride:&w.stride},w.route.as_ref().unwrap().steps(),a[1].parse().unwrap());format!("{}~{}",result,crossing_raw(&w.crossing))},
 "cross"=>{let mut keys=mask_keys(a[3]);let m=w.crossing.tick(&mut BridgeBody{inner:Body{player,script:&w.script},meta:&mut w.bridge,actions:&w.actions,stride:&w.stride},&mut keys,f(a[2]),w.route.as_ref().unwrap().steps(),a[1].parse().unwrap());format!("{}~{}~{}",movement_raw(keys,m),crossing_raw(&w.crossing),w.bridge.action_raw())},
 "crossclear"=>{w.crossing.clear();crossing_raw(&w.crossing)},
 "reject"=>{w.bridge.rejections=a[1].parse().unwrap();"-".into()},
 "placed"=>{w.bridge.placement=if a[1]=="-"{None}else{w.bridge.next_id+=1;Some(PlacementIdentity(w.bridge.next_id))};"-".into()},
 "straightwalk"=>ctrl::shortcut::is_straight_walkable(t,v(a[1]),v(a[2]),d(a[3]),a[4].parse().unwrap()).to_string(),
 "shortcut"=>ctrl::shortcut::shortcut(w.route.as_ref().unwrap(),a[1].parse().unwrap(),v(a[2]),t).map_or_else(||"-".into(),vector),
 _=>if a[0].starts_with('a'){actions_extra(a,w)}else if a[0].starts_with('f'){follower_extra(a,w)}else if a[0].starts_with("cap"){capture_extra(a,w)}else{targets_extra(a,w)}
 };format!("{}|{}",result,w.snapshot())
}
fn mask_keys(raw:&str)->MovementKeys{let input=crate::navigation::input::Input::from_mask(raw.parse().unwrap());MovementKeys{directional_input:input,jump:input.jump,sneak:input.shift}}
