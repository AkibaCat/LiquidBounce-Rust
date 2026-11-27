// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from pinned ModuleEntityControl.kt and MultiChoiceListValue.
use serde_json::Value;
pub const ENFORCE: [&str; 2] = ["Saddled", "JumpStrength"];
pub const SADDLED: u8 = 1;
pub const JUMP_STRENGTH: u8 = 2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityControl {
    pub enforce: u8,
}
impl Default for EntityControl {
    fn default() -> Self {
        Self { enforce: 3 }
    }
}
impl EntityControl {
    pub fn policy(self, running: bool) -> (bool, bool) {
        (
            running && self.enforce & SADDLED != 0,
            running && self.enforce & JUMP_STRENGTH != 0,
        )
    }
    /// Exact Gson MultiChoiceList semantics: clear first, retain a successfully
    /// read prefix on a malformed later element; unknown primitive tags ignored.
    pub fn deserialize_enforce(&mut self, value: &Value) -> Result<(), String> {
        self.enforce = 0;
        let mut include = |value: &Value| -> Result<(), String> {
            let tag = gson_string(value)?;
            if let Some(index) = ENFORCE.iter().position(|choice| java_tag_eq(choice, &tag)) {
                self.enforce |= 1 << index;
            }
            Ok(())
        };
        match value {
            Value::Array(values) => {
                for value in values {
                    include(value)?;
                }
            }
            Value::String(_) | Value::Number(_) | Value::Bool(_) => include(value)?,
            _ => {}
        }
        Ok(())
    }
}
fn gson_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Array(values) if values.len() == 1 => gson_string(&values[0]),
        _ => Err("Enforce entry has no Gson string value".to_owned()),
    }
}

// Tagged.makeLookupTable uses String.CASE_INSENSITIVE_ORDER. Its tags are
// ASCII; Java's single-character fold also accepts dotted/dotless I and ſ.
fn java_tag_eq(choice: &str, value: &str) -> bool {
    let fold = |character: char| {
        let mut upper = character.to_uppercase();
        let first = upper.next().unwrap_or(character);
        let upper = if upper.next().is_none() {
            first
        } else {
            character
        };
        upper.to_lowercase().next().unwrap_or(upper)
    };
    choice.chars().map(fold).eq(value.chars().map(fold))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn original_defaults_and_all_independent_subsets_use_running() {
        assert_eq!(EntityControl::default().policy(true), (true, true));
        for enforce in 0..4 {
            let state = EntityControl { enforce };
            assert_eq!(state.policy(true), (enforce & 1 != 0, enforce & 2 != 0));
            assert_eq!(state.policy(false), (false, false));
        }
    }
    #[test]
    fn import_clears_first_and_retains_malformed_prefix() {
        let mut state = EntityControl::default();
        assert!(
            state
                .deserialize_enforce(&json!(["JumpStrength", null]))
                .is_err()
        );
        assert_eq!(state.enforce, 2);
        state
            .deserialize_enforce(&json!([["saddled"], 17, true, "unknown"]))
            .unwrap();
        assert_eq!(state.enforce, 1);
        state.deserialize_enforce(&Value::Null).unwrap();
        assert_eq!(state.enforce, 0);
        state.deserialize_enforce(&json!({})).unwrap();
        assert_eq!(state.enforce, 0);
        state.deserialize_enforce(&json!("jumpſtrength")).unwrap();
        assert_eq!(state.enforce, 2);
    }
}
