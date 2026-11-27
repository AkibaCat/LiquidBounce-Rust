#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
use horizon_core::{
    geometry::{Aabb, Vec3},
    position::BlockPos,
};
pub use horizon_world::living_air;
mod block_bounce;
mod living_motion;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};
static RUNNING: AtomicBool = AtomicBool::new(false);
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone)]
struct PlayerMotion {
    client_side: bool,
    server_copy: bool,
    replaying: bool,
    client_block_bounce_motion: Option<f32>,
    bounds: Aabb,
    velocity: Vec3,
    strength: f64,
    block_factor: f32,
    boost: Option<i32>,
    sprint: bool,
    syncs: usize,
}
impl Default for PlayerMotion {
    fn default() -> Self {
        Self {
            client_side: true,
            server_copy: false,
            replaying: false,
            client_block_bounce_motion: Some(0.42),
            bounds: Aabb::new(Vec3::new(0.2, 64., 0.2), Vec3::new(0.8, 65.8, 0.8)),
            velocity: Vec3::ZERO,
            strength: 0.42,
            block_factor: 1.,
            boost: None,
            sprint: false,
            syncs: 0,
        }
    }
}
impl PlayerMotion {
    /* IDENTITY */
    fn bounds(&self) -> Aabb {
        self.bounds
    }
}
struct State {
    name: String,
}
#[derive(Default)]
struct Registry {
    states: HashMap<u32, State>,
}
impl Registry {
    fn state(&self, id: u32) -> Option<&State> {
        self.states.get(&id)
    }
}
#[derive(Default)]
struct Assets {
    registry: Registry,
}
struct MotionWorld<'a> {
    player: &'a mut PlayerMotion,
    assets: &'a Assets,
    blocks: HashMap<BlockPos, u32>,
}
impl MotionWorld<'_> {
    fn block(&self, p: BlockPos) -> u32 {
        *self.blocks.get(&p).unwrap_or(&0)
    }
}
impl living_motion::LivingMotionWorld for MotionWorld<'_> {
    type Error = Box<dyn std::error::Error>;
    /* MODIFIER */
    fn velocity(&self) -> Vec3 {
        self.player.velocity
    }
    fn set_velocity(&mut self, v: Vec3) -> Result<()> {
        self.player.velocity = v;
        Ok(())
    }
    fn jump_strength(&mut self) -> Result<f64> {
        Ok(self.player.strength)
    }
    fn block_jump_factor(&mut self) -> Result<f32> {
        Ok(self.player.block_factor)
    }
    fn has_jump_boost(&mut self) -> Result<bool> {
        Ok(self.player.boost.is_some())
    }
    fn jump_boost_amplifier(&mut self) -> Result<i32> {
        Ok(self.player.boost.unwrap())
    }
    fn sprinting(&mut self) -> Result<bool> {
        Ok(self.player.sprint)
    }
    fn yaw(&mut self) -> Result<f32> {
        Ok(0.)
    }
    fn mark_needs_sync(&mut self) {
        self.player.syncs += 1;
    }
    fn no_gravity(&mut self) -> Result<bool> {
        Ok(false)
    }
    fn gravity_attribute(&mut self) -> Result<f64> {
        Ok(0.08)
    }
    fn has_slow_falling(&mut self) -> Result<bool> {
        Ok(false)
    }
    fn controlling_passenger_is_player(&mut self) -> Result<bool> {
        Ok(false)
    }
    fn speed(&mut self) -> Result<f32> {
        Ok(0.1)
    }
    fn omnidirectional_air_mover(&mut self) -> Result<bool> {
        Ok(false)
    }
    fn air_drag_modifier(&mut self) -> Result<f64> {
        Ok(1.)
    }
}
fn assets() -> Assets {
    Assets {
        registry: Registry {
            states: HashMap::from([
                (
                    0,
                    State {
                        name: "minecraft:air".into(),
                    },
                ),
                (
                    1,
                    State {
                        name: "minecraft:slime_block".into(),
                    },
                ),
                (
                    2,
                    State {
                        name: "minecraft:honey_block".into(),
                    },
                ),
                (
                    3,
                    State {
                        name: "minecraft:red_bed".into(),
                    },
                ),
                (
                    4,
                    State {
                        name: "minecraft:stone".into(),
                    },
                ),
            ]),
        },
    }
}
fn jump(player: &mut PlayerMotion, position: BlockPos, state: u32) {
    let assets = assets();
    let mut world = MotionWorld {
        player,
        assets: &assets,
        blocks: HashMap::from([(position, state)]),
    };
    living_motion::jump_from_ground(&mut world).unwrap();
}
#[test]
fn actual_jump_modifier_runs_after_power_and_retains_existing_upward_speed_sprint_and_sync() {
    let mut player = PlayerMotion {
        strength: 0.42,
        block_factor: 0.5,
        boost: Some(1),
        sprint: true,
        ..Default::default()
    };
    jump(&mut player, BlockPos::new(0, 63, 0), 2);
    assert_eq!(
        player.velocity.y,
        f64::from(0.42f32 * 0.5f32 + 0.2f32 + 0.42f32)
    );
    assert_eq!(player.velocity.z, 0.2);
    assert_eq!(player.syncs, 1);
    let mut player = PlayerMotion {
        velocity: Vec3::new(2., 3., 4.),
        ..Default::default()
    };
    jump(&mut player, BlockPos::new(0, 63, 0), 1);
    assert_eq!(player.velocity, Vec3::new(2., 3., 4.));
    assert_eq!(player.syncs, 1);
    let mut player = PlayerMotion {
        client_block_bounce_motion: Some(-5.),
        ..Default::default()
    };
    jump(&mut player, BlockPos::new(0, 63, 0), 3);
    assert_eq!(player.velocity, Vec3::ZERO);
    assert_eq!(player.syncs, 0);
}
#[test]
fn exact_detection_queries_below_full_width_with_inclusive_edges_and_no_collision_shape_filter() {
    let mut player = PlayerMotion {
        bounds: Aabb::new(Vec3::new(-0.1, 64., -0.1), Vec3::new(1., 65.8, 1.)),
        ..Default::default()
    };
    jump(&mut player, BlockPos::new(1, 63, 1), 3);
    assert_eq!(player.velocity.y, f64::from(0.42f32 + 0.42f32));
    let mut player = PlayerMotion::default();
    jump(&mut player, BlockPos::new(0, 64, 0), 1);
    assert_eq!(player.velocity.y, f64::from(0.42f32));
    let mut player = PlayerMotion::default();
    jump(&mut player, BlockPos::new(0, 63, 0), 4);
    assert_eq!(player.velocity.y, f64::from(0.42f32));
}
#[test]
fn disabled_policy_and_every_non_client_identity_preserve_vanilla_jump() {
    for exclusion in 0..4 {
        let mut player = PlayerMotion::default();
        match exclusion {
            0 => player.client_side = false,
            1 => player.server_copy = true,
            2 => player.replaying = true,
            _ => player.client_block_bounce_motion = None,
        }
        jump(&mut player, BlockPos::new(0, 63, 0), 1);
        assert_eq!(player.velocity.y, f64::from(0.42f32));
    }
}
mod mod_api {
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(|m| &**m)
    }
    #[allow(unused_imports)]
    mod render {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/api-source/crates/horizon-client-render/src/gfx/native_render/generated.rs"
        ));
    }
    pub use render::*;
    #[allow(unused_imports)]
    mod host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs"));
    }
    pub use host::*;
    mod hooks {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs"));
    }
    pub use hooks::*;
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: std::sync::OnceLock<Vec<Box<dyn Hooks>>> = std::sync::OnceLock::new();
        MODS.get_or_init(|| vec![Box::new(TestHooks)])
    }
    struct TestHooks;
    impl Hooks for TestHooks {
        fn client_block_bounce(&self, _: &dyn Host, event: &mut ClientBlockBounceEvent) {
            if crate::RUNNING.load(crate::Ordering::SeqCst) {
                event.motion = 0.42;
                event.apply = true;
            }
        }
    }
}
mod game {
    struct Game<'a> {
        window: bool,
        lan_acting_uuid: Option<()>,
        player: Option<crate::PlayerMotion>,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Game<'_> {
        fn native_window(&self) -> bool {
            self.window
        }
    }
    impl Default for Game<'_> {
        fn default() -> Self {
            Self {
                window: true,
                lan_acting_uuid: None,
                player: Some(Default::default()),
                marker: std::marker::PhantomData,
            }
        }
    }
    mod native_hooks {
        use super::Game;
        use crate::mod_api; /* NATIVE */
    }
    mod mod_api_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_host_generated.rs"));
    }
    #[test]
    fn generated_event_copies_configuration_into_only_the_native_window_player() {
        crate::RUNNING.store(true, crate::Ordering::SeqCst);
        let game = Game::default();
        let (mut motion, mut apply) = (0., false);
        game.native_client_block_bounce(&mut motion, &mut apply);
        assert!(apply);
        assert_eq!(motion, 0.42);
        for excluded in 0..5 {
            let mut game = Game::default();
            match excluded {
                0 => game.window = false,
                1 => game.lan_acting_uuid = Some(()),
                2 => game.player = None,
                3 => game.player.as_mut().unwrap().server_copy = true,
                _ => game.player.as_mut().unwrap().replaying = true,
            }
            let (mut motion, mut apply) = (0., false);
            game.native_client_block_bounce(&mut motion, &mut apply);
            assert!(!apply);
            assert_eq!(motion, 0.);
        }
        crate::RUNNING.store(false, crate::Ordering::SeqCst);
        let (mut motion, mut apply) = (0., false);
        game.native_client_block_bounce(&mut motion, &mut apply);
        assert!(!apply);
    }
}
#[test]
fn actual_mc_registry_class_membership_aabb_bounds_and_float_formula_oracles() {
    /* ORACLE */
}
