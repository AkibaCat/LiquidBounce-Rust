use crate::terrain_speed::{ClimbMode, TerrainSpeed};
fn describe(key: &str) -> String {
    format!("中文:{key}")
}
fn common() -> CommonSettings {
    CommonSettings {
        enabled: true,
        key_bind: json!({"boundKey":"key.keyboard.g","modifiers":0}),
        hidden: false,
    }
}
fn parse(value: &Value) -> SettingsPatch {
    let mut p = SettingsPatch::default();
    terrain::parse_terrain_speed(
        &mut p,
        &nodes(value, "TerrainSpeed", "TerrainSpeed").unwrap(),
    );
    p
}
fn apply(p: SettingsPatch) -> TerrainSpeed {
    assert!(p.issues.is_empty(), "{:?}", p.issues);
    let mut s = TerrainSpeed::default();
    for SettingChange::TerrainSpeed(v) in p.changes {
        v.apply(&mut s);
    }
    s
}
fn find<'a>(root: &'a mut Value, name: &str) -> &'a mut Value {
    root["value"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|v| v["name"] == name)
        .unwrap()
}
#[test]
fn all_fourteen_settings_roundtrip_in_both_real_tree_formats() {
    let mut s = TerrainSpeed::default();
    s.fast_climb.enabled = false;
    s.fast_climb.mode = ClimbMode::Clip;
    s.fast_climb.motion = -0.0;
    s.ice_speed.enabled = false;
    s.ice_speed.slipperiness = -2.5;
    s.ice_speed.motion_enabled = true;
    s.ice_speed.horizontal_motion = 99.;
    s.water_speed.enabled = false;
    s.water_speed.auto_swim = false;
    s.water_speed.base_horizontal = 11.;
    s.water_speed.base_vertical = -12.;
    s.water_speed.sprint_enabled = false;
    s.water_speed.sprint_horizontal = 1e-30;
    s.water_speed.sprint_vertical = -0.0;
    for f in [JsonFormat::File, JsonFormat::Interop] {
        let v = terrain::serialize_terrain_speed(&s, &common(), f, &describe).unwrap();
        let p = parse(&v);
        assert_eq!(p.changes.len(), 14);
        let a = apply(p);
        assert_eq!(a, s);
        assert_eq!(a.fast_climb.motion.to_bits(), s.fast_climb.motion.to_bits());
        assert_eq!(
            a.water_speed.sprint_vertical.to_bits(),
            s.water_speed.sprint_vertical.to_bits()
        );
        assert_eq!(v["name"], "TerrainSpeed");
        let text = v.to_string();
        assert_eq!(text.contains("中文:"), f == JsonFormat::Interop);
    }
}
#[test]
fn original_defaults_ranges_and_complete_inactive_choice_are_present() {
    let s = TerrainSpeed::default();
    let mut v =
        terrain::serialize_terrain_speed(&s, &common(), JsonFormat::Interop, &describe).unwrap();
    let climb = find(&mut v, "FastClimb");
    assert_eq!(find(climb, "Enabled")["value"], true);
    let mode = find(climb, "Mode");
    assert_eq!(mode["active"], "Motion");
    assert!(
        mode["choices"]["Clip"]["value"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let motion = find(&mut mode["choices"]["Motion"], "Motion");
    assert_eq!(motion["valueType"], "FLOAT");
    assert_eq!(motion["range"], json!({"from":0.1f32,"to":0.5f32}));
    assert_eq!(apply(parse(&v)), s);
}
#[test]
fn malformed_leaf_reports_issue_and_preserves_independent_changes() {
    let mut v = terrain::serialize_terrain_speed(
        &TerrainSpeed::default(),
        &common(),
        JsonFormat::File,
        &describe,
    )
    .unwrap();
    find(find(&mut v, "IceSpeed"), "Slipperiness")["value"] = json!("not-a-float");
    find(find(&mut v, "WaterSpeed"), "AutoSwim")["value"] = json!(false);
    let p = parse(&v);
    assert_eq!(p.issues.len(), 1);
    assert_eq!(p.changes.len(), 13);
    assert!(p.changes.contains(&SettingChange::TerrainSpeed(
        terrain::TerrainSpeedSetting::AutoSwim(false)
    )));
}
#[test]
fn mode_names_are_case_sensitive_and_all_choice_data_is_imported() {
    let mut v = terrain::serialize_terrain_speed(
        &TerrainSpeed::default(),
        &common(),
        JsonFormat::File,
        &describe,
    )
    .unwrap();
    let mode = find(find(&mut v, "FastClimb"), "Mode");
    mode["active"] = json!("motion");
    find(&mut mode["choices"]["Motion"], "Motion")["value"] = json!("-17.25");
    let p = parse(&v);
    assert_eq!(p.issues.len(), 1);
    assert!(p.changes.contains(&SettingChange::TerrainSpeed(
        terrain::TerrainSpeedSetting::ClimbMotion(-17.25)
    )));
}
#[test]
fn export_rejects_non_finite_settings() {
    let mut s = TerrainSpeed::default();
    s.ice_speed.slipperiness = f32::NAN;
    assert!(terrain::serialize_terrain_speed(&s, &common(), JsonFormat::File, &describe).is_err());
}
#[test]
fn absent_choices_and_malformed_mode_children_report_each_original_import_failure() {
    let mut v = terrain::serialize_terrain_speed(
        &TerrainSpeed::default(),
        &common(),
        JsonFormat::File,
        &describe,
    )
    .unwrap();
    let mode = find(find(&mut v, "FastClimb"), "Mode");
    mode["choices"] = json!({});
    mode["value"] = json!(false);
    let p = parse(&v);
    assert_eq!(p.issues.len(), 3);
    assert_eq!(p.changes.len(), 13);
}
