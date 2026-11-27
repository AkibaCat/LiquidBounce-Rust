#![cfg(test)]
#![allow(clippy::too_many_arguments)] // Preserve the complete production APIs.
extern crate self as horizon_client;
extern crate self as horizon_world;
use horizon_core::geometry::{Aabb, Vec3, border::BorderGeometry};
use horizon_core::position::BlockPos;

pub type Result<T> = std::io::Result<T>;
pub struct CollisionContext;
pub struct PowderSnowSubject;
pub mod world { use super::Vec3; pub struct World { pub moved: Vec3 } impl World { pub fn block(&self, _: i32, _: i32, _: i32) -> usize { 0 } } }
pub mod blocks {
    pub struct Fact { pub name: Box<str> }
    pub struct BlockFacts(pub Fact);
    impl BlockFacts { pub fn get(&self, _: usize) -> &Fact { &self.0 } }
}
pub mod block_collision { pub struct BlockCollisionRegistry<'a>(pub std::marker::PhantomData<&'a ()>); }
pub mod movement_properties { pub struct BoundMovementProperties; }
pub mod assets { pub struct Assets; }
pub mod motion {
    use super::*;
    pub struct FluidResources;
    #[derive(Default)]
    pub struct ClientMovementQueries { pub reset_jump_delay:bool, pub suppress_bounce:bool, pub ignore_levitation:bool, pub ignore_slow_falling:bool }
    pub fn block_speed_factor_at(_: &world::World, _: &movement_properties::BoundMovementProperties, _: Vec3, _: Option<()>) -> f32 { 1.0 }
}
pub mod entity_move { use super::Vec3; pub fn scale_by_block_speed(v:Vec3, f:f32) -> Vec3 { Vec3::new(v.x*f64::from(f),v.y,v.z*f64::from(f)) } }
pub mod entity_rules { use super::Vec3; pub fn scale_air_drag(v:Vec3) -> Vec3 { v.scale(f64::from(0.98f32)) } }
pub mod collisions {
    use super::*;
    #[allow(dead_code)]
    pub struct SceneCollisions<'a> { pub world:&'a world::World, pub binding:&'a block_collision::BlockCollisionRegistry<'a>, pub subject:Option<PowderSnowSubject> }
    #[allow(dead_code)]
    pub struct EntityCollisions<'a> { pub blocks:SceneCollisions<'a>, pub entities:&'a [Aabb], pub border:Option<(&'a BorderGeometry,[f64;2])> }
    impl EntityCollisions<'_> {
        pub fn resolve_with_context(&self, _:Aabb, _:Vec3, _:f64, _:bool, _:CollisionContext) -> Result<Vec3> { Ok(self.blocks.world.moved) }
    }
}
pub mod entities {
    const ITEM_SLIME_BOUNCE:f32 = 0.8;
    __VERTICAL_COLLISION_VELOCITY__
    use super::Vec3;
    __MOVE_APPLIES__
}
use blocks::BlockFacts;
use world::World;
use block_collision::BlockCollisionRegistry;
use movement_properties::BoundMovementProperties;
use collisions::SceneCollisions;
use entities::{move_applies, vertical_collision_velocity};
const ITEM_GRAVITY:f64 = 0.04;
#[derive(Debug)]
struct ItemStep { old_velocity:Vec3, did_move:bool, vertical_contact:Option<BlockPos>, fall_move:Option<(Vec3,Vec3,bool)> }
#[derive(Clone)]
struct ItemEntity {
    ticks:u32, pickup_delay:i64, velocity:Vec3, position:Vec3, previous:Vec3,
    on_ground:bool, id:u32, stuck:Vec3, vertical_collision:bool, horizontal_collision:bool,
}
impl ItemEntity {
    fn new() -> Self { Self {ticks:0,pickup_delay:4,velocity:Vec3::new(0.4,-1.0,0.2),position:Vec3::new(0.5,1.0,0.5),previous:Vec3::ZERO,on_ground:false,id:1,stuck:Vec3::ZERO,vertical_collision:false,horizontal_collision:false} }
    fn has_infinite_pickup_delay(&self) -> bool { false }
    fn get_gravity(&self) -> f64 { ITEM_GRAVITY }
    fn set_fluid_movement(&mut self, _:f32) {}
    fn move_towards_closest_space(&mut self, _: &World, _: &BlockFacts, _:f32) {}
    fn bounds(&self) -> Aabb { Aabb::new(self.position,self.position) }
    fn collision_context(&self) -> CollisionContext { CollisionContext }
    fn vertical_contact(&self, _: &World, _: &BlockCollisionRegistry<'_>, _:Option<&BoundMovementProperties>, _:Vec3) -> Option<BlockPos> { Some(BlockPos::new(0,0,0)) }
    fn slime_step_on(&mut self, _: &World, _: &BlockFacts) {}
    __ITEM_WRAPPER__
    __ITEM_MOVE__
}
fn slime_at(_: &World, facts:&BlockFacts, _:BlockPos) -> bool { &*facts.get(0).name == "minecraft:slime_block" }

mod tnt_production {
    use super::*;
    #[derive(Clone)]
    pub(super) struct PrimedTnt {
        pub position:Vec3,pub previous:Vec3,pub velocity:Vec3,pub on_ground:bool,
        pub fuse:i32,pub fall_distance:f64,pub in_water:bool,
    }
    impl PrimedTnt {
        pub(super) fn new() -> Self { Self {position:Vec3::new(0.5,1.0,0.5),previous:Vec3::ZERO,velocity:Vec3::new(0.4,-1.0,0.2),on_ground:false,fuse:80,fall_distance:2.0,in_water:false} }
        fn get_gravity(&self) -> f64 { 0.04 }
        fn take_stuck(&mut self) -> Option<Vec3> { None }
        fn powder_snow_subject(&self) -> PowderSnowSubject { PowderSnowSubject }
        fn bounds(&self) -> Aabb { Aabb::new(self.position,self.position) }
        fn collision_context(&self) -> CollisionContext { CollisionContext }
        fn check_fall_damage(&mut self, _:f64) -> bool { self.on_ground }
        fn apply_effects_from_blocks(&mut self, _: &World, _: &assets::Assets, _: &BlockFacts, _: &BlockCollisionRegistry<'_>, _:Vec3, _:Option<Vec3>) -> Vec<BlockPos> { Vec::new() }
        fn push_by_fluid(&mut self, _: &World, _: &motion::FluidResources) -> (f64,f64) { (0.0,0.0) }
        __TNT_WRAPPER__
        __TNT_MOVE__
    }
}
struct Game { remote:Option<()>, native:bool, enabled:bool }
impl Game {
    fn native_client_movement_queries(&self, _: &mut bool, bounce:&mut bool, _: &mut bool, _: &mut bool) { if self.native { *bounce=self.enabled; } }
    #[allow(clippy::let_and_return)] // Compile the capture snippet unchanged.
    fn policy(&self) -> bool { __CAPTURE__ client_suppress_bounce }
}

#[test]
fn client_policy_never_enters_a_server_or_headless_game() {
    for (remote,native,enabled,expected) in [
        (Some(()),true,true,true), (Some(()),true,false,false),
        (Some(()),false,true,false), (None,true,true,false), (None,false,true,false),
    ] { assert_eq!(Game {remote,native,enabled}.policy(),expected); }
}
#[test]
fn entire_item_move_suppresses_only_slime_restitution() {
    let world=World {moved:Vec3::new(0.4,-0.5,0.2)};
    let facts=BlockFacts(blocks::Fact {name:"minecraft:slime_block".into()});
    let collision=BlockCollisionRegistry(std::marker::PhantomData);
    let mut vanilla=ItemEntity::new();
    let a=vanilla.tick_move_before_block_effects(&world,&facts,&collision,false,None,(0.0,0.0),&[],None,None);
    assert!(vanilla.velocity.y>0.0);
    let mut unchanged=ItemEntity::new();
    unchanged.tick_move_before_block_effects_with_bounce(&world,&facts,&collision,false,None,(0.0,0.0),&[],None,None,false);
    assert_eq!(vanilla.velocity,unchanged.velocity);
    let mut client=ItemEntity::new();
    let b=client.tick_move_before_block_effects_with_bounce(&world,&facts,&collision,false,None,(0.0,0.0),&[],None,None,true);
    assert_eq!(client.velocity,Vec3::new(0.4,0.0,0.2));
    assert_eq!(a.old_velocity,b.old_velocity);
    assert_eq!(a.did_move,b.did_move);
    assert_eq!(a.vertical_contact,b.vertical_contact);
    assert_eq!(a.fall_move,b.fall_move);
    assert_eq!(vanilla.position,client.position);
    assert_eq!(client.pickup_delay,3);
    assert!(client.on_ground && client.vertical_collision);
    let mut no_physics=ItemEntity::new();
    no_physics.tick_move_before_block_effects_with_bounce(&world,&facts,&collision,true,None,(0.0,0.0),&[],None,None,true);
    assert_eq!(no_physics.velocity.y,-1.04);
    assert!(!no_physics.on_ground);
}
#[test]
fn entire_tnt_move_retains_custom_ground_drag_fuse_and_default_entry() {
    let world=World {moved:Vec3::new(0.4,-0.5,0.2)};
    let facts=BlockFacts(blocks::Fact {name:"minecraft:slime_block".into()});
    let collision=BlockCollisionRegistry(std::marker::PhantomData);
    let mut vanilla=tnt_production::PrimedTnt::new();
    vanilla.tick_among(&world,&collision,&assets::Assets,&facts,&motion::FluidResources,None,None,&[]).unwrap();
    let mut explicit=tnt_production::PrimedTnt::new();
    explicit.tick_among_with_bounce(&world,&collision,&assets::Assets,&facts,&motion::FluidResources,None,None,&[],false).unwrap();
    assert_eq!(vanilla.velocity,explicit.velocity);
    assert!(vanilla.velocity.y<0.0); // TNT's own *-0.5 still follows slime restitution.
    let mut client=tnt_production::PrimedTnt::new();
    client.tick_among_with_bounce(&world,&collision,&assets::Assets,&facts,&motion::FluidResources,None,None,&[],true).unwrap();
    assert_eq!(client.velocity.y,0.0);
    assert_eq!(vanilla.velocity.x,client.velocity.x);
    assert_eq!(vanilla.velocity.z,client.velocity.z);
    assert_eq!(vanilla.position,client.position);
    assert_eq!(client.fuse,79);
    assert!(client.on_ground);
}
