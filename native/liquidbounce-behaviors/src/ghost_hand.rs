// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleGhostHand's complete registry-list state, LiquidBounce ae0eb396.
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhostHand {
    pub targeted_blocks: BTreeSet<String>,
}
impl Default for GhostHand {
    fn default() -> Self {
        let mut targeted_blocks = ["chest", "ender_chest", "trapped_chest", "shulker_box"]
            .into_iter()
            .map(|path| format!("minecraft:{path}"))
            .collect::<BTreeSet<_>>();
        for color in [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "light_gray",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ] {
            targeted_blocks.insert(format!("minecraft:{color}_shulker_box"));
        }
        Self { targeted_blocks }
    }
}
impl GhostHand {
    /// ListValue resolves all entries before mutating the original sorted set.
    /// The actual non-nullSafe registry adapter accepts string/number tokens.
    pub fn deserialize_targeted_blocks(
        &mut self,
        value: &Value,
        registry: &[String],
    ) -> Result<(), String> {
        let resolve = |value: &Value| -> Result<String, String> {
            let token = match value {
                Value::String(token) => token.clone(),
                Value::Number(token) => token.to_string(),
                _ => return Err("block registry entries require string or number tokens".into()),
            };
            let id = match token.find(':') {
                None => format!("minecraft:{token}"),
                Some(0) => format!("minecraft{token}"),
                Some(_) => token,
            };
            Ok(if registry.contains(&id) {
                id
            } else {
                "minecraft:air".to_owned()
            })
        };
        self.targeted_blocks = match value {
            Value::Array(values) => values.iter().map(resolve).collect::<Result<_, _>>()?,
            _ => BTreeSet::from([resolve(value)?]),
        };
        Ok(())
    }
    /// Identifier.compareTo sorts path before namespace.
    pub fn sorted_targeted_blocks(&self) -> Vec<&str> {
        let mut ids: Vec<_> = self.targeted_blocks.iter().map(String::as_str).collect();
        ids.sort_by(|a, b| {
            let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
            let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
            ap.cmp(bp).then_with(|| an.cmp(bn))
        });
        ids
    }
}
