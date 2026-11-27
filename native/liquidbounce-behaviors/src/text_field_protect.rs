// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleTextFieldProtect.kt (CCBlueX, 2015-2026).
use java_regex::{PatternSyntaxError, Regex};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Pattern(Regex);

impl Pattern {
    pub fn new(source: &str) -> Result<Self, PatternSyntaxError> {
        Regex::new(source).map(Self)
    }

    pub fn source(&self) -> &str {
        self.0.pattern()
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source()
    }
}
impl Eq for Pattern {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectError {
    NullPattern,
    NegativeCount,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextFieldProtect {
    // The source's linked set compares Regex instances by identity. Independently
    // deserialized duplicate strings therefore survive, in insertion order. Only
    // repeated nulls coalesce; the configuration adapter preserves that rule.
    pub patterns: Vec<Option<Pattern>>,
}

impl Default for TextFieldProtect {
    fn default() -> Self {
        Self {
            patterns: ["^/register.*", "^/login.*", "^/email.*"]
                .into_iter()
                .map(|s| Some(Pattern::new(s).expect("valid source default")))
                .collect(),
        }
    }
}

impl TextFieldProtect {
    /// Called separately for each actual EditBox.applyFormat argument. The
    /// count is displayPos in UTF-16 units, including for the second substring;
    /// neither the input's length nor the formatter's offset supplies this count.
    pub fn protect<'a>(
        &self,
        running: bool,
        input: &'a str,
        first_character_index: i32,
    ) -> Result<Cow<'a, str>, ProtectError> {
        if !running {
            return Ok(Cow::Borrowed(input));
        }
        for pattern in &self.patterns {
            let pattern = pattern.as_ref().ok_or(ProtectError::NullPattern)?;
            if pattern.0.matches(input) {
                let count = usize::try_from(first_character_index)
                    .map_err(|_| ProtectError::NegativeCount)?;
                return Ok(Cow::Owned("*".repeat(count)));
            }
        }
        Ok(Cow::Borrowed(input))
    }
}
