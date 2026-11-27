#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_bow.rs"]
mod auto_bow;
#[path = "../liquidbounce-behaviors/src/auto_bow_math.rs"]
mod auto_bow_math;
#[path = "../liquidbounce-behaviors/src/auto_rod.rs"]
mod auto_rod;
#[path = "../liquidbounce-behaviors/src/point_tracker.rs"]
mod point_tracker;
use auto_bow_math::{B, V};
use auto_rod::*;
use point_tracker::{Point, PointTracker, Samples};
#[derive(Default)]
struct Random {
    floats: usize,
    ints: usize,
    gaussians: usize,
    chances: usize,
    hit: bool,
}
impl Samples for Random {
    fn float(&mut self, r: [f32; 2]) -> f32 {
        self.floats += 1;
        r[0]
    }
    fn int(&mut self, r: [i32; 2]) -> i32 {
        self.ints += 1;
        r[0]
    }
    fn gaussian(&mut self) -> f64 {
        self.gaussians += 1;
        1.
    }
    fn chance(&mut self, _: f32) -> bool {
        self.chances += 1;
        self.hit
    }
}
fn point(x: f64) -> Point {
    Point::new(
        V(x, 1., 0.),
        B {
            lo: V(x, 0., 0.),
            hi: V(x + 1., 2., 1.),
        },
    )
}
fn main() {
    let mut random = Random::default();
    let mut r = AutoRod::default();
    r.initialize(&mut random);
    let mut c = Context {
        health: 20.,
        selected: 3,
        main_item: "minecraft:iron_sword".into(),
        ..Default::default()
    };
    c.rods[9] = true;
    c.rods[3] = true;
    r.first_tick(&c);
    assert_eq!(r.available_slot, Some(40));
    c.rods[9] = false;
    r.first_tick(&c);
    assert_eq!(r.available_slot, Some(3));
    c.rods[9] = true;
    r.first_tick(&c);
    assert_eq!(r.available_slot, Some(3));
    c.selected = 0;
    r.first_tick(&c);
    assert_eq!(r.available_slot, Some(40));
    assert!(r.requirements_met(&c));
    c.health = 10.;
    assert!(!r.requirements_met(&c));
    c.health = 20.;
    c.enemies = 2;
    assert!(!r.requirements_met(&c));
    r.settings.max_enemies = 0;
    assert!(r.requirements_met(&c));
    r.settings.requires[2] = true;
    assert!(!r.requirements_met(&c));
    c.requires[2] = true;
    assert!(r.requirements_met(&c));
    c.main_item = "minecraft:bow".into();
    assert!(!r.requirements_met(&c));
    c.main_item = "minecraft:iron_sword".into();
    r.settings.ignore[2] = true;
    c.ignore[2] = true;
    assert!(!r.requirements_met(&c));
    c.ignore[2] = false;
    r.begin(40, 42);
    assert!(!r.advance_hit(true, 16.));
    assert!(r.advance_hit(true, 30.));
    r.cooldown(2);
    assert!(r.advance_cooldown(true));
    assert!(!r.advance_cooldown(true));
    assert!(r.run.is_none());
    r.begin(40, 42);
    r.cached_hook = Some(Hook {
        id: 5,
        hooked: false,
        stationary: true,
    });
    assert!(r.advance_hit(true, 16.));
    r.cooldown(5);
    assert!(!r.advance_cooldown(false));
    assert!(r.run.is_none());
    r.cached_hook = None;
    r.settings.hit_timeout = 2;
    r.begin(40, 42);
    r.settings.hit_timeout = 100;
    assert!(!r.advance_hit(true, 16.));
    assert!(r.advance_hit(true, 16.));
    r.settings.hit_timeout = 2;
    r.settings.pull_out_of_range = false;
    r.begin(40, 42);
    assert!(!r.advance_hit(true, 1000.));
    assert!(r.advance_hit(true, 1000.));
    r.cached_hook = Some(Hook {
        id: 6,
        hooked: true,
        stationary: false,
    });
    r.target = Some(42);
    let scan = r.current_scan_extra;
    assert!(r.clear());
    assert_eq!(r.current_scan_extra, scan);
    assert!(r.target.is_none() && r.available_slot.is_none());
    let mut input = InputTracker::default();
    input.mouse_event(1, true, 1000);
    assert!(input.mouse_click(1, 2000));
    input.mouse_event(1, false, 2000);
    assert!(input.mouse_click(1, 1250));
    assert!(!input.mouse_click(1, 1251));
    input.moved(0., 0., 1000);
    assert!(!input.mouse_moved(1000));
    input.moved(1., 0., 1000);
    assert!(input.mouse_moved(1100));
    assert!(!input.mouse_moved(1101));
    let mut tracker = PointTracker::default();
    tracker.initialize(&mut random);
    tracker.settings.delay = true;
    tracker.settings.delay_ticks = [2, 2];
    tracker.delay_changed(&mut random);
    assert_eq!(tracker.process(point(0.), &mut random), point(0.));
    assert_eq!(tracker.process(point(1.), &mut random), point(0.));
    assert_eq!(tracker.process(point(2.), &mut random), point(0.));
    assert_eq!(tracker.delay_point, Some(point(2.)));
    assert_eq!(tracker.process(point(2.), &mut random), point(2.));
    tracker.settings.delay = false;
    tracker.settings.lazy = true;
    tracker.settings.threshold = [0.2, 0.2];
    tracker.threshold_changed(&mut random);
    assert_eq!(tracker.process(point(0.), &mut random), point(0.));
    assert_eq!(tracker.process(point(0.1), &mut random), point(0.));
    assert_eq!(tracker.process(point(0.3), &mut random), point(0.3));
    tracker.settings.lazy = false;
    tracker.settings.gaussian = true;
    tracker.settings.yaw_offset = [1., 1.];
    tracker.settings.pitch_offset = [1., 1.];
    tracker.settings.dynamic = true;
    tracker.settings.dynamic_yaw = 10.;
    tracker.settings.dynamic_pitch = 10.;
    random.hit = true;
    let out = tracker.process(point(0.), &mut random);
    assert_eq!(out, point(0.));
    assert_eq!(random.gaussians, 3);
    assert!(tracker.target_offset.0 < 1.);
    let old = tracker.target_offset;
    let out = tracker.process(point(0.), &mut random);
    assert!(out.pos.0 > 0.);
    assert_eq!(out.bounds.lo.0, out.pos.0);
    assert_eq!(tracker.target_offset, old);
    let shifted = point(0.).offset(V(5., 6., 7.));
    assert_eq!(shifted.pos, V(5., 7., 7.));
    assert_eq!(shifted.bounds.lo, V(5., 6., 7.));
    let mut tracker = PointTracker::default();
    tracker.settings.exempt_parts = [true, true, true];
    let box_ = B {
        lo: V(-0.3, 0., 3.7),
        hi: V(0.3, 1.8, 4.3),
    };
    let selected = tracker.find(V(0., 1.6, 0.), box_, &mut random);
    assert!(selected.pos.2 >= 3.7 && selected.pos.2 <= 4.3);
    let fishing = auto_bow_math::Trajectory {
        velocity: 1.5,
        gravity: 0.04,
        drag: 0.92,
        water_drag: 0.92,
        radius: 0.25,
        copies_velocity: true,
    };
    assert!(
        auto_bow_math::calculate(
            fishing,
            V(0., 1.62, 0.),
            [0.6, 1.8],
            |_| V(0., 0., 4.),
            |_, _| true
        )
        .is_some()
    );
    println!(
        "AutoRod focused smoke: slots, guards, coroutine phases, input windows, persistent PointTracker and fishing solver passed"
    );
}
