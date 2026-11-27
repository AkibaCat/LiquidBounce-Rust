#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/clicking.rs"]
mod clicking;
#[path = "../liquidbounce-behaviors/src/combat_rotations.rs"]
mod combat_rotations;
#[path = "../liquidbounce-behaviors/src/no_rotate_set.rs"]
mod no_rotate_set;
#[path = "../liquidbounce-behaviors/src/projectile_puncher.rs"]
mod projectile_puncher;
#[path = "../liquidbounce-behaviors/src/random.rs"]
mod random;
#[path = "../liquidbounce-behaviors/src/rotations.rs"]
mod rotations;
#[cfg(test)]
mod projectile_smoke {
    use super::*;
    use projectile_puncher as m;
    use rotations::Rotation;
    fn fireball() -> m::Candidate {
        m::Candidate {
            id: 7,
            kind: 0,
            within_border: true,
            position: [0., 1., 3.],
            previous: [0., 1., 3.2],
            physical: [-0.5, 1., 2.5, 0.5, 2., 3.5],
            pick_radius: 1.,
            dimensions: [1., 1.],
        }
    }
    #[test]
    fn velocity_kind_border_and_absolute_clip() {
        let mut c = fireball();
        let p = [-0.3, 0., -0.3, 0.3, 1.8, 0.3];
        assert!(m::should_attack(c, 3, p, p));
        c.previous = c.position;
        assert!(!m::should_attack(c, 3, p, p));
        c = fireball();
        assert!(!m::should_attack(c, 4, p, p));
        c.within_border = false;
        assert!(!m::should_attack(c, 3, p, p));
        c = fireball();
        c.position = [100., 1., 100.];
        c.previous = [100., 1., 100.2];
        let extended = m::inflate(p, 1.5);
        assert_eq!(
            m::clip(
                extended,
                c.position,
                m::scale(m::sub(c.position, c.previous), 20.)
            ),
            None
        );
    }
    #[test]
    fn next_box_uses_dimensions_without_pick_radius_or_wind_offset() {
        let mut c = fireball();
        c.kind = 2;
        c.dimensions = [0.3125; 2];
        c.physical = [-0.15625, 0.85, 2.84375, 0.15625, 1.1625, 3.15625];
        let b = m::predicted_box(c);
        assert_eq!(b[1], 1.);
        assert_eq!(b[4], 1.3125);
        assert_eq!(b[3] - b[0], 0.3125);
        assert!(b[2] < c.physical[2]);
    }
    #[test]
    fn silhouette_fov_is_box_angle_and_inside_ray_hits() {
        let b = [-1., -1., 4., 1., 1., 6.];
        assert_eq!(m::max_cos(b, [0.; 3], [0., 0., 1.]), 1.);
        let cos = m::max_cos(b, [0.; 3], [1., 0., 0.]);
        assert!((cos - 1. / 17f64.sqrt()).abs() < 1e-12);
        assert!(m::ray_hits(b, [0., 0., 5.], [1., 0., 0.]));
        assert!(!m::ray_hits(b, [0.; 3], [1., 0., 0.]));
    }
    #[test]
    fn outline_visibility_and_strict_range() {
        let b = [-0.5, 0., 2.5, 0.5, 1., 3.5];
        let eye = [0., 0.5, 0.];
        let hit = m::raytrace_box(eye, b, 3., [0., 0.], [0., 0., 1.], &mut |_, _| true).unwrap();
        assert_eq!(hit, [0., -0.]);
        assert!(m::raytrace_box(eye, b, 2.5, [0., 0.], [0., 0., 1.], &mut |_, _| true).is_none());
        assert!(m::raytrace_box(eye, b, 3., [0., 0.], [0., 0., 1.], &mut |_, _| false).is_none());
        let turn =
            m::raytrace_box(eye, b, 3., [45., 0.], [-0.707, 0., 0.707], &mut |_, _| true).unwrap();
        assert!(turn[0].abs() < 45.);
    }
    #[test]
    fn captured_modes_fallback_and_processor_order() {
        use combat_rotations::{Settings, SmoothMode};
        let mut s = Settings::default();
        s.interpolation.horizontal = [80; 2];
        s.interpolation.vertical = [25; 2];
        s.interpolation.direction_change = [100; 2];
        let from = Rotation::ZERO;
        let to = Rotation::new(60., 20.);
        let process = |mode, s: &mut Settings| {
            s.active = mode;
            let id = s.processor_id();
            s.process(
                id,
                true,
                from,
                to,
                Some(from),
                from,
                None,
                from,
                &mut || 0.5,
            )
            .unwrap()
        };
        let interpolation = process(SmoothMode::Interpolation, &mut s);
        assert_eq!(interpolation, process(SmoothMode::Ai, &mut s));
        assert!(interpolation.yaw > 0. && interpolation.yaw < 60.);
        s.active = SmoothMode::Linear;
        let captured = s.processor_id();
        s.fail.enabled = true;
        s.short_stop.enabled = true;
        let result = s
            .process(
                captured,
                true,
                from,
                to,
                Some(from),
                from,
                None,
                from,
                &mut || 0.5,
            )
            .unwrap();
        assert_eq!(result, to);
        s.short_stop.rate = 100;
        s.short_stop.duration = [1; 2];
        let id = s.processor_id();
        let stopped = s
            .process(
                id,
                true,
                from,
                to,
                Some(from),
                from,
                None,
                from,
                &mut || 0.5,
            )
            .unwrap();
        assert!(stopped.yaw.abs() < 0.1);
    }
    #[test]
    fn fail_ticks_and_disabled_processors_preserve_history() {
        let mut s = combat_rotations::Settings::default();
        s.fail.enabled = true;
        s.fail.rate = 100;
        s.fail.duration = [2; 2];
        s.fail.horizontal = no_rotate_set::FloatRange(0., 0.);
        s.fail.vertical = no_rotate_set::FloatRange(0., 0.);
        s.first_tick(true, &mut || 0.).unwrap();
        assert_eq!(s.fail.elapsed, 0);
        assert_eq!(s.fail.current_duration, 2);
        s.first_tick(false, &mut || 0.).unwrap();
        assert_eq!(s.fail.elapsed, 0);
        let id = s.processor_id();
        let shifted = s
            .process(
                id,
                true,
                Rotation::ZERO,
                Rotation::ZERO,
                Some(Rotation::new(10., 20.)),
                Rotation::ZERO,
                None,
                Rotation::ZERO,
                &mut || 0.,
            )
            .unwrap();
        assert_eq!(shifted, Rotation::new(0.39999998, 0.79999995));
    }
    #[test]
    fn independent_clicker_has_no_item_cooldown_and_cooperates() {
        use clicking::*;
        let mut a = Clicker::new(
            Settings {
                technique: Technique::Constant,
                cps: [20; 2],
                item_cooldown: None,
                ..Default::default()
            },
            false,
            JavaRandom::seeded(1),
            Math::default(),
        );
        let mut b = a.clone();
        let cooldown = Cooldown {
            has_cooldown: true,
            real_ticks: 0.,
            reset_ticks: 0.,
            server_tick_speed: 1.,
            attack_delay: 100.,
        };
        a.tick(0);
        b.tick(0);
        assert!(a.passes_gate(cooldown, 0));
        assert!(!a.passes_gate(cooldown, 1));
        assert!(a.take_attempt(cooldown));
        a.record_success(None);
        b.plan.pressed_by(&a.plan, 0);
        assert!(!b.take_attempt(cooldown));
        a.finish_click(1);
        assert!(!a.attack_key_down(false));
    }
}
