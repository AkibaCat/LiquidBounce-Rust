#[path = "../liquidbounce-behaviors/src/vehicle_control.rs"]
mod vehicle_control;
use vehicle_control::*;
#[test]
fn complete_velocity_and_rehook_smoke() {
    let mut s = VehicleControl::default();
    let mut i = Input {
        moving: true,
        ..Default::default()
    };
    assert_eq!(
        s.velocity(i, [0., 1.], None),
        [0., f64::from(-0.15f32), 0.5]
    );
    i.sprint = true;
    i.jump = true;
    assert_eq!(s.velocity(i, [0., 1.], None), [0., 2., 5.]);
    i.sneak = true;
    assert_eq!(s.velocity(i, [0., 1.], None)[1], 2.);
    i.jump = false;
    assert_eq!(s.velocity(i, [0., 1.], None)[1], -2.);
    i.sneak = false;
    i.vehicle_water = true;
    assert_eq!(s.velocity(i, [0., 1.], None)[1], 0.);
    assert_eq!(s.velocity(i, [0., 1.], Some([2., 3., 4.])), [2., 3., 4.]);
    assert!(s.movement_notice(true, true));
    assert!(!s.movement_notice(true, true));
    assert!(!s.movement_notice(false, true));
    assert!(s.movement_notice(true, true));
    assert_eq!(s.input(true, false, true), (false, false));
    assert_eq!(s.input(true, true, true), (true, false));
    s.settings.rehook = true;
    s.settings.unhook_after = 2;
    s.settings.hook_after = 2;
    let (_, id) = s.start_rehook(true, false, false);
    let id = id.unwrap();
    assert!(s.callback(id, Some(7)).0);
    let (keep, next) = s.callback(id, Some(7));
    assert!(!keep);
    assert_eq!(s.vehicle_id, 7);
    assert_eq!(s.input(false, false, false), (true, true));
    assert!(!s.callback(next.unwrap(), None).0);
    assert_eq!(
        s.start_rehook(false, true, true).0,
        Some(RehookAction::Interact(7))
    );
    assert_eq!(
        s.start_rehook(false, true, true).0,
        Some(RehookAction::ForceMount(7))
    );
    assert_eq!(
        s.start_rehook(false, true, false).0,
        Some(RehookAction::TooFar)
    );
    s.vehicle_id = 8;
    assert_eq!(
        s.start_rehook(false, false, true).0,
        Some(RehookAction::Gone)
    );
    let (_, id) = s.start_rehook(true, false, false);
    s.cancel();
    assert!(!s.callback(id.unwrap(), Some(1)).0);
}
