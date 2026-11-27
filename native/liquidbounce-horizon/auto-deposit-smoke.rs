// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(dead_code)]
mod vomit {
    #[derive(Clone, Debug, Default)]
    pub struct Constraints;
}
#[path = "../liquidbounce-behaviors/src/auto_deposit.rs"]
mod auto_deposit;
#[path = "../liquidbounce-behaviors/src/chest_cleaner.rs"]
mod chest_cleaner;
#[test]
fn filter_screen_and_punch_boundaries() {
    let mut s = auto_deposit::AutoDeposit::default();
    s.inventory.items.insert("minecraft:stone".into());
    let mut items = vec![("minecraft:stone".to_owned(), true); 36];
    items[8].1 = false;
    items[9].1 = false;
    assert_eq!(s.matching(&items, false), vec![8]);
    assert_eq!(s.matching(&items, true), vec![8, 9]);
    let mut translated = vec![vec![]; 16];
    translated[4] = vec!["箱子".into()];
    assert!(
        s.inventory
            .accepts_screen(false, Some("minecraft:generic_9x3"), "箱子", &translated)
    );
    assert!(
        !s.inventory
            .accepts_screen(false, Some("minecraft:furnace"), "箱子", &translated)
    );
    assert!(
        !s.inventory
            .accepts_screen(true, Some("minecraft:generic_9x3"), "箱子", &translated)
    );
    s.inventory.filter = chest_cleaner::Filter::Blacklist;
    assert!(s.matching(&items, true).is_empty());
    items[0] = ("minecraft:diamond".into(), false);
    assert_eq!(s.matching(&items, false), vec![0]);
    assert!(!s.punch.running(true, false, false));
    s.punch.enabled = true;
    assert!(s.punch.running(true, true, true));
    s.punch.pauses = 3;
    assert!(!s.punch.running(true, true, false));
    assert!(!s.punch.running(true, false, true));
    assert!(s.punch.running(true, false, false));
    assert!(!s.punch.running(false, false, false));
    assert_eq!(s.punch.blocks.len(), 2);
    assert!(s.punch.blocks.contains("minecraft:ender_chest"));
    assert_eq!(auto_deposit::SWINGS.len(), 4);
    assert_eq!(s.punch.delay, 5);
}
