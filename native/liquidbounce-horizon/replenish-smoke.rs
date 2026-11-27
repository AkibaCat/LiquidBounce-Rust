// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(dead_code)]
mod vomit {
    #[derive(Clone, Debug, Default)]
    pub struct Constraints;
}
#[path = "../liquidbounce-behaviors/src/replenish.rs"]
mod replenish;
use replenish::{Click, Input, Item, Replenish, Step};
fn stack(count: i32) -> Item {
    Item {
        components: "stone+components".into(),
        count,
        maximum: 64,
    }
}
fn next(s: &mut Replenish, stacks: &[Item], now: i64) -> Vec<Click> {
    match s.next(stacks, true, now) {
        Step::Chain(c) => c,
        other => panic!("expected chain, got {other:?}"),
    }
}
#[test]
fn replenish_slot_order_modes_memory_and_wait() {
    let mut stacks = vec![Item::default(); 41];
    stacks[40] = stack(2);
    stacks[0] = stack(5);
    stacks[9] = stack(50);
    stacks[10] = stack(3);
    stacks[11] = stack(3);
    let mut s = Replenish::default();
    assert!(s.begin(0));
    let c = next(&mut s, &stacks, 0);
    assert_eq!(c.iter().map(|c| c.slot).collect::<Vec<_>>(), vec![11, 40]);
    let c = next(&mut s, &stacks, 1);
    assert_eq!(c[0].slot, 10);
    let c = next(&mut s, &stacks, 2);
    assert_eq!(c.iter().map(|c| c.slot).collect::<Vec<_>>(), vec![9, 40]);
    assert_eq!(s.next(&stacks, true, 3), Step::Wait);
    assert!(!s.ready(42));
    assert!(s.ready(43));
    let c = next(&mut s, &stacks, 43);
    assert_eq!(c[1].slot, 0);
    s.cancel();
    s.features = 4;
    assert!(s.begin(84));
    let c = next(&mut s, &stacks, 84);
    assert_eq!(
        c,
        vec![Click {
            slot: 9,
            button: 40,
            input: Input::Swap
        }]
    );
    s.cancel();
    s.features = 2;
    assert!(s.begin(125));
    let c = next(&mut s, &stacks, 125);
    assert_eq!(
        c.iter().map(|c| c.input).collect::<Vec<_>>(),
        vec![Input::Pickup, Input::PickupAll, Input::Pickup]
    );
    // Non-stackables are remembered, and an empty remembered slot can still refill.
    let mut memory = Replenish::default();
    stacks.fill(Item::default());
    stacks[0] = Item {
        maximum: 1,
        ..stack(1)
    };
    assert!(memory.begin(0));
    assert_eq!(memory.next(&stacks, true, 0), Step::Done);
    stacks[0] = Item::default();
    stacks[9] = Item {
        maximum: 1,
        ..stack(1)
    };
    assert!(!memory.begin(40));
    assert!(memory.begin(41));
    assert_eq!(next(&mut memory, &stacks, 41)[1].slot, 0);
    memory.cancel();
    memory.clear_tracked();
    assert!(memory.begin(82));
    assert_eq!(memory.next(&stacks, true, 82), Step::Done);
    // The literal source overflow click targets the hotbar again.
    let mut overflow = Replenish::default();
    stacks[0] = stack(5);
    stacks[9] = stack(64);
    assert!(overflow.begin(0));
    assert_eq!(
        next(&mut overflow, &stacks, 0)
            .iter()
            .map(|c| c.slot)
            .collect::<Vec<_>>(),
        vec![9, 0, 0]
    );
    assert!(!s.running(true, true, false));
    assert!(!s.running(true, true, true));
    assert!(s.running(true, false, false));
    s.inside_of = 3;
    assert!(s.running(true, true, true));
    assert!(!s.running(false, false, false));
}
