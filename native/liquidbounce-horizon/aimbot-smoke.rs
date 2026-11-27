#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/aimbot.rs"]
mod aimbot;
#[path = "../liquidbounce-behaviors/src/aimbot_ray.rs"]
mod aimbot_ray;
#[path = "../liquidbounce-behaviors/src/auto_bow.rs"]
mod auto_bow;
#[path = "../liquidbounce-behaviors/src/auto_bow_math.rs"]
mod auto_bow_math;
#[path = "../liquidbounce-behaviors/src/combat_rotations.rs"]
mod combat_rotations;
#[path = "../liquidbounce-behaviors/src/no_rotate_set.rs"]
mod no_rotate_set;
#[path = "../liquidbounce-behaviors/src/random.rs"]
mod random;
#[path = "../liquidbounce-behaviors/src/rotations.rs"]
mod rotations;
use aimbot::{Aimbot, Smooth};
use auto_bow_math::{B, V};
use rotations::Rotation;
fn main() {
    let mut a = Aimbot::default();
    assert_eq!(a.settings.range, 4.2);
    assert_eq!(a.settings.smooth, Smooth::Interpolation);
    assert_eq!(
        a.settings.priority,
        vec![auto_bow::Priority::Type, auto_bow::Priority::Direction]
    );
    assert!(a.requirements_met(false, [false; 6]));
    assert!(!a.requirements_met(true, [true; 6]));
    for i in 0..6 {
        a.settings.requires = [false; 6];
        a.settings.requires[i] = true;
        assert!(!a.requirements_met(false, [false; 6]));
        let mut r = [false; 6];
        r[i] = true;
        assert!(a.requirements_met(false, r));
    }
    a.settings.requires = [false; 6];
    a.prepare(Rotation::new(10., 20.), true);
    a.target = Some(1);
    a.settings.smooth = Smooth::Linear;
    a.aim(Some(Rotation::new(50., -20.)), None, &mut || 0.)
        .unwrap();
    let r = a.render(0.5, 1., false, false).unwrap();
    assert_eq!((r.yaw, r.pitch), (30., 0.));
    let r = a.render(0.25, 2., false, false).unwrap();
    assert_eq!((r.yaw, r.pitch), (30., 0.));
    a.settings.axis = [false, true];
    let r = a.render(0.5, 1., false, false).unwrap();
    assert_eq!((r.yaw, r.pitch), (10., 0.));
    a.settings.axis = [true, false];
    let r = a.render(0.5, 1., false, false).unwrap();
    assert_eq!((r.yaw, r.pitch), (30., 20.));
    a.settings.axis = [false; 2];
    let r = a.render(10., 1., false, false).unwrap();
    assert_eq!((r.yaw, r.pitch), (10., 20.));
    assert!(a.render(0.5, 1., true, false).is_none());
    assert!(a.render(0.5, 1., false, true).is_none());
    a.settings.ignore = [true, false];
    assert!(a.render(0.5, 1., true, false).is_some());
    assert!(a.render(0.5, 1., true, true).is_none());
    a.settings.ignore = [true; 2];
    assert!(a.render(0.5, 1., true, true).is_some());
    a.mouse(20., 1000.);
    assert_eq!(a.player_rotation.unwrap(), Rotation::new(13., 90.));
    assert_eq!(a.target_rotation.unwrap(), Rotation::new(53., 90.));
    let saved = a.player_rotation;
    a.clear();
    assert_eq!(a.player_rotation, saved);
    assert!(a.target.is_none() && a.target_rotation.is_none());
    a.prepare(Rotation::new(1., 2.), false);
    assert_eq!(a.player_rotation, Some(Rotation::new(1., 2.)));
    a.settings.axis = [true; 2];
    a.settings.ignore = [false; 2];
    a.prepare(Rotation::ZERO, true);
    a.settings.smooth = Smooth::Interpolation;
    a.settings.interpolation.horizontal = [50, 50];
    a.settings.interpolation.vertical = [50, 50];
    a.settings.interpolation.direction_change = [100, 100];
    a.aim(Some(Rotation::new(10., 0.)), None, &mut || 0.)
        .unwrap();
    let base = a.target_rotation.unwrap();
    a.aim(
        Some(Rotation::new(10., 0.)),
        Some(Rotation::new(180., 0.)),
        &mut || 0.,
    )
    .unwrap();
    assert!(a.target_rotation.unwrap().yaw > base.yaw);
    a.aim(None, None, &mut || 0.).unwrap();
    assert!(a.target_rotation.is_none());
    let eye = V(0., 1., 0.);
    let box_ = B {
        lo: V(-0.5, 0., 3.),
        hi: V(0.5, 2., 4.),
    };
    let center = V(0., 1., 3.5);
    assert!(aimbot_ray::raytrace(eye, box_, center, 3., 0., |_, _| true).is_none());
    assert_eq!(
        aimbot_ray::raytrace(eye, box_, center, 3.01, 0., |_, _| true),
        // Source Rotation.lookingAt widens Mth.RAD_TO_DEG (a Float) to
        // Double before multiplying atan2, then narrows before subtracting 90f.
        // A precisely forward vector therefore retains this Float residue.
        Some([-7.6293945e-6, -0.])
    );
    assert!(aimbot_ray::raytrace(eye, box_, center, 4., 3., |_, _| false).is_none());
    assert!(aimbot_ray::raytrace(eye, box_, center, 2., 3.01, |_, _| false).is_some());
    let alternative = aimbot_ray::raytrace(eye, box_, center, 5., 0., |_, p| p.0 > 0.2).unwrap();
    assert!(alternative[0] < 0.);
    let mut samples = 0;
    let hit = aimbot_ray::raytrace(eye, box_, center, 5., 4., |_, p| {
        samples += 1;
        p.0 > 0.2
    });
    // Hidden preferred points are accepted immediately inside wallsRange.
    assert!(hit.is_some());
    assert_eq!(samples, 1);
    let inside = V(0., 1., 3.5);
    let exit = aimbot_ray::first_hit(box_, inside, V(0., 1., 5.)).unwrap();
    assert_eq!(exit.2, 4.);
    assert!(aimbot_ray::raytrace(inside, box_, inside, 1., 0., |_, _| true).is_some());
    let defaults = auto_bow_math::projected_points(eye, box_);
    let more = auto_bow_math::projected_points_limit(eye, box_, 256);
    assert!(more.len() > defaults.len());
    assert_eq!(
        defaults,
        auto_bow_math::projected_points_limit(eye, box_, 128)
    );
    println!(
        "Aimbot guards, endpoints, timer/axis/mouse/disable, smoothing, strict ray ranges and surface sampling passed"
    );
}
