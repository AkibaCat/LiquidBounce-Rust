// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleMultiActions (alias MultiTask). Its three queries read enabled,
//! independently of ClientModule.running and event-handler conditions.
use serde_json::Value;

pub const ACTIONS: [&str; 3] = [
    "PlacingWhileBreaking",
    "AttackingWhileUsing",
    "BreakingWhileUsing",
];
pub const PLACING_WHILE_BREAKING: u8 = 1;
pub const ATTACKING_WHILE_USING: u8 = 2;
pub const BREAKING_WHILE_USING: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiActions {
    pub actions: u8,
}
impl Default for MultiActions {
    fn default() -> Self {
        Self { actions: 7 }
    }
}
impl MultiActions {
    pub fn may_place_while_breaking(&self, enabled: bool) -> bool {
        enabled && self.actions & PLACING_WHILE_BREAKING != 0
    }
    pub fn may_attack_while_using(&self, enabled: bool) -> bool {
        enabled && self.actions & ATTACKING_WHILE_USING != 0
    }
    pub fn may_break_while_using(&self, enabled: bool) -> bool {
        enabled && self.actions & BREAKING_WHILE_USING != 0
    }
    pub fn policy(&self, enabled: bool) -> (bool, bool, bool) {
        (
            self.may_place_while_breaking(enabled),
            self.may_attack_while_using(enabled),
            self.may_break_while_using(enabled),
        )
    }
    /// MultiChoiceListValue clears its live EnumSet before reading entries.
    /// A malformed array therefore retains any successfully read prefix.
    pub fn deserialize_actions(&mut self, value: &Value) -> Result<(), String> {
        self.actions = 0;
        let mut include = |value: &Value| -> Result<(), String> {
            let tag = gson_string(value)?;
            if let Some(index) = ACTIONS.iter().position(|choice| java_tag_eq(choice, &tag)) {
                self.actions |= 1 << index;
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
        _ => Err("Actions entry has no Gson string value".to_owned()),
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
    fn enabled_queries_and_every_action_subset_are_independent() {
        assert_eq!(MultiActions::default().policy(true), (true, true, true));
        for actions in 0..8 {
            let state = MultiActions { actions };
            assert_eq!(state.policy(false), (false, false, false));
            assert_eq!(
                state.policy(true),
                (actions & 1 != 0, actions & 2 != 0, actions & 4 != 0)
            );
        }
    }

    #[test]
    fn source_multichoice_clears_then_adds_and_keeps_malformed_prefix() {
        let mut state = MultiActions::default();
        assert!(
            state
                .deserialize_actions(&json!(["BreakingWhileUsing", null]))
                .is_err()
        );
        assert_eq!(state.actions, BREAKING_WHILE_USING);
        state
            .deserialize_actions(&json!([["placingwhilebreaking"], 17, true, "unknown"]))
            .unwrap();
        assert_eq!(state.actions, PLACING_WHILE_BREAKING);
        for value in [json!(null), json!({}), json!([]), json!(false)] {
            state.deserialize_actions(&value).unwrap();
            assert_eq!(state.actions, 0);
        }
        state
            .deserialize_actions(&json!("AttackıngWhileUsing"))
            .unwrap();
        assert_eq!(state.actions, ATTACKING_WHILE_USING);
    }
}
