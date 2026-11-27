// SPDX-License-Identifier: GPL-3.0-or-later
//! Real Horizon world access for Agent navigation. Live access is scoped to one
//! game-thread call; only owned copy-on-write chunk snapshots enter workers.
use super::*;
use crate::navigation as nav;
use nav::{aabb::Direction as NavDirection,geometry::{BlockPos as NavPos,BlockRegion,Vec3 as NavVec},terrain::{BlockKind,Terrain,TerrainLevel,TerrainState},voxel_shape::VoxelShape as NavShape};
use std::{cell::{Cell,RefCell},marker::PhantomData,rc::Rc,sync::Arc};

struct ActiveGame{pointer:*mut (),borrowed:Cell<bool>}
thread_local!{static ACTIVE_GAME:RefCell<Option<ActiveGame>>=const{RefCell::new(None)};}
struct ScopeExit;
impl Drop for ScopeExit{fn drop(&mut self){ACTIVE_GAME.with(|slot|{let old=slot.borrow_mut().take();assert!(old.is_some_and(|s|!s.borrowed.get()),"navigation scope exited while borrowed");});}}
struct BorrowExit;
impl Drop for BorrowExit{fn drop(&mut self){ACTIVE_GAME.with(|slot|slot.borrow().as_ref().expect("navigation scope").borrowed.set(false));}}
/// The input exclusive reference is suspended for the entire callback. The
/// callback receives no Game reference; every reborrow passes the same guard.
/// Nested scopes and recursive read/write access are explicitly rejected.
pub(super) fn with_scope<R>(game:&mut Game<'_>,run:impl FnOnce()->R)->R{
 ACTIVE_GAME.with(|slot|{let mut slot=slot.borrow_mut();assert!(slot.is_none(),"nested navigation game scope");*slot=Some(ActiveGame{pointer:(game as *mut Game<'_>).cast(),borrowed:Cell::new(false)});});
 let _scope=ScopeExit;run()
}
fn pointer()->*mut (){ACTIVE_GAME.with(|slot|{let slot=slot.borrow();let active=slot.as_ref().expect("navigation live access outside game tick");assert!(!active.borrowed.replace(true),"recursive navigation Game borrow");active.pointer})}
pub(super) fn read<R>(access:impl for<'s> FnOnce(&'s Game<'s>)->R)->R{
 let pointer=pointer();let _borrow=BorrowExit;
 // SAFETY: with_scope owns the sole &mut Game, and pointer() admits exactly
 // one reborrow. HRTB forces the result to be independent of the borrow.
 // ScopeExit clears the pointer on both normal return and unwinding.
 unsafe{access(&*pointer.cast::<Game<'_>>())}
}
pub(super) fn write<R>(access:impl for<'s> FnOnce(&'s mut Game<'s>)->R)->R{
 let pointer=pointer();let _borrow=BorrowExit;
 // SAFETY: same exclusivity, thread locality and non-escape rule as read().
 unsafe{access(&mut *pointer.cast::<Game<'_>>())}
}
pub(super) fn current_epoch(game:&Game<'_>)->u64{game.remote.as_ref().and_then(|r|r.native_epoch()).or_else(||game.native_lifecycle.epoch()).map_or(0,|e|e.get())}
pub(super) fn vec(v:Vec3)->NavVec{NavVec::new(v.x,v.y,v.z)}
pub(super) fn core_vec(v:NavVec)->Vec3{Vec3::new(v.x,v.y,v.z)}
pub(super) fn pos(p:NavPos)->horizon_core::position::BlockPos{horizon_core::position::BlockPos::new(p.x,p.y,p.z)}
pub(super) fn shape(shape:&horizon_core::geometry::VoxelShape)->NavShape{NavShape::new(shape.boxes().iter().map(|b|{let a=b.min_position();let z=b.max_position();[a.x,a.y,a.z,z.x,z.y,z.z]}).collect())}

pub(super) struct NavigationRegistry{
 pub resources:Arc<Resources>,
 pub registry:&'static horizon_world::registry::BlockRegistry,
 pub collision:Arc<BlockCollisionRegistry<'static>>,
 facts:Vec<horizon_client::blocks::StateFacts>,
 movement:Vec<horizon_world::movement_properties::MovementBlock>,
 climbable:std::collections::BTreeSet<String>,
}
impl NavigationRegistry{
 pub fn capture(game:&Game<'_>)->Arc<Self>{
  let registry=game.pack.assets.registry;
  Arc::new(Self{resources:game.res.clone(),registry,collision:game.res.core.collision.clone(),facts:(0..registry.len()).map(|id|game.res.facts().get(BlockStateId(id as u32)).clone()).collect(),movement:(0..registry.len()).map(|id|game.motion.blocks.get(BlockStateId(id as u32)).expect("bound movement registry")).collect(),climbable:game.motion.climbable.clone()})
 }
 pub fn state(self:&Arc<Self>,id:BlockStateId)->NativeState{NativeState{registry:self.clone(),id}}
 pub fn default_state(self:&Arc<Self>,name:&str)->NativeState{self.state(self.registry.default_state(name).expect("vanilla navigation block"))}
}
#[derive(Clone)]
pub(super) struct NativeState{pub registry:Arc<NavigationRegistry>,pub id:BlockStateId}
impl NativeState{
 pub fn facts(&self)->&horizon_client::blocks::StateFacts{&self.registry.facts[self.id.0 as usize]}
 pub fn name(&self)->&str{&self.facts().name}
 pub fn has_class(&self,class:&str)->bool{self.facts().classes.iter().any(|c|&**c==class||c.rsplit('.').next()==Some(class))}
 pub fn property(&self,key:&str)->Option<&str>{self.registry.registry.state(self.id)?.properties.get(key).map(String::as_str)}
 fn coefficients(&self)->horizon_world::movement_properties::MovementCoefficients{self.registry.movement[self.id.0 as usize].coefficients}
}
impl TerrainState for NativeState{
 fn same_instance(&self,other:&Self)->bool{Arc::ptr_eq(&self.registry,&other.registry)&&self.id==other.id}
 fn same_block(&self,other:&Self)->bool{self.facts().name==other.facts().name}
 fn same_fluid(&self,other:&Self)->bool{self.facts().fluid==other.facts().fluid}
 fn is_hazard(&self)->bool{self.facts().fluid==Fluid::Lava||self.has_class("BaseFireBlock")||matches!(self.name(),"minecraft:cobweb"|"minecraft:sweet_berry_bush"|"minecraft:powder_snow"|"minecraft:wither_rose"|"minecraft:magma_block"|"minecraft:campfire"|"minecraft:soul_campfire"|"minecraft:cactus")}
 fn is_water(&self)->bool{self.facts().fluid==Fluid::Water}
 fn has_liquid(&self)->bool{self.facts().fluid!=Fluid::None}
 fn is_climbable(&self)->bool{self.registry.climbable.contains(self.name())}
 fn kind(&self,kind:BlockKind)->bool{match kind{BlockKind::Door=>self.has_class("DoorBlock"),BlockKind::OpensByHand=>(self.has_class("DoorBlock")&&self.name()!="minecraft:iron_door")||(self.has_class("TrapDoorBlock")&&self.name()!="minecraft:iron_trapdoor")||self.has_class("FenceGateBlock"),BlockKind::FenceGate=>self.has_class("FenceGateBlock"),BlockKind::Trapdoor=>self.has_class("TrapDoorBlock"),BlockKind::IronTrapdoor=>self.name()=="minecraft:iron_trapdoor",BlockKind::Ladder=>self.has_class("LadderBlock")}}
 fn open(&self)->Option<bool>{self.property("open").map(|v|v=="true")}
 fn with_open(&self,open:bool)->Self{let mut state=self.registry.registry.state(self.id).expect("navigation state id").clone();state.properties.insert("open".into(),open.to_string());self.registry.state(self.registry.registry.id(&state).expect("open state property"))}
 fn is_upper_half(&self)->bool{self.property("half")==Some("upper")}
 fn horizontal_facing(&self)->Option<NavDirection>{match self.property("facing")?{"north"=>Some(NavDirection::North),"south"=>Some(NavDirection::South),"west"=>Some(NavDirection::West),"east"=>Some(NavDirection::East),_=>None}}
 fn bounce_restitution(&self)->f32{self.coefficients().bounce_restitution}
 fn collision_shape<L:TerrainLevel<Self>>(&self,level:&L,p:NavPos)->NavShape{level.collision_shape_for(self,p).expect("native level supplies contextual collision")}
}
impl nav::walk::WalkState for NativeState{
 fn is_air(&self)->bool{matches!(self.name(),"minecraft:air"|"minecraft:cave_air"|"minecraft:void_air")}
 fn can_be_replaced(&self)->bool{self.facts().replaceable}
 fn is_falling(&self)->bool{self.has_class("FallingBlock")}
 fn is_slime(&self)->bool{self.name()=="minecraft:slime_block"}
 fn friction(&self)->f32{self.coefficients().friction}
 fn speed_factor(&self)->f32{self.coefficients().speed_factor}
 fn jump_factor(&self)->f32{self.coefficients().jump_factor}
}
impl nav::simulation::block_grid::SimulationState for NativeState{
 fn is_fence(&self)->bool{self.has_class("FenceBlock")}
 fn is_wall(&self)->bool{self.has_class("WallBlock")}
 fn is_fence_gate(&self)->bool{self.has_class("FenceGateBlock")}
 fn is_cobweb(&self)->bool{self.name()=="minecraft:cobweb"}
 fn is_step_air(&self)->bool{matches!(self.name(),"minecraft:air"|"minecraft:void_air")}
 fn fall_distance_reduction(&self)->f32{self.coefficients().fall_distance_reduction}
}
impl nav::walk_control::body::ControlState for NativeState{
 fn is_farmland(&self)->bool{self.has_class("FarmBlock")||self.name()=="minecraft:farmland"}
 fn is_bottom_stair(&self)->bool{self.has_class("StairBlock")&&self.property("half")==Some("bottom")}
}
impl nav::walk_control::break_times::BreakState for NativeState{
 fn empty_level_hardness(&self)->f32{self.facts().hardness}
 fn requires_correct_tool(&self)->bool{self.facts().requires_tool}
}
impl nav::ride::moves::RideState for NativeState{fn friction(&self)->f32{self.coefficients().friction}}
impl nav::ride::boating::BoatState for NativeState{fn is_air(&self)->bool{nav::walk::WalkState::is_air(self)}}

#[derive(Clone)]
pub(super) struct NativeLevel{pub registry:Arc<NavigationRegistry>,pub epoch:u64,thread:PhantomData<Rc<()>>}
impl NativeLevel{
 pub fn new(registry:Arc<NavigationRegistry>,epoch:u64)->Self{Self{registry,epoch,thread:PhantomData}}
 pub fn game<R>(&self,f:impl for<'s>FnOnce(&'s Game<'s>)->R)->R{read(|game|{assert_eq!(current_epoch(game),self.epoch,"navigation handle belongs to a retired world");f(game)})}
}
fn collision(game:&Game<'_>,state:BlockStateId,p:NavPos)->NavShape{
 let player=game.player.as_ref().expect("navigation player");
 let context=CollisionContext::for_entity(CollisionEntity{identity:horizon_world::entity_obstruction::EntityIdentity(0),kind:horizon_world::collision_context::EntityContextKind::Player,y:player.position.y,descending:player.shift_key_down,main_hand:Some(CollisionItem::EMPTY)},false);
 let mut world=horizon_client::collisions::SceneCollisions{world:&game.world,binding:game.collision,subject:Some(player.powder_snow_subject())};
 shape(&game.collision.collision_shape_with_context(&mut world,state,pos(p),&context).expect("native contextual collision"))
}
impl TerrainLevel<NativeState> for NativeLevel{
 fn block_state(&self,p:NavPos)->NativeState{self.registry.state(self.game(|game|game.world.block(p.x,p.y,p.z)))}
 fn void_air(&self)->NativeState{self.registry.default_state("minecraft:void_air")}
 fn is_known(&self,p:NavPos)->bool{self.game(|game|p.y>=game.world.shape.min_y&&p.y<game.world.shape.min_y+game.world.shape.height&&game.world.chunks.contains_key(&(p.x>>4,p.z>>4)))}
 fn is_inside_border(&self,x:f64,z:f64)->bool{self.game(|game|game.border.contains(x,z))}
 fn min_y(&self)->i32{self.game(|game|game.world.shape.min_y)}
 fn height(&self)->i32{self.game(|game|game.world.shape.height)}
 fn collision_shape_for(&self,state:&NativeState,p:NavPos)->Option<NavShape>{Some(self.game(|game|collision(game,state.id,p)))}
}
impl nav::glide::body::GlideLevel<NativeState> for NativeLevel{
 fn has_ceiling(&self)->bool{self.game(|game|game.world.shape.has_ceiling)}
 fn has_chunk_at(&self,p:NavPos)->bool{self.game(|game|game.world.chunks.contains_key(&(p.x>>4,p.z>>4)))}
 fn has_column(&self,x:i32,z:i32)->bool{self.game(|game|game.world.chunks.contains_key(&(x>>4,z>>4)))}
 fn column_height(&self,x:i32,z:i32)->i32{self.game(|game|game.world.motion_blocking(x,z,|id|{let facts=game.res.facts().get(id);facts.legacy_solid||facts.fluid!=Fluid::None}))}
}
pub(super) struct SnapshotLevel{
 pub registry:Arc<NavigationRegistry>,pub world:World,pub epoch:u64,border:horizon_client::worldborder::WorldBorder,
 player_y:f64,descending:bool,subject:horizon_client::collisions::PowderSnowSubject,
}
impl SnapshotLevel{
 pub fn capture(level:&NativeLevel,region:BlockRegion)->Option<Self>{level.game(|game|{
  let player=game.player.as_ref()?;let mut world=World::new(game.world.air,0);world.shape=game.world.shape;
  for z in (region.min.z>>4)..=(region.max.z>>4){for x in (region.min.x>>4)..=(region.max.x>>4){if let Some(chunk)=game.world.chunks.get(&(x,z)){world.chunks.insert((x,z),chunk.clone());}}}
  world.moving_pistons=game.world.moving_pistons.clone();world.shulkers=game.world.shulkers.clone();
  Some(Self{registry:level.registry.clone(),world,epoch:level.epoch,border:game.border.clone(),player_y:player.position.y,descending:player.shift_key_down,subject:player.powder_snow_subject()})
 })}
}
impl TerrainLevel<NativeState> for SnapshotLevel{
 fn block_state(&self,p:NavPos)->NativeState{self.registry.state(self.world.block(p.x,p.y,p.z))}
 fn void_air(&self)->NativeState{self.registry.default_state("minecraft:void_air")}
 fn is_known(&self,p:NavPos)->bool{p.y>=self.world.shape.min_y&&p.y<self.world.shape.min_y+self.world.shape.height&&self.world.chunks.contains_key(&(p.x>>4,p.z>>4))}
 fn is_inside_border(&self,x:f64,z:f64)->bool{self.border.contains(x,z)}
 fn min_y(&self)->i32{self.world.shape.min_y}
 fn height(&self)->i32{self.world.shape.height}
 fn collision_shape_for(&self,state:&NativeState,p:NavPos)->Option<NavShape>{
  let context=CollisionContext::for_entity(CollisionEntity{identity:horizon_world::entity_obstruction::EntityIdentity(0),kind:horizon_world::collision_context::EntityContextKind::Player,y:self.player_y,descending:self.descending,main_hand:Some(CollisionItem::EMPTY)},false);
  let mut world=horizon_client::collisions::SceneCollisions{world:&self.world,binding:&self.registry.collision,subject:Some(self.subject)};
  Some(shape(&self.registry.collision.collision_shape_with_context(&mut world,state.id,pos(p),&context).expect("snapshot contextual collision")))
 }
}
pub(super) type SearchTerrain=Terrain<NativeState,nav::staged_search::SharedLevel<SnapshotLevel>>;
pub(super) struct SearchLevel{pub live:NativeLevel,pub width:f64,pub height:f64,pub can_open_doors:bool}
impl nav::background::BackgroundLevel<SearchTerrain> for SearchLevel{
 fn snapshot(&mut self,region:BlockRegion)->Option<SearchTerrain>{Some(Terrain::new(nav::staged_search::SharedLevel::new(SnapshotLevel::capture(&self.live,region)?),self.width,self.height,self.can_open_doors))}
 fn has_level(&mut self)->bool{self.live.game(|game|game.player.is_some())}
 fn invalidate(&mut self,terrain:&mut SearchTerrain,p:NavPos)->bool{
  terrain.invalidate(p,&self.live.block_state(p))
 }
}
impl nav::replanner::ReplannerLevel<SearchTerrain> for SearchLevel{fn is_known(&mut self,p:NavPos)->bool{self.live.is_known(p)}}
