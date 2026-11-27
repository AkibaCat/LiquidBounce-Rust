// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::eagle::{CONDITIONS, Settings};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EagleSetting {
    Distance([f32; 2]),
    Conditional(bool),
    Conditions(u32),
    Pitch([f32; 2]),
}
impl EagleSetting {
    pub fn apply(self, s: &mut Settings) {
        match self {
            Self::Distance(v) => s.distance = v,
            Self::Conditional(v) => s.conditional = v,
            Self::Conditions(v) => s.conditions = v,
            Self::Pitch(v) => s.pitch = v,
        }
    }
}
pub fn serialize_eagle(
    s: &Settings,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut values = t.common(ModuleKind::Eagle, c);
    let key = |s: &str| format!("liquidbounce.module.eagle.{s}");
    values.push(t.ranged(
        "EdgeDistance",
        "FLOAT_RANGE",
        float_range(s.distance[0], s.distance[1], "Eagle.EdgeDistance")?,
        json!({"from":0.01,"to":1.3}),
        "",
        &key("edgeDistance"),
    ));
    values.push(t.group(
        "Conditional",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.conditional),
                &key("conditional"),
                None,
            ),
            t.multi(
                "Conditions",
                s.conditions,
                &CONDITIONS,
                &key("conditional.conditions"),
            ),
            t.ranged(
                "Pitch",
                "FLOAT_RANGE",
                float_range(s.pitch[0], s.pitch[1], "Eagle.Conditional.Pitch")?,
                json!({"from":-90,"to":90}),
                "",
                &key("conditional.pitch"),
            ),
        ],
    ));
    Ok(t.group("Eagle", "TOGGLEABLE", values))
}
pub(super) fn parse_eagle(p: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(p, root, "EdgeDistance", "Eagle.EdgeDistance", |v, k| {
        parse_float_pair(v, k).map(|v| SettingChange::Eagle(EagleSetting::Distance(v)))
    });
    if let Some(v) = root.get("Conditional") {
        match nodes(v, "Conditional", "Eagle.Conditional") {
            Ok(n) => {
                leaf(p, &n, "Enabled", "Eagle.Conditional.Enabled", |v, k| {
                    parse_bool(v, k).map(|v| SettingChange::Eagle(EagleSetting::Conditional(v)))
                });
                leaf(p, &n, "Pitch", "Eagle.Conditional.Pitch", |v, k| {
                    parse_float_pair(v, k).map(|v| SettingChange::Eagle(EagleSetting::Pitch(v)))
                });
                leaf(
                    p,
                    &n,
                    "Conditions",
                    "Eagle.Conditional.Conditions",
                    |v, k| {
                        let entries = match v {
                            Value::Array(a) => a.clone(),
                            _ => vec![v.clone()],
                        };
                        let mut mask = 0;
                        for e in entries {
                            let tag = scalar_string(&e)
                                .ok_or_else(|| ConfigError::new(k, "expected condition tag"))?;
                            if let Some(i) =
                                CONDITIONS.iter().position(|s| s.eq_ignore_ascii_case(&tag))
                            {
                                mask |= 1 << i;
                            }
                        }
                        Ok(SettingChange::Eagle(EagleSetting::Conditions(mask)))
                    },
                );
            }
            Err(e) => p.issues.push(e),
        }
    }
}
