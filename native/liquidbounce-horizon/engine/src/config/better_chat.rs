// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::better_chat::{
    CHAT_TYPES, FEATURES, Filter, Settings, TranslationSettings,
};
#[derive(Clone, Debug, PartialEq)]
pub struct BetterChatSetting(pub String, pub Value);
fn key(p: &str) -> String {
    format!(
        "liquidbounce.module.betterChat.{}",
        p.split('.')
            .map(|s| s[..1].to_ascii_lowercase() + &s[1..])
            .collect::<Vec<_>>()
            .join(".")
    )
}
impl BetterChatSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        let path = format!("BetterChat.{p}");
        match p {
            "Features" => s.features = parse_multi_mask(v, &path, &FEATURES)?,
            "AutoTranslate" => s.auto_translate = parse_multi_mask(v, &path, &CHAT_TYPES)?,
            "AppendPrefix.Enabled" => s.append_prefix = parse_bool(v, &path)?,
            "AppendSuffix.Enabled" => s.append_suffix = parse_bool(v, &path)?,
            "AntiSpam.Enabled" => s.anti_spam = parse_bool(v, &path)?,
            "AntiSpam.StackMessages" => s.stack_messages = parse_bool(v, &path)?,
            "Copy.Enabled" => s.copy = parse_bool(v, &path)?,
            "Copy.Notify" => s.notify = parse_bool(v, &path)?,
            "Copy.Highlight" => s.highlight = parse_bool(v, &path)?,
            "AppendPrefix.Prefix" | "AppendSuffix.Suffix" => {
                let text =
                    scalar_string(v).ok_or_else(|| ConfigError::new(&path, "expected text"))?;
                if p == "AppendPrefix.Prefix" {
                    s.prefix = text;
                } else {
                    s.suffix = text;
                }
            }
            "AntiSpam.Filters" => {
                let list = v
                    .as_array()
                    .ok_or_else(|| ConfigError::new(&path, "expected regex list"))?;
                let mut filters = Vec::new();
                for v in list {
                    let text = scalar_string(v)
                        .ok_or_else(|| ConfigError::new(&path, "expected regex text"))?;
                    let regex =
                        Filter::new(&text).map_err(|e| ConfigError::new(&path, e.to_string()))?;
                    filters.push(regex);
                }
                s.filters = filters;
            }
            _ => return Err(ConfigError::new(&path, "unknown setting")),
        }
        Ok(())
    }
}
fn group(t: &Tree, name: &str, values: Vec<Value>) -> Value {
    let mut v = t.group(name, "TOGGLEABLE", values);
    if t.format == JsonFormat::Interop {
        v["key"] = json!(key(name));
        v["description"] = json!((t.describe)(&format!("{}.description", key(name))));
    }
    v
}
pub fn serialize_better_chat(
    s: &Settings,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::BetterChat, c);
    let leaf = |n: &str, ty: &str, v: Value, p: &str| t.leaf(n, ty, v, &key(p), None);
    values.push(t.multi("Features", s.features, &FEATURES, &key("Features")));
    values.push(group(
        &t,
        "AppendPrefix",
        vec![
            leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.append_prefix),
                "AppendPrefix.Enabled",
            ),
            leaf("Prefix", "TEXT", json!(s.prefix), "AppendPrefix.Prefix"),
        ],
    ));
    values.push(group(
        &t,
        "AppendSuffix",
        vec![
            leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.append_suffix),
                "AppendSuffix.Enabled",
            ),
            leaf("Suffix", "TEXT", json!(s.suffix), "AppendSuffix.Suffix"),
        ],
    ));
    values.push(t.multi(
        "AutoTranslate",
        s.auto_translate,
        &CHAT_TYPES,
        &key("AutoTranslate"),
    ));
    let mut filters = leaf(
        "Filters",
        "MUTABLE_LIST",
        json!(s.filters.iter().map(Filter::pattern).collect::<Vec<_>>()),
        "AntiSpam.Filters",
    );
    if format == JsonFormat::Interop {
        filters["innerValueType"] = json!("TEXT");
    }
    values.push(group(
        &t,
        "AntiSpam",
        vec![
            leaf("Enabled", "BOOLEAN", json!(s.anti_spam), "AntiSpam.Enabled"),
            leaf(
                "StackMessages",
                "BOOLEAN",
                json!(s.stack_messages),
                "AntiSpam.StackMessages",
            ),
            filters,
        ],
    ));
    values.push(group(
        &t,
        "Copy",
        vec![
            leaf("Enabled", "BOOLEAN", json!(s.copy), "Copy.Enabled"),
            leaf("Notify", "BOOLEAN", json!(s.notify), "Copy.Notify"),
            leaf("Highlight", "BOOLEAN", json!(s.highlight), "Copy.Highlight"),
        ],
    ));
    Ok(t.group("BetterChat", "TOGGLEABLE", values))
}
fn add(patch: &mut SettingsPatch, n: &Nodes, name: &str, path: &str) {
    leaf(patch, n, name, &format!("BetterChat.{path}"), |v, _| {
        let s = BetterChatSetting(path.into(), v.clone());
        s.apply(&mut Settings::default())?;
        Ok(SettingChange::BetterChat(s))
    });
}
pub(super) fn parse_better_chat(patch: &mut SettingsPatch, root: &Nodes) {
    for n in ["Features", "AutoTranslate"] {
        add(patch, root, n, n);
    }
    for (group, leaves) in [
        ("AppendPrefix", &["Enabled", "Prefix"][..]),
        ("AppendSuffix", &["Enabled", "Suffix"][..]),
        ("AntiSpam", &["Enabled", "StackMessages", "Filters"][..]),
        ("Copy", &["Enabled", "Notify", "Highlight"][..]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("BetterChat.{group}")) {
                Ok(n) => {
                    for name in leaves {
                        add(patch, &n, name, &format!("{group}.{name}"));
                    }
                }
                Err(e) => patch.issues.push(e),
            }
        }
    }
}
fn global_key(p: &str) -> String {
    format!("liquidbounce.autoTranslate.{p}")
}
pub fn serialize_global_translate(
    s: &TranslationSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Value {
    let t = Tree { format, describe };
    let leaf = |n: &str, v: &str, p: &str| t.leaf(n, "TEXT", json!(v), &global_key(p), None);
    let mut libre = t.group(
        "LibreTranslate",
        "CONFIGURABLE",
        vec![
            leaf("BaseUrl", &s.base_url, "provider.libreTranslate.baseUrl"),
            leaf("ApiKey", &s.api_key, "provider.libreTranslate.apiKey"),
        ],
    );
    if format == JsonFormat::Interop {
        if let Some(a) = libre["value"].as_array_mut() {
            if let Some(k) = a.iter_mut().find(|v| v["name"] == "ApiKey") {
                k["isPassword"] = json!(true);
            }
        }
    }
    let mut provider = t.mode(
        if s.provider == 0 {
            "Google"
        } else {
            "LibreTranslate"
        },
        Map::from_iter([
            ("Google".into(), t.group("Google", "CONFIGURABLE", vec![])),
            ("LibreTranslate".into(), libre),
        ]),
    );
    provider["name"] = json!("Provider");
    if format == JsonFormat::Interop {
        provider["key"] = json!(global_key("provider"));
        provider["description"] = json!(describe(&global_key("provider.description")));
    }
    let mut g = t.group(
        "AutoTranslate",
        "CONFIGURABLE",
        vec![
            leaf("LanguageCode", &s.language_code, "languageCode"),
            provider,
        ],
    );
    if format == JsonFormat::Interop {
        g["key"] = json!("liquidbounce.autoTranslate");
        g["description"] = json!(describe("liquidbounce.autoTranslate.description"));
    }
    g
}
fn global_text(issues: &mut Vec<ConfigError>, n: &Nodes, name: &str, path: &str, dst: &mut String) {
    if let Some(v) = n.get(name) {
        match value_of(v, path)
            .and_then(|v| scalar_string(v).ok_or_else(|| ConfigError::new(path, "expected text")))
        {
            Ok(v) => *dst = v,
            Err(e) => issues.push(e),
        }
    }
}
pub fn apply_global_translate(s: &mut TranslationSettings, root: &Value) -> Vec<ConfigError> {
    let mut issues = vec![];
    let Some(g) = root["value"]
        .as_array()
        .and_then(|a| a.iter().find(|n| n["name"] == "AutoTranslate"))
    else {
        return issues;
    };
    let n = match nodes(g, "AutoTranslate", "AutoTranslate") {
        Ok(n) => n,
        Err(e) => {
            issues.push(e);
            return issues;
        }
    };
    global_text(
        &mut issues,
        &n,
        "LanguageCode",
        "AutoTranslate.LanguageCode",
        &mut s.language_code,
    );
    if let Some(p) = n.get("Provider") {
        if let Some(v) = p["choices"].get("LibreTranslate") {
            match nodes(v, "LibreTranslate", "AutoTranslate.Provider.LibreTranslate") {
                Ok(n) => {
                    global_text(
                        &mut issues,
                        &n,
                        "BaseUrl",
                        "AutoTranslate.Provider.LibreTranslate.BaseUrl",
                        &mut s.base_url,
                    );
                    global_text(
                        &mut issues,
                        &n,
                        "ApiKey",
                        "AutoTranslate.Provider.LibreTranslate.ApiKey",
                        &mut s.api_key,
                    );
                }
                Err(e) => issues.push(e),
            }
        }
        match p["active"].as_str() {
            Some("Google") => s.provider = 0,
            Some("LibreTranslate") => s.provider = 1,
            _ => issues.push(ConfigError::new(
                "AutoTranslate.Provider",
                "unknown case-sensitive provider",
            )),
        }
    }
    issues
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_module_tree_roundtrips_inactive_fields_regex_and_multiselect() {
        let mut expected = Settings::default();
        expected.features = 9;
        expected.auto_translate = 7;
        expected.prefix = "前缀".into();
        expected.suffix = "後綴".into();
        expected.append_suffix = true;
        expected.filters = vec![
            Filter::new("(?i)spam").unwrap(),
            Filter::new("[你好]+").unwrap(),
            Filter::new("(?i)spam").unwrap(),
        ];
        expected.stack_messages = true;
        expected.copy = false;
        expected.notify = false;
        let common = CommonSettings {
            enabled: false,
            hidden: false,
            key_bind: json!({"boundKey":"key.keyboard.unknown","action":"Toggle"}),
        };
        for f in [JsonFormat::File, JsonFormat::Interop] {
            let tree = serialize_better_chat(&expected, &common, f, &str::to_owned).unwrap();
            let patch = parse_settings(ModuleKind::BetterChat, &tree).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
            let mut actual = Settings::default();
            for c in patch.changes {
                if let SettingChange::BetterChat(v) = c {
                    v.apply(&mut actual).unwrap();
                }
            }
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn invalid_regex_is_atomic_and_translation_global_roundtrips() {
        let mut s = Settings::default();
        assert!(
            BetterChatSetting("AntiSpam.Filters".into(), json!(["ok", "["]))
                .apply(&mut s)
                .is_err()
        );
        assert!(s.filters.is_empty());
        let expected = TranslationSettings {
            language_code: "zh-Hant".into(),
            provider: 1,
            base_url: "https://translator.example".into(),
            api_key: "secret".into(),
        };
        for f in [JsonFormat::File, JsonFormat::Interop] {
            let tree = json!({"name":"Settings","value":[serialize_global_translate(&expected,f,&str::to_owned)]});
            let mut actual = TranslationSettings::default();
            assert!(apply_global_translate(&mut actual, &tree).is_empty());
            assert_eq!(actual, expected);
        }
    }
}

pub fn global_translate_changes(before: &Value, after: &Value) -> Vec<Value> {
    fn walk(before: &Value, after: &Value, out: &mut Vec<Value>) {
        if let Some(values) = after["value"].as_array() {
            for leaf in values {
                let old = before["value"]
                    .as_array()
                    .and_then(|a| a.iter().find(|n| n["name"] == leaf["name"]))
                    .unwrap_or(&Value::Null);
                walk(old, leaf, out);
            }
            return;
        }
        if after["valueType"] == "CHOICE" || after.get("choices").is_some() {
            if before["active"] != after["active"] {
                out.push(after.clone());
            }
            for (name, choice) in after["choices"].as_object().into_iter().flatten() {
                walk(&before["choices"][name], choice, out);
            }
            return;
        }
        if before["value"] != after["value"] {
            out.push(after.clone());
        }
    }
    let mut out = vec![];
    walk(before, after, &mut out);
    out
}
