#![allow(dead_code)]
extern crate self as horizon_client;
pub mod actions {
    #[derive(Clone, Copy)]
    pub enum Action { Jump, Sneak }
}
use horizon_core::{geometry::Vec3, position::BlockPos};
pub use horizon_world::{collision_context, registry};
type Result<T> = std::result::Result<T, ()>;
#[path = "__AIR_JUMP_PATH__"]
mod air_jump;
#[path = "__NO_PUSH_PATH__"]
mod no_push;
#[allow(clippy::type_complexity)] // The complete unchanged production iterator signature.
mod scan {
    __COLLISION_SCAN__
}

mod health {
    #[derive(Clone, Debug)]
    pub enum HealthEvent { Statistic { name: &'static str, amount: u32 } }
}
#[derive(Clone, Debug)]
enum MotionEvent { ClientJump { on_ground: bool }, Health(health::HealthEvent) }
__QUERIES__
#[derive(Clone, Default)]
struct Ground { on_ground: bool }
#[derive(Clone, Default)]
struct Fluids { was_touching_water: bool }
#[derive(Clone, Copy)]
struct ModFluid { height: f64 }
#[derive(Clone)]
struct PlayerMotion {
    client_side: bool, server_copy: bool, replaying: bool,
    client_airjump_push_queries: ClientAirJumpPushQueries,
    ground: Ground, fluids: Fluids, jumping: bool, flying: bool,
    no_jump_delay: i32, velocity: Vec3, water: f64, lava: f64,
    swim: Option<ModFluid>, entity_id: i32, sprinting: bool,
    events: Vec<MotionEvent>, exhaustion: f32, jump_strength: f64,
}
impl PlayerMotion {
    fn new() -> Self { Self {
        client_side: true, server_copy: false, replaying: false,
        client_airjump_push_queries: ClientAirJumpPushQueries::default(),
        ground: Ground::default(), fluids: Fluids::default(), jumping: true, flying: false,
        no_jump_delay: 0, velocity: Vec3::ZERO, water: 0.0, lava: 0.0,
        swim: None, entity_id: 7, sprinting: false, events: Vec::new(), exhaustion: 0.0, jump_strength: 0.42,
    } }
    __PLAYER_QUERY__
    fn is_in_lava(&self) -> bool { self.lava > 0.0 }
    fn swim_fluid(&self) -> Option<ModFluid> { self.swim }
    fn fluid_height(&self, kind: u8) -> f64 { if kind == LAVA { self.lava } else { self.water } }
    fn fluid_jump_threshold(&self) -> f64 { 0.4 }
    fn cause_food_exhaustion(&mut self, value: f32) { self.exhaustion += value; }
}
const LAVA: u8 = 1;
const WATER: u8 = 0;
fn native_living_jump(_: u32, _: &str) {}
struct MotionWorld { player: PlayerMotion }
mod living_motion {
    pub use horizon_world::living_motion::jump_from_ground;
}
impl horizon_world::living_motion::LivingMotionWorld for MotionWorld {
    type Error = ();
    fn velocity(&self) -> Vec3 { self.player.velocity }
    fn set_velocity(&mut self, value: Vec3) -> Result<()> { self.player.velocity = value; Ok(()) }
    fn jump_strength(&mut self) -> Result<f64> { Ok(self.player.jump_strength) }
    fn block_jump_factor(&mut self) -> Result<f32> { Ok(1.0) }
    fn has_jump_boost(&mut self) -> Result<bool> { Ok(false) }
    fn jump_boost_amplifier(&mut self) -> Result<i32> { Ok(0) }
    fn sprinting(&mut self) -> Result<bool> { Ok(self.player.sprinting) }
    fn yaw(&mut self) -> Result<f32> { Ok(0.0) }
    fn mark_needs_sync(&mut self) {}
    fn no_gravity(&mut self) -> Result<bool> { Ok(false) }
    fn gravity_attribute(&mut self) -> Result<f64> { Ok(0.08) }
    fn has_slow_falling(&mut self) -> Result<bool> { Ok(false) }
    fn controlling_passenger_is_player(&mut self) -> Result<bool> { Ok(false) }
    fn speed(&mut self) -> Result<f32> { Ok(0.1) }
    fn omnidirectional_air_mover(&mut self) -> Result<bool> { Ok(false) }
    fn air_drag_modifier(&mut self) -> Result<f64> { Ok(0.0) }
}
#[derive(Clone, Copy)]
struct FluidState;
fn flow_at(_: &mut MotionWorld, _: FluidState, _: BlockPos) -> Result<Vec3> { Ok(Vec3::new(0.25,0.0,-0.5)) }
impl MotionWorld {
    __JUMP_FROM_GROUND__
    __JUMP_PHASE__
    __FLUID_FLOW__
}

mod mount {
    use super::*;
    #[derive(Default)]
    pub struct Extra { pub client_side: bool, pub client_air_jump: bool }
    #[derive(Clone, Copy)]
    pub struct Rider { pub client: bool }
    #[derive(Default)]
    pub struct Ai { pub jumping: bool, pub no_jump_delay: i32 }
    #[derive(Default)]
    pub struct Fluids { pub in_lava: bool, pub in_water: bool, pub water: f64, pub lava: f64 }
    pub struct Mob { pub extra: Extra, pub rider: Option<Rider>, pub ai: Ai,
        pub fluids: Fluids, pub on_ground: bool, pub velocity: Vec3, pub jumps: usize }
    pub struct MobWorld;
    impl Mob {
        pub fn new() -> Self { Self { extra: Extra::default(), rider: None, ai: Ai { jumping:true,..Ai::default() },
            fluids:Fluids::default(),on_ground:false,velocity:Vec3::ZERO,jumps:0 } }
        fn affected_by_fluids(&self) -> bool { true }
        fn swim_fluid(&self) -> Option<ModFluid> { None }
        fn fluid_jump_threshold(&self) -> f64 { 0.4 }
        fn jump_from_ground(&mut self, _: &MobWorld) { self.jumps += 1; self.velocity.y = f64::from(0.42f32); }
        fn jump_in_liquid(&mut self, _: bool) { self.velocity.y += f64::from(0.04f32); }
        __MOB_HANDLE_JUMP__
    }
}

mod fishing {
    use super::*;
    pub enum State { Flying, Hooked(u32) }
    pub struct Remote { pub entity_id: i32 }
    pub struct Hook { pub id: u32, pub owner: u32, pub state: State, pub position: Vec3 }
    pub struct Mob { pub id: u32, pub position: Vec3 }
    pub struct Game { pub allowed: bool, pub remote: Option<Remote>, pub remote_hooks: Vec<Hook>,
        pub mobs: Vec<Mob>, pub player: Option<PlayerMotion> }
    __FISHING_PULL__
    impl Game {
        fn native_client_airjump_push_policy(&self) -> [bool;7] { [false,false,false,true,true,self.allowed,true] }
        __FISHING_EVENT__
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::{geometry::{Aabb,VoxelShape},position::ChunkPos};
    use scan::{BlockCollisionWorld,BlockCollisionScan,CollisionBlock,with_client_ghost_block,update_client_ghost_block_y};
    use collision_context::CollisionContext;
    use registry::BlockStateId;

    #[test]
    fn air_jump_precedes_ground_and_liquid_decisions_and_sets_cooldown() {
        let mut world = MotionWorld { player:PlayerMotion::new() };
        world.player.client_airjump_push_queries.air_jump = true;
        world.jump_phase().unwrap();
        assert_eq!(world.player.velocity.y,f64::from(0.42f32));
        assert_eq!(world.player.no_jump_delay,10);
        assert!(matches!(world.player.events[0],MotionEvent::ClientJump {on_ground:false}));
        world.player.velocity.y = -0.2;
        world.jump_phase().unwrap();
        assert_eq!(world.player.velocity.y,-0.2);
        assert_eq!(world.player.no_jump_delay,10);
        world.player.no_jump_delay=0;
        world.player.fluids.was_touching_water=true;
        world.player.water=0.9;
        world.jump_phase().unwrap();
        assert_eq!(world.player.velocity.y,f64::from(0.42f32)+f64::from(0.04f32));
    }
    #[test]
    fn double_jump_consumes_on_original_ground_even_with_zero_power() {
        let mut world = MotionWorld { player:PlayerMotion::new() };
        world.player.client_airjump_push_queries=ClientAirJumpPushQueries {air_jump:true,double_jump_mode:true,suppress_liquid_flow:false};
        world.player.jump_strength=0.0;
        world.jump_phase().unwrap();
        assert!(!world.player.client_airjump_push_queries.air_jump);
        assert_eq!(world.player.velocity,Vec3::ZERO);
        assert_eq!(world.player.no_jump_delay,10);
        assert!(matches!(world.player.events[0],MotionEvent::ClientJump {on_ground:false}));
        world.player.ground.on_ground=true;
        world.player.client_airjump_push_queries.air_jump=true;
        world.player.no_jump_delay=0;
        world.jump_phase().unwrap();
        assert!(world.player.client_airjump_push_queries.air_jump);
    }
    #[test]
    fn client_policy_is_absent_for_server_replay_and_shared_mounts() {
        let policy=ClientAirJumpPushQueries {air_jump:true,double_jump_mode:true,suppress_liquid_flow:true};
        for (client_side,server_copy,replaying) in [(false,false,false),(true,true,false),(true,false,true)] {
            let mut world=MotionWorld {player:PlayerMotion {client_side,server_copy,replaying,
                client_airjump_push_queries:policy,..PlayerMotion::new()}};
            world.jump_phase().unwrap();
            assert_eq!(world.player.velocity,Vec3::ZERO);
            assert!(world.player.events.is_empty());
            assert_eq!(world.player.client_airjump_push_queries,policy);
        }
        for (client_side,server_rider) in [(false,false),(true,true),(true,false)] {
            let mut mob=mount::Mob::new();
            mob.extra.client_side=client_side;
            mob.extra.client_air_jump=true;
            mob.rider=Some(mount::Rider {client:server_rider});
            mob.handle_jumping(&mount::MobWorld);
            assert_eq!(mob.jumps,usize::from(client_side&&!server_rider));
        }
    }
    #[test]
    fn flow_suppression_keeps_fluid_immersion_and_restores_original_flow() {
        let mut world=MotionWorld {player:PlayerMotion::new()};
        world.player.water=0.9;
        world.player.fluids.was_touching_water=true;
        world.player.client_airjump_push_queries.suppress_liquid_flow=true;
        assert_eq!(world.flow(FluidState,BlockPos::default()).unwrap(),Vec3::ZERO);
        assert!(world.player.fluids.was_touching_water);
        assert_eq!(world.player.water,0.9);
        world.player.client_airjump_push_queries.suppress_liquid_flow=false;
        assert_eq!(world.flow(FluidState,BlockPos::default()).unwrap(),Vec3::new(0.25,0.0,-0.5));
        world.player.client_airjump_push_queries.suppress_liquid_flow=true;
        world.player.server_copy=true;
        assert_eq!(world.flow(FluidState,BlockPos::default()).unwrap(),Vec3::new(0.25,0.0,-0.5));
    }

    struct Air { missing:bool, original_calls:usize }
    impl BlockCollisionWorld for Air {
        type Error=std::io::Error;
        type Chunk=();
        fn collision_chunk(&mut self,_:ChunkPos)->std::io::Result<Option<()>> {Ok((!self.missing).then_some(()))}
        fn collision_block(&mut self,_:(),_:BlockPos)->std::io::Result<CollisionBlock> {
            Ok(CollisionBlock {state:BlockStateId(0),large_collision_shape:false,moving_piston:false,empty_shape:true})
        }
        fn is_suffocating(&mut self,_:(),_:CollisionBlock,_:BlockPos)->std::io::Result<bool> {Ok(false)}
        fn context_shape(&mut self,_:CollisionBlock,_:BlockPos,_:&CollisionContext<'_>)->std::io::Result<VoxelShape> {
            self.original_calls+=1; Ok(VoxelShape::empty())
        }
        fn section_is_empty(&mut self,_:(),_:i32)->std::io::Result<bool> {Ok(true)}
        fn may_reach_out(&mut self,_:(),_:i32)->std::io::Result<bool> {Ok(false)}
    }
    fn hits(world:&mut Air,y:f64)->Vec<scan::BlockCollisionHit> {
        let bounds=Aabb::new(Vec3::new(0.1,y,0.1),Vec3::new(0.9,y+0.6,0.9));
        BlockCollisionScan::new(world,CollisionContext::empty(false),bounds,false,10000).unwrap().map(|h|h.unwrap()).collect()
    }
    #[test]
    fn ghost_block_modifies_iterator_air_below_player_and_preserves_original_query() {
        let mut air=Air {missing:false,original_calls:0};
        assert!(hits(&mut air,-0.3).is_empty());
        assert_eq!(air.original_calls,0);
        with_client_ghost_block(Some(0),|| {
            let below=hits(&mut air,-0.3);
            assert!(!below.is_empty());
            assert!(below.iter().all(|hit|hit.position.y<0));
            assert!(air.original_calls>0);
            assert!(hits(&mut air,0.1).is_empty());
            // Source event reads the player's current blockPosition, updated
            // after an actual local move; later support scans see that value.
            update_client_ghost_block_y(1);
            assert!(!hits(&mut air,0.1).is_empty());
        });
        assert!(hits(&mut air,0.1).is_empty());
        with_client_ghost_block(Some(0),|| {
            assert!(hits(&mut Air {missing:true,original_calls:0},-0.3).is_empty());
        });
    }
    #[test]
    fn ghost_scope_restores_on_nested_server_scope_thread_and_unwind() {
        with_client_ghost_block(Some(0),|| {
            with_client_ghost_block(None,||assert!(hits(&mut Air {missing:false,original_calls:0},-0.3).is_empty()));
            assert!(!hits(&mut Air {missing:false,original_calls:0},-0.3).is_empty());
            assert!(std::thread::spawn(||hits(&mut Air {missing:false,original_calls:0},-0.3).is_empty()).join().unwrap());
        });
        let unwind=std::panic::catch_unwind(||with_client_ghost_block(Some(0),||panic!("test unwind")));
        assert!(unwind.is_err());
        assert!(hits(&mut Air {missing:false,original_calls:0},-0.3).is_empty());
    }
    #[test]
    fn fishing_event31_only_pulls_the_hooked_local_player_when_allowed() {
        let mut game=fishing::Game {allowed:false,remote:Some(fishing::Remote {entity_id:7}),
            remote_hooks:vec![fishing::Hook {id:100,owner:20,state:fishing::State::Hooked(7),position:Vec3::new(0.0,1.0,0.0)}],
            mobs:vec![fishing::Mob {id:20,position:Vec3::new(10.0,3.0,0.0)}],player:Some(PlayerMotion::new())};
        game.remote_hook_pull_event(100);
        assert_eq!(game.player.as_ref().unwrap().velocity,Vec3::ZERO);
        game.allowed=true;
        game.remote_hook_pull_event(100);
        assert_eq!(game.player.as_ref().unwrap().velocity,Vec3::new(1.0,0.2,0.0));
        game.remote_hooks[0].state=fishing::State::Hooked(8);
        game.remote_hook_pull_event(100);
        assert_eq!(game.player.as_ref().unwrap().velocity,Vec3::new(1.0,0.2,0.0));
    }
}
