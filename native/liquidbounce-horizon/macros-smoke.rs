// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../liquidbounce-behaviors/src/auto_queue.rs"]
mod auto_queue;
#[path = "../liquidbounce-behaviors/src/client_commands.rs"]
mod commands;
#[path = "../liquidbounce-behaviors/src/macros.rs"]
mod macros;
use macros::{COUNT, Slot, Start, State};
fn random(a: i32, _: i32) -> i32 {
    a
}
fn chat(start: Result<Option<Start>, &str>) -> u64 {
    match start.unwrap() {
        Some(Start::Chat(id)) => id,
        _ => panic!("expected chat"),
    }
}
fn slot(slot: i32, item: &str, name: &str) -> Slot {
    Slot {
        slot,
        item: item.into(),
        name: name.into(),
    }
}
#[test]
fn macros_and_shared_command_focused_smoke() {
    let mut s = State::default();
    assert_eq!(s.chats.len(), 5);
    assert_eq!(s.items.len(), 5);
    s.chats[0].trigger.key = "key.keyboard.r".into();
    s.chats[0].messages = vec!["first".into(), "second".into()];
    s.chats[0].delay = [5, 5];
    let event = s.new_event();
    assert_eq!(
        s.begin(0, event, "key.keyboard.r", 0, &[], random),
        Ok(None)
    );
    let a = chat(s.begin(0, event, "key.keyboard.r", 1, &[], random));
    assert_eq!(s.message(a), Ok(Some(("first".into(), false))));
    assert_eq!(s.sent(a, 1, random), Ok(false));
    assert!(s.due(5).is_empty());
    assert_eq!(s.next_wake(), Some(6));
    assert_eq!(s.due(6), vec![a]);
    // The iterator retains the original list; execution reads the current flag
    // and samples the current delay after sending each message.
    s.replace_messages(0, vec!["replacement".into()]);
    s.chats[0].client_command = true;
    s.chats[0].delay = [9, 9];
    assert_eq!(s.message(a), Ok(Some(("second".into(), true))));
    assert_eq!(s.sent(a, 6, random), Ok(false));
    assert_eq!(s.next_wake(), Some(15));
    assert!(s.due(14).is_empty());
    assert_eq!(s.due(15), vec![a]);
    assert_eq!(s.message(a), Ok(None));
    let first = s.new_event();
    let previous = chat(s.begin(0, first, "key.keyboard.r", 20, &[], random));
    let second = s.new_event();
    let failed = chat(s.begin(0, second, "key.keyboard.r", 21, &[], random));
    s.chats[1].trigger.key = "key.keyboard.r".into();
    s.chats[1].messages = vec!["sibling".into()];
    let sibling = chat(s.begin(1, second, "key.keyboard.r", 21, &[], random));
    s.failed(failed);
    assert!(s.message(previous).unwrap().is_some());
    assert_eq!(s.message(failed), Ok(None));
    assert_eq!(s.message(sibling), Ok(None));
    s.chats[2].trigger.key = "key.keyboard.r".into();
    assert_eq!(
        s.begin(2, second, "key.keyboard.r", 21, &[], random),
        Ok(None)
    );
    assert_eq!(s.chats[2].trigger.last, 21);
    let event = s.new_event();
    s.items[0].trigger.key = "key.keyboard.r".into();
    let slots = [
        slot(-1, "minecraft:paper", "Paper"),
        slot(0, "minecraft:paper", "Paper"),
    ];
    assert_eq!(
        s.begin(COUNT, event, "key.keyboard.r", 30, &slots, random),
        Ok(Some(Start::Item {
            slot: -1,
            hold: 1,
            action: 0,
            swing: 0
        }))
    );
    s.items[0].pick.mode = 1;
    s.items[0].pick.items = vec!["minecraft:air".into()];
    assert!(matches!(
        s.begin(
            COUNT,
            event,
            "key.keyboard.r",
            31,
            &[slot(-1, "minecraft:air", "Air")],
            random
        ),
        Ok(Some(Start::Item { slot: -1, .. }))
    ));
    s.items[0].hold = [2, 1];
    assert!(
        s.begin(
            COUNT,
            event,
            "key.keyboard.r",
            32,
            &[slot(-1, "minecraft:air", "Air")],
            random
        )
        .is_err()
    );
    s.chats[0].delay = [2, 1];
    assert!(s.sent(previous, 33, random).is_err());
    s.failed(previous);
    s.chats[0].trigger.delay = [2, 1];
    let last = s.chats[0].trigger.current;
    assert!(
        s.begin(0, event, "key.keyboard.r", 34, &[], random)
            .is_err()
    );
    assert_eq!(s.chats[0].trigger.current, last);
    assert_eq!(s.chats[0].trigger.last, 34);
    let other = s.new_event();
    let changed = chat(s.begin(1, other, "key.keyboard.r", 40, &[], random));
    s.import_messages(1, vec!["edited".into()]);
    assert!(s.message(changed).is_err());
    s.cancel();
    assert_eq!(s.next_wake(), None);
    assert_eq!(s.chats[0].trigger.last, 34);
    let mut r = commands::Reader::new(" VALUE   SET module.macros.chat-1.messages \"a   b,c\" ");
    assert_eq!(r.word().as_deref(), Some("VALUE"));
    assert_eq!(r.word().as_deref(), Some("SET"));
    assert_eq!(r.word().as_deref(), Some("module.macros.chat-1.messages"));
    assert_eq!(r.string().unwrap().as_deref(), Some("a   b,c"));
    r.finish().unwrap();
    assert_eq!(
        commands::Reader::new("'a   b'")
            .string()
            .unwrap()
            .as_deref(),
        Some("a b")
    );
    assert_eq!(
        commands::Reader::new("\"a\\\"b\\\\c\"")
            .string()
            .unwrap()
            .as_deref(),
        Some("a\"b\\c")
    );
    assert!(commands::Reader::new("\"bad\\q\"").string().is_err());
    assert!(commands::Reader::new("\"unclosed").string().is_err());
    assert!(commands::Reader::new("\"Macros\"r").string().is_err());
    assert_eq!(
        commands::Reader::new("\"friend name\"").word().as_deref(),
        Some("\"friend")
    );
    native_scheduler::check();
}
/*NATIVE_SCHEDULER*/
