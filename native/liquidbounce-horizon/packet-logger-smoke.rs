#[path = "../liquidbounce-behaviors/src/packet_logger.rs"]
mod model;
#[test]
fn packet_filter_and_output() {
    let mut s = model::Settings::default();
    assert!(s.accepts(true, "minecraft:keep_alive"));
    s.s2c.push("minecraft:keep_alive".into());
    assert!(!s.accepts(true, "minecraft:keep_alive"));
    assert!(s.accepts(false, "minecraft:keep_alive"));
    s.whitelist = true;
    assert!(s.accepts(true, "minecraft:keep_alive"));
    assert!(!s.accepts(false, "minecraft:keep_alive"));
    let r = model::Record {
        time: 1,
        incoming: true,
        class: "KeepAlive".into(),
        id: "minecraft:keep_alive".into(),
        cancelled: true,
        fields: vec![model::Field {
            name: "id".into(),
            kind: "i64".into(),
            value: "\"a,b\"".into(),
        }],
    };
    assert_eq!(
        r.csv(),
        "1,Incoming,KeepAlive,minecraft:keep_alive,true,\"id:i64=\"\"a,b\"\";\"\n"
    );
    assert!(s.show_type && s.chat && !s.file);
}
