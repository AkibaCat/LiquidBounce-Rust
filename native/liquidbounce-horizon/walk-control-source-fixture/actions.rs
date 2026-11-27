use crate::navigation::walk_control::block_actions::{Action,ActionHost,BlockActions,DoubleHalf,PlacedBlock,PlacementOrder};
#[derive(Default)]struct ActionOrderData{active:Vec<BlockPos>,script:Option<Vec<BlockPos>>}
#[derive(Clone,Default)]struct ActionOrder(Rc<RefCell<ActionOrderData>>);
impl PlacementOrder for ActionOrder{
 fn inserted(&mut self,pos:BlockPos){self.0.borrow_mut().active.push(pos)}
 fn removed(&mut self,pos:BlockPos){self.0.borrow_mut().active.retain(|p|*p!=pos)}
 fn clear(&mut self){self.0.borrow_mut().active.clear()}
 fn keys(&self)->Vec<BlockPos>{let order=self.0.borrow();if let Some(keys)=&order.script{let ordered:Vec<_>=keys.iter().filter(|p|order.active.contains(p)).copied().collect();assert_eq!(ordered.len(),order.active.len(),"script must list every pending key");ordered}else{order.active.clone()}}
}
struct ActionMeta{latency:i32,use_result:bool,start:bool,continuing:bool,bounds:Aabb,retained:Vec<Arc<PlacedBlock>>,order:ActionOrder,external_floors:bool}
impl Default for ActionMeta{fn default()->Self{Self{latency:0,use_result:true,start:true,continuing:true,bounds:Aabb::new(Vec3::new(4.0,0.0,4.0),Vec3::new(4.6,1.8,4.6)),retained:Vec::new(),order:ActionOrder::default(),external_floors:false}}}
impl ActionMeta{
 fn id(&mut self,placed:&Arc<PlacedBlock>)->usize{if let Some(index)=self.retained.iter().position(|p|Arc::ptr_eq(p,placed)){index}else{self.retained.push(placed.clone());self.retained.len()-1}}
 fn event(&mut self,player:&Player,kind:&str,placed:&Arc<PlacedBlock>){let id=self.id(placed);player.event(&format!("{kind}:{id}:{}:{}:{}",position(placed.pos),placed.tick,placed.verdict().tag()))}
}
struct ActionBody<'a>{inner:TargetBody<'a>,meta:&'a mut ActionMeta,script:&'a Script,method:Bridging,risky:bool,sneak_speed:f64}
impl SightHost for ActionBody<'_>{type State=S;fn state(&self,pos:BlockPos)->S{self.inner.state(pos)}fn outline_shape(&self,state:&S,pos:BlockPos)->VoxelShape{self.inner.outline_shape(state,pos)}fn clip(&self,from:Vec3,to:Vec3)->BlockHit{self.inner.clip(from,to)}}
impl TargetHost for ActionBody<'_>{type Inventory=TargetInv;type Policy=TargetPolicy;
 fn eye_position(&self)->Vec3{self.inner.eye_position()}fn reach(&self)->f64{self.inner.reach()}fn block_position(&self)->BlockPos{self.inner.block_position()}fn uuid_most_significant_bits(&self)->i64{self.inner.uuid_most_significant_bits()}
 fn sent_rotation(&self)->ViewRotation{self.inner.sent_rotation()}fn view_step(&self)->f32{self.inner.view_step()}fn pick(&self,rotation:ViewRotation,reach:f64,eye:Option<Vec3>)->BlockHit{self.inner.pick(rotation,reach,eye)}
 fn inventory(&self)->&TargetInv{self.inner.inventory()}fn placeable_blocks(&self)->&TargetPolicy{self.inner.placeable_blocks()}fn is_tool_for(&self,stack:&TargetStack,state:&S)->bool{self.inner.is_tool_for(stack,state)}fn destroy_speed(&self,stack:&TargetStack,state:&S)->f32{self.inner.destroy_speed(stack,state)}fn efficiency_level(&self,stack:&TargetStack)->i32{self.inner.efficiency_level(stack)}fn is_wear_free(&self,stack:&TargetStack)->bool{self.inner.is_wear_free(stack)}
}
impl ActionHost for ActionBody<'_>{
 fn collision_empty(&self,state:&S,pos:BlockPos)->bool{state.collision_shape(&self.inner.player.level,pos).is_empty()}
 fn double_half(&self,state:&S)->Option<DoubleHalf>{match state.def().upper{0=>Some(DoubleHalf::Lower),1=>Some(DoubleHalf::Upper),_=>None}}
 fn player_box(&self)->Aabb{self.inner.player.event("box");self.meta.bounds}
 fn look(&mut self,rotation:ViewRotation,purpose:LookPurpose){self.inner.player.event(&format!("L:{}:{}",rotation_raw(rotation),match purpose{LookPurpose::Aim=>"AIM",LookPurpose::Movement=>"MOVEMENT",LookPurpose::View=>"VIEW",LookPurpose::Steer=>"STEER"}));}
 fn select_hotbar_slot(&mut self,slot:i32){self.inner.player.event(&format!("select:{slot}"))}fn release_hotbar_slot(&mut self){self.inner.player.event("release")}
 fn start_destroy(&mut self,pos:BlockPos,face:Direction)->bool{self.inner.player.event(&format!("start:{}:{}",position(pos),face_name(face)));self.meta.start}
 fn continue_destroy(&mut self,pos:BlockPos,face:Direction)->bool{self.inner.player.event(&format!("continue:{}:{}",position(pos),face_name(face)));self.meta.continuing}
 fn stop_destroy(&mut self){self.inner.player.event("stop")}
 fn use_item_on(&mut self,hand:Hand,hit:BlockHit)->bool{self.inner.player.event(&format!("use:{}:{}:{}:{}",match hand{Hand::Main=>"MAIN_HAND",Hand::Off=>"OFF_HAND"},position(hit.pos),face_name(hit.direction),vector(hit.location)));self.meta.use_result}
 fn latency_millis(&self)->i32{self.inner.player.event("latency");self.meta.latency}
 fn block_placed(&mut self,placed:&Arc<PlacedBlock>){self.meta.event(self.inner.player,"placed",placed)}fn block_kept(&mut self,placed:&Arc<PlacedBlock>){self.meta.event(self.inner.player,"kept",placed)}fn block_rejected(&mut self,placed:&Arc<PlacedBlock>){self.meta.event(self.inner.player,"rejected",placed)}
 fn server_put_back_broken(&mut self,pos:BlockPos){self.inner.player.event(&format!("warn:broken:{}",position(pos)))}fn server_took_back_placed(&mut self,pos:BlockPos){self.inner.player.event(&format!("warn:placed:{}",position(pos)))}
}
fn actual_action_raw(actions:&BlockActions<ActionOrder>)->String{match actions.action(){None=>"-".into(),Some(Action::Mine(pos))=>format!("M:{}",position(*pos)),Some(Action::Open(pos))=>format!("O:{}",position(*pos)),Some(Action::Place{pos,ready,plan,held})=>format!("P:{}:{}:{}:{}",position(*pos),ready,held.map_or_else(||"-".into(),rotation_raw),plan.clone().map_or_else(||"-".into(),plan_raw))}}
fn action_snapshot(actions:&BlockActions<ActionOrder>,meta:&mut ActionMeta)->String{let(ticks,cooldown,digging)=actions.timing();let placement=actions.placement.as_ref().map_or_else(||"-".into(),|p|meta.id(p).to_string());format!("{}~{}~{}~{}~{}~{}~{}~{}~{}",actual_action_raw(actions),actions.rotation.map_or_else(||"-".into(),rotation_raw),ticks,cooldown,digging.map_or_else(||"-".into(),position),actions.rejected_in_row,actions.rejections,placement,meta.retained.iter().enumerate().map(|(i,p)|format!("{i}:{}:{}:{}",position(p.pos),p.tick,p.verdict().tag())).collect::<Vec<_>>().join(";"))}
fn actions_extra(a:&[&str],w:&mut WalkFixture)->String{
 if a[0]=="anew"{w.action_meta=ActionMeta::default();w.block_actions=BlockActions::new(w.action_meta.order.clone());}
 let mut host=ActionBody{inner:TargetBody{player:&w.player,targets:&w.targets,sent:w.bridge.sent},meta:&mut w.action_meta,script:&w.script,method:w.bridge.method,risky:w.bridge.risky,sneak_speed:w.bridge.sneak_speed};
 let result=match a[0]{
 "anew"=>"-".into(),
 "aset"=>{host.meta.latency=a[1].parse().unwrap();host.meta.use_result=a[2].parse().unwrap();host.meta.start=a[3].parse().unwrap();host.meta.continuing=a[4].parse().unwrap();host.meta.bounds=Aabb::new(v(a[5]),v(a[6]));"-".into()},
 "amine"=>{w.block_actions.mine(&mut host,p(a[1]));"-".into()},"aopen"=>{w.block_actions.open(&mut host,p(a[1]));"-".into()},
 "aplace"=>{w.block_actions.place(&mut host,p(a[1]),a[2].parse().unwrap(),if a[3]=="-"{None}else{Some(ViewRotation::new(f(a[3]),f(a[4])))});"-".into()},
 "aidle"=>{w.block_actions.idle(&mut host);"-".into()},"aaim"=>{w.block_actions.aim(&mut host);"-".into()},"atick"=>{w.block_actions.tick(&mut host);"-".into()},
 "aanswer"=>w.block_actions.answer(&mut host,p(a[1]),&w.player.level.state_id(a[2].parse().unwrap())).to_string(),
 "alevel"=>{w.block_actions.level_changed();"-".into()},"aforget"=>{w.block_actions.forget_rejections();"-".into()},
 "astatus"=>{let pos=p(a[1]);format!("{},{},{},{}",w.block_actions.is_active(),w.block_actions.is_unconfirmed(pos),w.block_actions.is_open_unconfirmed(pos),w.block_actions.unbreakable.contains(&pos.packed()))},
 "aorder"=>{let keys=ps(a[1]);assert_eq!(host.meta.order.0.borrow().active.len(),keys.len());host.meta.order.0.borrow_mut().script=Some(keys);"-".into()},
 _=>panic!("action command {a:?}")
 };
 result+"@"+&action_snapshot(&w.block_actions,&mut w.action_meta)
}
