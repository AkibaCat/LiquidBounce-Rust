#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_bow.rs"]
mod auto_bow;
#[path = "../liquidbounce-behaviors/src/auto_shoot.rs"]
mod auto_shoot;
use auto_bow::{Candidate, Priority};
fn candidate(id: i32, distance: f64, type_weight: i32) -> Candidate {
    Candidate {
        id,
        distance,
        type_weight,
        health: 20.,
        cosine: 1.,
        hurt: 0,
        age: 0,
    }
}
fn main() {
    let mut s = auto_shoot::AutoShoot::default();
    assert_eq!(s.settings.range, [3., 6.]);
    assert!(s.projectile(true));
    assert!(!s.projectile(false));
    s.settings.gravity = 1;
    assert!(!s.projectile(true));
    s.settings.gravity = 2;
    assert!(s.projectile(false));
    assert!(s.accepts("minecraft:egg", 1, true));
    assert!(!s.accepts("minecraft:egg", 0, true));
    assert!(!s.accepts("minecraft:stone", 1, false));
    s.settings.custom = true;
    assert!(!s.accepts("minecraft:egg", 0, true));
    assert!(!s.accepts("minecraft:stone", 1, false));
    s.settings.whitelist = false;
    assert!(s.accepts("minecraft:stone", 1, false));
    assert!(!s.accepts("minecraft:egg", 1, true));
    let mut slots = [false; 10];
    slots[4] = true;
    slots[9] = true;
    assert_eq!(s.choose_slot(slots, 4), Some(4));
    assert_eq!(s.choose_slot(slots, 3), Some(40));
    slots[9] = false;
    slots[2] = true;
    assert_eq!(s.choose_slot(slots, 3), Some(2));
    assert_eq!(s.choose_slot([false; 10], 0), None);
    s.select(
        vec![
            candidate(1, 8.999, 0),
            candidate(2, 9., 1),
            candidate(3, 36., 0),
            candidate(4, 36.001, 0),
        ],
        |_| true,
    );
    assert_eq!(s.target, Some(3));
    assert_eq!(s.closest_squared_enemy_distance, 9.);
    s.select(vec![candidate(2, 9., 1), candidate(3, 36., 0)], |id| {
        id == 2
    });
    assert_eq!(s.target, Some(2));
    s.settings.priority = vec![Priority::Distance];
    s.select(vec![candidate(2, 9., 1), candidate(3, 36., 0)], |_| true);
    assert_eq!(s.target, Some(2));
    s.select(vec![], |_| true);
    assert!(s.target.is_none());
    assert_eq!(s.closest_squared_enemy_distance, 9.);
    s.target = Some(9);
    s.settings.requires_aura = true;
    assert!(s.pause_for_aura(false));
    assert!(s.target.is_none());
    assert!(!s.pause_for_aura(true));
    s.target = Some(10);
    s.disabled();
    assert!(s.target.is_none());
    assert!(s.settings.requires_aura);
    println!(
        "AutoShoot source defaults, range/type/visibility selection, slot order, filters, gravity and lifecycle passed"
    );
}
