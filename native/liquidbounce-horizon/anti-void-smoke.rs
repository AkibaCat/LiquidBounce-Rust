// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/anti_void.rs"]
pub mod anti_void;
#[path = "../liquidbounce-behaviors/src/ghost_hand.rs"]
pub mod ghost_hand;
#[cfg(test)]
mod anti_void_smoke {
    use super::anti_void::*;
    fn ready(mode: usize) -> State {
        let mut s = State::default();
        s.settings.mode = mode;
        s.observe(false, [1., 64.9, 3.]);
        s.observe(true, [2., 64., 3.]);
        s
    }
    fn rescue(
        s: &mut State,
        ground: bool,
        positions: bool,
        motion: f64,
        slots: &[(i32, String)],
    ) -> Option<Rescue> {
        s.rescue(
            true,
            Exempt::default(),
            true,
            0.5,
            motion,
            ground,
            positions,
            slots,
            4,
            |r| r[0],
        )
    }
    #[test]
    fn anti_void_smoke_defaults_first_fall_and_ten_tick_boundary() {
        let s = State::default();
        assert_eq!(s.settings.mode, 0);
        assert_eq!(s.settings.void_level, 0);
        assert_eq!(s.settings.slot_reset, [4, 6]);
        let mut snapshots = vec![
            Snapshot {
                position: [0., 0., 0.],
                fall_distance: 0.,
                on_ground: false
            };
            11
        ];
        snapshots[0].fall_distance = 1.;
        snapshots[1].fall_distance = 2.;
        snapshots[1].position[0] = 1.;
        let mut visited = vec![];
        assert!(!predicts_fall(&snapshots, |p| {
            visited.push(p);
            p[0] == 1.
        }));
        assert_eq!(visited.len(), 1);
        snapshots[0].fall_distance = 0.;
        snapshots[1].fall_distance = 0.;
        snapshots[10].fall_distance = 1.;
        assert!(!predicts_fall(&snapshots, |_| true));
        snapshots[9].fall_distance = 1.;
        assert!(predicts_fall(&snapshots, |_| true));
    }
    #[test]
    fn anti_void_smoke_ghost_floor_and_lifecycle_keep_safe_position() {
        let mut s = ready(0);
        assert_eq!(s.ghost_floor(true, Exempt::default()), Some(64.));
        assert!(ghost_shape(true, 63, Some(64.)));
        assert!(!ghost_shape(true, 64, Some(64.)));
        assert!(!ghost_shape(false, 0, Some(64.)));
        assert_eq!(rescue(&mut s, false, false, -1., &[]), None);
        s.enabled();
        assert!(!s.likely_falling);
        assert_eq!(s.rescue_position, Some([1., 64.9, 3.]));
        s.observe(false, [0., -0.1, 0.]);
        s.observe(true, [0., -1., 0.]);
        assert_eq!(s.ghost_floor(true, Exempt::default()), Some(-1.));
    }
    #[test]
    fn anti_void_smoke_flag_exact_float_and_dynamic_until_listeners() {
        let mut s = ready(1);
        assert_eq!(
            rescue(&mut s, false, false, 0., &[]),
            Some(Rescue::Raise(f64::from(0.42_f32)))
        );
        assert!(s.rescue_position.is_none());
        s = ready(1);
        s.settings.silent = true;
        assert_eq!(rescue(&mut s, false, false, 0., &[]), Some(Rescue::Silent));
        s.observe(false, [0., 20., 0.]);
        s.observe(true, [0., 19., 0.]);
        assert_eq!(rescue(&mut s, false, false, 0., &[]), Some(Rescue::Silent));
        s.settings.height = 0.3;
        let mut y = 10.;
        s.network(true, false, &mut y);
        assert_eq!(s.pending_silent, 2);
        let mut expected = 10.;
        expected += f64::from(0.3_f32);
        expected += f64::from(0.3_f32);
        assert_eq!(y, expected);
        s.settings.height = 0.5;
        s.network(true, true, &mut y);
        assert_eq!(y, expected + 1.);
        assert_eq!(s.pending_silent, 0);
        s.pending_silent = 1;
        s.network(false, true, &mut y);
        assert_eq!(y, expected + 1.);
        assert_eq!(s.pending_silent, 0);
    }
    #[test]
    fn anti_void_smoke_blink_uses_shared_positions_and_exemptions() {
        let mut s = ready(2);
        assert!(s.requires_lag(true, Exempt::default()));
        assert_eq!(rescue(&mut s, false, false, -1., &[]), None);
        assert_eq!(
            rescue(&mut s, false, true, -1., &[]),
            Some(Rescue::CancelBlink)
        );
        for exempt in [
            Exempt {
                dead: true,
                ..Default::default()
            },
            Exempt {
                fly: true,
                ..Default::default()
            },
            Exempt {
                clutch_guarding: true,
                ..Default::default()
            },
            Exempt {
                scaffold: true,
                ..Default::default()
            },
        ] {
            let s = ready(2);
            assert!(!s.requires_lag(true, exempt));
            assert!(!s.eligible(true, exempt));
        }
        assert!(ready(1).eligible(
            true,
            Exempt {
                scaffold: true,
                ..Default::default()
            }
        ));
    }
    #[test]
    fn anti_void_smoke_use_item_threshold_ground_and_closest_order() {
        let slots = [
            (40, "minecraft:magma_cream".into()),
            (3, "minecraft:magma_cream".into()),
            (5, "minecraft:magma_cream".into()),
            (4, "minecraft:magma_cream".into()),
        ];
        let mut s = ready(3);
        assert_eq!(rescue(&mut s, false, false, -0.5, &slots), None);
        assert_eq!(rescue(&mut s, true, false, -1., &slots), None);
        assert_eq!(
            rescue(&mut s, false, false, -1., &slots),
            Some(Rescue::Use { slot: 4, delay: 4 })
        );
        assert_eq!(closest_slot(&slots[..3], 4, &s.settings.items), Some(40));
        assert_eq!(closest_slot(&slots[1..3], 4, &s.settings.items), Some(3));
        s.settings.items.insert("minecraft:air".into());
        assert_eq!(
            closest_slot(&[(0, "minecraft:air".into())], 0, &s.settings.items),
            Some(0)
        );
    }
}
