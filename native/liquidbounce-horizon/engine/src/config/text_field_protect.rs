// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::text_field_protect::{Pattern, TextFieldProtect};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextFieldProtectSetting {
    Patterns(Vec<Option<Pattern>>),
}

impl TextFieldProtectSetting {
    pub fn apply(self, state: &mut TextFieldProtect) {
        match self {
            Self::Patterns(patterns) => state.patterns = patterns,
        }
    }
}

pub fn serialize_text_field_protect(
    state: &TextFieldProtect,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    let mut values = tree.common(ModuleKind::TextFieldProtect, common);
    let patterns: Vec<_> = state
        .patterns
        .iter()
        .map(|p| p.as_ref().map(Pattern::source))
        .collect();
    let mut value = tree.leaf(
        "Patterns",
        "MUTABLE_LIST",
        json!(patterns),
        "liquidbounce.module.textFieldProtect.patterns",
        None,
    );
    if format == JsonFormat::Interop {
        value["innerValueType"] = json!("TEXT");
    }
    values.push(value);
    Ok(tree.group("TextFieldProtect", "TOGGLEABLE", values))
}

fn patterns(value: &Value, path: &str) -> Result<Vec<Option<Pattern>>, ConfigError> {
    let items: Vec<_> = match value {
        Value::Array(items) => items.iter().collect(),
        item => vec![item],
    };
    let mut patterns = Vec::with_capacity(items.len());
    let mut has_null = false;
    // Deserialize the entire array before committing. Regex uses reference
    // identity in the source linked set: duplicate strings survive, repeated
    // nulls coalesce at the first insertion position.
    for item in items {
        match item {
            Value::Null if !has_null => {
                patterns.push(None);
                has_null = true;
            }
            Value::Null => {}
            Value::String(source) => patterns.push(Some(
                Pattern::new(source).map_err(|error| ConfigError::new(path, error.to_string()))?,
            )),
            _ => return Err(ConfigError::new(path, "Regex input must be string or null")),
        }
    }
    Ok(patterns)
}

pub(super) fn parse_text_field_protect(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    if let Some(node) = root.get("Patterns") {
        match node
            .get("value")
            .ok_or_else(|| ConfigError::new("TextFieldProtect.Patterns", "missing value"))
            .and_then(|value| patterns(value, "TextFieldProtect.Patterns"))
        {
            Ok(patterns) => patch.changes.push(SettingChange::TextFieldProtect(
                TextFieldProtectSetting::Patterns(patterns),
            )),
            Err(error) => patch.issues.push(error),
        }
    }
}
