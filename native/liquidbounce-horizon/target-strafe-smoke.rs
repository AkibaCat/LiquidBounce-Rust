#[cfg(test)]
mod bridge_tests {
    use super::*;
    use crate::target_strafe_math::B;
    struct Flat;
    impl model::World for Flat {
        fn collision_free(&mut self, _: V) -> bool {
            true
        }
        fn edge_safe(&mut self, _: V, _: f32) -> bool {
            true
        }
        fn void_safe(&mut self, _: V, _: f32) -> bool {
            true
        }
        fn sees(&mut self, _: V, _: B, _: i32) -> bool {
            true
        }
    }
    fn snapshot(input_mode: bool, initial_moving: bool) -> Snapshot {
        Snapshot {
            settings: Settings {
                input_mode,
                ..Default::default()
            },
            target: Some(Target {
                id: 1,
                position: V::default(),
                bounds: B {
                    lo: V(-0.3, 0., -0.3),
                    hi: V(0.3, 1.8, 0.3),
                },
            }),
            input: 1,
            initial_moving,
            speed_running: false,
        }
    }
    fn context(input: u8) -> Context {
        Context {
            feet: V(3., 0., 0.),
            speed: 0.53,
            yaw: 0.,
            horizontal_collision: false,
            input,
        }
    }
    #[test]
    fn actual_move_uses_event_speed_and_initial_movement_gate() {
        clear();
        configure(Some(snapshot(false, false)));
        let old = V(0.2, 0.1, 0.4);
        assert_eq!(move_event(&mut Flat, context(1), old, true), old);
        assert!(render().is_none());
        configure(Some(snapshot(false, true)));
        let out = move_event(&mut Flat, context(1), old, true);
        assert!((out.0.hypot(out.2) - 0.53).abs() < 1e-12);
        assert_eq!(out.1, 0.1);
        assert!(render().is_some());
        clear();
        assert!(render().is_none());
    }
    #[test]
    fn input_mode_uses_cancelled_keys_and_does_not_change_motion() {
        clear();
        configure(Some(snapshot(true, false)));
        assert!(input_event(&mut Flat, context(15)).is_none());
        assert!(render().is_none());
        assert!(input_event(&mut Flat, context(1)).is_some());
        let old = V(1., 2., 3.);
        assert_eq!(move_event(&mut Flat, context(1), old, false), old);
        clear();
    }
}
