// SPDX-License-Identifier: GPL-3.0-or-later
use horizon_core::{geometry::Vec3,position::BlockPos};
use horizon_world::registry::BlockStateId;
use std::{cell::Cell,collections::{BTreeMap,BTreeSet}};
type Result<T>=std::result::Result<T,Box<dyn std::error::Error>>;
pub mod actions{pub(crate) use crate::Action;}
#[path="__BRIDGE__"]pub mod terrain_speed_bridge;
#[path="__ORDER__"]pub mod terrain_speed_order;
#[path="__ADAPTER__"]pub mod terrain_speed_motion;
thread_local!{static SETTINGS:Cell<Option<terrain_speed_bridge::Snapshot>>=const{Cell::new(None)};}
pub mod mod_api{
    use super::*;
    pub trait RenderHooks:Send+Sync{}
    #[allow(unused_imports)]mod host{include!(concat!(env!("OUT_DIR"),"/mod_api_generated_host.rs"));}pub use host::*;
    #[allow(clippy::too_many_arguments)]mod generated{include!(concat!(env!("OUT_DIR"),"/mod_api_generated.rs"));}pub use generated::*;
    pub fn entered<'a>(mods:&'a [Box<dyn Hooks>],_:&str)->impl Iterator<Item=&'a dyn Hooks>{mods.iter().map(Box::as_ref)}
    struct Adapter;
    impl Hooks for Adapter{fn client_terrain_speed(&self,_:&dyn Host,e:&mut ClientTerrainSpeedEvent){
        if let Some(s)=SETTINGS.with(Cell::get){let t=s.settings;
            e.flags=u32::from(t.fast_climb.enabled)|(u32::from(t.ice_speed.enabled)<<1)|(u32::from(t.ice_speed.motion_enabled)<<2)|(u32::from(t.water_speed.enabled)<<3)|(u32::from(t.water_speed.auto_swim)<<4)|(u32::from(t.water_speed.sprint_enabled)<<5);
            e.mode=u8::from(t.fast_climb.mode==terrain_speed::ClimbMode::Clip);e.climb_motion=t.fast_climb.motion;e.slipperiness=t.ice_speed.slipperiness;e.ice_motion=t.ice_speed.horizontal_motion;
            e.base_horizontal=t.water_speed.base_horizontal;e.base_vertical=t.water_speed.base_vertical;e.sprint_horizontal=t.water_speed.sprint_horizontal;e.sprint_vertical=t.water_speed.sprint_vertical;e.input=s.input;e.running=true;
        }
    }}
    pub fn mods()->&'static[Box<dyn Hooks>]{static MODS:std::sync::LazyLock<Vec<Box<dyn Hooks>>>=std::sync::LazyLock::new(||vec![Box::new(Adapter)]);&MODS}
}
#[derive(Clone,Default)]struct Fluids{was_touching_water:bool}
#[derive(Clone,Default)]struct Collision{horizontal:bool}
#[derive(Clone,Default)]struct Ground{on_ground:bool}
#[derive(Clone)]pub struct Player{
    position:Vec3,velocity:Vec3,collision:Collision,fluids:Fluids,move_vector:[f32;2],yaw:f32,
    allowed:bool,on_climbable:bool,client_side:bool,ground:Ground,client_strafe:Option<(strafe::Strafe,u8)>,entity_id:i32,
}
impl Player{fn native_client_queries_allowed(&self)->bool{self.allowed}}
struct Scene{blocks:BTreeMap<(i32,i32,i32),BlockStateId>}
impl Scene{fn block(&self,x:i32,y:i32,z:i32)->BlockStateId{self.blocks.get(&(x,y,z)).copied().unwrap_or(BlockStateId(0))}}
struct Coefficients{friction:f32}struct MovementBlock{coefficients:Coefficients}
struct Blocks;impl Blocks{fn get(&self,_:BlockStateId)->Result<MovementBlock>{Ok(MovementBlock{coefficients:Coefficients{friction:0.98}})}}
struct Resources{terrain_climb_blocks:BTreeSet<BlockStateId>,blocks:Blocks}
struct State{name:String}struct Registry{state:State}impl Registry{fn state(&self,_:BlockStateId)->Option<&State>{Some(&self.state)}}
struct Assets{registry:Registry}struct Pack{assets:Assets}
pub mod motion{
use super::*;
use horizon_world::entity_move::MoverType;
pub struct MotionWorld<'p,'w,'r>{pub(super) player:&'p mut Player,pub(super) scene:&'w mut Scene,pub(super) resources:&'r Resources,pub(super) assets:&'r Assets,pub(super) entities:&'r[()],pub(super) collision_binding:(),pub(super) health:&'r()}
impl MotionWorld<'_,'_,'_>{
    fn block(&self,p:BlockPos)->BlockStateId{self.scene.block(p.x,p.y,p.z)}
    pub(super) fn query_friction(&mut self,p:BlockPos)->Result<f32>{self.block_friction(p)}
    pub fn climbable(&mut self)->Result<bool>{Ok(self.player.on_climbable)}
    fn update_auto_jump(&mut self,dx:f32,dz:f32)->Result<()>{AUTO_JUMP.with(|c|c.set([dx,dz]));Ok(())}
    __STRAFE_MODIFIER__
    __MOVE_SELF__
    __SHULKER_PUSH__
    __PLAYER_FRICTION__
}
mod entity_move{
    use super::*;
    pub fn move_entity(world:&mut MotionWorld<'_,'_,'_>,mover:MoverType,movement:Vec3)->Result<()>{
        LAST_MOVE.with(|m|m.set(Some((mover,[movement.x,movement.y,movement.z]))));
        world.player.position=world.player.position.add_vector(movement);Ok(())
    }
}
}
thread_local!{static AUTO_JUMP:Cell<[f32;2]>=const{Cell::new([0.;2])};static LAST_MOVE:Cell<Option<(horizon_world::entity_move::MoverType,[f64;3])>>=const{Cell::new(None)};}
#[derive(Clone,Copy)]enum Action{Forward,Sprint,Jump,Sneak}
struct Keys{forward:bool,sprint:bool,jump:bool,sneak:bool}impl Keys{fn held(&self,a:Action)->bool{match a{Action::Forward=>self.forward,Action::Sprint=>self.sprint,Action::Jump=>self.jump,Action::Sneak=>self.sneak}}}
pub struct Game<'a>{player:Option<Player>,yaw:f32,keys:Keys,lan_acting_uuid:Option<u128>,window:bool,previous_position:Vec3,world:Scene,pack:Pack,collision:(),motion:&'a Resources,health_resources:()}
thread_local!{static TIMER_CANCELLED:Cell<bool>=const{Cell::new(false)};static TIMER_FINAL:Cell<(bool,[f64;3])>=const{Cell::new((false,[0.;3]))};}
impl Game<'_>{fn native_window(&self)->bool{self.window}fn player_entity_collisions(&self)->Vec<()>{vec![]}
fn native_client_player_tick(&self,cancelled:&mut bool){*cancelled=TIMER_CANCELLED.with(Cell::get);}
fn native_client_player_tick_final(&self,cancelled:bool){let v=self.player.as_ref().unwrap().velocity;TIMER_FINAL.with(|c|c.set((cancelled,[v.x,v.y,v.z])));}}
mod native_hooks{use super::*;impl Game<'_>{__HOST__}}
mod piston_scope{use super::*;use motion::MotionWorld;impl Game<'_>{__PISTON_HEAD__}pub(super) fn run_head(g:&mut Game<'_>,v:Vec3)->Result<Vec3>{g.terrain_local_move_head(v)}}
mod generated_host{include!(concat!(env!("OUT_DIR"),"/mod_api_host_generated.rs"));}

#[cfg(test)]mod native_tests{
use super::*;
fn snapshot()->terrain_speed_bridge::Snapshot{terrain_speed_bridge::Snapshot{settings:Default::default(),forward:true,sprint:false,jump:false,sneak:false,input:1}}
fn game(res:&Resources)->Game<'_>{Game{player:Some(Player{position:Vec3::new(2.25,-3.5,4.75),velocity:Vec3::new(-0.0,0.2,-0.4),collision:Collision{horizontal:true},fluids:Fluids{was_touching_water:true},move_vector:[1.,0.],yaw:0.,allowed:true,on_climbable:true,client_side:true,ground:Default::default(),client_strafe:None,entity_id:7}),yaw:0.,keys:Keys{forward:true,sprint:false,jump:false,sneak:false},lan_acting_uuid:None,window:true,previous_position:Vec3::ZERO,world:Scene{blocks:BTreeMap::new()},pack:Pack{assets:Assets{registry:Registry{state:State{name:"minecraft:ice".into()}}}},collision:(),motion:res,health_resources:()}}
fn resources()->Resources{Resources{terrain_climb_blocks:BTreeSet::from([BlockStateId(1),BlockStateId(2)]),blocks:Blocks}}
#[test]fn generated_host_keeps_physical_keys_separate_and_water_reads_previous_input(){
 let res=resources();let mut g=game(&res);let mut s=snapshot();s.settings.water_speed.base_horizontal=0.44;s.input=1;SETTINGS.with(|c|c.set(Some(s)));
 g.keys.jump=true;g.keys.sneak=true;g.keys.sprint=true;
 g.player.as_mut().unwrap().yaw=-90.;g.yaw=0.;
 let captured=g.native_terrain_snapshot().unwrap();assert!(captured.jump&&captured.sneak&&captured.sprint);assert_eq!(captured.input,1);
 let _scope=terrain_speed_bridge::enter(Some(captured));g.native_terrain_player_tick();
 let v=g.player.as_ref().unwrap().velocity;assert_eq!(v.y,1.0);assert_eq!(v.z,1.0);assert_eq!(v.x.to_bits(),(-0.0f64).to_bits());
 // The following new key read/rotation does not retroactively affect tick.
 g.player.as_mut().unwrap().move_vector=[0.;2];g.player.as_mut().unwrap().yaw=90.;assert_eq!(v.z,1.0);
 SETTINGS.with(|c|c.set(None));
}
#[test]fn native_identity_and_running_gate_capture_without_a_screen_guard(){
 let res=resources();let mut g=game(&res);SETTINGS.with(|c|c.set(Some(snapshot())));assert!(g.native_terrain_snapshot().is_some());
 for denied in 0..3{g.window=denied!=0;g.lan_acting_uuid=(denied==1).then_some(4);g.player.as_mut().unwrap().allowed=denied!=2;assert!(g.native_terrain_snapshot().is_none());}
 g.window=true;g.lan_acting_uuid=None;g.player.as_mut().unwrap().allowed=true;SETTINGS.with(|c|c.set(None));assert!(g.native_terrain_snapshot().is_none());
}
#[test]fn water_handler_runs_on_cancelled_player_tick_before_actual_final_listener(){
 let res=resources();let mut g=game(&res);let s=snapshot();let _scope=terrain_speed_bridge::enter(Some(s));
 TIMER_CANCELLED.with(|c|c.set(true));assert!(g.client_timer_player_tick());
 let(cancelled,velocity)=TIMER_FINAL.with(Cell::get);assert!(cancelled);assert_eq!(velocity[2],f64::from(0.44f32));
 TIMER_CANCELLED.with(|c|c.set(false));
}
#[test]fn global_query_repetitions_flush_before_local_friction_and_preserve_thread_scope(){
 let mut s=snapshot();s.settings.ice_speed.motion_enabled=true;s.settings.ice_speed.horizontal_motion=-2.5;
 let _scope=terrain_speed_bridge::enter(Some(s));let mut v=Vec3::new(0.3,-0.0,-0.4);
 for _ in 0..3{assert_eq!(terrain_speed_bridge::friction("minecraft:blue_ice",0.989),0.6);}
 assert_eq!(terrain_speed_bridge::friction("minecraft:stone",0.6),0.6);
 let mut expected=[0.3,-0.0,-0.4];for _ in 0..4{s.settings.ice_speed.block_friction(true,"minecraft:ice",0.98,&mut expected);}
 assert_eq!(terrain_speed_bridge::player_friction("minecraft:ice",0.98,&mut v),0.6);assert_eq!([v.x.to_bits(),v.y.to_bits(),v.z.to_bits()],expected.map(f64::to_bits));
 let other=std::thread::spawn(||terrain_speed_bridge::friction("minecraft:ice",0.98)).join().unwrap();assert_eq!(other,0.98);
 {let _nested=terrain_speed_bridge::enter(None);assert_eq!(terrain_speed_bridge::friction("minecraft:ice",0.98),0.98);}
 assert_eq!(terrain_speed_bridge::friction("minecraft:packed_ice",0.98),0.6);terrain_speed_bridge::flush(&mut v);
}
#[test]fn actual_move_heads_keep_strafe_then_terrain_and_clip_outside_auto_jump_delta(){
 let res=resources();let mut g=game(&res);let mut s=snapshot();s.settings.fast_climb.mode=terrain_speed::ClimbMode::Clip;
 let _scope=terrain_speed_bridge::enter(Some(s));g.world.blocks.insert((2,-3,4),BlockStateId(1));g.world.blocks.insert((2,-2,4),BlockStateId(2));
 let mut w=motion::MotionWorld{player:g.player.as_mut().unwrap(),scene:&mut g.world,resources:&res,assets:&g.pack.assets,entities:&[],collision_binding:(),health:&()};
 w.move_self(Vec3::new(0.3,0.,-0.4)).unwrap();assert_eq!(w.player.position.y,0.5);assert_eq!(w.player.position.z,5.35);
 assert_eq!(AUTO_JUMP.with(Cell::get),[0.3f32,-0.4f32]);g.native_terrain_flush();assert_eq!(g.previous_position,Vec3::new(2.25,0.5,5.75));
}
#[test]fn actual_shulker_head_and_optimized_piston_head_use_every_move(){
 let res=resources();let mut g=game(&res);let s=snapshot();let _scope=terrain_speed_bridge::enter(Some(s));
 let v=Vec3::new(0.0,0.0,0.0);let changed=piston_scope::run_head(&mut g,v).unwrap();assert_eq!(changed.y,f64::from(0.2872f32));
 let mut w=motion::MotionWorld{player:g.player.as_mut().unwrap(),scene:&mut g.world,resources:&res,assets:&g.pack.assets,entities:&[],collision_binding:(),health:&()};w.shulker_box_push(v).unwrap();
 let (moved,movement)=LAST_MOVE.with(Cell::get).unwrap();assert_eq!(moved,horizon_world::entity_move::MoverType::ShulkerBox);assert_eq!(movement[1],f64::from(0.2872f32));
}
#[test]fn auto_swim_uses_physical_sneak_independent_of_mutated_event_sneak(){
 let res=resources();let g=game(&res);let s=snapshot();let _scope=terrain_speed_bridge::enter(Some(s));let(mut f,mut x,mut jump,mut sneak)=(0.,0.,false,true);
 g.native_movement_input(&mut f,&mut x,&mut jump,&mut sneak);assert!(jump);assert!(sneak);
 drop(_scope);let mut s=s;s.sneak=true;let _scope=terrain_speed_bridge::enter(Some(s));jump=false;sneak=false;g.native_movement_input(&mut f,&mut x,&mut jump,&mut sneak);assert!(!jump);
}
#[test]fn actual_motion_world_friction_drains_prior_nonlocal_queries_before_this_query(){
 let res=resources();let mut g=game(&res);let mut s=snapshot();s.settings.ice_speed.motion_enabled=true;
 let _scope=terrain_speed_bridge::enter(Some(s));terrain_speed_bridge::friction("minecraft:ice",0.98);
 let initial=g.player.as_ref().unwrap().velocity;
 let mut w=motion::MotionWorld{player:g.player.as_mut().unwrap(),scene:&mut g.world,resources:&res,assets:&g.pack.assets,entities:&[],collision_binding:(),health:&()};
 assert_eq!(w.query_friction(BlockPos::new(0,0,0)).unwrap(),0.6);assert_eq!(w.player.velocity.x.to_bits(),(initial.x*0.5*0.5).to_bits());assert_eq!(w.player.velocity.z.to_bits(),(initial.z*0.5*0.5).to_bits());
}
#[test]fn original_entity_turns_bracket_water_and_local_friction_without_combining_multipliers(){
 let res=resources();let mut g=game(&res);let mut s=snapshot();s.settings.ice_speed.motion_enabled=true;
 s.settings.ice_speed.horizontal_motion=0.3;
 let factor=f64::from(0.3f32);let _scope=terrain_speed_bridge::enter(Some(s));
 // The first entity turn completes before Water's PlayerTick handler.
 {let _selection=terrain_speed_bridge::select(terrain_speed_order::Selection::Entity(11));
 assert!(terrain_speed_bridge::selected(11));assert!(!terrain_speed_bridge::selected(12));
 assert_eq!(terrain_speed_bridge::friction_for(11,"minecraft:ice",0.98),0.6);}
 g.native_terrain_flush();assert_eq!(g.player.as_ref().unwrap().velocity.z.to_bits(),(-0.4*factor).to_bits());
 g.native_terrain_player_tick();let water=f64::from(0.44f32);
 assert_eq!(g.player.as_ref().unwrap().velocity.z.to_bits(),water.to_bits());
 let mut w=motion::MotionWorld{player:g.player.as_mut().unwrap(),scene:&mut g.world,resources:&res,assets:&g.pack.assets,entities:&[],collision_binding:(),health:&()};
 w.query_friction(BlockPos::new(0,0,0)).unwrap();assert_eq!(w.player.velocity.z.to_bits(),(water*factor).to_bits());
 g.native_terrain_finish_player();assert_eq!(g.player.as_ref().unwrap().velocity.z.to_bits(),(water*factor).to_bits());
 // Each later entity turn is drained before its impulse or the next turn.
 for _ in 0..2{let _selection=terrain_speed_bridge::select(terrain_speed_order::Selection::Entity(12));terrain_speed_bridge::friction_for(12,"minecraft:ice",0.98);g.native_terrain_flush();}
 assert_eq!(g.player.as_ref().unwrap().velocity.z.to_bits(),(water*factor*factor*factor).to_bits());
}
#[test]fn early_chunk_return_finishes_global_callbacks_and_absent_player_discards_them(){
 let res=resources();let mut g=game(&res);let mut s=snapshot();s.settings.ice_speed.motion_enabled=true;
 {let _scope=terrain_speed_bridge::enter(Some(s));
  terrain_speed_bridge::friction_for(12,"minecraft:ice",0.98);g.native_terrain_flush();
  assert_eq!(g.player.as_ref().unwrap().velocity.z,-0.2);
  g.native_terrain_finish_player();assert_eq!(g.player.as_ref().unwrap().velocity.z,-0.2);}
 {let _scope=terrain_speed_bridge::enter(Some(s));
  terrain_speed_bridge::friction_for(12,"minecraft:ice",0.98);g.player=None;g.native_terrain_finish_player();}
 {let _scope=terrain_speed_bridge::enter(None);
  assert_eq!(terrain_speed_bridge::friction_for(12,"minecraft:ice",0.98),0.98);}
}
#[test]fn actual_mc_snap_boundaries_old_positions_and_direction_rounding_match_adapter(){
 use terrain_speed::ClimbWorld;
 let facts:serde_json::Value=serde_json::from_str(include_str!("minecraft-facts.json")).unwrap();let res=resources();let mut g=game(&res);
 let _scope=terrain_speed_bridge::enter(Some(snapshot()));
 for row in facts["snaps"].as_array().unwrap(){
  let p:[f64;3]=std::array::from_fn(|i|f64::from_bits(row["input"][i].as_str().unwrap().parse().unwrap()));
  let mut w=motion::MotionWorld{player:g.player.as_mut().unwrap(),scene:&mut g.world,resources:&res,assets:&g.pack.assets,entities:&[],collision_binding:(),health:&()};w.abs_snap_to(p).unwrap();
  let got=serde_json::json!([w.player.position.x,w.player.position.y,w.player.position.z].map(|v|v.to_bits().to_string()));assert_eq!(got,row["position"]);
  g.native_terrain_flush();let old=g.previous_position;assert_eq!(serde_json::json!([old.x,old.y,old.z].map(|v|v.to_bits().to_string())),row["oldPosition"]);
 }
 for row in facts["directions"].as_array().unwrap(){
  g.player.as_mut().unwrap().yaw=f32::from_bits(row["yawBits"].as_str().unwrap().parse().unwrap());
  let w=motion::MotionWorld{player:g.player.as_mut().unwrap(),scene:&mut g.world,resources:&res,assets:&g.pack.assets,entities:&[],collision_binding:(),health:&()};let got=format!("{:?}",w.direction()).to_ascii_uppercase();assert_eq!(got,row["direction"].as_str().unwrap());
 }
 assert_eq!(facts["climbBlocks"],serde_json::json!(["minecraft:ladder","minecraft:vine"]));
}
}
