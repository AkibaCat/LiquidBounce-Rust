// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSleepWalker has only the original common ClientModule settings.
use super::*;

pub fn serialize_sleep_walker(
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let tree = Tree { format, describe };
    Ok(tree.group(
        "SleepWalker",
        "TOGGLEABLE",
        tree.common(ModuleKind::SleepWalker, common),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_sleep_walker_has_only_common_settings_with_stable_keys() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        for format in [JsonFormat::Interop, JsonFormat::File] {
            let value = serialize_sleep_walker(&common, format, &str::to_owned).unwrap();
            let names: Vec<_> = value["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|leaf| leaf["name"].as_str().unwrap())
                .collect();
            if format == JsonFormat::File {
                assert_eq!(names, ["Enabled", "Bind", "Hidden"]);
            } else {
                assert_eq!(names, ["Bind", "Hidden"]);
            }
            assert_eq!(value["name"], "SleepWalker");
            assert!(!value.to_string().contains("BedGodMode"));
            let patch = parse_settings(ModuleKind::SleepWalker, &value).unwrap();
            assert!(patch.issues.is_empty(), "{:?}", patch.issues);
        }
    }
}
