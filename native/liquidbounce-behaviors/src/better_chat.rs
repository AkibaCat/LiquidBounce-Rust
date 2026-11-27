// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBetterChat, AntiSpam and GlobalSettingsAutoTranslate source behavior.
use java_regex::{PatternSyntaxError, Regex};
use serde_json::{Value, json};
use std::collections::VecDeque;
pub const FEATURES: [&str; 4] = [
    "Infinite",
    "AntiClear",
    "KeepAfterDeath",
    "ForceUnicodeChat",
];
pub const CHAT_TYPES: [&str; 3] = ["ChatMessage", "DisguisedChatMessage", "GameMessage"];
#[derive(Clone, Debug)]
pub struct Filter(Regex);
impl Filter {
    pub fn new(s: &str) -> Result<Self, PatternSyntaxError> {
        Regex::new(s).map(Self)
    }
    pub fn pattern(&self) -> &str {
        self.0.pattern()
    }
    pub fn matches(&self, s: &str) -> bool {
        self.0.matches(s)
    }
}
impl PartialEq for Filter {
    fn eq(&self, o: &Self) -> bool {
        self.pattern() == o.pattern()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub features: u32,
    pub append_prefix: bool,
    pub prefix: String,
    pub append_suffix: bool,
    pub suffix: String,
    pub auto_translate: u32,
    pub anti_spam: bool,
    pub stack_messages: bool,
    pub filters: Vec<Filter>,
    pub copy: bool,
    pub notify: bool,
    pub highlight: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            features: 7,
            append_prefix: false,
            prefix: "> ".into(),
            append_suffix: false,
            suffix: " | 𝙻𝚒𝚚𝚞𝚒𝚍𝙱𝚘𝚞𝚗𝚌𝚎".into(),
            auto_translate: 0,
            anti_spam: true,
            stack_messages: false,
            filters: vec![],
            copy: true,
            notify: true,
            highlight: true,
        }
    }
}
impl Settings {
    pub fn feature(&self, bit: u32) -> bool {
        self.features & bit != 0
    }
    pub fn outgoing(&self, content: &str) -> String {
        let mut out = String::new();
        if self.append_prefix {
            out.push_str(&self.prefix);
        }
        for c in content.chars() {
            out.push(if self.feature(8) && (33..=128).contains(&(c as u32)) {
                char::from_u32(c as u32 + 65248).unwrap()
            } else {
                c
            });
        }
        if self.append_suffix {
            out.push_str(&self.suffix);
        }
        out
    }
    pub fn filtered(&self, plain: &str) -> bool {
        if !self.anti_spam {
            return false;
        }
        let content = plain
            .split_once('>')
            .map_or(plain, |(_, s)| s)
            .trim_matches(kotlin_whitespace);
        self.filters.iter().any(|p| p.matches(content))
    }
    pub fn stack_id(&self, kind: usize, plain: &str) -> Option<String> {
        (self.anti_spam && self.stack_messages && kind != 1).then(|| format!("{plain}-external"))
    }
    pub fn translation_text(&self, kind: usize, decorated: &str) -> Option<String> {
        if kind >= CHAT_TYPES.len() || self.auto_translate & (1 << kind) == 0 {
            return None;
        }
        let s = strip_colors(decorated);
        (!s.chars().all(kotlin_whitespace)).then_some(s)
    }
}
fn kotlin_whitespace(c: char) -> bool {
    matches!(c,'\t'..='\r'|'\u{001c}'..='\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}')
}
pub fn strip_colors(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§'
            && chars.peek().is_some_and(
                |c| matches!(c.to_ascii_lowercase(), '0'..='9' | 'a'..='f' | 'k'..='o' | 'r'),
            )
        {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationSettings {
    pub language_code: String,
    pub provider: usize,
    pub base_url: String,
    pub api_key: String,
}
impl Default for TranslationSettings {
    fn default() -> Self {
        Self {
            language_code: "en".into(),
            provider: 0,
            base_url: "https://libretranslate.com".into(),
            api_key: String::new(),
        }
    }
}
pub fn libre_code(s: &str) -> String {
    let s = s.to_ascii_lowercase();
    let subtags: Vec<_> = s.split(['-', '_']).collect();
    if subtags.first() == Some(&"zh")
        && subtags
            .iter()
            .any(|s| matches!(*s, "tw" | "hk" | "mo" | "hant"))
    {
        "zt".into()
    } else {
        subtags.first().copied().unwrap_or("").into()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Translation {
    pub origin: String,
    pub translation: String,
    pub from: String,
    pub to: String,
}
impl Translation {
    pub fn valid(&self) -> bool {
        self.origin != self.translation && !self.from.eq_ignore_ascii_case(&self.to)
    }
    pub fn display(&self) -> String {
        format!("({}->{}) {}", self.from, self.to, self.translation)
    }
}
pub fn translation_response(
    s: &TranslationSettings,
    origin: &str,
    json: &Value,
) -> Result<Translation, &'static str> {
    let (translated, from) = if s.provider == 0 {
        (json[0][0].as_str(), json[0][1].as_str())
    } else {
        (
            json["translatedText"].as_str(),
            json["detectedLanguage"]["language"]
                .as_str()
                .or(Some("auto")),
        )
    };
    Ok(Translation {
        origin: origin.into(),
        translation: translated.ok_or("Invalid translation response")?.into(),
        from: from.ok_or("Missing detected language")?.into(),
        to: s.language_code.clone(),
    })
}
pub fn libre_body(s: &TranslationSettings, text: &str) -> Value {
    let mut v =
        json!({"q":text,"source":"auto","target":libre_code(&s.language_code),"format":"text"});
    if !s.api_key.chars().all(kotlin_whitespace) {
        v["api_key"] = json!(s.api_key);
    }
    v
}
#[derive(Default)]
pub struct TranslationCache {
    entries: VecDeque<(TranslationSettings, String, i64, Translation)>,
}
impl TranslationCache {
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn get(&mut self, s: &TranslationSettings, text: &str, now: i64) -> Option<Translation> {
        self.entries
            .retain(|(_, _, at, _)| now.saturating_sub(*at) < 900_000);
        let i = self
            .entries
            .iter()
            .position(|(key, origin, _, _)| key == s && origin == text)?;
        let entry = self.entries.remove(i).unwrap();
        let value = entry.3.clone();
        self.entries.push_back(entry);
        Some(value)
    }
    pub fn put(&mut self, s: TranslationSettings, result: Translation, now: i64) {
        if let Some(i) = self
            .entries
            .iter()
            .position(|(key, text, _, _)| key == &s && text == &result.origin)
        {
            self.entries.remove(i);
        }
        self.entries
            .push_back((s, result.origin.clone(), now, result));
        while self.entries.len() > 512 {
            self.entries.pop_front();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outgoing_unicode_prefix_suffix_includes_128_and_preserves_cjk() {
        let mut s = Settings::default();
        s.features |= 8;
        s.append_prefix = true;
        s.append_suffix = true;
        s.suffix = "尾".into();
        assert_eq!(s.outgoing("!~\u{7f}\u{80} 中文"), "> ！～｟｠ 中文尾");
    }
    #[test]
    fn filters_match_entire_body_and_disguised_does_not_stack() {
        let mut s = Settings::default();
        s.filters = vec![Filter::new("spam").unwrap()];
        assert!(s.filtered("<User> spam "));
        assert!(!s.filtered("<User> spammer"));
        assert!(s.filtered(" spam "));
        s.stack_messages = true;
        assert_eq!(s.stack_id(0, "same"), Some("same-external".into()));
        assert!(s.stack_id(1, "same").is_none());
        s.anti_spam = false;
        assert!(!s.filtered("spam"));
    }
    #[test]
    fn translation_selection_providers_and_validity() {
        let mut s = Settings::default();
        s.auto_translate = 4;
        assert_eq!(s.translation_text(2, "§a你好"), Some("你好".into()));
        assert_eq!(
            s.translation_text(2, "§A好§z文本§"),
            Some("好§z文本§".into())
        );
        assert!(s.translation_text(0, "hello").is_none());
        assert!(s.translation_text(2, "§c  ").is_none());
        let mut g = TranslationSettings::default();
        let t = translation_response(&g, "你好", &json!([["Hello", "zh-CN"]])).unwrap();
        assert!(t.valid());
        assert_eq!(t.display(), "(zh-CN->en) Hello");
        g.provider = 1;
        g.language_code = "zh-Hant-TW".into();
        g.api_key = "secret".into();
        assert_eq!(libre_body(&g, "Hello")["target"], "zt");
        assert_eq!(libre_body(&g, "Hello")["api_key"], "secret");
        let t = translation_response(
            &g,
            "Hello",
            &json!({"translatedText":"你好","detectedLanguage":{"language":"en"}}),
        )
        .unwrap();
        assert!(t.valid());
        assert_eq!(libre_code("pt_BR"), "pt");
    }
    #[test]
    fn cache_expires_after_write_and_evicts_lru_at_512() {
        let mut c = TranslationCache::default();
        let s = TranslationSettings::default();
        for i in 0..513 {
            c.put(
                s.clone(),
                Translation {
                    origin: i.to_string(),
                    translation: "translated".into(),
                    from: "zh".into(),
                    to: "en".into(),
                },
                0,
            );
        }
        assert!(c.get(&s, "0", 1).is_none());
        assert!(c.get(&s, "1", 899999).is_some());
        assert!(c.get(&s, "1", 900000).is_none());
    }
}
