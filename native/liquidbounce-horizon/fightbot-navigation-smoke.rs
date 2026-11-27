#[path = "navigation-owners.rs"]
mod owners;
use owners::{Holds, Owner, Slots};
#[test]
fn fight_navigation_smoke_holds_are_independent() {
    let mut holds = Holds::empty();
    assert!(!holds.any());
    holds.set(Owner::AutoWalk, true);
    holds.set(Owner::FightBot, true);
    assert!(holds.owned(Owner::AutoWalk) && holds.owned(Owner::FightBot));
    assert!(!holds.releasing_last(Owner::AutoWalk));
    holds.set(Owner::AutoWalk, false);
    assert!(holds.any());
    assert!(holds.owned(Owner::FightBot));
    assert!(holds.releasing_last(Owner::FightBot));
    holds.set(Owner::FightBot, false);
    assert!(!holds.any());
}
#[test]
fn fight_navigation_smoke_disabled_owner_cannot_stop_other_mining() {
    let mut holds = Holds::empty();
    holds.set(Owner::FightBot, true);
    assert!(!holds.releasing_last(Owner::AutoWalk));
    holds.set(Owner::AutoWalk, false);
    assert!(holds.releasing_last(Owner::FightBot));
    assert!(!Holds::empty().releasing_last(Owner::FightBot));
}
#[test]
fn fight_navigation_smoke_release_preserves_other_hotbar_claim() {
    let mut slots = Slots::default();
    slots.claim(Owner::AutoWalk, 2, 7);
    slots.claim(Owner::FightBot, 4, 2);
    assert_eq!(slots.release(Owner::AutoWalk), Some(4));
    assert_eq!(slots.release(Owner::FightBot), Some(7));
    assert_eq!(slots.release(Owner::AutoWalk), None);
}
#[test]
fn fight_navigation_smoke_last_selection_wins_and_original_returns() {
    let mut slots = Slots::default();
    slots.claim(Owner::AutoWalk, 2, 7);
    slots.claim(Owner::FightBot, 4, 2);
    slots.claim(Owner::AutoWalk, 5, 4);
    assert_eq!(slots.release(Owner::AutoWalk), Some(4));
    assert_eq!(slots.release(Owner::FightBot), Some(7));
    slots.claim(Owner::FightBot, 6, 8);
    assert_eq!(slots.release(Owner::FightBot), Some(8));
}
#[test]
fn fight_navigation_smoke_world_retirement_never_restores_old_inventory() {
    let mut slots = Slots::default();
    slots.claim(Owner::AutoWalk, 2, 7);
    slots.claim(Owner::FightBot, 4, 2);
    slots = Slots::default();
    assert_eq!(slots.release(Owner::AutoWalk), None);
    assert_eq!(slots.release(Owner::FightBot), None);
    slots.claim(Owner::FightBot, 5, 3);
    assert_eq!(slots.release(Owner::FightBot), Some(3));
}
