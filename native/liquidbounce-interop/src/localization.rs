// SPDX-License-Identifier: GPL-3.0-or-later
// Language descriptions are derived from LiquidBounce ae0eb396, CCBlueX 2015-2026.
//! Display-only translations. REST names, setting paths and stored values are
//! deliberately kept outside this layer. The caller supplies the game's loaded
//! language code, never a separate preference belonging to this port.

use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

type Dictionary = BTreeMap<String, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    English,
    SimplifiedChinese,
    TraditionalChinese,
}

impl Locale {
    pub fn from_game_language(code: &str) -> Self {
        let code = code.trim().replace('-', "_").to_ascii_lowercase();
        if code == "zh" || code.starts_with("zh_") {
            if code
                .split('_')
                .any(|part| matches!(part, "tw" | "hk" | "mo" | "hant"))
            {
                Self::TraditionalChinese
            } else {
                Self::SimplifiedChinese
            }
        } else {
            Self::English
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en_us",
            Self::SimplifiedChinese => "zh_cn",
            Self::TraditionalChinese => "zh_tw",
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct DisplayDictionary {
    module_names: Dictionary,
    labels: Dictionary,
    descriptions: Dictionary,
    english_descriptions: Dictionary,
}

struct Resources {
    english: Dictionary,
    simplified: Dictionary,
    traditional: Dictionary,
    simplified_display: DisplayDictionary,
    traditional_display: DisplayDictionary,
    module_keys: Dictionary,
}

fn resources() -> &'static Resources {
    static RESOURCES: OnceLock<Resources> = OnceLock::new();
    RESOURCES.get_or_init(|| {
        let simplified_display: DisplayDictionary =
            serde_json::from_str(include_str!("../resources/localization/display_zh_cn.json"))
                .expect("embedded simplified Chinese display labels");
        let mut traditional_display: DisplayDictionary =
            serde_json::from_str(include_str!("../resources/localization/display_zh_tw.json"))
                .expect("embedded traditional Chinese display labels");
        let mut english: Dictionary =
            serde_json::from_str(include_str!("../resources/localization/en_us.json"))
                .expect("embedded English LiquidBounce language resource");
        // The pinned files contain module descriptions but no descriptions for
        // the settings implemented here. Explicitly owned additions fill those
        // gaps; original language entries retain priority.
        for (key, value) in &simplified_display.english_descriptions {
            english.entry(key.clone()).or_insert_with(|| value.clone());
        }
        let mut simplified = english.clone();
        simplified.extend(simplified_display.descriptions.clone());
        simplified.extend(
            serde_json::from_str::<Dictionary>(include_str!(
                "../resources/localization/zh_cn.json"
            ))
            .expect("embedded simplified Chinese LiquidBounce language resource"),
        );
        // Missing traditional entries follow simplified Chinese, then English.
        // This is explicit: it does not pretend machine-converted text is an
        // independently reviewed traditional translation.
        let mut traditional = simplified.clone();
        traditional.extend(traditional_display.descriptions.clone());
        traditional.extend(
            serde_json::from_str::<Dictionary>(include_str!(
                "../resources/localization/zh_tw.json"
            ))
            .expect("embedded traditional Chinese LiquidBounce language resource"),
        );
        let mut traditional_names = simplified_display.module_names.clone();
        traditional_names.append(&mut traditional_display.module_names);
        traditional_display.module_names = traditional_names;
        let mut traditional_labels = simplified_display.labels.clone();
        traditional_labels.append(&mut traditional_display.labels);
        traditional_display.labels = traditional_labels;
        Resources {
            english,
            simplified,
            traditional,
            simplified_display,
            traditional_display,
            module_keys: serde_json::from_str(include_str!(
                "../resources/localization/module_keys.json"
            ))
            .expect("embedded stable module translation keys"),
        }
    })
}

fn translations(locale: Locale) -> &'static Dictionary {
    let resources = resources();
    match locale {
        Locale::English => &resources.english,
        Locale::SimplifiedChinese => &resources.simplified,
        Locale::TraditionalChinese => &resources.traditional,
    }
}

/// Resolve an upstream language key with the documented locale fallback.
pub fn translate(game_language: &str, key: &str) -> Option<&'static str> {
    translations(Locale::from_game_language(game_language))
        .get(key)
        .map(String::as_str)
}

/// Compatible with settings serializers' description callback.
pub fn describe(game_language: &str, key: &str) -> String {
    translate(game_language, key).unwrap_or(key).to_owned()
}

pub fn module_description(game_language: &str, name: &str) -> Option<&'static str> {
    let key = resources().module_keys.get(name)?;
    translations(Locale::from_game_language(game_language))
        .get(&format!("{key}.description"))
        .map(String::as_str)
}

fn display_dictionary(game_language: &str) -> Option<&'static DisplayDictionary> {
    match Locale::from_game_language(game_language) {
        Locale::English => None,
        Locale::SimplifiedChinese => Some(&resources().simplified_display),
        Locale::TraditionalChinese => Some(&resources().traditional_display),
    }
}

/// Native screens can use the same labels as the Web theme. Identity and
/// persistence keys must continue to use the caller's original strings.
pub fn display_label(game_language: &str, english: &str) -> String {
    display_dictionary(game_language)
        .and_then(|dictionary| dictionary.labels.get(english))
        .map_or_else(|| english.to_owned(), Clone::clone)
}

pub fn display_module_name(game_language: &str, name: &str) -> String {
    display_dictionary(game_language)
        .and_then(|dictionary| dictionary.module_names.get(name))
        .map_or_else(|| name.to_owned(), Clone::clone)
}

/// A display dictionary is not a module registry. Supplying all upstream module
/// names here never makes an unimplemented module available in the client.
pub fn snapshot(game_language: &str, generation: u64) -> Value {
    let locale = Locale::from_game_language(game_language);
    let resources = resources();
    let empty = DisplayDictionary::default();
    let display = match locale {
        Locale::English => &empty,
        Locale::SimplifiedChinese => &resources.simplified_display,
        Locale::TraditionalChinese => &resources.traditional_display,
    };
    json!({
        "gameLanguage": game_language,
        "locale": locale.code(),
        "generation": generation,
        "moduleNames": display.module_names,
        "labels": display.labels,
        "moduleKeys": resources.module_keys,
        "translations": translations(locale),
        "fallback": if locale == Locale::TraditionalChinese { vec!["zh_tw", "zh_cn", "en_us"] }
            else if locale == Locale::SimplifiedChinese { vec!["zh_cn", "en_us"] }
            else { vec!["en_us"] },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_loaded_game_language_and_normalizes_chinese_variants() {
        for language in ["zh_cn", "ZH-CN", "zh", "zh_sg", "zh-Hans-CN"] {
            assert_eq!(
                Locale::from_game_language(language),
                Locale::SimplifiedChinese
            );
        }
        for language in ["zh_tw", "zh-hk", "zh_mo", "zh-Hant-TW"] {
            assert_eq!(
                Locale::from_game_language(language),
                Locale::TraditionalChinese
            );
        }
        for language in ["en_us", "ja_jp", "de_de", "", "unknown"] {
            assert_eq!(Locale::from_game_language(language), Locale::English);
        }
    }

    #[test]
    fn descriptions_reuse_original_language_files() {
        assert_eq!(
            module_description("zh_cn", "NoBob"),
            Some("禁用视角摇晃效果。")
        );
        assert_eq!(
            module_description("zh_cn", "FullBright"),
            Some("无限夜视效果，让你在黑暗中看清一切。")
        );
        assert_eq!(
            module_description("en_us", "NoBob"),
            resources()
                .english
                .get("liquidbounce.module.noBob.description")
                .map(String::as_str)
        );
        assert_eq!(describe("zh_cn", "missing.key"), "missing.key");
        let chinese: Dictionary =
            serde_json::from_str(include_str!("../resources/localization/zh_cn.json")).unwrap();
        let traditional: Dictionary =
            serde_json::from_str(include_str!("../resources/localization/zh_tw.json")).unwrap();
        let english: Dictionary =
            serde_json::from_str(include_str!("../resources/localization/en_us.json")).unwrap();
        let mut english_fallbacks = 0;
        let mut chinese_fallbacks = 0;
        for (key, value) in &english {
            assert!(resources().simplified.contains_key(key));
            assert!(resources().traditional.contains_key(key));
            if !chinese.contains_key(key) {
                assert_eq!(translate("zh_cn", key), Some(value.as_str()));
                english_fallbacks += 1;
            }
            if !traditional.contains_key(key) {
                assert_eq!(translate("zh_tw", key), translate("zh_cn", key));
                chinese_fallbacks += 1;
            }
        }
        assert!(english_fallbacks > 0);
        assert!(chinese_fallbacks > 0);
        assert_eq!(
            translate("zh_cn", "liquidbounce.common.bind.description"),
            Some("设置此模块的快捷键和触发方式。")
        );
        assert_eq!(
            translate("zh_tw", "liquidbounce.common.bind.description"),
            Some("設定此模組的快捷鍵和觸發方式。")
        );
        assert_eq!(
            translate("en_us", "liquidbounce.common.bind.description"),
            Some("Set this module's key binding and activation action.")
        );
    }

    #[test]
    fn snapshot_keeps_stable_identifiers_and_restores_english() {
        let chinese = snapshot("zh_cn", 42);
        assert_eq!(chinese["moduleNames"].as_object().unwrap().len(), 236);
        assert_eq!(chinese["moduleNames"]["NoFOV"], "固定视场角");
        assert_eq!(chinese["labels"]["Render"], "渲染");
        assert_eq!(chinese["labels"]["Search"], "搜索");
        assert_eq!(chinese["moduleKeys"]["NoFOV"], "liquidbounce.module.noFOV");
        let english = snapshot("en_us", 43);
        assert_eq!(english["moduleNames"], json!({}));
        assert_eq!(english["labels"], json!({}));
        assert_eq!(english["moduleKeys"], chinese["moduleKeys"]);
        assert_eq!(english["generation"], 43);
        assert_eq!(snapshot("zh_tw", 44)["moduleNames"]["FullBright"], "全亮");
        assert_eq!(
            display_label("zh_cn", "Native game implementations"),
            "原生游戏实现"
        );
        assert_eq!(display_module_name("zh_tw", "NoFOV"), "固定視野角度");
        assert_eq!(display_module_name("en_us", "NoFOV"), "NoFOV");
    }
}
