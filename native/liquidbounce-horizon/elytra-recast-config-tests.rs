#[test]
fn interop_contains_only_original_common_settings_and_localized_descriptions() {
    let common = CommonSettings {
        enabled: true,
        hidden: false,
        key_bind: json!({"key":"key.keyboard.x","action":"Toggle","modifiers":[]}),
    };
    let tree =
        elytra::serialize_elytra_recast(&common, JsonFormat::Interop, &|key| format!("中文:{key}"))
            .unwrap();
    assert_eq!(tree["name"], "ElytraRecast");
    assert_eq!(tree["valueType"], "TOGGLEABLE");
    let values = tree["value"].as_array().unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["name"], "Bind");
    assert_eq!(values[0]["value"], common.key_bind);
    assert_eq!(values[0]["key"], "liquidbounce.module.elytraRecast.bind");
    assert_eq!(
        values[0]["description"],
        "中文:liquidbounce.common.bind.description"
    );
    assert_eq!(values[1]["name"], "Hidden");
    assert_eq!(values[1]["value"], false);
    assert_eq!(
        values[1]["description"],
        "中文:liquidbounce.common.hidden.description"
    );
}

#[test]
fn file_format_keeps_enabled_bind_hidden_and_excludes_interop_metadata() {
    let common = CommonSettings {
        enabled: true,
        hidden: true,
        key_bind: json!(null),
    };
    let tree = elytra::serialize_elytra_recast(&common, JsonFormat::File, &|_| {
        panic!("file descriptions")
    })
    .unwrap();
    assert_eq!(
        tree,
        json!({"name":"ElytraRecast","value":[
        {"name":"Enabled","value":true},{"name":"Bind","value":null},
        {"name":"Hidden","value":true}]})
    );
}
