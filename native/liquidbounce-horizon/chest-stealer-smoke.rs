#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_weapon.rs"]
mod auto_weapon;
#[path = "../liquidbounce-behaviors/src/chest_cleaner.rs"]
mod chest_cleaner;
#[path = "../liquidbounce-behaviors/src/chest_stealer.rs"]
mod chest_stealer;
#[path = "../liquidbounce-behaviors/src/inventory_cleaner.rs"]
mod inventory_cleaner;
#[path = "../liquidbounce-behaviors/src/inventory_manager.rs"]
mod inventory_manager;
#[path = "../liquidbounce-behaviors/src/random.rs"]
mod random;
#[path = "../liquidbounce-behaviors/src/vomit.rs"]
mod vomit;
use chest_stealer::*;
use inventory_cleaner::*;
use inventory_manager::*;
use std::collections::{BTreeSet, VecDeque};
fn stack(slot: Slot, id: &str, count: i32) -> Item {
    Item {
        slot,
        id: id.into(),
        components: id.into(),
        patch: id.into(),
        count,
        maximum: 64,
        default_maximum: 64,
        stackable: true,
        ..Default::default()
    }
}
fn main() {
    let armor = ArmorSettings::default();
    let screen = Screen {
        object: 1,
        container_id: 1,
        geometry: Geometry {
            object: 2,
            slots: vec![Position::default(); 63],
        },
        non_player_slots: (0..27).collect(),
    };
    let mut settings = ChestStealer::default();
    let mut random = random::ThreadRandom::seeded(42, 1);
    let mut sword = stack(Slot::Container(4), "sword", 1);
    sword.weapon.kinds = auto_weapon::SWORD;
    sword.classes.insert("SwordItem".into());
    let mut helmet = stack(Slot::Container(10), "helmet", 1);
    helmet.armor = Some(3);
    let stone = stack(Slot::Container(2), "stone", 40);
    let items = vec![sword.clone(), helmet.clone(), stone.clone()];
    let original = vec![stone.slot, sword.slot, helmet.slot];
    let mut slots = original.clone();
    settings.select(&mut slots, &items, false, 0., &armor, &mut random);
    assert_eq!(slots, vec![helmet.slot, sword.slot, stone.slot]);
    settings.selection = 2;
    settings.descending = true;
    settings.select(&mut slots, &items, false, 0., &armor, &mut random);
    assert_eq!(slots, vec![helmet.slot, sword.slot, stone.slot]);
    settings.descending = false;
    settings.select(&mut slots, &items, false, 0., &armor, &mut random);
    assert_eq!(slots, original);
    settings.selection = 1;
    settings.start = 2;
    settings.select(&mut slots, &items, false, 0., &armor, &mut random);
    assert_eq!(slots[0], helmet.slot);
    assert_eq!(slots[1], stone.slot);
    settings.selection = 3;
    settings.select(&mut slots, &items, false, 0., &armor, &mut random);
    assert_eq!(
        slots.iter().copied().collect::<BTreeSet<_>>(),
        original.iter().copied().collect()
    );
    let actions = settings
        .quick_swap(
            &items,
            &BTreeSet::new(),
            &screen,
            sword.slot,
            Slot::Player(0),
            true,
        )
        .unwrap();
    assert!(matches!(
        actions.as_slice(),
        [Action::Click {
            slot: Slot::Container(4),
            input: Input::Swap,
            button: 0,
            ..
        }]
    ));
    let mut with_hotbar = items.clone();
    with_hotbar.push(stack(Slot::Player(0), "old", 1));
    let actions = settings
        .quick_swap(
            &with_hotbar,
            &BTreeSet::from([Slot::Player(0)]),
            &screen,
            sword.slot,
            Slot::Player(0),
            true,
        )
        .unwrap();
    assert!(matches!(
        actions.as_slice(),
        [
            Action::Click {
                slot: Slot::Player(9),
                input: Input::Swap,
                ..
            },
            Action::Click {
                slot: Slot::Container(4),
                input: Input::Swap,
                ..
            }
        ]
    ));
    assert!(
        settings
            .quick_swap(
                &items,
                &BTreeSet::new(),
                &screen,
                sword.slot,
                Slot::Player(40),
                true
            )
            .is_some()
    );
    with_hotbar.push(stack(Slot::Player(40), "offhand", 1));
    assert!(
        settings
            .quick_swap(
                &with_hotbar,
                &BTreeSet::new(),
                &screen,
                sword.slot,
                Slot::Player(40),
                true
            )
            .is_some()
    );
    let useless = settings
        .quick_swap(
            &with_hotbar,
            &BTreeSet::new(),
            &screen,
            sword.slot,
            Slot::Player(0),
            true,
        )
        .unwrap();
    assert!(matches!(
        useless.as_slice(),
        [
            Action::Click {
                input: Input::Throw,
                button: 1,
                ..
            },
            Action::Click {
                input: Input::Swap,
                ..
            }
        ]
    ));
    settings.drag = true;
    let actions = settings.pick(&items, &screen, stone.slot).unwrap();
    assert!(matches!(
        actions.as_slice(),
        [
            Action::Click {
                slot: Slot::Container(2),
                input: Input::Pickup,
                ..
            },
            Action::Click {
                slot: Slot::Player(0),
                input: Input::Pickup,
                ..
            }
        ]
    ));
    let mut full = (0..36)
        .map(|i| stack(Slot::Player(i), "full", 64))
        .collect::<Vec<_>>();
    full.push(stone.clone());
    assert!(settings.pick(&full, &screen, stone.slot).is_none());
    full[2] = stack(Slot::Player(2), "stone", 54);
    let actions = settings.pick(&full, &screen, stone.slot).unwrap();
    assert!(matches!(
        actions.last(),
        Some(Action::Click {
            slot: Slot::Container(2),
            input: Input::Pickup,
            ..
        })
    ));
    assert_eq!(actions.len(), 3);
    let plan = CleanupPlan {
        useful: BTreeSet::new(),
        swaps: vec![],
        mergeable: vec![vec![stone.slot, Slot::Player(2)]],
    };
    assert_eq!(required_space(&full, &plan, 1), 1);
    full[2].count = 20;
    assert_eq!(required_space(&full, &plan, 1), 0);
    let mut queue = VecDeque::from([Slot::Player(40), Slot::Player(0)]);
    assert!(matches!(
        settings
            .throw_next(&full, &screen, &mut queue)
            .unwrap()
            .as_slice(),
        [Action::Click {
            slot: Slot::Player(0),
            input: Input::Throw,
            ..
        }]
    ));
    settings.throw_full = false;
    assert!(
        settings
            .throw_next(&full, &screen, &mut VecDeque::from([Slot::Player(0)]))
            .is_none()
    );
    assert!(settings.aura.running(true, false, false));
    assert!(!settings.aura.running(true, true, false));
    assert_eq!(settings.silent.color, i32::MIN);
    assert_eq!(settings.silent.scale, 1.5);
    println!(
        "ChestStealer: selection, priority, swaps, drag chains, capacity, merge budget and on-full smoke passed"
    );
}
