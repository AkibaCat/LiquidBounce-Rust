#![cfg(test)]
#![allow(dead_code)]
extern crate self as horizon_client;
use horizon_core::{geometry::Vec3, position::BlockPos};
use horizon_world::{
    entity_move::{self, CollisionFlags, EntityMoveWorld, MovementHistory, MoverType},
    movement_emission::MovementEmission,
    registry::BlockStateId,
};
use strafe::{Strafe, StrafeContext};
type Result<T> = std::result::Result<T, std::io::Error>;
mod strafe;

#[test]
fn unchanged_kotlin_module_and_actual_mc_vector_bits() {
    let commands = include_str!("commands.txt");
    let vectors = include_str!("vectors.txt");
    assert_eq!(commands.lines().count(), vectors.lines().count());
    for (index, (command, want)) in commands.lines().zip(vectors.lines()).enumerate() {
        let t: Vec<_> = command.split(' ').collect();
        let f = |i: usize| t[i].parse::<f32>().unwrap();
        let state = Strafe {
            strength_in_air: f(1),
            strength_on_ground: f(2),
            strict_movement: t[3] == "true",
        };
        let movement = [
            t[10].parse().unwrap(),
            t[11].parse().unwrap(),
            t[12].parse().unwrap(),
        ];
        let actual = state.move_event(
            t[9] == "true",
            StrafeContext {
                self_movement: t[8] == "0",
                on_ground: t[4] == "true",
                moving: f(6) != 0.0 || f(7) != 0.0,
                input: t[5].parse().unwrap(),
                yaw: f(0),
            },
            movement,
            |angle| {
                [
                    horizon_core::jmath::sin(angle),
                    horizon_core::jmath::cos(angle),
                ]
            },
        );
        let expected: Vec<u64> = want.split(',').map(|v| v.parse().unwrap()).collect();
        assert_eq!(
            actual.map(f64::to_bits).as_slice(),
            expected,
            "Kotlin row {index}: {command}"
        );
    }
}
#[derive(Default)]
struct Ground {
    on_ground: bool,
}
#[derive(Default)]
struct PlayerMotion {
    client_side: bool,
    server_copy: bool,
    replaying: bool,
    client_strafe: Option<(Strafe, u8)>,
    ground: Ground,
    move_vector: [f32; 2],
    yaw: f32,
    position: Vec3,
    velocity: Vec3,
    stuck: Vec3,
    flags: CollisionFlags,
    history: MovementHistory,
}
impl PlayerMotion {
    /* IDENTITY */
}
struct MotionWorld<'a> {
    player: &'a mut PlayerMotion,
    no_physics: bool,
    auto_jump: Option<[f32; 2]>,
    collision_input: Option<Vec3>,
    recorded: Vec<Vec3>,
}
impl MotionWorld<'_> {
    /* MODIFIER */
    /* MOVE */
    /* SHULKER */
    fn update_auto_jump(&mut self, x: f32, z: f32) -> Result<()> {
        self.auto_jump = Some([x, z]);
        Ok(())
    }
}
impl EntityMoveWorld for MotionWorld<'_> {
    type Error = std::io::Error;
    fn no_physics(&self) -> bool {
        self.no_physics
    }
    fn position(&self) -> Vec3 {
        self.player.position
    }
    fn set_position(&mut self, p: Vec3) -> Result<()> {
        self.player.position = p;
        Ok(())
    }
    fn record_movement(&mut self, _: MoverType, v: Vec3) -> Result<()> {
        self.recorded.push(v);
        Ok(())
    }
    fn collision_flags(&self) -> CollisionFlags {
        self.player.flags
    }
    fn set_collision_flags(&mut self, f: CollisionFlags) {
        self.player.flags = f;
    }
    fn limit_piston_movement(&mut self, v: Vec3) -> Result<Vec3> {
        Ok(v)
    }
    fn stuck_multiplier(&self) -> Vec3 {
        self.player.stuck
    }
    fn set_stuck_multiplier(&mut self, v: Vec3) {
        self.player.stuck = v;
    }
    fn velocity(&mut self) -> Result<Vec3> {
        Ok(self.player.velocity)
    }
    fn set_velocity(&mut self, v: Vec3) -> Result<()> {
        self.player.velocity = v;
        Ok(())
    }
    fn back_off_edge(&mut self, v: Vec3, _: MoverType) -> Result<Vec3> {
        Ok(v)
    }
    fn collide(&mut self, v: Vec3) -> Result<Vec3> {
        self.collision_input = Some(v);
        Ok(v)
    }
    fn fall_distance(&self) -> f64 {
        0.0
    }
    fn fall_reset_clip(&mut self, _: Vec3, _: Vec3) -> Result<bool> {
        Ok(false)
    }
    fn reset_fall_distance(&mut self) -> Result<()> {
        Ok(())
    }
    fn movement_history(&mut self) -> &mut MovementHistory {
        &mut self.player.history
    }
    fn locally_authoritative(&mut self) -> Result<bool> {
        Ok(true)
    }
    fn set_on_ground_with_movement(&mut self, below: bool, _: bool, _: Vec3) -> Result<()> {
        self.player.ground.on_ground = below;
        Ok(())
    }
    fn minor_horizontal_collision(&mut self, _: Vec3) -> Result<bool> {
        Ok(false)
    }
    fn legacy_on_position(&mut self) -> Result<BlockPos> {
        Ok(BlockPos::new(0, 0, 0))
    }
    fn block_state(&mut self, _: BlockPos) -> Result<BlockStateId> {
        Ok(BlockStateId(0))
    }
    fn on_ground(&mut self) -> Result<bool> {
        Ok(self.player.ground.on_ground)
    }
    fn check_fall_damage(&mut self, _: f64, _: bool, _: BlockStateId, _: BlockPos) -> Result<()> {
        Ok(())
    }
    fn removed(&mut self) -> Result<bool> {
        Ok(false)
    }
    fn can_simulate(&mut self) -> Result<bool> {
        Ok(true)
    }
    fn restitute(&mut self, _: BlockStateId, _: bool, _: bool, _: Vec3) -> Result<()> {
        Ok(())
    }
    fn client_side(&mut self) -> Result<bool> {
        Ok(self.player.client_side)
    }
    fn movement_emission(&mut self) -> Result<MovementEmission> {
        Ok(MovementEmission::None)
    }
    fn passenger(&mut self) -> Result<bool> {
        Ok(false)
    }
    fn apply_movement_emission(
        &mut self,
        _: MovementEmission,
        _: Vec3,
        _: BlockPos,
        _: BlockStateId,
    ) -> Result<()> {
        Ok(())
    }
    fn block_speed_factor(&mut self) -> Result<f32> {
        Ok(1.0)
    }
}
fn world(player: &mut PlayerMotion, no_physics: bool) -> MotionWorld<'_> {
    MotionWorld {
        player,
        no_physics,
        auto_jump: None,
        collision_input: None,
        recorded: vec![],
    }
}
#[test]
fn actual_entity_move_applies_before_no_physics_stuck_and_auto_jump_without_scaling_velocity() {
    let movement = Vec3::new(0.3, 0.42, -0.4);
    for no_physics in [false, true] {
        for stuck in [false, true] {
            let mut player = PlayerMotion {
                client_side: true,
                client_strafe: Some((Strafe::default(), 1)),
                move_vector: [0.0, 1.0],
                velocity: Vec3::new(9.0, 8.0, 7.0),
                stuck: if stuck {
                    Vec3::new(0.25, 0.5, 0.25)
                } else {
                    Vec3::ZERO
                },
                ..Default::default()
            };
            let mut w = world(&mut player, no_physics);
            w.move_self(movement).unwrap();
            let factor = if stuck && !no_physics { 0.25 } else { 1.0 };
            assert!((w.player.position.z - 0.5 * factor).abs() < 1e-14);
            assert_eq!(w.player.position.x, 0.0);
            assert_eq!(
                w.player.position.y,
                0.42 * if stuck && !no_physics { 0.5 } else { 1.0 }
            );
            assert_eq!(w.auto_jump, Some([0.0, (0.5 * factor) as f32]));
            assert_eq!(
                w.player.velocity,
                if stuck && !no_physics {
                    Vec3::ZERO
                } else {
                    Vec3::new(9.0, 8.0, 7.0)
                }
            );
            if !no_physics {
                assert_eq!(w.collision_input.unwrap(), w.player.position);
            }
        }
    }
}
#[test]
fn body_identity_guards_and_nonself_pushes_preserve_movement() {
    let movement = Vec3::new(0.3, 0.42, -0.4);
    for (client_side, server_copy, replaying) in [
        (false, false, false),
        (true, true, false),
        (true, false, true),
    ] {
        let mut player = PlayerMotion {
            client_side,
            server_copy,
            replaying,
            client_strafe: Some((Strafe::default(), 1)),
            move_vector: [0.0, 1.0],
            ..Default::default()
        };
        world(&mut player, true).move_self(movement).unwrap();
        assert_eq!(player.position, movement);
    }
    let mut player = PlayerMotion {
        client_side: true,
        client_strafe: Some((Strafe::default(), 1)),
        move_vector: [0.0, 1.0],
        ..Default::default()
    };
    let mut w = world(&mut player, true);
    w.shulker_box_push(movement).unwrap();
    assert_eq!(w.player.position, movement);
    assert!(w.auto_jump.is_none());
}
pub mod mod_api {
    pub trait RenderHooks: Send + Sync {}
    #[allow(unused_imports)]
    mod generated_host {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated_host.rs"));
    }
    pub use generated_host::*;
    mod generated {
        include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs"));
    }
    pub use generated::*;
    pub fn entered<'a>(mods: &'a [Box<dyn Hooks>], _: &str) -> impl Iterator<Item = &'a dyn Hooks> {
        mods.iter().map(Box::as_ref)
    }
    pub fn mods() -> &'static [Box<dyn Hooks>] {
        static MODS: std::sync::LazyLock<Vec<Box<dyn Hooks>>> =
            std::sync::LazyLock::new(|| vec![Box::new(Policy)]);
        &MODS
    }
    struct Policy;
    impl RenderHooks for Policy {}
    impl Hooks for Policy {
        fn client_strafe(&self, _: &dyn Host, event: &mut ClientStrafeEvent) {
            event.air = -0.5;
            event.ground = 2.25;
            event.strict = true;
            event.input = 5;
            event.apply = true;
        }
    }
}
mod game {
    use crate::PlayerMotion;
    struct Game<'a> {
        window: bool,
        lan_acting_uuid: Option<()>,
        player: Option<PlayerMotion>,
        marker: std::marker::PhantomData<&'a ()>,
    }
    impl Game<'_> {
        fn native_window(&self) -> bool {
            self.window
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
    fn generated_native_event_copies_raw_settings_and_untransformed_input_only_for_window_player() {
        let mut game = Game {
            window: true,
            lan_acting_uuid: None,
            player: Some(PlayerMotion::default()),
            marker: std::marker::PhantomData,
        };
        let query = |game: &Game<'_>| {
            let (mut air, mut ground, mut strict, mut input, mut apply) =
                (1.0, 1.0, false, 0, false);
            game.native_client_strafe(&mut air, &mut ground, &mut strict, &mut input, &mut apply);
            (air, ground, strict, input, apply)
        };
        assert_eq!(query(&game), (-0.5, 2.25, true, 5, true));
        game.window = false;
        assert_eq!(query(&game), (1.0, 1.0, false, 0, false));
        game.window = true;
        game.lan_acting_uuid = Some(());
        assert!(!query(&game).4);
        game.lan_acting_uuid = None;
        game.player.as_mut().unwrap().server_copy = true;
        assert!(!query(&game).4);
        game.player.as_mut().unwrap().server_copy = false;
        game.player.as_mut().unwrap().replaying = true;
        assert!(!query(&game).4);
        game.player = None;
        assert!(!query(&game).4);
    }
}
