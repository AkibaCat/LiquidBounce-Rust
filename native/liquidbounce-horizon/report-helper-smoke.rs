// SPDX-License-Identifier: GPL-3.0-or-later
#![allow(dead_code)]
#[path = "../liquidbounce-behaviors/src/report_helper.rs"]
mod report_helper;
use report_helper::*;
fn player(name: &str, local: bool, friend: bool) -> Player {
    Player {
        name: name.into(),
        local,
        friend,
    }
}
#[test]
fn report_helper_source_sequence_and_both_confirmation_menus() {
    let mut state = State::default();
    assert!(!state.settings.report && !state.settings.confirm);
    assert_eq!(state.settings.delay, [1, 3]);
    state.set_report(true);
    state.set_confirm(true);
    let players = vec![
        player("Self", true, false),
        player("Friend", false, true),
        player("Second", false, false),
        player("First", false, false),
    ];
    let mut draws = vec![];
    let r = state.message(
        true,
        "Self Friend First Second",
        "Self",
        &players,
        |a, b| {
            draws.push([a, b]);
            if a == 0 { 0 } else { 2 }
        },
    );
    let Some(Report::Wait(id)) = r else {
        panic!("expected wait");
    };
    assert_eq!(draws, vec![[0, 99], [1, 3]]);
    assert_eq!(state.report_callback(id, true, true), (true, None));
    state.settings.pattern = "check %1$s %<S %%".into();
    let (keep, name) = state.report_callback(id, true, true);
    assert!(!keep);
    assert_eq!(name.as_deref(), Some("Second"));
    assert_eq!(
        format_command(&state.settings.pattern, &name.unwrap()).unwrap(),
        "check Second SECOND %"
    );
    // Chance is drawn even for an already reported case-insensitive name.
    let mut calls = 0;
    assert!(
        state
            .message(
                true,
                "Self sEcOnD",
                "Self",
                &[player("sEcOnD", false, false)],
                |_, _| {
                    calls += 1;
                    0
                }
            )
            .is_none()
    );
    assert_eq!(calls, 1);
    assert!(
        state
            .message(true, "self First", "Self", &players, |_, _| panic!(
                "case-sensitive self gate"
            ))
            .is_none()
    );
    state.settings.chance = 1;
    assert!(
        state
            .message(true, "Self First", "Self", &players, |_, _| 1)
            .is_none()
    );
    state.settings.chance = 100;
    state.settings.delay = [0, 0];
    assert_eq!(
        state.message(true, "Self First", "Self", &players, |a, _| a),
        Some(Report::Send("First".into()))
    );
    state.set_report(false);
    state.set_report(true);
    assert!(matches!(
        state.message(true, "Self Second", "Self", &players, |a, _| a),
        Some(Report::Send(_))
    ));
    state.session(1);
    state.session(2);
    assert!(matches!(
        state.message(true, "Self Second", "Self", &players, |a, _| a),
        Some(Report::Send(_))
    ));
    state.settings.delay = [3, 3];
    state.session(3);
    let Some(Report::Wait(pending)) = state.message(true, "Self First", "Self", &players, |a, _| a)
    else {
        panic!();
    };
    state.disabled();
    assert_eq!(state.report_callback(pending, true, true), (false, None));
    let first = state.screen(true, 10).unwrap();
    assert_eq!(
        state.confirm_callback(first, true, true, true),
        Confirm::Inspect {
            screen: 10,
            mode: 0
        }
    );
    let late = state.screen(true, 11).unwrap();
    for _ in 0..4 {
        assert_eq!(
            state.confirm_callback(late, true, true, false),
            Confirm::Keep
        );
    }
    assert_eq!(
        state.confirm_callback(late, true, true, true),
        Confirm::Stop
    );
    let canceled = state.screen(true, 12).unwrap();
    for _ in 0..4 {
        assert_eq!(
            state.confirm_callback(canceled, true, true, false),
            Confirm::Keep
        );
    }
    assert_eq!(
        state.confirm_callback(canceled, true, true, false),
        Confirm::Stop
    );
    let changed = state.screen(true, 13).unwrap();
    state.set_mode(1);
    assert_eq!(
        state.confirm_callback(changed, true, true, true),
        Confirm::Stop
    );
    let mut slots = (0..27)
        .map(|index| Slot {
            index,
            item: "minecraft:air".into(),
            empty: true,
        })
        .collect::<Vec<_>>();
    for (index, item) in [
        (11, "minecraft:green_terracotta"),
        (13, "minecraft:player_head"),
        (15, "minecraft:red_terracotta"),
    ] {
        slots[index].item = item.into();
        slots[index].empty = false;
    }
    assert_eq!(confirm_slot(0, &slots), Some(11));
    slots[0].empty = false;
    assert_eq!(confirm_slot(0, &slots), None);
    slots[0].empty = true;
    slots[11].item = "minecraft:lime_terracotta".into();
    assert_eq!(confirm_slot(0, &slots), None);
    let mut hey = slots[..9].to_vec();
    hey[2] = Slot {
        index: 41,
        item: "minecraft:diamond_sword".into(),
        empty: false,
    };
    hey[5] = Slot {
        index: 44,
        item: "minecraft:diamond_sword".into(),
        empty: false,
    };
    assert_eq!(confirm_slot(1, &hey), Some(41));
    assert_eq!(confirm_slot(1, &hey[..8]), None);
    assert_eq!(
        format_command("%1$-8.3S/%<b/%<h", "Player").unwrap(),
        "PLA     /true/8ea38701"
    );
    assert_eq!(format_command("%s:%1$s", "Alex").unwrap(), "Alex:Alex");
    for pattern in [
        "%s %s", "%2$s", "%<s", "%d", "%#s", "%0s", "%-s", "%.s", "%",
    ] {
        assert!(format_command(pattern, "Alex").is_err(), "{pattern}");
    }
    native_identity::check();
    for (pattern, expected) in /*FORMAT_GOLDENS*/ [] {
        assert_eq!(
            format_command(pattern, "Player").ok().as_deref(),
            expected,
            "{pattern}"
        );
    }
}
/*NATIVE_IDENTITY*/
