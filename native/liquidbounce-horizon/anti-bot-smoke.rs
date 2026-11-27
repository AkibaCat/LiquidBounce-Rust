// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/anti_bot.rs"]
mod anti_bot;
use anti_bot::*;
fn player(id: i32) -> Player {
    Player {
        id,
        token: id as u64,
        uuid: format!("uuid{id}"),
        name: format!("Player{id}"),
        local: false,
        ticks: 20,
        position: [0.; 3],
        previous: [0.; 3],
        on_ground: true,
        pitch: 0.,
        health: 20.,
        scale: 1.,
        armor: [Armor::default(); 4],
        properties_empty: true,
        online: true,
        listed: true,
        game_mode: Some(0),
        ping: 1,
    }
}
fn quiet() -> State {
    let mut s = State::default();
    s.settings.conditions = 0;
    s.settings.invalid_ground = false;
    s.settings.name = false;
    s
}
fn entry(id: i32, ping: i32) -> Entry {
    Entry {
        uuid: format!("uuid{id}"),
        name: Some(format!("Player{id}")),
        properties_empty: true,
        mode: Some(0),
        latency: ping,
    }
}
fn full(p: &mut Player) {
    p.armor = std::array::from_fn(|i| Armor {
        identity: (i + 1) as u64,
        predicates: 1 << 5,
        player_armor: true,
        enchanted: true,
    });
}
#[test]
fn anti_bot_smoke_custom_conditions_and_local_guards() {
    let mut s = quiet();
    let mut p = player(2);
    assert!(!s.is_bot(true, &p, &[], 20.));
    assert!(!s.is_bot(false, &p, &[], 20.));
    for i in 0..11 {
        p = player(2);
        s.settings.conditions = 1 << i;
        let mut online = vec![];
        match i {
            0 => online.push(Profile {
                uuid: "other".into(),
                name: p.name.clone(),
            }),
            1 => p.game_mode = None,
            2 => p.ping = 0,
            3 => p.pitch = -90.01,
            4 => p.id = -1,
            5 => {}
            6 => p.health = 20.01,
            7 => {}
            8 => {}
            9 => {}
            _ => p.scale = f64::from_bits(1f64.to_bits() + 1),
        }
        assert!(s.is_bot(true, &p, &online, 20.), "condition {i}");
        p.local = true;
        assert!(!s.is_bot(true, &p, &online, 20.));
    }
    s.settings.conditions = (1 << 5) | (1 << 7) | (1 << 8) | (1 << 9);
    p = player(2);
    s.attack(2);
    s.swing(2);
    s.animate(2, 5);
    s.attributes(2);
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.remove_entities(&[2]);
    assert!(s.is_bot(true, &p, &[], 20.));
}
#[test]
fn anti_bot_smoke_ground_metadata_and_removal_lifecycle() {
    let mut s = quiet();
    let p = player(2);
    s.settings.invalid_ground = true;
    s.settings.ground_vl = 3;
    s.ground(2, true, 1., 2.);
    s.ground(2, true, 1., 2.);
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.ground(2, true, 1., 2.);
    assert!(s.is_bot(true, &p, &[], 20.));
    s.ground(2, false, 1., 2.);
    assert_eq!(s.custom.flying[&2], 1);
    s.ground(2, false, 1., 2.);
    assert!(!s.custom.flying.contains_key(&2));
    s.metadata(2, 4);
    s.metadata(2, 1);
    s.settings.metadata = true;
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.metadata(3, 3);
    assert!(s.is_bot(true, &player(3), &[], 20.));
    s.remove_entities(&[2, 3]);
    s.metadata(2, 3);
    assert!(s.is_bot(true, &p, &[], 20.));
    s.reset();
    assert!(!s.is_bot(true, &p, &[], 20.));
}
#[test]
fn anti_bot_smoke_radius_movement_and_armor_history() {
    let mut s = quiet();
    let mut p = player(2);
    p.position[0] = 21.;
    s.custom_tick([0.; 3], &[p.clone()]);
    s.settings.radius = true;
    s.settings.moved = true;
    assert!(!s.is_bot(true, &p, &[], 20.));
    assert!(s.is_bot(true, &player(3), &[], 20.));
    p = player(2);
    p.ticks = 160;
    s.custom_tick([0.; 3], &[p.clone()]);
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.settings.radius = false;
    s.settings.moved = false;
    s.settings.armor = true;
    p.armor[0] = Armor {
        identity: 9,
        predicates: 1 << 5,
        player_armor: true,
        enchanted: false,
    };
    s.custom_tick([0.; 3], &[p.clone()]);
    assert!(s.is_bot(true, &p, &[], 20.));
    s.settings.armor = false;
    p.armor[0] = Armor::default();
    s.custom_tick([0.; 3], &[p.clone()]);
    assert!(!s.custom.armor.contains(&2));
    s.settings.armor = true;
    s.settings.armor_choices = [0; 4];
    p.armor[0].predicates = 0;
    s.custom_tick([0.; 3], &[p.clone()]);
    assert!(!s.is_bot(true, &p, &[], 20.));
}
#[test]
fn anti_bot_smoke_utf16_length_and_validator_intersection() {
    let mut s = quiet();
    let mut p = player(2);
    s.settings.name = true;
    s.settings.validators = 0;
    s.settings.length = [2, 2];
    p.name = "𠀀".into();
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.settings.length = [1, 32];
    s.settings.validators = 1;
    p.name = "Vanilla_1".into();
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.settings.validators = 3;
    assert!(s.is_bot(true, &p, &[], 20.));
    s.settings.validators = 2;
    p.name = "Я".into();
    assert!(!s.is_bot(true, &p, &[], 20.));
    s.settings.validators = 4;
    p.name = "中".into();
    assert!(!s.is_bot(true, &p, &[], 20.));
    p.name = "𠀀".into();
    assert!(s.is_bot(true, &p, &[], 20.));
}
#[test]
fn anti_bot_smoke_online_and_listed_collections_are_distinct() {
    let mut s = quiet();
    let mut p = player(2);
    p.listed = false;
    s.settings.literal_npc = true;
    assert!(!s.is_bot(true, &p, &[], 20.));
    p.online = false;
    assert!(s.is_bot(true, &p, &[], 20.));
    s.settings.literal_npc = false;
    s.settings.not_in_tab = true;
    s.settings.grace = 21;
    assert!(!s.is_bot(true, &p, &[], 20.));
    p.ticks = 21;
    assert!(s.is_bot(true, &p, &[], 20.));
    p.local = true;
    assert!(!s.is_bot(true, &p, &[], 20.));
}
#[test]
fn anti_bot_smoke_profile_exact_count_and_matrix_packet_filters() {
    let mut s = quiet();
    s.set_mode(1);
    let p = player(2);
    let same = Profile {
        uuid: p.uuid.clone(),
        name: p.name.clone(),
    };
    let other = Profile {
        uuid: "other".into(),
        name: p.name.clone(),
    };
    assert!(unique(&same, &[same.clone()]));
    assert!(!unique(&same, &[same.clone(), same.clone()]));
    assert!(duplicate(&same, &[other.clone()]));
    assert!(!duplicate(&same, &[other.clone(), other.clone()]));
    s.info(1, &[entry(2, 2)], &[same], 0);
    assert!(s.matrix.suspects.is_empty());
    s.info(1, &[entry(2, 1)], &[], 0);
    assert!(s.matrix.suspects.is_empty());
    let mut e = entry(2, 2);
    e.properties_empty = false;
    s.info(1, &[e], &[], 0);
    assert!(s.matrix.suspects.is_empty());
    s.info(1, &[entry(2, 2)], &[other], 0);
    assert!(s.is_bot(true, &p, &[], 20.));
    s.remove_profiles(&[p.uuid]);
    assert!(s.matrix.bots.is_empty());
}
#[test]
fn anti_bot_smoke_matrix_fifo_identity_and_removed_entity_retention() {
    let mut s = quiet();
    s.set_mode(1);
    let first = player(2);
    let mut second = player(3);
    full(&mut second);
    s.info(1, &[entry(2, 2), entry(3, 2)], &[], 0);
    let callback = s.matrix_start(&[first.clone(), second.clone()]).unwrap();
    assert!(s.matrix.bots.is_empty());
    let mut changed = first.clone();
    changed.armor[0].identity = 42;
    assert!(!s.matrix_resume(callback, &[changed.clone(), second.clone()]));
    assert!(s.matrix.bots.contains(&first.uuid));
    assert!(s.matrix.bots.contains(&second.uuid));
    s.reset();
    s.info(1, &[entry(2, 2)], &[], 0);
    let callback = s.matrix_start(&[first.clone()]).unwrap();
    let mut reused = changed;
    reused.token = 999;
    assert!(!s.matrix_resume(callback, &[reused.clone()]));
    assert!(s.matrix.bots.is_empty());
    assert!(!s.matrix.suspects.contains(&first.uuid));
    s.observe(&[reused]);
    assert_eq!(s.entities.len(), 1);
    s.reset();
    s.info(1, &[entry(3, 2)], &[], 0);
    assert_eq!(s.matrix_start(&[second.clone()]), None);
    assert!(s.is_bot(true, &second, &[], 20.));
}
#[test]
fn anti_bot_smoke_intave_first_action_and_fifteen_ms_boundary() {
    let mut s = quiet();
    s.set_mode(2);
    s.info(1 | 16, &[entry(2, 8)], &[], 100);
    assert!(s.intave.suspects.contains_key("uuid2"));
    s.info(4 | 16, &[entry(2, 0)], &[], 105);
    assert!(s.intave.suspects.contains_key("uuid2"));
    s.info(16, &[entry(2, 0)], &[], 115);
    assert!(s.intave.bots.contains("uuid2"));
    s.info(1, &[entry(3, 8), entry(4, 8)], &[], 100);
    s.info(16, &[entry(3, 0), entry(4, 0)], &[], 116);
    assert!(!s.intave.bots.contains("uuid3"));
    assert!(s.intave.suspects.is_empty());
    s.info(1, &[entry(5, 8), entry(6, 8)], &[], 200);
    s.info(16, &[entry(5, 0), entry(6, 0)], &[], 200);
    assert!(s.intave.bots.contains("uuid5") && s.intave.bots.contains("uuid6"));
    s.remove_profiles(&["uuid2".into()]);
    assert!(!s.intave.bots.contains("uuid2"));
}
#[test]
fn anti_bot_smoke_horizon_and_mode_history_reset() {
    let mut s = quiet();
    s.set_mode(3);
    let mut e = entry(2, 0);
    e.mode = None;
    s.info(4, &[e.clone()], &[], 0);
    assert!(s.horizon.is_empty());
    s.info(1, &[e], &[], 0);
    assert!(s.horizon.contains("uuid2"));
    s.set_mode(0);
    s.attack(7);
    s.set_mode(1);
    s.info(1, &[entry(3, 2)], &[], 0);
    assert!(s.matrix_start(&[player(3)]).is_some());
    s.set_mode(3);
    assert!(s.matrix.continuation.is_none());
    assert!(s.horizon.contains("uuid2"));
    assert!(s.custom.hit.contains(&7));
    s.remove_profiles(&["uuid2".into()]);
    assert!(s.horizon.is_empty());
    s.reset();
    assert!(s.custom.hit.is_empty() && s.matrix.suspects.is_empty() && s.intave.bots.is_empty());
}
