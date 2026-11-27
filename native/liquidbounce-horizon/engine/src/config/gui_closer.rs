// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::gui_closer::{Filter, GuiCloser, Mode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiCloserSetting {
    Mode(Mode),
    Filter(Vec<Option<Filter>>),
    PrintScreenTitle(bool),
}
impl GuiCloserSetting {
    pub fn apply(self, state: &mut GuiCloser) {
        match self {
            Self::Mode(value) => state.mode = value,
            Self::Filter(value) => state.filters = value,
            Self::PrintScreenTitle(value) => state.print_screen_title = value,
        }
    }
}

pub fn serialize_gui_closer(
    state: &GuiCloser,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::GuiCloser, common);
    let mut mode = tree.leaf("Mode", "CHOOSE", json!(state.mode.tag()),
        "liquidbounce.module.guiCloser.mode", None);
    let filters: Vec<_> = state.filters.iter().map(|v| v.as_ref().map(Filter::pattern)).collect();
    let mut filter = tree.leaf("Filter", "MUTABLE_LIST", json!(filters),
        "liquidbounce.module.guiCloser.filter", None);
    if format == JsonFormat::Interop {
        mode["choices"] = json!(["Matches", "Contains"]);
        filter["innerValueType"] = json!("TEXT");
    }
    values.extend([
        mode, filter,
        // doNotIncludeAlways is excluded only by PUBLIC_SERIALIZER.
        // FILE and INTEROP both explicitly include private values.
        tree.leaf("PrintScreenTitle", "BOOLEAN", json!(state.print_screen_title),
            "liquidbounce.module.guiCloser.printScreenTitle", None),
    ]);
    Ok(tree.group("GUICloser", "TOGGLEABLE", values))
}

fn filters(value: &Value, path: &str) -> Result<Vec<Option<Filter>>, ConfigError> {
    fn one(value: &Value, path: &str) -> Result<Option<Filter>, ConfigError> {
        match value {
            Value::Null => Ok(None),
            Value::String(pattern) => Filter::new(pattern).map(Some)
                .map_err(|error| ConfigError::new(path, error.to_string())),
            _ => Err(ConfigError::new(path, "Regex input must be string or null")),
        }
    }
    // Deserialize every item before replacing the old mutable list, as ListValue does.
    match value {
        Value::Array(values) => values.iter().map(|v| one(v, path)).collect(),
        value => one(value, path).map(|v| vec![v]),
    }
}

pub(super) fn parse_gui_closer(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Mode", "GUICloser.Mode", |v, p| {
        scalar_string(v).and_then(|tag| Mode::parse(&tag))
            .map(|v| SettingChange::GuiCloser(GuiCloserSetting::Mode(v)))
            .ok_or_else(|| ConfigError::new(p, "unknown choice"))
    });
    // This adapter accepts explicit JsonNull; the common scalar helper rejects it.
    if let Some(node) = root.get("Filter") {
        match node.get("value").ok_or_else(|| ConfigError::new("GUICloser.Filter", "missing value"))
            .and_then(|v| filters(v, "GUICloser.Filter")) {
            Ok(value) => patch.changes.push(SettingChange::GuiCloser(GuiCloserSetting::Filter(value))),
            Err(error) => patch.issues.push(error),
        }
    }
    leaf(patch, root, "PrintScreenTitle", "GUICloser.PrintScreenTitle", |v, p| {
        parse_bool(v, p).map(|v| SettingChange::GuiCloser(GuiCloserSetting::PrintScreenTitle(v)))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn common() -> CommonSettings {
        CommonSettings { enabled: false, hidden: false, key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}) }
    }
    #[test]
    fn complete_file_interop_tree_and_private_setting() {
        let state = GuiCloser::default();
        for format in [JsonFormat::File, JsonFormat::Interop] {
            let value = serialize_gui_closer(&state, &common(), format, &str::to_owned).unwrap();
            let n = nodes(&value, "GUICloser", "GUICloser").unwrap();
            assert_eq!(n["Mode"]["value"], "Matches");
            assert_eq!(n["Filter"]["value"], json!(["^Vote$"]));
            assert_eq!(n["PrintScreenTitle"]["value"], false);
            if format == JsonFormat::Interop {
                assert_eq!(n["Mode"]["choices"], json!(["Matches", "Contains"]));
                assert_eq!(n["Filter"]["valueType"], "MUTABLE_LIST");
                assert_eq!(n["Filter"]["innerValueType"], "TEXT");
            } else { assert!(n["Filter"].get("innerValueType").is_none()); }
        }
    }
    #[test]
    fn source_list_deserialization_is_atomic_ordered_and_nullable() {
        for input in [json!([]), json!("a"), Value::Null, json!(["a", null, "a", "(?i)b"])] {
            let mut state = GuiCloser::default();
            let root = json!({"name":"GUICloser","value":[{"name":"Filter","value":input}]});
            let patch = parse_settings(ModuleKind::GuiCloser, &root).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            for change in patch.changes { if let SettingChange::GuiCloser(v) = change { v.apply(&mut state); } }
            assert_eq!(state.filters, filters(&input, "Filter").unwrap());
        }
        for input in [json!(true), json!(2), json!({}), json!(["ok", "["]), json!(["ok", false])] {
            let mut state = GuiCloser::default();
            let old = state.filters.clone();
            let root = json!({"name":"GUICloser","value":[{"name":"Filter","value":input},{"name":"PrintScreenTitle","value":true}]});
            let patch = parse_settings(ModuleKind::GuiCloser, &root).unwrap();
            assert_eq!(patch.issues.len(), 1, "{input}");
            for change in patch.changes { if let SettingChange::GuiCloser(v) = change { v.apply(&mut state); } }
            assert_eq!(state.filters, old);
            assert!(state.print_screen_title);
        }
    }
    #[test]
    fn unicode_choice_lookup_and_runtime_forwarding() {
        for tag in ["Matches", "matcheſ", "CONTAİNS", "Contaıns"] {
            assert!(Mode::parse(tag).is_some(), "{tag}");
        }
        let value = GuiCloserSetting::Filter(vec![None, Some(Filter::new("a").unwrap())]);
        assert_eq!(apply_setting(&SettingChange::GuiCloser(value.clone()), &mut NoFov::default(), &mut Zoom::default(), &mut FullBright::default()), ApplyResult::Runtime(RuntimeRequest::GuiCloser(value)));
        assert_eq!(ModuleKind::GuiCloser.key(), "liquidbounce.module.guiCloser");
        assert_eq!(ModuleKind::GuiCloser.category(), "Misc");
    }
}
