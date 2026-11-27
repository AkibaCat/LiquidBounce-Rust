#![cfg(test)]
extern crate self as horizon_client;
#[path = "BEHAVIOR_PATH"]
pub mod no_rotate_set;
#[path = "PACKET_BRIDGE_PATH"]
pub mod packet_bridge;
#[path = "PLAYER_SYNC_PATH"]
pub mod player_sync;
#[path = "RANDOM_PATH"]
pub mod random;
#[path = "ROTATIONS_PATH"]
pub mod rotations;

#[test]
fn java_mode_steps_match_original_jdk_kotlin_float_expressions() {
    use no_rotate_set::{FloatRange, RotationsSettings, SmoothMode};
    use rotations::Rotation;
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("java-vectors.json")).unwrap();
    assert_eq!(reference["rangeNaNThrows"], true);
    let modes = [
        SmoothMode::Linear,
        SmoothMode::Sigmoid,
        SmoothMode::Acceleration,
        SmoothMode::Acceleration,
    ];
    for (kind, mode) in modes.into_iter().enumerate() {
        let mut settings = RotationsSettings::default();
        settings.linear.horizontal_turn_speed = FloatRange(13.0, 79.0);
        settings.linear.vertical_turn_speed = FloatRange(3.0, 45.0);
        settings.sigmoid.speeds = settings.linear;
        settings.sigmoid.steepness = 7.3;
        settings.sigmoid.midpoint = 0.24;
        settings.acceleration.sigmoid_deceleration = kind == 3;
        let units = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99999, 0.33, 0.67];
        let mut cursor = 0;
        let mut random = || {
            let value = units[cursor % units.len()];
            cursor += 1;
            value
        };
        let mut current = Rotation::new(179.0, -88.0);
        let mut previous = Rotation::new(170.0, -85.0);
        for tick in 0..5 {
            let next = settings
                .process(
                    mode,
                    current,
                    Rotation::new(-179.0, 88.0),
                    Some(previous),
                    Rotation::ZERO,
                    &mut random,
                )
                .unwrap();
            let bits = [next.yaw.to_bits(), next.pitch.to_bits()];
            let expected = [
                reference["steps"][kind][tick][0].as_u64().unwrap() as u32,
                reference["steps"][kind][tick][1].as_u64().unwrap() as u32,
            ];
            assert_eq!(bits, expected, "kind={kind}, tick={tick}");
            previous = current;
            current = next;
        }
    }
}

#[test]
fn raw_ranges_processor_identity_and_post_correction_guards() {
    use no_rotate_set::*;
    use rotations::{MovementCorrection, Rotation};
    let mut module = NoRotateSet::default();
    let before = Rotation::new(12.0, -34.0);
    let corrected = Rotation::new(180.0, 90.0);
    assert!(
        module
            .after_correction(false, false, before, corrected, true)
            .is_none()
    );
    assert!(
        module
            .after_correction(true, true, before, corrected, true)
            .is_none()
    );
    let silent = module
        .after_correction(true, false, before, corrected, true)
        .unwrap();
    assert!(silent.target.is_none());
    assert_eq!(silent.restored.yaw, 12.0 + 0.000001f32);
    assert_eq!(silent.restored.pitch, -34.0 + 0.000001f32);
    module.mode = NoRotateMode::ResetRotation;
    module.rotations.active = SmoothMode::Sigmoid;
    module.rotations.movement_correction = MovementCorrection::ChangeLook;
    module.rotations.ticks_until_reset = -5;
    module.rotations.reset_threshold = -25.0;
    let target = module
        .after_correction(true, false, before, corrected, true)
        .unwrap()
        .target
        .unwrap();
    assert_eq!(target.processor, SmoothMode::Sigmoid);
    assert!(target.target.consider_inventory && target.target.rotation.is_normalized);
    assert_eq!(target.target.ticks_until_reset, -5);
    assert_eq!(target.target.reset_threshold, -25.0);
    module.rotations.active = SmoothMode::Acceleration;
    assert_eq!(target.processor, SmoothMode::Sigmoid);
    assert!(
        module
            .after_correction(true, false, before, corrected, false)
            .unwrap()
            .target
            .is_none()
    );
    let mut draws = 0;
    assert_eq!(
        FloatRange(20.0, -5.0)
            .sample(&mut || {
                draws += 1;
                0.5
            })
            .unwrap(),
        20.0
    );
    assert_eq!(draws, 0);
    assert!(FloatRange(f32::NAN, 2.0).sample(&mut || 0.5).is_err());
    assert!(FloatRange(0.0, f32::MAX).sample(&mut || 0.5).is_err());
    module.rotations.acceleration.coef_distance = f32::NAN;
    assert!(
        module
            .rotations
            .process(
                SmoothMode::Acceleration,
                before,
                corrected,
                None,
                before,
                &mut || 0.5
            )
            .is_err()
    );
    module.rotations.acceleration.coef_distance = -1.393;
    module.rotations.acceleration.yaw_acceleration = FloatRange(-2.0, -3.0);
    assert!(
        module
            .rotations
            .process(
                SmoothMode::Acceleration,
                before,
                corrected,
                None,
                before,
                &mut || 0.5
            )
            .is_err()
    );
    module.rotations.linear.horizontal_turn_speed = FloatRange(-1.0, -1.0);
    assert!(
        module
            .rotations
            .process(
                SmoothMode::Linear,
                before,
                corrected,
                None,
                before,
                &mut || 0.5
            )
            .is_err()
    );
}

#[test]
fn manager_keeps_captured_processor_live_during_reset_and_change_look_endpoints() {
    use no_rotate_set::{FloatRange, RotationsSettings, SmoothMode};
    use rotations::{DefaultRotationTarget, MovementCorrection, Rotation, RotationManager};
    let mut manager = RotationManager::default();
    let player = Rotation::new(0.0, 0.0);
    let mut target = DefaultRotationTarget::sprint(Rotation::new(90.0, 0.0));
    target.ticks_until_reset = 1;
    let mut settings = RotationsSettings::default();
    settings.linear.horizontal_turn_speed = FloatRange(20.0, 20.0);
    manager.request_processed(7, -20, target, SmoothMode::Linear.processor_id(), true);
    let mut seen = vec![];
    manager
        .update_with(
            player,
            1.0,
            true,
            false,
            |_| true,
            |id, _, from, to, last| {
                seen.push(id);
                settings
                    .process(
                        SmoothMode::from_processor_id(id).unwrap(),
                        from,
                        to,
                        last,
                        player,
                        &mut || 0.5,
                    )
                    .map_err(rotations::RotationError::Processor)
            },
        )
        .unwrap();
    assert_eq!(manager.current.unwrap().yaw, 20.0);
    settings.active = SmoothMode::Acceleration;
    settings.linear.horizontal_turn_speed = FloatRange(5.0, 5.0);
    manager
        .update_with(
            player,
            1.0,
            true,
            false,
            |_| false,
            |id, _, from, to, last| {
                seen.push(id);
                settings
                    .process(
                        SmoothMode::from_processor_id(id).unwrap(),
                        from,
                        to,
                        last,
                        player,
                        &mut || 0.5,
                    )
                    .map_err(rotations::RotationError::Processor)
            },
        )
        .unwrap();
    assert_eq!(
        manager.current.unwrap().yaw,
        15.0,
        "captured Linear reads newly changed speed while resetting"
    );
    assert_eq!(seen, vec![1, 1]);

    target.movement_correction = MovementCorrection::ChangeLook;
    target.consider_inventory = true;
    manager.request(2, 0, target, true);
    manager.current = Some(Rotation::new(100.0, 89.0));
    manager.last = Some(Rotation::new(80.0, 80.0));
    manager.last_model = Some(Rotation::new(60.0, 70.0));
    assert_eq!(
        manager
            .change_look_rotation(0.5, 1.0, true, false, |_| true)
            .unwrap(),
        Some(Rotation {
            yaw: 90.0,
            pitch: 85.0,
            is_normalized: true
        })
    );
    let before = manager.clone();
    manager.mouse_rotation(100.0, 100.0, true, true, |_| true);
    assert_eq!(
        manager, before,
        "inventory blocks mouse endpoint adjustment"
    );
    manager.mouse_rotation(100.0, 100.0, true, false, |_| true);
    assert_eq!(manager.current, Some(Rotation::new(115.0, 90.0)));
    assert_eq!(manager.last, Some(Rotation::new(95.0, 90.0)));
    assert_eq!(manager.last_model, Some(Rotation::new(75.0, 85.0)));
    assert_eq!(
        manager
            .change_look_rotation(0.5, 1.0, false, false, |_| true)
            .unwrap(),
        None
    );
}

#[allow(dead_code, clippy::too_many_arguments)]
mod fixture {
    use super::*;
    use horizon_core::geometry::Vec3;
    use horizon_protocol::{
        connection::{ConnWriter, Connection, Pipe},
        packets::play,
    };
    use no_rotate_set::{NoRotateSet, NoRotateTarget};
    use play::Serverbound as S;
    use rotations::Rotation;
    use std::{
        cell::RefCell,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
    };
    type Result<T> = std::result::Result<T, &'static str>;
    #[derive(Default)]
    struct PlayerMotion {
        position: Vec3,
        velocity: Vec3,
        spectator: bool,
        yaw: f32,
        pitch: f32,
        server_copy: bool,
        replaying: bool,
    }
    impl PlayerMotion {
        fn new(_: (), position: Vec3) -> Result<Self> {
            Ok(Self {
                position,
                ..Default::default()
            })
        }
        /* IDENTITY */
    }
    #[derive(Default)]
    struct Model {
        yaw: f32,
        previous_yaw: f32,
        head_pitch: f32,
        previous_head_pitch: f32,
    }
    struct Load(bool);
    impl Load {
        fn is_active(&self) -> bool {
            self.0
        }
    }
    struct Remote {
        session: Session,
        level_load: Load,
        sequence: i32,
        last_teleport_sequence: i32,
        ghast_dismount: Option<i32>,
    }
    struct Replay(bool);
    impl Replay {
        fn playing(&self) -> bool {
            self.0
        }
    }
    #[derive(Default)]
    struct Reset;
    impl Reset {
        fn reset(&mut self) {}
        fn reset_yaw(&mut self, _: f32) {}
    }
    struct Session {
        writer: Arc<Mutex<Option<ConnWriter>>>,
        configuring: Arc<AtomicBool>,
        serial: u64,
        debug: bool,
    }
    impl Session {
        fn serial(&self) -> u64 {
            self.serial
        } /* SESSION */
    }
    pub struct Game {
        player: Option<PlayerMotion>,
        lan_acting_uuid: Option<u128>,
        window: bool,
        remote: Option<Remote>,
        yaw: f32,
        pitch: f32,
        rope_rot_old: (f32, f32),
        player_model: Option<Model>,
        motion: (),
        riding: Option<u32>,
        replay: Replay,
        previous_position: Vec3,
        sway: Reset,
        eye: f32,
        previous_eye: f32,
        clock: Reset,
        mining: Option<()>,
        behavior: Arc<Mutex<NoRotateSet>>,
        running: Arc<AtomicBool>,
        targets: RefCell<Vec<NoRotateTarget>>,
        rotation_manager: RefCell<rotations::RotationManager>,
    }
    mod mod_api {
        use super::*;
        pub trait Host {
            fn game(&self) -> &Game;
        }
        pub trait RenderHooks: Send + Sync {}
        pub fn entered<'a>(
            mods: &'a [Box<dyn Hooks>],
            _: &str,
        ) -> impl Iterator<Item = &'a dyn Hooks> {
            mods.iter().map(Box::as_ref)
        }
        #[allow(clippy::too_many_arguments)]
        mod generated {
            include!(concat!(env!("OUT_DIR"), "/mod_api_generated.rs"));
        }
        pub use generated::*;
        pub fn mods() -> &'static [Box<dyn Hooks>] {
            static MODS: std::sync::LazyLock<Vec<Box<dyn Hooks>>> =
                std::sync::LazyLock::new(|| vec![Box::new(Recorder)]);
            &MODS
        }
        struct Recorder;
        impl Hooks for Recorder {
            fn client_no_rotate_capture(
                &self,
                host: &dyn Host,
                event: &mut ClientNoRotateCaptureEvent,
            ) {
                event.running = host.game().running.load(Ordering::Acquire);
            }
            fn client_mouse_rotation(&self, host: &dyn Host, event: &mut ClientMouseRotationEvent) {
                let game = host.game();
                game.rotation_manager.borrow_mut().mouse_rotation(
                    event.cursor_x,
                    event.cursor_y,
                    true,
                    false,
                    |_| game.running.load(Ordering::Acquire),
                );
            }
            fn client_change_look(&self, host: &dyn Host, event: &mut ClientChangeLookEvent) {
                let game = host.game();
                if let Some(rotation) = game
                    .rotation_manager
                    .borrow_mut()
                    .change_look_rotation(event.partial, 1.0, true, false, |_| {
                        game.running.load(Ordering::Acquire)
                    })
                    .unwrap()
                {
                    event.yaw = rotation.yaw;
                    event.pitch = rotation.pitch;
                    event.apply = true;
                }
            }
            fn client_server_rotation(
                &self,
                host: &dyn Host,
                event: &mut ClientServerRotationEvent,
            ) {
                let game = host.game();
                if let Some(result) = game.behavior.lock().unwrap().after_correction(
                    event.preaccepted || game.running.load(Ordering::Acquire),
                    false,
                    Rotation::new(event.previous_yaw, event.previous_pitch),
                    Rotation::new(event.corrected_yaw, event.corrected_pitch),
                    true,
                ) {
                    if let Some(target) = result.target {
                        game.targets.borrow_mut().push(target);
                    }
                    event.yaw = result.restored.yaw;
                    event.pitch = result.restored.pitch;
                    event.restore = true;
                }
            }
        }
    }
    impl mod_api::Host for Game {
        fn game(&self) -> &Game {
            self
        }
    }
    impl Game {
        fn native_window(&self) -> bool {
            self.window
        }
        fn remote_send_carried(&self) {}
        fn spectator(&self) -> bool {
            false
        }
        fn take_carried_attributes(&self, _: &mut PlayerMotion) -> Result<()> {
            Ok(())
        }
        fn input_player_replaced(&mut self) {}
        fn apply_remote_health(&mut self) {}
        fn apply_remote_abilities(&mut self) {}
        fn place_ghast_under(&mut self, _: i32, _: Vec3) {}
        fn stop_remote_mining(&mut self) {}
        /* HELPERS */
        /* REMOTE */
    }
    /* POSITION_PITCH */

    #[test]
    fn exact_handlers_ack_before_restore_and_different_running_check_phases() {
        use no_rotate_set::{NoRotateMode, SmoothMode};
        struct DisableOnAck(Arc<AtomicBool>);
        impl packet_bridge::PacketInterceptor for DisableOnAck {
            fn outgoing(&self, _: &mut packet_bridge::PacketEvent<'_, S>) {
                self.0.store(false, Ordering::Release);
            }
        }
        let running = Arc::new(AtomicBool::new(true));
        let behavior = Arc::new(Mutex::new(NoRotateSet::default()));
        behavior.lock().unwrap().mode = NoRotateMode::ResetRotation;
        behavior.lock().unwrap().rotations.active = SmoothMode::Acceleration;
        let (client, server) = Pipe::pair();
        let (_, writer) = Connection::local(client).split();
        let mut server = Connection::local(server);
        let mut game = Game {
            player: Some(PlayerMotion::default()),
            lan_acting_uuid: None,
            window: true,
            remote: Some(Remote {
                session: Session {
                    writer: Arc::new(Mutex::new(Some(writer))),
                    configuring: Arc::new(AtomicBool::new(false)),
                    serial: 77,
                    debug: false,
                },
                level_load: Load(false),
                sequence: 8,
                last_teleport_sequence: 0,
                ghast_dismount: None,
            }),
            yaw: 25.0,
            pitch: 35.0,
            rope_rot_old: (25.0, 35.0),
            player_model: Some(Model::default()),
            motion: (),
            riding: None,
            replay: Replay(false),
            previous_position: Vec3::ZERO,
            sway: Reset,
            eye: 1.62,
            previous_eye: 1.62,
            clock: Reset,
            mining: Some(()),
            behavior,
            running: running.clone(),
            targets: RefCell::new(vec![]),
            rotation_manager: RefCell::new(rotations::RotationManager::default()),
        };
        let previous =
            packet_bridge::set_interceptor(Some(Arc::new(DisableOnAck(running.clone()))));
        game.remote_rotation(&play::PlayerRotation {
            yaw: 5.0,
            pitch: 80.0,
            relative_yaw: true,
            relative_pitch: true,
        });
        let (id, bytes) = server.recv().unwrap();
        assert!(matches!(
            S::decode(id, &bytes).unwrap(),
            S::MovePlayerRot(play::MovePlayerRot {
                yaw: 30.0,
                pitch: 90.0,
                flags: 0
            })
        ));
        assert!(!running.load(Ordering::Acquire));
        assert_eq!(
            (game.yaw, game.pitch),
            (25.0 + 0.000001f32, 35.0 + 0.000001f32)
        );
        assert_eq!(game.player_model.as_ref().unwrap().previous_yaw, game.yaw);
        assert_eq!(
            game.player_model.as_ref().unwrap().previous_head_pitch,
            game.pitch
        );
        assert_eq!(
            game.targets.borrow()[0].target.rotation,
            Rotation {
                yaw: 30.0,
                pitch: 90.0,
                is_normalized: true
            }
        );
        assert_eq!(game.targets.borrow()[0].processor, SmoothMode::Acceleration);
        running.store(true, Ordering::Release);
        let position = play::PlayerPosition {
            teleport: 4,
            position: [1.0, 2.0, 3.0],
            delta: [4.0, 5.0, 6.0],
            yaw: -50.0,
            pitch: -80.0,
            relatives: 0,
        };
        game.remote_teleport(&position).unwrap();
        let (id, bytes) = server.recv().unwrap();
        assert!(matches!(
            S::decode(id, &bytes).unwrap(),
            S::AcceptTeleportation(play::AcceptTeleportation {
                yaw: -50.0,
                pitch: -80.0,
                ..
            })
        ));
        assert_eq!(
            (game.yaw, game.pitch),
            (-50.0, -80.0),
            "position checks running after vanilla/ack"
        );
        assert_eq!(
            game.player.as_ref().unwrap().position,
            Vec3::new(1.0, 2.0, 3.0)
        );
        assert_eq!(
            game.player.as_ref().unwrap().velocity,
            Vec3::new(4.0, 5.0, 6.0)
        );
        assert_eq!(game.targets.borrow().len(), 1);
        assert_eq!(game.remote.as_ref().unwrap().last_teleport_sequence, 8);
        assert!(game.mining.is_none());
        packet_bridge::set_interceptor(previous);

        running.store(true, Ordering::Release);
        game.remote.as_mut().unwrap().level_load.0 = true;
        game.remote_rotation(&play::PlayerRotation {
            yaw: 90.0,
            pitch: 10.0,
            relative_yaw: false,
            relative_pitch: false,
        });
        server.recv().unwrap();
        assert_eq!((game.yaw, game.pitch), (90.0, 10.0));
        game.remote.as_mut().unwrap().level_load.0 = false;
        game.player.as_mut().unwrap().server_copy = true;
        game.remote_rotation(&play::PlayerRotation {
            yaw: 100.0,
            pitch: 20.0,
            relative_yaw: false,
            relative_pitch: false,
        });
        server.recv().unwrap();
        assert_eq!((game.yaw, game.pitch), (100.0, 20.0));
        game.player.as_mut().unwrap().server_copy = false;
        let mut target = rotations::DefaultRotationTarget::sprint(Rotation::new(80.0, 40.0));
        target.movement_correction = rotations::MovementCorrection::ChangeLook;
        game.rotation_manager
            .borrow_mut()
            .request(1, -20, target, true);
        game.rotation_manager.borrow_mut().current = Some(Rotation::new(80.0, 40.0));
        game.rotation_manager.borrow_mut().last = Some(Rotation::new(60.0, 20.0));
        game.native_change_look_now(0.5);
        assert_eq!((game.yaw, game.pitch), (70.0, 30.0));
        assert_eq!(game.rope_rot_old, (100.0, 20.0));
        assert_eq!(game.native_client_player_last_rotation(), [100.0, 20.0]);
        assert_eq!(game.player_model.as_ref().unwrap().previous_yaw, 100.0);
        assert_eq!(
            game.player_model.as_ref().unwrap().previous_head_pitch,
            20.0
        );
        game.native_client_mouse_rotation(100.0, 100.0);
        assert_eq!(
            game.rotation_manager.borrow().current,
            Some(Rotation::new(95.0, 55.0))
        );
    }
}
