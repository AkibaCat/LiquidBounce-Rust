use crate::navigation::walk_control::{block_targets::{self,BlockTargets,TargetHost,TargetInventory,PlaceableBlocks,Plan,BlockHit,Hand},block_sight::SightHost};
#[derive(Clone)]struct TargetStack{id:i32,tool:bool,correct:bool,speed:f32,efficiency:i32,placeable:bool,unfavourable:bool,count:i32,empty:bool,events:Option<Events>}
impl TargetStack{fn empty(id:i32)->Self{Self{id,tool:false,correct:false,speed:1.0,efficiency:0,placeable:false,unfavourable:false,count:1,empty:true,events:None}}}
struct TargetData{stacks:Vec<TargetStack>,selected:i32,reach:f64,uuid:i64,view_step:f32,hit_pos:BlockPos,hit_face:Direction,hit:bool,clip_count:i32,clip_limit:i32}
impl Default for TargetData{fn default()->Self{Self{stacks:(0..41).map(TargetStack::empty).collect(),selected:0,reach:4.5,uuid:0,view_step:0.0,hit_pos:BlockPos::default(),hit_face:Direction::Up,hit:true,clip_count:0,clip_limit:i32::MAX}}}
struct TargetInv{data:Rc<RefCell<TargetData>>,events:Events}
impl TargetInventory for TargetInv{type Stack=TargetStack;fn item(&self,slot:i32)->TargetStack{self.events.borrow_mut().push(format!("I:{slot}"));{let mut stack=self.data.borrow().stacks[slot as usize].clone();stack.events=Some(self.events.clone());stack}}fn selected_slot(&self)->i32{self.events.borrow_mut().push("slot".into());self.data.borrow().selected}}
struct TargetPolicy{events:Events}
impl PlaceableBlocks<TargetStack> for TargetPolicy{fn is_placeable(&self,stack:&TargetStack)->bool{self.events.borrow_mut().push(format!("P:{}",stack.id));stack.placeable}fn is_unfavourable(&self,stack:&TargetStack)->bool{self.events.borrow_mut().push(format!("Q:{}",stack.id));stack.unfavourable}}
struct TargetFixture{data:Rc<RefCell<TargetData>>,inventory:TargetInv,policy:TargetPolicy}
impl TargetFixture{fn new(events:Events)->Self{let data=Rc::new(RefCell::new(TargetData::default()));Self{data:data.clone(),inventory:TargetInv{data,events:events.clone()},policy:TargetPolicy{events}}}}
struct TargetBody<'a>{player:&'a Player,targets:&'a TargetFixture,sent:ViewRotation}
impl SightHost for TargetBody<'_>{type State=S;
 fn state(&self,pos:BlockPos)->S{self.player.level.block_state(pos)}
 fn outline_shape(&self,state:&S,pos:BlockPos)->VoxelShape{state.collision_shape(&self.player.level,pos)}
 fn clip(&self,from:Vec3,to:Vec3)->BlockHit{self.player.event(&format!("Z:{}:{}",vector(from),vector(to)));let mut data=self.targets.data.borrow_mut();data.clip_count+=1;BlockHit{is_block:data.hit&&data.clip_count<=data.clip_limit,location:to,direction:data.hit_face,pos:data.hit_pos}}
}
impl TargetHost for TargetBody<'_>{type Inventory=TargetInv;type Policy=TargetPolicy;
 fn eye_position(&self)->Vec3{self.player.eye_position()}fn reach(&self)->f64{self.player.event("r");self.targets.data.borrow().reach}
 fn block_position(&self)->BlockPos{self.player.event("V");BlockPos::containing(self.player.facts.pos)}
 fn uuid_most_significant_bits(&self)->i64{self.player.event("U");self.targets.data.borrow().uuid}
 fn sent_rotation(&self)->ViewRotation{self.player.event("R");self.sent}fn view_step(&self)->f32{self.player.event("vstep");self.targets.data.borrow().view_step}
 fn pick(&self,rotation:ViewRotation,reach:f64,eye:Option<Vec3>)->BlockHit{let eye=eye.unwrap_or_else(||self.eye_position());let look=rotation.view_vector();let to=Vec3::new(eye.x+look.x*reach,eye.y+look.y*reach,eye.z+look.z*reach);self.player.event("l");self.clip(eye,to)}
 fn inventory(&self)->&TargetInv{self.player.event("N");&self.targets.inventory}fn placeable_blocks(&self)->&TargetPolicy{self.player.event("o");&self.targets.policy}
 fn is_tool_for(&self,stack:&TargetStack,_state:&S)->bool{self.player.event(&format!("correct:{}",stack.id));stack.correct}
 fn destroy_speed(&self,stack:&TargetStack,_state:&S)->f32{self.player.event(&format!("speed:{}",stack.id));stack.speed}
 fn efficiency_level(&self,stack:&TargetStack)->i32{self.player.event(&format!("eff:{}",stack.id));stack.efficiency}
 fn is_wear_free(&self,stack:&TargetStack)->bool{self.player.event(&format!("tool:{}",stack.id));!stack.tool}
}
fn face(raw:&str)->Direction{match raw{"DOWN"=>Direction::Down,"UP"=>Direction::Up,"NORTH"=>Direction::North,"SOUTH"=>Direction::South,"WEST"=>Direction::West,"EAST"=>Direction::East,_=>panic!("face")}}
fn face_name(face:Direction)->&'static str{match face{Direction::Down=>"DOWN",Direction::Up=>"UP",Direction::North=>"NORTH",Direction::South=>"SOUTH",Direction::West=>"WEST",Direction::East=>"EAST"}}
fn plan_raw(plan:Plan)->String{format!("{}^{}^{}^{}^{}^{}^{}",position(plan.pos),position(plan.against),face_name(plan.face),vector(plan.point),rotation_raw(plan.rotation),plan.slot,match plan.hand(){Hand::Main=>"MAIN_HAND",Hand::Off=>"OFF_HAND"})}
fn targets_extra(a:&[&str],w:&mut WalkFixture)->String{
 let host=TargetBody{player:&w.player,targets:&w.targets,sent:w.bridge.sent};
 match a[0]{
 "targetnew"=>{*w.targets.data.borrow_mut()=TargetData::default();"-".into()},
 "targetstack"=>{let id:usize=a[1].parse().unwrap();w.targets.data.borrow_mut().stacks[id]=TargetStack{id:id as i32,tool:a[2].parse().unwrap(),correct:a[3].parse().unwrap(),speed:f(a[4]),efficiency:a[5].parse().unwrap(),placeable:a[6].parse().unwrap(),unfavourable:a[7].parse().unwrap(),count:1,empty:true,events:None};"-".into()},
 "targetsetup"=>{let mut data=w.targets.data.borrow_mut();data.selected=a[1].parse().unwrap();data.reach=d(a[2]);data.uuid=a[3].parse().unwrap();data.view_step=f(a[4]);data.hit_pos=p(a[5]);data.hit_face=face(a[6]);data.hit=a[7].parse().unwrap();data.clip_limit=a[8].parse().unwrap();data.clip_count=0;"-".into()},
 "targetplan"=>BlockTargets::plan_placement(&host,p(a[1]),a[2].parse().unwrap(),a[3].parse().unwrap()).map_or_else(||"-".into(),plan_raw),
 "targetheld"=>BlockTargets::held_click(&host,p(a[1]),ViewRotation::new(f(a[2]),f(a[3]))).map_or_else(||"-".into(),plan_raw),
 "targetaim"=>BlockTargets::aim_at(&host,p(a[1])).map_or_else(||"-".into(),rotation_raw),
 "targetclicks"=>BlockTargets::clicks_from(&host,v(a[1]),ViewRotation::new(f(a[2]),f(a[3])),p(a[4])).to_string(),
 "targetdig"=>BlockTargets::digging_slot(&host,&w.player.level.state_id(a[1].parse().unwrap())).map_or_else(||"-".into(),|s|s.to_string()),
 "targetslot"=>BlockTargets::placeable_slot(&host).map_or_else(||"-".into(),|s|s.to_string()),
 "clickpoint"=>vector(block_targets::click_point(p(a[1]),face(a[2]),a[3].parse().unwrap())),
 _=>panic!("target command {a:?}")
 }
}
