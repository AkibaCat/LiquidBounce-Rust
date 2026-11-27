#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/clock.rs"]
mod clock;
#[path = "../liquidbounce-behaviors/src/extinguish.rs"]
mod extinguish;
#[path = "../liquidbounce-behaviors/src/timed_pickup_tracker.rs"]
mod timed_pickup_tracker;
use extinguish::{Action, Context, Extinguish};
fn main() {
    let mut tracker = timed_pickup_tracker::TimedPickupTracker::new(2);
    tracker.record([1, 2, 3], 10);
    tracker.record([4, 5, 6], 20);
    assert_eq!(tracker.first_eligible(10, 20, |_| true), None);
    assert_eq!(tracker.first_eligible(10, 21, |_| true), Some([1, 2, 3]));
    tracker.prune(10, 20, |_| true);
    assert_eq!(tracker.first_eligible(0, 20, |_| true), Some([1, 2, 3]));
    tracker.prune(10, 21, |_| true);
    assert_eq!(tracker.first_eligible(0, 21, |_| true), Some([4, 5, 6]));
    tracker.record([7, 8, 9], 22);
    tracker.record([10, 11, 12], 23);
    assert_eq!(tracker.first_eligible(0, 24, |_| true), Some([7, 8, 9]));
    let mut s = Extinguish::<Action>::default();
    let c = |now| Context {
        now,
        on_fire: true,
        ..Default::default()
    };
    assert_eq!(s.settings.pickup_span, [0.1, 10.]);
    s.rotation_update(c(1000), Some);
    assert_eq!(s.current_target, Some(Action::Extinguish));
    s.succeeded([2, 3, 4], 1000);
    assert_eq!(s.cooldown.last_update(), 2000);
    s.rotation_update(c(1100), Some);
    assert_eq!(s.current_target, None);
    s.rotation_update(c(1101), Some);
    assert_eq!(s.current_target, Some(Action::Pickup([2, 3, 4])));
    s.rotation_update(
        Context {
            water_evaporates: true,
            ..c(9999)
        },
        |_| panic!("Nether must retain plan before pruning"),
    );
    assert_eq!(s.current_target, Some(Action::Pickup([2, 3, 4])));
    s.world_changed();
    assert!(s.current_target.is_none());
    s.rotation_update(
        Context {
            fire_resistant: true,
            ..c(1200)
        },
        |_| panic!("Fire resistance must guard pickup"),
    );
    assert!(s.current_target.is_none());
    s.rotation_update(
        Context {
            in_combat: true,
            ..c(1300)
        },
        |_| panic!("Combat must guard pickup"),
    );
    s.settings.pickup = false;
    s.rotation_update(c(2000), Some);
    assert!(s.current_target.is_none());
    s.rotation_update(c(2001), Some);
    assert_eq!(s.current_target, Some(Action::Extinguish));
    s.settings.pickup = true;
    s.rotation_update(c(2001), |a| {
        if matches!(a, Action::Pickup(_)) {
            None
        } else {
            Some(a)
        }
    });
    assert_eq!(s.current_target, Some(Action::Extinguish));
    s.enabled();
    assert!(s.current_target.is_none());
    assert_eq!(s.cooldown.last_update(), 2000);
    assert_eq!(s.pickup_tracker.first_eligible(0, 3000, |_| true), None);
    s.succeeded([9, 9, 9], 3000);
    s.succeeded([8, 8, 8], 2900);
    assert_eq!(s.cooldown.last_update(), 4000);
    assert_eq!(
        s.pickup_tracker.first_eligible(0, 3000, |_| true),
        Some([8, 8, 8])
    );
    println!(
        "Extinguish strict windows, FIFO, guards, fallback, lifecycle and future cooldown passed"
    );
}
