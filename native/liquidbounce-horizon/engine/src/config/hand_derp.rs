// SPDX-License-Identifier: GPL-3.0-or-later
//! Both original HandDerp modes and every active/inactive setting.
use super::*;
use liquidbounce_behaviors::hand_derp::{HandDerp, Mode, MODES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandDerpSetting { Silent(bool), Mode(Mode), Delay(i32) }
impl HandDerpSetting {
    pub fn apply(self, state: &mut HandDerp) {
        match self {
            Self::Silent(value) => state.settings.silent = value,
            Self::Mode(value) => state.set_mode(value),
            Self::Delay(value) => state.settings.delay = value,
        }
    }
}
pub fn serialize_hand_derp(state: &HandDerp, common: &CommonSettings, format: JsonFormat, describe: &dyn Fn(&str) -> String) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::HandDerp, common);
    values.push(tree.leaf("Silent", "BOOLEAN", json!(state.settings.silent), "liquidbounce.module.handDerp.silent", None));
    let mut choices = Map::new();
    choices.insert("Delay".into(), tree.group("Delay", "CONFIGURABLE", vec![tree.ranged("Delay", "INT", json!(state.settings.delay), json!({"from":0,"to":20}), "ticks", "liquidbounce.module.handDerp.mode.delay.delay")]));
    choices.insert("Swing".into(), tree.group("Swing", "CONFIGURABLE", vec![]));
    values.push(tree.mode(state.settings.mode.tag(), choices));
    Ok(tree.group("HandDerp", "TOGGLEABLE", values))
}
pub(super) fn parse_hand_derp(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Silent", "HandDerp.Silent", |v,p| parse_bool(v,p).map(|v| SettingChange::HandDerp(HandDerpSetting::Silent(v))));
    let Some(mode) = root.get("Mode") else { return; };
    match mode.get("active").and_then(scalar_string).and_then(|tag| Mode::parse(&tag)) {
        Some(value) => patch.changes.push(SettingChange::HandDerp(HandDerpSetting::Mode(value))),
        None => patch.issues.push(ConfigError::new("HandDerp.Mode.active", "missing or unknown case-sensitive mode")),
    }
    if let Err(error) = nodes(mode,"Mode","HandDerp.Mode") { patch.issues.push(error); }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else { patch.issues.push(ConfigError::new("HandDerp.Mode.choices", "expected choices object")); return; };
    for name in MODES {
        let path = format!("HandDerp.Mode.{name}");
        match choices.get(name) {
            Some(choice) => match nodes(choice,name,&path) {
                Ok(choice) if name == "Delay" => leaf(patch,&choice,"Delay",&format!("{path}.Delay"),|v,p| parse_int(v,p).map(|v| SettingChange::HandDerp(HandDerpSetting::Delay(v)))),
                Ok(_) => {},
                Err(error) => patch.issues.push(error),
            },
            None => patch.issues.push(ConfigError::new(&path,"choice not found")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings { CommonSettings { enabled:true,hidden:false,key_bind:json!({"key":"key.keyboard.unknown","action":"Toggle"}) } }
    #[test]
    fn complete_two_mode_tree_roundtrips_inactive_delay_and_raw_values() {
        for mode in [Mode::Delay,Mode::Swing] {
            for delay in [i32::MIN,-1,0,1,20,i32::MAX] {
                let mut state=HandDerp::default(); state.set_mode(mode); state.settings.delay=delay; state.settings.silent=true;
                let tree=serialize_hand_derp(&state,&common(),JsonFormat::Interop,&str::to_owned).unwrap();
                assert_eq!(tree["value"].as_array().unwrap().iter().map(|v|v["name"].as_str().unwrap()).collect::<Vec<_>>(),["Bind","Hidden","Silent","Mode"]);
                assert_eq!(tree["value"][3]["choices"]["Delay"]["value"][0]["range"],json!({"from":0,"to":20}));
                assert_eq!(tree["value"][3]["choices"]["Delay"]["value"][0]["suffix"],"ticks");
                assert_eq!(tree["value"][3]["choices"]["Swing"]["value"],json!([]));
                let stored=serialize_hand_derp(&state,&common(),JsonFormat::File,&str::to_owned).unwrap();
                let patch=parse_settings(ModuleKind::HandDerp,&stored).unwrap(); assert!(patch.issues.is_empty(),"{:?}",patch.issues);
                let mut restored=HandDerp::default();
                for change in patch.changes { if let SettingChange::HandDerp(value)=change { value.apply(&mut restored); } }
                assert_eq!(restored.settings,state.settings);
            }
        }
    }
    #[test]
    fn bad_mode_still_imports_inactive_delay_and_mode_lifecycle() {
        let mut state=HandDerp::default();
        state.start_delay(true,||1); HandDerpSetting::Mode(Mode::Delay).apply(&mut state); assert!(state.busy());
        HandDerpSetting::Mode(Mode::Swing).apply(&mut state); assert!(!state.busy());
        let mut tree=serialize_hand_derp(&state,&common(),JsonFormat::File,&str::to_owned).unwrap();
        let values=tree["value"].as_array_mut().unwrap();
        let mode=values.iter_mut().find(|v|v["name"]=="Mode").unwrap();
        mode["active"]=json!("swing"); mode["choices"]["Delay"]["value"][0]["value"]=json!(4294967297_i64);
        let patch=parse_settings(ModuleKind::HandDerp,&tree).unwrap(); assert_eq!(patch.issues.len(),1);
        for change in patch.changes { if let SettingChange::HandDerp(value)=change { value.apply(&mut state); } }
        assert_eq!(state.settings.delay,1); assert_eq!(state.settings.mode,Mode::Swing);
        assert!(matches!(apply_setting(&SettingChange::HandDerp(HandDerpSetting::Silent(true)),&mut NoFov::default(),&mut Zoom::default(),&mut FullBright::default()),ApplyResult::Runtime(RuntimeRequest::HandDerp(HandDerpSetting::Silent(true)))));
        assert_eq!(ModuleKind::HandDerp.category(),"Fun");
        assert_eq!(ModuleKind::HandDerp.key(),"liquidbounce.module.handDerp");
    }
}
