#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_bow.rs"]
mod auto_bow;
#[path = "../liquidbounce-behaviors/src/auto_bow_math.rs"]
mod math;
use auto_bow::*;
use math::{B, Trajectory, V};
fn candidate(id: i32, type_weight: i32, distance: f64) -> Candidate {
    Candidate {
        id,
        type_weight,
        health: 10.,
        distance,
        cosine: 1.,
        hurt: 0,
        age: 10,
    }
}
fn shot(weapon: Weapon) -> Shot {
    Shot {
        weapon,
        hand: 1,
        charged: false,
        ticks: 20,
        pull_time: 25,
        now: 10000,
        hit: false,
        active_aim: Some(0.),
    }
}
fn main() {
    let mut settings = Settings::default();
    let c = vec![
        candidate(1, 1, 1.),
        candidate(2, 0, 20.),
        candidate(3, 0, 4.),
        candidate(4, 0, 4.),
    ];
    assert_eq!(
        targets(&settings, c.clone())
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        [3, 4, 2, 1]
    );
    settings.priority = vec![Priority::Distance, Priority::Type];
    assert_eq!(
        targets(&settings, c)
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>(),
        [1, 3, 4, 2]
    );
    let mut immune = candidate(9, 0, 1.);
    immune.hurt = 11;
    let mut behind = candidate(10, 0, 2.);
    behind.cosine = -1.;
    settings.fov = 90.;
    assert!(targets(&settings, vec![immune, behind]).is_empty());
    settings.priority = vec![Priority::Health];
    settings.fov = 180.;
    let mut nan = candidate(11, 0, 0.);
    nan.health = f32::from_bits(0xffc00001);
    assert_eq!(
        targets(&settings, vec![nan, candidate(12, 0, 0.)])[0].id,
        12
    );
    let mut bow = AutoBow::default();
    let calls = std::cell::Cell::new(0);
    let mut gaussian = || {
        calls.set(calls.get() + 1);
        99.
    };
    bow.settings.charged_random = [-3., 5.];
    bow.shoot(Some(shot(Weapon::Bow)), &mut gaussian);
    assert_eq!(bow.charge_random, Some(5));
    assert!(!bow.force_uncharged);
    assert_eq!(calls.get(), 1);
    bow.target = Some(3);
    let mut s = shot(Weapon::Bow);
    s.ticks = 19;
    bow.shoot(Some(s), &mut gaussian);
    assert!(!bow.force_uncharged);
    s.ticks = 20;
    s.active_aim = Some(2.);
    bow.shoot(Some(s), &mut gaussian);
    assert!(!bow.force_uncharged);
    s.active_aim = Some(0.);
    bow.shoot(Some(s), &mut gaussian);
    assert!(bow.force_uncharged);
    assert_eq!(calls.get(), 2);
    bow.stop_using(true, 10000);
    bow.settings.delay = 1.;
    s.now = 11000;
    bow.shoot(Some(s), &mut gaussian);
    assert!(!bow.force_uncharged);
    s.now = 11001;
    bow.shoot(Some(s), &mut gaussian);
    assert!(bow.force_uncharged);
    bow.settings.requires_hit = true;
    s.hit = false;
    bow.shoot(Some(s), &mut gaussian);
    assert!(!bow.force_uncharged);
    s.hit = true;
    bow.target = None;
    s.active_aim = None;
    bow.shoot(Some(s), &mut gaussian);
    assert!(bow.force_uncharged);
    let cached = bow.charge_random;
    bow.disabled();
    assert_eq!(bow.charge_random, cached);
    assert_eq!(bow.last_shot, 10000);
    assert!(!bow.force_uncharged);
    bow.settings.aimbot = false;
    bow.settings.requires_hit = false;
    bow.settings.delay = 0.;
    s = shot(Weapon::Trident);
    s.now = 12000;
    s.ticks = 10;
    bow.shoot(Some(s), &mut gaussian);
    assert!(!bow.force_uncharged);
    s.ticks = 11;
    bow.shoot(Some(s), &mut gaussian);
    assert!(bow.force_uncharged);
    s = shot(Weapon::Crossbow);
    s.now = 12000;
    s.ticks = 24;
    bow.shoot(Some(s), &mut gaussian);
    assert!(!bow.force_uncharged);
    s.charged = true;
    assert_eq!(bow.shoot(Some(s), &mut gaussian), ShootEffect::Use(1));
    assert_eq!(bow.last_shot, 12000);
    bow.settings.fast_charge = true;
    assert!(!bow.fast_charge(true, false, false, false));
    assert!(bow.fast_charge(true, true, false, false));
    bow.settings.stationary_only = true;
    bow.settings.no_regeneration = true;
    assert!(!bow.fast_charge(true, true, true, false));
    assert!(!bow.fast_charge(true, true, false, true));
    assert!(!bow.fast_charge(false, true, false, false));
    let b = B {
        lo: V(10., -1., -1.),
        hi: V(11., 1., 1.),
    };
    assert_eq!(math::max_cos(V::default(), V(1., 0., 0.), b), 1.);
    assert!(math::max_cos(V::default(), V(-1., 0., 0.), b) < 0.);
    assert_eq!(b.clip(V::default(), V(20., 0., 0.)), Some(V(10., 0., 0.)));
    assert!(b.clip(V::default(), V(10., 0., 0.)).is_none());
    let points = math::projected_points(V::default(), b);
    assert!(!points.is_empty() && points.len() <= 128);
    assert!(
        points
            .iter()
            .all(|p| (p.0 - 10.).abs() < 1e-6 && p.1.abs() <= 1. && p.2.abs() <= 1.)
    );
    assert!(math::visible_point(V::default(), b, |_, _| false).is_none());
    assert!(math::visible_point(V::default(), b, |_, _| true).is_some());
    assert!(Trajectory::bow(0).is_none());
    assert_eq!(Trajectory::bow(20).unwrap().velocity, 3.);
    assert_eq!(Trajectory::crossbow(true).gravity, 0.);
    assert_eq!(Trajectory::trident().velocity, 2.5);
    let source = V(0., 1.62, 0.);
    let info = Trajectory::bow(20).unwrap();
    let close = math::calculate(info, source, [0.6, 1.8], |_| V(0., 0., 3.), |_, _| false).unwrap();
    assert!(close.iter().all(|n| n.is_finite()));
    let far = math::calculate(info, source, [0.6, 1.8], |_| V(20., 0., 0.), |_, _| true).unwrap();
    assert!(far.iter().all(|n| n.is_finite()));
    assert!(math::calculate(info, source, [0.6, 1.8], |_| V(20., 0., 0.), |_, _| false).is_none());
    println!(
        "AutoBow focused smoke passed: release gates, priority order, charge cache, exact timers, fast-charge guards and both projectile solvers"
    );
}
