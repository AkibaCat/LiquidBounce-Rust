// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(dead_code)]
mod vomit {
    #[derive(Clone, Debug, Default)]
    pub struct Constraints;
}
#[path = "../liquidbounce-behaviors/src/offhand.rs"]
mod offhand;
use offhand::*;

fn safe() -> Context {
    Context {
        now: 1_000,
        health: 20.,
        max_health: 20.,
        modern: true,
        ..Default::default()
    }
}
fn stack(item: &str) -> Item {
    Item {
        item: item.into(),
        components: format!("[{item},components]"),
        count: 1,
        ..Default::default()
    }
}
fn inventory() -> Vec<Item> {
    vec![Item::default(); 41]
}
fn quiet() -> State {
    let mut s = State::default();
    s.settings.missing_armor = false;
    s.settings.entities = false;
    s.settings.fall = false;
    s
}

#[test]
fn defaults_priority_and_full_slot_selection() {
    let mut s = quiet();
    let mut c = safe();
    s.enable(&c);
    assert_eq!(s.static_mode, Mode::Crystal);
    assert_eq!(s.settings.switch_mode, SwitchMode::Switch);
    assert_eq!(s.settings.cycle, "key.keyboard.h");
    assert_eq!(s.settings.back_delay, 40);
    c.sword = true;
    c.kill_aura = true;
    assert_eq!(s.wanted(&c), Mode::Gapple);
    s.settings.strength = true;
    assert_eq!(s.wanted(&c), Mode::Strength);
    c.health = 14.;
    assert_eq!(s.wanted(&c), Mode::Totem);
    c.creative = true;
    assert_eq!(s.wanted(&c), Mode::Strength);
    let mut items = inventory();
    items[0] = stack("minecraft:golden_apple");
    items[40] = stack("minecraft:golden_apple");
    items[12] = stack("minecraft:enchanted_golden_apple");
    items[3] = stack("minecraft:enchanted_golden_apple");
    assert_eq!(s.slot(Mode::Gapple, &items), Some(12));
    items[40] = stack("minecraft:enchanted_golden_apple");
    assert_eq!(s.slot(Mode::Gapple, &items), Some(40));
    items[0].death_protection = true;
    items[12].death_protection = true;
    assert_eq!(s.slot(Mode::Totem, &items), Some(0));
    items[0].death_protection = false;
    items[12].death_protection = false;
    items[14] = stack("minecraft:end_crystal");
    assert_eq!(s.slot(Mode::Totem, &items), Some(14));
    items[14] = stack("minecraft:potion");
    items[14].strength = true;
    assert_eq!(s.slot(Mode::Strength, &items), Some(14));
    items[14].item = "minecraft:splash_potion".into();
    assert_eq!(s.slot(Mode::Strength, &items), None);
}

#[test]
fn health_safety_damage_maxima_and_urgency() {
    let mut s = quiet();
    let mut c = safe();
    c.health = 12.;
    assert!(s.settings.below_threshold(&c));
    c.hole = true;
    assert!(!s.settings.below_threshold(&c));
    s.settings.entities = true;
    s.settings.blocks = true;
    s.settings.sphere_initialized = true;
    s.settings.fall = true;
    c.health = 20.;
    c.hole = false;
    c.fall_distance = 8.;
    c.fall_damage = 3.;
    c.entity_damage = vec![4., 8.];
    c.block_damage = vec![6., 10.];
    assert_eq!(s.settings.predicted_damage(&c, 20.), 13.);
    assert_eq!(s.settings.predicted_damage(&c, 7.), 8.);
    assert!(!s.settings.below_threshold(&c));
    s.settings.subtract = true;
    assert!(s.settings.below_threshold(&c));
    c.no_fall = true;
    assert_eq!(s.settings.predicted_damage(&c, 20.), 10.);
    c.no_fall = false;
    c.fall_flying_pose = true;
    s.settings.ignore_elytra = true;
    assert_eq!(s.settings.predicted_damage(&c, 20.), 10.);
    s.settings.urgency = true;
    s.settings.min_factor = 0.2;
    c.enemy_distance_squared = vec![0.];
    assert!((s.settings.urgency_factor(&c) - 0.2).abs() < 1e-6);
    c.health = 0.;
    c.max_health = 0.;
    assert!(s.settings.urgency_factor(&c).is_nan());
    c.entity_damage.clear();
    c.block_damage = vec![10.];
    assert_eq!(s.settings.predicted_damage(&c, 0.), 0.);
}

#[test]
fn strict_wall_clock_and_switch_back_delay() {
    let mut s = quiet();
    let mut c = safe();
    let mut items = inventory();
    s.static_mode = Mode::Crystal;
    items[10] = stack("minecraft:end_crystal");
    s.settings.delay = [10, 10];
    s.changed_at = 100;
    c.now = 110;
    assert!(s.plan(&c, &items).is_none());
    c.now = 111;
    let p = s.plan(&c, &items).unwrap();
    s.complete(&p, 111, 0.75);
    assert_eq!(s.changed_at, 111);
    assert_eq!(s.roll, 0.75);
    s.last_mode = Some(Mode::Totem);
    c.now = 1_000;
    assert!(s.plan(&c, &items).is_none());
    c.now = 1_040;
    assert!(s.plan(&c, &items).is_none());
    c.now = 1_041;
    assert_eq!(s.plan(&c, &items).unwrap().mode, Mode::Crystal);
    s.last_mode = Some(Mode::Crystal);
    s.changed_at = 200;
    c.now = 1_000;
    items[40] = stack("minecraft:end_crystal");
    assert!(s.plan(&c, &items).is_none());
    assert_eq!(s.changed_at, 200);
    assert_eq!(s.last_mode, Some(Mode::Crystal));
}

#[test]
fn real_switch_pickup_smart_direct_and_back_quirk() {
    let mut s = quiet();
    let mut c = safe();
    let mut items = inventory();
    c.health = 1.;
    c.selected = 1;
    items[3] = stack("minecraft:totem_of_undying");
    items[3].death_protection = true;
    let p = s.plan(&c, &items).unwrap();
    assert_eq!(p.actions, vec![Action::Swap(3)]);
    s.settings.switch_mode = SwitchMode::PickUp;
    assert_eq!(
        s.plan(&c, &items).unwrap().actions,
        vec![Action::Pickup(3), Action::Pickup(40)]
    );
    items[40] = stack("minecraft:shield");
    assert_eq!(
        s.plan(&c, &items).unwrap().actions,
        vec![Action::Pickup(3), Action::Pickup(40), Action::Pickup(3)]
    );
    s.settings.switch_mode = SwitchMode::Smart;
    let smart = s.plan(&c, &items).unwrap();
    assert!(smart.smart);
    assert_eq!(
        smart.actions,
        vec![Action::Held(3), Action::SwapOffhand, Action::Held(1)]
    );
    c.selected = 3;
    assert_eq!(
        s.plan(&c, &items).unwrap().actions,
        vec![Action::SwapOffhand]
    );
    s.settings.switch_mode = SwitchMode::Automatic;
    c.modern = false;
    assert_eq!(s.plan(&c, &items).unwrap().actions.len(), 3);
    c.modern = true;
    s.settings.direct = true;
    let direct = s.plan(&c, &items).unwrap();
    assert!(direct.direct);
    s.complete(&direct, c.now, 0.2);
    assert_eq!(s.slot(Mode::Back, &items), Some(3));
    items[3] = stack("minecraft:shield");
    assert_eq!(s.slot(Mode::Back, &items), None);
}

#[test]
fn continuation_finishes_changed_source_and_cancels_unchanged_source() {
    let mut s = quiet();
    let mut c = safe();
    let mut items = inventory();
    c.health = 1.;
    items[9] = stack("minecraft:totem_of_undying");
    items[9].death_protection = true;
    let p = s.plan(&c, &items).unwrap();
    let same = Some((p.stack.components.as_str(), p.stack.count));
    assert!(s.may_continue(&c, &p, same));
    c.health = 20.;
    assert!(!s.may_continue(&c, &p, same));
    assert!(s.may_continue(&c, &p, Some(("empty", 0))));
    assert!(s.may_continue(&c, &p, Some((&p.stack.components, 2))));
    assert!(s.may_continue(&c, &p, None));
}

#[test]
fn key_priority_history_lifecycle_and_dependency_conditions() {
    let mut s = quiet();
    let mut c = safe();
    s.enable(&c);
    s.settings.gapple_bind = "key.keyboard.x".into();
    s.settings.crystal_bind = s.settings.gapple_bind.clone();
    s.key("key.keyboard.x", &c);
    assert_eq!(s.static_mode, Mode::Gapple);
    s.active = Mode::Gapple;
    s.key("key.keyboard.x", &c);
    assert_eq!(s.static_mode, Mode::Crystal);
    s.settings.strength = true;
    s.settings.strength_bind = "key.keyboard.z".into();
    s.key("key.keyboard.z", &c);
    assert_eq!(s.static_mode, Mode::Crystal);
    s.key("key.keyboard.h", &c);
    assert_eq!(s.static_mode, Mode::Totem);
    s.last_mode = Some(Mode::Totem);
    s.last = Some(("saved".into(), 2));
    s.changed_at = 123;
    s.enable(&c);
    assert_eq!(s.last_mode, Some(Mode::Totem));
    assert_eq!(s.changed_at, 123);
    assert!(s.last.is_some());
    assert!(!s.operating(false));
    s.settings.mace = true;
    c.mace_diving = true;
    assert_eq!(s.wanted(&c), Mode::Totem);
    c.mace_diving = false;
    s.settings.strength = false;
    s.settings.block = true;
    c.eagle_enabled = true;
    assert_eq!(s.wanted(&c), Mode::Block);
}
