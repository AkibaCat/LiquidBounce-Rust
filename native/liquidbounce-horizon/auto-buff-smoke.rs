#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/auto_buff.rs"]
mod auto_buff;
#[path = "../liquidbounce-behaviors/src/random.rs"]
mod random;
#[path = "../liquidbounce-behaviors/src/rotations.rs"]
mod rotations;
#[path = "../liquidbounce-behaviors/src/vomit.rs"]
mod vomit;
use auto_buff::*;
fn item(slot: u8, id: &str) -> Item {
    Item {
        slot,
        id: format!("minecraft:{id}"),
        components: id.into(),
        count: 1,
        maximum: 64,
        ..Default::default()
    }
}
fn main() {
    let mut s = AutoBuff::default();
    let mut c = Context {
        now: 10000,
        health: 5.,
        max_health: 20.,
        tick_count: 40,
        selected: 0,
        server_selected: 0,
        landing_y: Some(60),
        y: 61.,
        items: vec![item(3, "mushroom_stew"), item(40, "golden_apple")],
        ..Default::default()
    };
    let mut random = |r: [i32; 2]| r[0];
    let e = s.start(&c, &mut random);
    assert!(e.contains(&Effect::Select {
        slot: 3,
        ticks: 300
    }));
    assert!(!e.iter().any(|e| matches!(e, Effect::Use { .. })));
    assert!(s.needs_wait());
    let e = s.drive(&c, true, &mut random);
    assert!(e.contains(&Effect::Use { slot: 3, reset: 2 }));
    c.items[0] = item(3, "bowl");
    let e = s.drive(&c, true, &mut random);
    assert!(e.contains(&Effect::DropBowl));
    assert!(e.contains(&Effect::ReleaseSelection { slot: 3, ticks: 1 }));
    assert!(s.run.is_none());
    s.first_tick();
    assert_eq!(s.ticks_until_next, 0);
    s.settings.enabled = [false, false, false, false, true];
    s.start(&c, &mut random);
    assert_eq!(s.force_use, Some(Feature::Gapple));
    let mut pressed = false;
    s.force_use_key(true, &mut pressed);
    assert!(pressed);
    c.health = 20.;
    s.drive(&c, true, &mut random);
    assert_eq!(s.force_use, None);
    c.health = 5.;
    c.items = vec![Item {
        effects: vec!["instant_health".into()],
        ..item(0, "splash_potion")
    }];
    s.settings.enabled = [false, false, true, false, false];
    let e = s.start(&c, &mut random);
    assert!(e.contains(&Effect::PreparePotion));
    let r = rotations::Rotation::new(40., 88.);
    let e = s.potion_prepared(&c, r, &mut random);
    assert!(e.contains(&Effect::Rotate(r)));
    assert!(s.drive(&c, false, &mut random).is_empty());
    c.current_pitch = 88.;
    let e = s.drive(&c, true, &mut random);
    assert!(e.contains(&Effect::UsePotion {
        slot: 0,
        rotation: r,
        pre_packet: false,
        post_packet: false
    }));
    assert_eq!(s.ticks_until_next, 1);
    s.first_tick();
    c.enemy_near = true;
    assert!(s.start(&c, &mut random).is_empty());
    c.enemy_near = false;
    c.effects.insert("instant_health".into());
    assert!(s.start(&c, &mut random).is_empty());
    c.effects.clear();
    c.items = vec![Item {
        effects: vec!["strength".into()],
        ..item(12, "potion")
    }];
    s.settings.enabled = [false, false, false, true, false];
    assert_eq!(s.refill_slots(&c), vec![12]);
    assert!(AutoBuff::hotbar_can_receive(&c, 12));
    s.clear();
    s.settings.enabled = [true, false, false, false, false];
    s.settings.auto_swap = false;
    c.items = vec![item(8, "mushroom_stew")];
    assert_eq!(s.start(&c, &mut random), vec![Effect::Pause(0)]);
    assert!(s.run.is_none());
    s.settings.bowl_mode = BowlMode::Stack;
    s.bowl_slot = Some(8);
    assert_eq!(s.stack_bowl_valid(&c), Some(8));
    s.settings.bowl[1].assume_empty = false;
    assert_eq!(s.stack_bowl_valid(&c), None);
    println!("AutoBuff focused smoke passed");
}
