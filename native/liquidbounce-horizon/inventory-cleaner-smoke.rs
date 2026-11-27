#![allow(dead_code)]
mod vomit {
    #[derive(Clone, Debug, Default)]
    pub struct Constraints;
}
mod inventory_manager {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Slot {
        Player(u8),
        Container(i32),
    }
}
#[path = "../liquidbounce-behaviors/src/auto_weapon.rs"]
mod auto_weapon;
#[path = "../liquidbounce-behaviors/src/inventory_cleaner.rs"]
mod inventory_cleaner;
use inventory_cleaner::*;
use inventory_manager::Slot;
use std::collections::BTreeSet;
fn stack(slot: u8, id: &str, count: i32) -> Item {
    Item {
        slot: Slot::Player(slot),
        id: id.into(),
        components: id.into(),
        patch: id.into(),
        count,
        maximum: 64,
        stackable: true,
        ..Default::default()
    }
}
fn main() {
    let mut settings = InventoryCleaner::default();
    settings.slots = [23; 10];
    settings.slots[8] = 20;
    settings.slots[9] = 20;
    settings.max_blocks = 64;
    settings.throw_unknown = true;
    let mut a = stack(9, "stone", 30);
    a.classes.insert("BlockItem".into());
    a.block = Some(BlockFacts {
        valid: true,
        favourable: true,
        full: true,
        hardness: 1.5,
        ..Default::default()
    });
    let mut b = a.clone();
    b.slot = Slot::Player(10);
    b.count = 40;
    let junk = stack(11, "unknown", 1);
    let mut named = stack(12, "lobby", 1);
    named.custom_name = true;
    let mut worn = stack(36, "worn", 1);
    worn.armor = Some(0);
    worn.defense = 3.;
    worn.weapon.durability = 100;
    let mut upgrade = worn.clone();
    upgrade.slot = Slot::Player(13);
    upgrade.id = "diamond".into();
    upgrade.defense = 4.;
    let items = vec![a.clone(), b.clone(), junk, named, worn, upgrade];
    let armor = ArmorSettings::default();
    let plan = settings.plan(
        &items,
        false,
        f64::INFINITY,
        &armor,
        &BTreeSet::new(),
        false,
        true,
    );
    assert!(!plan.useful.contains(&Slot::Player(11)));
    assert!(plan.useful.contains(&Slot::Player(12)));
    assert!(plan.useful.contains(&Slot::Player(36)));
    assert!(plan.useful.contains(&Slot::Player(13)));
    assert_eq!(
        plan.swaps
            .iter()
            .filter(|s| s.priority == 0)
            .map(|s| s.from)
            .collect::<Vec<_>>(),
        vec![Slot::Player(10), Slot::Player(9)]
    );
    let merge = CleanupPlan {
        mergeable: vec![vec![Slot::Player(9), Slot::Player(10)]],
        ..Default::default()
    };
    assert!(merge.merge_slots(&items).is_empty());
    b.count = 20;
    let merge_items = vec![a, b];
    assert_eq!(merge.merge_slots(&merge_items), vec![Slot::Player(9)]);
    settings.slots = [23; 10];
    settings.max_blocks = 1;
    let limited = settings.plan(
        &merge_items,
        false,
        f64::INFINITY,
        &armor,
        &BTreeSet::new(),
        false,
        true,
    );
    assert_eq!(
        limited
            .useful
            .iter()
            .filter(|s| matches!(s, Slot::Player(9 | 10)))
            .count(),
        1
    );
    let locked = BTreeSet::from([Slot::Player(9)]);
    settings.slots[8] = 20;
    let plan = settings.plan(
        &merge_items,
        false,
        f64::INFINITY,
        &armor,
        &locked,
        false,
        true,
    );
    assert!(plan.swaps.iter().all(|s| s.from != Slot::Player(9)));
    let mut raw = armor;
    raw.mode = ArmorMode::RawDefense;
    raw.skip_cursed = 1;
    let mut cursed = stack(14, "cursed", 1);
    cursed.armor = Some(0);
    cursed.defense = 9.;
    cursed.curse_mask = 1;
    assert!(!raw.allowed(&cursed));
    cursed.slot = Slot::Player(36);
    assert!(raw.allowed(&cursed));
    println!(
        "InventoryCleaner: classification, allocation, quantities, names, armor, locks and merge smoke passed"
    );
}
