#[path = "../liquidbounce-behaviors/src/notifier.rs"]
mod notifier;
#[path = "../liquidbounce-behaviors/src/notifier_format.rs"]
mod notifier_format;
#[test]
fn events_and_cache() {
    use notifier::*;
    use notifier_format::format_message;
    let mut m = State::default();
    m.sync(true);
    m.initialize([("id".into(), "Tester".into(), 0)]);
    assert!(m.add("short".into(), "ab".into()).is_none());
    assert_eq!(m.add("new".into(), "NewPlayer".into()).unwrap().kind, 0);
    m.settings.switches[2] = true;
    assert!(m.mode("id", 0, false).is_none());
    assert_eq!(m.mode("id", 1, false).unwrap().kind, 2);
    m.settings.switches[4] = true;
    let item = Stack {
        id: "minecraft:end_crystal".into(),
        components: "a".into(),
        name: "Crystal".into(),
        count: 2,
        duration: 3,
        consumable: true,
    };
    let mut p = Player {
        id: 2,
        uuid: "id".into(),
        name: "Tester".into(),
        bot: false,
        using: true,
        active_hand: 0,
        ticks_using: 0,
        hands: [Some(item), None],
    };
    assert_eq!(m.tick(&[p.clone()]).len(), 1);
    assert!(m.tick(&[p.clone()]).is_empty());
    p.ticks_using = 2;
    assert!(m.tick(&[p.clone()]).is_empty());
    p.using = false;
    assert_eq!(m.tick(&[p.clone()])[0].kind, 3);
    p.hands[0].as_mut().unwrap().count = 1;
    assert_eq!(m.tick(&[p.clone()])[0].kind, 4);
    p.bot = true;
    assert!(m.tick(&[p]).is_empty());
    assert_eq!(
        m.entity_event("id", "Tester", 35, false, false)
            .unwrap()
            .args[1],
        Arg::Int(1)
    );
    assert!(m.entity_event("id", "Tester", 35, false, true).is_none());
    assert_eq!(
        m.entity_event("id", "Tester", 35, false, false)
            .unwrap()
            .args[1],
        Arg::Int(3)
    );
    m.remove_entity("id");
    assert_eq!(
        m.entity_event("id", "Tester", 35, false, false)
            .unwrap()
            .args[1],
        Arg::Int(1)
    );
    assert_eq!(
        format_message(
            "%2$04d %1$s %% %<S",
            &[Arg::Text("test".into()), Arg::Int(7)]
        )
        .unwrap(),
        "0007 test % TEST"
    );
    assert!(format_message("%d", &[Arg::Text("invalid".into())]).is_err());
    assert!(m.remove("new").is_some());
    m.sync(false);
    m.sync(true);
    m.initialize([]);
    assert!(m.remove("id").is_none());
}
