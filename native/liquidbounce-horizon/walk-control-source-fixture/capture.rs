use crate::navigation::walk_control::{ability_capture::{self,CaptureHost,CaptureInventory,CaptureOptions,CaptureStack},break_times::{BreakStack,BreakState,BreakTimes}};
impl BreakState for S{
 fn empty_level_hardness(&self)->f32{self.events.lock().unwrap().push(format!("hard:{}",self.id));self.registry.borrow().capture_hardness.get(&self.id).map_or(0.0,|v|v.0)}
 fn requires_correct_tool(&self)->bool{self.events.lock().unwrap().push(format!("requires:{}",self.id));self.registry.borrow().capture_hardness.get(&self.id).is_some_and(|v|v.1)}
}
impl TargetStack{fn cap_event(&self,s:String){if let Some(events)=&self.events{events.lock().unwrap().push(s)}}}
impl BreakStack<S> for TargetStack{
 fn is_empty(&self)->bool{self.cap_event(format!("empty:{}",self.id));self.empty}
 fn copy_stack(&self)->Self{self.cap_event(format!("copy:{}",self.id));self.clone()}
 fn efficiency_level(&self)->i32{self.cap_event(format!("eff:{}",self.id));self.efficiency}
 fn correct_for_drops(&self,_state:&S)->bool{self.cap_event(format!("harvest:{}",self.id));self.correct}
 fn destroy_speed(&self,_state:&S)->f32{self.cap_event(format!("speed:{}",self.id));self.speed}
 fn empty_hand()->Self{Self::empty(-1)}
}
impl CaptureStack<S> for TargetStack{fn count(&self)->i32{self.cap_event(format!("count:{}",self.id));self.count}}
impl CaptureInventory for TargetInv{type Stack=TargetStack;fn item(&self,slot:i32)->TargetStack{TargetInventory::item(self,slot)}}
struct CaptureFixture{mask:i32,max_fall:i32,safe:f64,health:f32,jump_climbs:bool,allows_break:Arc<std::sync::atomic::AtomicBool>,method:Bridging,avoid:HashSet<i64>,unbreakable:HashSet<i64>,reserved:Arc<HashSet<i64>>,captured:Option<WalkAbilities<S>>}
impl Default for CaptureFixture{fn default()->Self{Self{mask:4,max_fall:3,safe:3.0,health:20.0,jump_climbs:true,allows_break:Arc::new(std::sync::atomic::AtomicBool::new(true)),method:Bridging::Auto,avoid:HashSet::new(),unbreakable:HashSet::new(),reserved:Arc::new(HashSet::new()),captured:None}}}
struct CaptureBody<'a>{inner:Body<'a>,targets:&'a TargetFixture,data:&'a CaptureFixture,method:Bridging,risky:bool}
impl WalkBody for CaptureBody<'_>{type Player=Player;type LiveLevel=LiveWorld;type Simulation=Sim;
 fn player(&self)->&Player{self.inner.player()}fn live_level(&self)->LiveWorld{self.inner.live_level()}fn exact_simulation(&self)->bool{self.inner.exact_simulation()}fn input_y_rot(&self)->f32{self.inner.input_y_rot()}fn movement_y_rot(&self)->f32{self.inner.movement_y_rot()}fn simulate(&self,input:SimulatedInput,yaw:f32)->Sim{self.inner.simulate(input,yaw)}
}
impl CaptureHost for CaptureBody<'_>{type Stack=TargetStack;type Inventory=TargetInv;
 fn capture_options(&self)->CaptureOptions<S,TargetStack>{self.player().event("o");let events=self.player().level.events.clone();let break_events=events.clone();let allowed=self.data.allows_break.clone();CaptureOptions{can_break:self.data.mask&1!=0,can_place:self.data.mask&2!=0,can_parkour:self.data.mask&4!=0,risky_jumps:self.data.mask&8!=0,max_fall:self.data.max_fall,fast_bridges:self.data.mask&16!=0,bridging:self.method,risky_bridges:self.risky,placeable:Arc::new(move|stack|{events.lock().unwrap().push(format!("P:{}",stack.id));stack.placeable}),breakable:Arc::new(move|state|{break_events.lock().unwrap().push(format!("breakable:{}",state.id));allowed.load(std::sync::atomic::Ordering::Relaxed)}),reserved:self.data.reserved.clone()}}
 fn capture_inventory(&self)->&TargetInv{self.player().event("N");&self.targets.inventory}
 fn safe_fall_distance(&self)->f64{self.player().event("safe");self.data.safe}fn health(&self)->f32{self.player().event("health");self.data.health}fn jump_climbs_ladders(&self)->bool{self.player().event("X");self.data.jump_climbs}
}
fn longs(raw:&str)->HashSet<i64>{if raw=="-"{HashSet::new()}else{raw.split(',').map(|s|s.parse().unwrap()).collect()}}
fn longs_raw(raw:&HashSet<i64>)->String{let mut a:Vec<_>=raw.iter().copied().collect();a.sort();a.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")}
fn capture_raw(a:&WalkAbilities<S>)->String{format!("{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}~{}",a.can_break,a.can_place,a.placeable_blocks,a.can_parkour,a.risky_jumps,a.can_sprint,a.safe_fall,a.water_fall,a.air,a.motion.walking_speed.to_bits(),a.motion.jump_strength.to_bits(),a.motion.jump_boost_power.to_bits(),if a.motion.mechanics==Mechanics::V1_8{"V1_8"}else{"MODERN"},a.jump_climbs,a.bridge_ticks.to_bits(),a.health.to_bits(),longs_raw(&a.avoid),longs_raw(&a.unbreakable),longs_raw(&a.reserved))}
fn capture_extra(a:&[&str],w:&mut WalkFixture)->String{
 let mut result="-".into();match a[0]{
 "capopts"=>{w.capture.mask=a[1].parse().unwrap();w.capture.max_fall=a[2].parse().unwrap();w.capture.safe=d(a[3]);w.capture.health=f(a[4]);w.capture.jump_climbs=a[5].parse().unwrap();w.capture.method=bridging(a[6]);w.capture.allows_break.store(a[7].parse().unwrap(),std::sync::atomic::Ordering::Relaxed);w.bridge.method=w.capture.method;w.bridge.risky=w.capture.mask&32!=0;},
 "capsets"=>{w.capture.avoid=longs(a[1]);w.capture.unbreakable=longs(a[2]);w.capture.reserved=Arc::new(longs(a[3]));},
 "capstack"=>{let mut data=w.targets.data.borrow_mut();let stack=&mut data.stacks[a[1].parse::<usize>().unwrap()];stack.count=a[2].parse().unwrap();stack.empty=a[3].parse().unwrap();},
 "caphard"=>{w.player.level.registry.borrow_mut().capture_hardness.insert(a[1].parse().unwrap(),(f(a[2]),a[3].parse().unwrap()));},
 _=>{let host=CaptureBody{inner:Body{player:&w.player,script:&w.script},targets:&w.targets,data:&w.capture,method:w.bridge.method,risky:w.bridge.risky};match a[0]{
 "capwalk"=>{let captured=ability_capture::capture_walk(&host,&w.capture.avoid,&w.capture.unbreakable);result=capture_raw(&captured);w.capture.captured=Some(captured);},
 "capquery"=>result=capture_raw(w.capture.captured.as_ref().unwrap()),
 "capticks"=>result=(w.capture.captured.as_ref().unwrap().break_ticks)(&w.player.level.state_id(a[1].parse().unwrap())).to_bits().to_string(),
 "capsafe"=>result=ability_capture::safe_fall(&host).to_string(),
 "caphot"=>result=ability_capture::hotbar(&host).iter().map(|s|s.id.to_string()).collect::<Vec<_>>().join(","),
 "capbreak"=>result=BreakTimes::capture(&ability_capture::hotbar(&host)).ticks(&w.player.level.state_id(a[1].parse().unwrap())).to_bits().to_string(),
 _=>panic!("{a:?}")
 }}
 }result
}
