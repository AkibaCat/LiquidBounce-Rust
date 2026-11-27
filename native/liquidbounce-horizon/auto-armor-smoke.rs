#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_armor.rs"]
mod auto_armor;
#[path = "../liquidbounce-behaviors/src/auto_weapon.rs"]
mod auto_weapon;
#[path = "../liquidbounce-behaviors/src/inventory_cleaner.rs"]
mod inventory_cleaner;
#[path = "../liquidbounce-behaviors/src/inventory_manager.rs"]
mod inventory_manager;
#[path = "../liquidbounce-behaviors/src/random.rs"]
mod random;
#[path = "../liquidbounce-behaviors/src/vomit.rs"]
mod vomit;
use auto_armor::*;
use inventory_cleaner::*;
use inventory_manager::*;
use std::collections::BTreeSet;
fn piece(slot: u8, armor: u8, defense: f32, durability: i32) -> Item {
    Item {
        slot: Slot::Player(slot),
        id: format!("armor-{slot}"),
        components: format!("armor-{slot}"),
        count: 1,
        armor: Some(armor),
        defense,
        weapon: auto_weapon::WeaponStack {
            durability,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn main() {
    let mut s = AutoArmor::default();
    let empty = BTreeSet::new();
    let worn = piece(39, 3, 3., 12);
    let replacement = piece(12, 3, 2., 100);
    let mut cursed = piece(1, 3, 5., 150);
    cursed.curse_mask = 1;
    let items = vec![worn.clone(), replacement.clone(), cursed.clone()];
    s.skip_cursed = 1;
    assert_eq!(
        s.pieces(&items, &empty, false),
        vec![Piece {
            source: Slot::Player(12),
            armor: 3
        }]
    );
    assert!(
        s.pieces(&items, &BTreeSet::from([Slot::Player(12)]), false)
            .is_empty()
    );
    let mut bound = worn.clone();
    bound.curse_mask = 1;
    assert!(
        s.candidates(&[bound], &BTreeSet::from([Slot::Player(39)]))
            .len()
            == 1
    );
    s.save = false;
    assert!(s.pieces(&items, &empty, false).is_empty());
    s.save = true;
    s.mode = ArmorMode::RawDefense;
    assert_eq!(s.pieces(&items, &empty, false)[0].source, Slot::Player(12));
    let mut c = Context {
        in_game: true,
        no_screen: true,
        player_stacks: vec![Stack::default(); 41],
        ..Default::default()
    };
    let p = Piece {
        source: Slot::Player(3),
        armor: 3,
    };
    assert!(matches!(
        s.action(p, &c, &[], &empty, false),
        Some(Action::UseItem { slot: 3 })
    ));
    assert!(matches!(
        s.action(
            Piece {
                source: Slot::Player(40),
                ..p
            },
            &c,
            &[],
            &empty,
            false
        ),
        Some(Action::UseItem { slot: 40 })
    ));
    assert!(matches!(
        s.action(p, &c, &[worn.clone()], &empty, false),
        Some(Action::Click {
            slot: Slot::Player(39),
            input: Input::QuickMove,
            ..
        })
    ));
    for stack in &mut c.player_stacks[..36] {
        stack.count = 1;
    }
    assert!(matches!(
        s.action(p, &c, &[worn.clone()], &empty, false),
        Some(Action::Click {
            slot: Slot::Player(39),
            input: Input::Throw,
            button: 1,
            ..
        })
    ));
    s.can_swap = true;
    assert!(matches!(
        s.action(p, &c, &[worn.clone()], &empty, false),
        Some(Action::UseItem { slot: 3 })
    ));
    assert!(matches!(
        s.action(p, &c, &[], &empty, true),
        Some(Action::Click {
            input: Input::QuickMove,
            ..
        })
    ));
    let mut glider = worn.clone();
    glider.classes.insert("Glider".into());
    assert!(s.action(p, &c, &[glider], &empty, false).is_none());
    assert!(
        s.action(p, &c, &[], &BTreeSet::from([p.source]), false)
            .is_none()
    );
    let mut state = SaveState::default();
    s.constraints.requirements = 32;
    s.can_swap = false;
    let replacements = vec![Piece {
        source: replacement.slot,
        armor: 3,
    }];
    assert_eq!(
        state.tick(&s, &c, false, 3, &items, &replacements, || panic!(
            "unexpected delay draw"
        )),
        None
    );
    assert_eq!(state.waiting, Some(SaveWait::Open));
    assert_eq!(state.resume(&c), (Some(ScreenAction::Open), false));
    assert!(state.opened);
    c.no_screen = false;
    c.inventory_screen = true;
    c.now = 100;
    assert_eq!(state.tick(&s, &c, false, 3, &[], &[], || 75), None);
    assert_eq!(state.waiting, Some(SaveWait::Close(175)));
    assert_eq!(state.resume(&c), (None, true));
    c.now = 175;
    assert_eq!(state.resume(&c), (Some(ScreenAction::Close), false));
    state.opened = true;
    state.tick(&s, &c, false, 3, &[], &[], || 0);
    c.inventory_screen = false;
    assert_eq!(state.resume(&c), (None, false));
    c.container_screen = true;
    state.previous_armor = 5;
    assert_eq!(
        state.tick(&s, &c, false, 3, &items, &replacements, || panic!(
            "unexpected delay"
        )),
        Some(ScreenAction::Close)
    );
    state.reset();
    c.container_screen = false;
    c.no_screen = true;
    s.can_swap = true;
    state.tick(&s, &c, false, 3, &items, &[p], || {
        panic!("unexpected delay")
    });
    assert_eq!(state.waiting, None);
    s.constraints.requirements = 0;
    state.tick(&s, &c, false, 3, &items, &replacements, || {
        panic!("unexpected delay")
    });
    assert_eq!(state.waiting, None);
    println!(
        "AutoArmor: shared policy, curses, locked armor, hotbar/offhand, removal/full inventory, gliders and SaveArmor waits passed"
    );
}
