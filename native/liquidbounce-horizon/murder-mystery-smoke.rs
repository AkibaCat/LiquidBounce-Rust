#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/murder_mystery.rs"]
mod murder_mystery;
mod font {
    include!("../liquidbounce-behaviors/src/murder_mystery_font.rs");
    pub fn map(text: &str) -> Vec<u8> {
        let mut colors = vec![13 * 4 + 3; 128 * 128];
        let mut x = 1;
        for c in text.chars() {
            if c == ' ' {
                x += 5;
                continue;
            }
            let (_, data) = LETTERS.iter().find(|(k, _)| *k == c.to_string()).unwrap();
            let width = data.len() / 7;
            for y in 0..7 {
                for dx in 0..width {
                    if data[y * width + dx] {
                        colors[(105 + y) * 128 + x + dx] = 0;
                    }
                }
            }
            x += width + 1;
        }
        colors
    }
}
use murder_mystery::*;
fn item(id: &str) -> Item {
    Item {
        id: format!("minecraft:{id}"),
        ..Default::default()
    }
}
fn player(uuid: &str, name: &str, x: f64) -> Player {
    Player {
        uuid: uuid.into(),
        name: name.into(),
        position: [x, 0., 0.],
        using_bow: false,
        has_arrow: false,
    }
}
#[test]
fn murder_mystery_focused_source_smoke() {
    let mut s = State::default();
    let a = player("a", "Alice", 5.);
    let b = player("b", "Bob", 2.);
    assert!(!s.disallows_arrow_dodge(true));
    assert!(!item("mace").is_sword());
    assert!(item("bone").is_sword());
    assert!(item("chorus_plant").is_sword());
    let mut shovel = item("wooden_shovel");
    shovel.mining = true;
    assert!(!shovel.is_sword());
    shovel.id = "minecraft:stone_shovel".into();
    assert!(shovel.is_sword());
    let mut boat = item("pale_oak_chest_boat");
    boat.boat = true;
    assert!(boat.is_sword());
    assert_eq!(
        s.item(&a, &item("bone")),
        Some(Message::Murderer("Alice".into()))
    );
    assert_eq!(s.item(&a, &item("bow")), Some(Message::Bow("Alice".into())));
    assert_eq!(s.role("a"), Role::Murderer);
    assert!(s.should_attack("a"));
    s.tick(&[item("bow"), item("bone")], &[]);
    assert_eq!(s.classic.own_role, Role::Detective);
    s.tick(&[item("bone"), item("arrow")], &[]);
    assert_eq!(s.classic.own_role, Role::Murderer);
    assert!(!s.should_attack("a"));
    assert!(s.should_attack("b"));
    assert_eq!(s.take_sounds(), (true, true));
    assert_eq!(s.take_sounds(), (false, false));
    s.reset();
    assert_eq!(s.classic.own_role, Role::Murderer);
    assert_eq!(s.role("a"), Role::Neutral);
    s.set_mode(1);
    let mut a = a.clone();
    a.has_arrow = true;
    assert_eq!(
        s.tick(&[item("air"), item("arrow")], &[a.clone(), b.clone()]),
        vec![Message::Bow("Alice".into())]
    );
    assert!(s.disallows_arrow_dodge(true));
    assert!(!s.disallows_arrow_dodge(false));
    assert_eq!(s.item(&a, &item("iron_sword")), None); // classification is genuine tags, not an id suffix.
    assert_eq!(
        s.item(&a, &item("bone")),
        Some(Message::FirstInfected("Alice".into()))
    );
    assert_eq!(s.item(&b, &item("stick")), None);
    assert_eq!(s.infection.murderers.len(), 2);
    assert_eq!(s.tag("Alice", None), Some((true, Role::Murderer)));
    assert_eq!(s.prefixes.len(), 1);
    assert_eq!(Role::Murderer.color(), Some(0xffcb0909));
    s.reset();
    assert!(s.disallows_arrow_dodge(true));
    s.set_mode(2);
    assert!(!s.disallows_arrow_dodge(true));
    assert!(!s.should_attack("a"));
    let map = font::map("NAME:ALICE");
    assert_eq!(read_contract(&map), "NAME:ALICE");
    assert_eq!(
        s.contract(
            true,
            Some(7),
            Some(&map),
            &[("a".into(), "Alice".into()), ("b".into(), "Bob".into())]
        ),
        Some(Message::Target("Alice".into()))
    );
    assert!(s.should_attack("a"));
    assert!(!s.should_attack("b"));
    let volume = (1. / (0.98272992 + 0.04342088 * 2.)) as f32;
    assert_eq!(
        s.sound(volume, [0.; 3], &[a.clone(), b.clone()]),
        Some(Message::Assassin("Bob".into()))
    );
    assert!(s.should_attack("b"));
    assert_eq!(s.sound(volume, [0.; 3], &[a, b]), None);
    s.reset();
    assert_eq!(s.last_map, Some(7));
    assert_eq!(s.contract(true, Some(7), Some(&map), &[]), None);
    s.contract(false, None, None, &[]);
    assert_eq!(s.last_map, None);
    assert_eq!(
        s.contract(true, Some(7), Some(&map), &[]),
        Some(Message::MissingTarget("alice".into()))
    );
}
