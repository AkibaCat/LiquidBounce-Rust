// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleNoSlowBreak.kt and MixinPlayer.java at LiquidBounce ae0eb396.
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoSlowBreak {
    pub mining_fatigue: bool,
    pub on_air: bool,
    pub underwater: bool,
}
impl Default for NoSlowBreak {
    fn default() -> Self {
        Self {
            mining_fatigue: true,
            on_air: true,
            underwater: false,
        }
    }
}
impl NoSlowBreak {
    pub fn apply(self, running: bool, fatigue: &mut bool, water: &mut bool, on_ground: &mut bool) {
        if running {
            if self.mining_fatigue {
                *fatigue = false;
            }
            if self.underwater {
                *water = false;
            }
            if self.on_air {
                *on_ground = true;
            }
        }
    }
    pub fn tags(self) -> Vec<&'static str> {
        [
            (self.mining_fatigue, "MiningFatigue"),
            (self.on_air, "OnAir"),
            (self.underwater, "Underwater"),
        ]
        .into_iter()
        .filter_map(|(active, tag)| active.then_some(tag))
        .collect()
    }
    /// The original multichoice clears before reading and ignores unknown tags.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        let mut result = Self {
            mining_fatigue: false,
            on_air: false,
            underwater: false,
        };
        let mut include = |value: &Value| -> Result<(), String> {
            match value {
                Value::String(tag) => {
                    if tag.eq_ignore_ascii_case("MiningFatigue") {
                        result.mining_fatigue = true;
                    }
                    if tag.eq_ignore_ascii_case("OnAir") {
                        result.on_air = true;
                    }
                    if tag.eq_ignore_ascii_case("Underwater") {
                        result.underwater = true;
                    }
                    Ok(())
                }
                Value::Number(_) | Value::Bool(_) => Ok(()),
                _ => Err("NoSlowBreak.When entries must be JSON primitives".into()),
            }
        };
        match value {
            Value::Array(values) => {
                for value in values {
                    include(value)?;
                }
            }
            Value::String(_) | Value::Number(_) | Value::Bool(_) => include(value)?,
            Value::Object(_) | Value::Null => {}
        }
        Ok(result)
    }
}
