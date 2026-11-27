use super::super::{abilities::GlideAbilities, contacts::{ContactBody,ContactPlayer,ContactListener,Contacts},roof_bounce::{RoofBody,RoofPlayer,RoofReport,RoofBounce,bounce_line}};
#[derive(Clone)]struct LiveWorld(World);
impl TerrainLevel<S> for LiveWorld {
 fn block_state(&self,p:BlockPos)->S{self.0.block_state(p)} fn void_air(&self)->S{self.0.void_air()}
 fn is_known(&self,p:BlockPos)->bool {let data=self.0.data.borrow();if p.y<data.min||p.y>data.min+data.count-1{return false}self.0.events.borrow_mut().push(format!("H:{},{}",p.x,p.z));!data.unknown.contains(&BlockPos::new(p.x,0,p.z))}
 fn is_inside_border(&self,x:f64,z:f64)->bool{self.0.is_inside_border(x,z)} fn min_y(&self)->i32{self.0.min_y()} fn height(&self)->i32{self.0.height()}
}
struct Actor{position:Vec3,bounds:Aabb,ground:bool,flying:bool,horizontal:bool,vertical:bool,below:bool,events:Events}
impl RoofPlayer for Actor{fn position(&self)->Vec3{self.events.borrow_mut().push("p".into());self.position}fn on_ground(&self)->bool{self.events.borrow_mut().push("g".into());self.ground}fn is_fall_flying(&self)->bool{self.events.borrow_mut().push("f".into());self.flying}}
impl ContactPlayer for Actor{fn is_fall_flying(&self)->bool{RoofPlayer::is_fall_flying(self)}fn horizontal_collision(&self)->bool{self.events.borrow_mut().push("h".into());self.horizontal}fn vertical_collision(&self)->bool{self.events.borrow_mut().push("v".into());self.vertical}fn vertical_collision_below(&self)->bool{self.events.borrow_mut().push("b".into());self.below}fn bounding_box(&self)->Aabb{self.events.borrow_mut().push("A".into());self.bounds}}
struct Body{actor:Actor,world:World,recasts:bool,hashes:[i32;6]}
impl RoofBody for Body{type Player=Actor;type State=S;type Level=LiveWorld;fn player(&self)->&Actor{self.world.events.borrow_mut().push("P".into());&self.actor}fn recasts_elytra(&self)->bool{self.world.events.borrow_mut().push("r".into());self.recasts}fn live_level(&self)->LiveWorld{self.world.events.borrow_mut().push("L".into());LiveWorld(self.world.clone())}fn press(&mut self,i:crate::navigation::input::Input,e:bool){self.world.events.borrow_mut().push(format!("K:{}:{e}",i.java_hash()))}fn look_steer(&mut self,y:f32,x:f32){self.world.events.borrow_mut().push(format!("V:{},{}:STEER",y.to_bits(),x.to_bits()))}}
impl ContactBody for Body{type Player=Actor;type State=S;type Level=LiveWorld;fn player(&self)->&Actor{RoofBody::player(self)}fn live_level(&self)->LiveWorld{RoofBody::live_level(self)}fn direction_hash(&self,d:Direction)->i32{self.hashes[d as usize]}}
struct Report(Events);
impl RoofReport for Report{fn blocks_empty(&mut self){self.0.borrow_mut().push("R:B:0,0".into())}fn following(&mut self,points:&[Vec3],progress:&RoofBounce){self.0.borrow_mut().push(format!("R:F:{}:{}:{}",points.iter().copied().map(vector).collect::<Vec<_>>().join(";"),progress.is_active(),progress.target().map_or_else(String::new,vector)))}}
impl ContactListener for Report{fn touched(&mut self,p:BlockPos,d:Direction){self.0.borrow_mut().push(format!("T:{}:{}",position(p),direction(Some(d))))}}
struct Runtime{roof:RoofBounce,contacts:Contacts,body:Body}
fn extra(a:&[&str],terrain:&mut Terrain<S,World>,runtime:&mut Option<Runtime>)->String {
 let world=terrain.level().clone();
 if a[0]=="runtime"{let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let hashes:Vec<i32>=std::fs::read_to_string(root.join("direction-hashes.txt")).unwrap().split(',').map(|s|s.parse().unwrap()).collect();*runtime=Some(Runtime{roof:RoofBounce::default(),contacts:Contacts::default(),body:Body{actor:Actor{position:Vec3::ZERO,bounds:Aabb::from_block(BlockPos::default()),ground:false,flying:false,horizontal:false,vertical:false,below:false,events:world.events.clone()},world,recasts:false,hashes:hashes.try_into().unwrap()}});return "-".into();}
 let r=runtime.as_mut().unwrap();r.body.world=world;let b=|s:&str|s.parse::<bool>().unwrap();
 let state=|roof:&RoofBounce|roof.bounce.as_ref().map_or_else(||"-".into(),|b|format!("{}:{}:{}:{}",vector(b.goal),b.floor.to_bits(),vector(b.line_end),b.ticks));
 match a[0]{"player"=>{let p=&mut r.body.actor;p.position=v(a[1]);p.ground=b(a[2]);p.flying=b(a[3]);r.body.recasts=b(a[4]);p.horizontal=b(a[5]);p.vertical=b(a[6]);p.below=b(a[7]);p.bounds=crate_box(a[8]);"-".into()},"line"=>bounce_line(terrain,v(a[1]),v(a[2]),d(a[3])).map_or_else(||"-".into(),vector),"roof"=>{let result=r.roof.navigate(&mut r.body,&mut Report(terrain.level().events.clone()),v(a[1]),b(a[2]).then_some(GlideAbilities{rockets:0,rocket_ticks:0,glide_ticks:100}),f32::from_bits(a[3].parse().unwrap()));format!("{result}:{}:{}",r.roof.is_active(),state(&r.roof))},"halt"=>{r.roof.halt();state(&r.roof)},"contacts"=>{r.contacts.tick(&r.body,&mut Report(terrain.level().events.clone()));r.contacts.fixture_state()},_=>panic!("{a:?}")}
}
fn crate_box(s:&str)->Aabb{b(s)}
