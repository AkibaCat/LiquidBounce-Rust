#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_bow_math.rs"]
mod auto_bow_math;
#[path = "../liquidbounce-behaviors/src/easy_pearl.rs"]
mod easy_pearl;
#[path = "../liquidbounce-behaviors/src/rotations.rs"]
mod rotations;
use auto_bow_math::V;
use easy_pearl::*;
use rotations::Rotation;
fn main() {
    let mut model = EasyPearl::default();
    let target = V(6., 2., 12.);
    let rotation = Rotation::new(20., -25.);
    assert_eq!(
        model.settings,
        Settings {
            aim_off_threshold: 2.,
            reachable_check: true,
            show_distance: true
        }
    );
    assert_eq!(
        model.interact(false, true, target, false, Some(rotation), 10.),
        Interaction::default()
    );
    assert_eq!(
        model.interact(true, false, target, false, Some(rotation), 10.),
        Interaction::default()
    );
    model.current_target_rotation = Some(rotation);
    assert_eq!(
        model.interact(true, true, target, false, None, 0.),
        Interaction {
            cancel: true,
            warning: true
        }
    );
    assert_eq!(model.target_position, None);
    assert_eq!(model.current_target_rotation, Some(rotation));
    assert_eq!(
        model.interact(true, true, target, true, None, f32::INFINITY),
        Interaction::default()
    );
    assert_eq!(model.target_position, None); // MISS bypasses reachable warning; missing solve counts as done.
    model.settings.reachable_check = false;
    assert_eq!(
        model.interact(true, true, target, false, None, f32::INFINITY),
        Interaction::default()
    );
    model.settings.reachable_check = true;
    assert_eq!(
        model.interact(true, true, target, false, Some(rotation), 2.),
        Interaction::default()
    );
    assert!(
        model
            .interact(true, true, target, false, Some(rotation), 2.0001)
            .cancel
    );
    assert_eq!(model.target_position, Some(target));
    model.disabled();
    assert_eq!(model.target_position, Some(target));
    assert_eq!(model.current_target_rotation, None);
    assert_eq!(model.tick(None, false, Some(-1)), None);
    assert_eq!(model.current_target_rotation, None);
    assert_eq!(model.tick(Some(rotation), false, Some(-1)), None);
    assert_eq!(model.current_target_rotation, Some(rotation));
    assert_eq!(model.tick(None, false, Some(-1)), None);
    assert_eq!(model.current_target_rotation, Some(rotation));
    assert_eq!(model.tick(Some(rotation), true, None), None);
    assert_eq!(model.target_position, Some(target));
    assert_eq!(
        model.tick(Some(rotation), true, Some(-1)),
        Some((-1, rotation))
    );
    // Real use may recursively replace the target. Both fields clear only after it returns.
    let next = V(2., 4., 20.);
    model.interact(true, true, next, false, Some(rotation), 3.);
    assert_eq!(model.target_position, Some(next));
    model.thrown();
    assert_eq!(model.target_position, None);
    assert_eq!(model.current_target_rotation, None);
    model.current_target_rotation = Some(rotation);
    assert_eq!(model.tick(Some(rotation), true, Some(2)), None);
    assert_eq!(model.current_target_rotation, Some(rotation));
    assert_eq!(target_color(true, 50), 0x3220c206);
    assert_eq!(target_color(false, 200), 0xc8d70909);
    println!("EasyPearl production state transitions passed");
}
