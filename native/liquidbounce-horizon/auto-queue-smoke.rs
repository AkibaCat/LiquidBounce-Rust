// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/auto_queue.rs"]
mod queue;
use queue::*;
fn random(a: i32, _b: i32) -> i32 {
    a
}
fn slot(index: i32, item: &str, name: &str) -> Slot {
    Slot {
        slot: index,
        item: item.into(),
        name: name.into(),
    }
}
fn id(e: &[Effect]) -> u64 {
    e.iter()
        .find_map(|e| {
            if let Effect::Register(id) = e {
                Some(*id)
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn auto_queue_focused_smoke() {
    let mut s = State::default();
    let mut c = Context {
        slots: vec![slot(0, "minecraft:paper", "Paper")],
        ..Default::default()
    };
    assert_eq!(s.settings.pause, 15);
    c.pause = 2;
    assert!(s.tick(true, &c, random).is_empty());
    c.pause = 0;
    let e = s.tick(true, &c, random);
    assert_eq!(e[0], Effect::Command("play solo_normal".into()));
    let task = id(&e);
    assert!(s.tick(true, &c, random).is_empty());
    for _ in 0..19 {
        assert!(s.callback(task, true, &c, random).0);
    }
    assert!(!s.callback(task, true, &c, random).0);
    s.settings.game = 1;
    assert_eq!(
        s.tick(true, &c, random)[0],
        Effect::Command("play solo_insane".into())
    );
    s.disabled();
    s.set_preset(1);
    c.header = Some("GommeHD.net Lobby".into());
    c.npc = Some(77);
    c.now = 100;
    let e = s.tick(true, &c, random);
    assert!(e.contains(&Effect::Interact(77)));
    let task = id(&e);
    c.now = 5099;
    assert!(s.callback(task, true, &c, random).0);
    c.now = 5100;
    assert!(!s.callback(task, true, &c, random).0);
    c.header = Some("GommeHD.net Duels".into());
    c.slots = vec![slot(4, "minecraft:player_head", "Head")];
    let e = s.tick(true, &c, random);
    assert!(e.contains(&Effect::Control("KillAura", false)));
    assert!(e.contains(&Effect::Select {
        slot: 4,
        owner: true
    }));
    let task = id(&e);
    for _ in 0..4 {
        assert!(s.callback(task, true, &c, random).0);
    }
    let (keep, e) = s.callback(task, true, &c, random);
    assert!(!keep);
    assert!(e.contains(&Effect::RawUse));
    assert!(s.event(true, 0, 0, "Das Match kann beginnen", 0).is_empty());
    assert!(!s.in_match);
    assert!(
        s.event(true, 0, 2, "Das Match kann beginnen", 0)
            .contains(&Effect::Notice("Match started", false))
    );
    assert!(s.in_match);
    let e = s.event(true, 0, 2, "und das Match gewonnen", 20);
    let task = id(&e);
    c.now = 2019;
    assert!(s.callback(task, true, &c, random).0);
    s.settings.win = "late value".into();
    c.now = 2020;
    assert!(
        s.callback(task, true, &c, random)
            .1
            .contains(&Effect::Chat("late value".into()))
    );
    s.disabled();
    s.set_preset(2);
    s.settings.wait_world = true;
    s.settings.start_delay = [7, 7];
    s.settings.message_delay = [3, 3];
    s.settings.sends = vec!["/one".into(), "two".into()];
    c.now = 3000;
    c.header = None;
    s.event(true, 1, 0, "Win", c.now);
    c.pause = 1;
    assert!(
        !s.tick(true, &c, random)
            .iter()
            .any(|e| matches!(e, Effect::ChatOrCommand(_)))
    );
    c.pause = 0;
    let e = s.tick(true, &c, random);
    assert!(e.contains(&Effect::Control("KillAura", false)));
    assert_eq!(s.next_wake(), Some(3007));
    c.now = 3006;
    assert!(s.wake(true, &c, random).is_empty());
    c.now = 3007;
    assert_eq!(
        s.wake(true, &c, random),
        vec![Effect::ChatOrCommand("/one".into())]
    );
    assert_eq!(s.next_wake(), Some(3010));
    s.settings.sends = vec!["replacement".into()];
    c.now = 3010;
    let e = s.wake(true, &c, random);
    assert_eq!(e[0], Effect::ChatOrCommand("two".into()));
    let world = id(&e);
    assert!(s.callback(world, true, &c, random).0);
    let e = s.world_change();
    let cooldown = id(&e);
    assert!(!s.callback(world, true, &c, random).0);
    for _ in 0..19 {
        assert!(s.callback(cooldown, true, &c, random).0);
    }
    assert!(!s.callback(cooldown, true, &c, random).0);
    assert!(
        s.tick(true, &c, random)
            .contains(&Effect::Control("KillAura", true))
    );
    s.settings.action = 1;
    s.settings.trigger = 3;
    s.settings.trigger_pick.mode = 1;
    s.settings.action_pick.mode = 1;
    c.slots = vec![
        slot(-1, "minecraft:paper", "Paper"),
        slot(0, "minecraft:paper", "Paper"),
    ];
    let e = s.tick(true, &c, random);
    assert!(e.contains(&Effect::Select {
        slot: -1,
        owner: false
    }));
    let task = id(&e);
    c.slots[0].item = "minecraft:stone".into();
    let (keep, e) = s.callback(task, true, &c, random);
    assert!(!keep && e.is_empty());
    c.slots[0].item = "minecraft:paper".into();
    let e = s.tick(true, &c, random);
    let task = id(&e);
    let (keep, e) = s.callback(task, true, &c, random);
    assert!(!keep);
    assert!(e.contains(&Effect::Use(-1)));
    let world = id(&e);
    c.slots[0].item = "minecraft:stone".into();
    assert!(!s.callback(world, true, &c, random).0);
    assert!(s.world_change().is_empty());
    s.settings.trigger = 2;
    s.settings.messages = vec!["game".into()];
    s.event(true, 0, 0, "game", 0);
    assert!(!s.latches[2]);
    s.event(true, 0, 2, "game", 0);
    assert!(s.latches[2]);
    for (mode, source, tag) in [(1, 2, "恭喜"), (4, 0, "unused"), (5, 0, "unused")] {
        s.cancel();
        s.settings.trigger = mode;
        if mode == 1 {
            s.event(true, source, 0, tag, 0);
        } else if mode == 4 {
            c.header = Some("Duels".into());
        } else {
            c.footer = Some("Duels".into());
        }
        assert!(
            s.tick(true, &c, random)
                .contains(&Effect::Control("KillAura", false))
        );
    }
    let mut pick = Pick::default();
    pick.names = vec!["paper".into()];
    for mode in [1, 3, 5, 7] {
        pick.match_mode = mode;
        assert!(pick.matches(&slot(0, "minecraft:paper", "Paper")));
    }
    pick.match_mode = 0;
    assert!(!pick.matches(&slot(0, "minecraft:paper", "Paper")));
    s.cancel();
    s.settings.action = 0;
    s.settings.start_delay = [1, 1];
    s.settings.trigger = 0;
    s.event(true, 1, 0, "Win", 5000);
    c.now = 5000;
    s.tick(true, &c, random);
    c.now = 5001;
    assert!(s.wake(false, &c, random).is_empty());
    assert_eq!(s.next_wake(), None);
    native_scheduler::check();
}
/*NATIVE_SCHEDULER*/
