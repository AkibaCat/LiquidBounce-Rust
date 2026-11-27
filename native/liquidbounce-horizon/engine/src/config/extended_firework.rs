// SPDX-License-Identifier: GPL-3.0-or-later
//! Both original ExtendedFirework modes and the complete Vec3d value.
use super::*;
use liquidbounce_behaviors::extended_firework::{ExtendedFirework, Mode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExtendedFireworkSetting {
    Mode(Mode),
    VelocityMultiplier([f64; 3]),
}
impl ExtendedFireworkSetting {
    pub fn apply(self, module: &mut ExtendedFirework) {
        match self {
            Self::Mode(value) => module.mode = value,
            Self::VelocityMultiplier(value) => module.custom = value,
        }
    }
}
pub fn serialize_extended_firework(
    state: &ExtendedFirework,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let path = "ExtendedFirework.Mode.Custom.VelocityMultiplier";
    if state.custom.iter().any(|component| !component.is_finite()) {
        return Err(ConfigError::new(
            path,
            "non-finite double cannot be serialized by the original Gson",
        ));
    }
    let tree = Tree { format, describe };
    let mut choices = Map::new();
    choices.insert(
        "Normal".into(),
        tree.group("Normal", "CONFIGURABLE", vec![]),
    );
    choices.insert(
        "Custom".into(),
        tree.group(
            "Custom",
            "CONFIGURABLE",
            vec![tree.leaf(
                "VelocityMultiplier",
                "VECTOR3_D",
                json!({"x":state.custom[0],"y":state.custom[1],"z":state.custom[2]}),
                "liquidbounce.module.extendedFirework.mode.custom.velocityMultiplier",
                None,
            )],
        ),
    );
    let mut values = tree.common(ModuleKind::ExtendedFirework, common);
    values.push(tree.mode(state.mode.tag(), choices));
    Ok(tree.group("ExtendedFirework", "TOGGLEABLE", values))
}

// Java Double.parseDouble accepts a hexadecimal significand. Retain 53 bits
// and a guard/sticky bit, then round once, including the subnormal boundary.
fn hex_double(text: &str) -> Option<f64> {
    let (negative, text) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let text = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))?;
    let (mantissa, exponent) = text.split_once(['p', 'P'])?;
    let exponent_negative = exponent.starts_with('-');
    let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
    if digits.is_empty() || !digits.bytes().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    let exponent = digits.bytes().fold(0_i64, |n, digit| {
        n.saturating_mul(10).saturating_add(i64::from(digit - b'0'))
    });
    let exponent = if exponent_negative {
        -exponent
    } else {
        exponent
    };
    let mut bits = Vec::new();
    let mut fractional = None;
    for digit in mantissa.chars() {
        if digit == '.' {
            if fractional.is_some() {
                return None;
            }
            fractional = Some(0_i64);
            continue;
        }
        let value = digit.to_digit(16)?;
        for shift in (0..4).rev() {
            bits.push((value >> shift) & 1 != 0);
        }
        if let Some(count) = fractional.as_mut() {
            *count += 1;
        }
    }
    if bits.is_empty() {
        return None;
    }
    let sign = u64::from(negative) << 63;
    let Some(first) = bits.iter().position(|value| *value) else {
        return Some(f64::from_bits(sign));
    };
    let bits = &bits[first..];
    let binary_exponent = exponent.saturating_sub(4 * fractional.unwrap_or(0));
    let highest = binary_exponent.saturating_add(bits.len() as i64 - 1);
    if highest > 1023 {
        return Some(f64::from_bits(sign | (0x7ff_u64 << 52)));
    }
    if highest < -1075 {
        return Some(f64::from_bits(sign));
    }
    let lsb = (highest - 52).max(-1074);
    let retained = bits.len() as i64 - lsb.saturating_sub(binary_exponent);
    let mut significant = 0_u64;
    for index in 0..retained.max(0) {
        significant =
            (significant << 1) | u64::from(bits.get(index as usize).copied().unwrap_or(false));
    }
    let guard = retained >= 0 && bits.get(retained as usize).copied().unwrap_or(false);
    let sticky = retained >= 0 && bits.iter().skip(retained as usize + 1).any(|value| *value);
    if guard && (sticky || significant & 1 != 0) {
        significant += 1;
    }
    if significant == 0 {
        return Some(f64::from_bits(sign));
    }
    if lsb == -1074 && significant < 1_u64 << 52 {
        return Some(f64::from_bits(sign | significant));
    }
    let shift = 63 - significant.leading_zeros();
    let unbiased = lsb + i64::from(shift);
    if unbiased > 1023 {
        return Some(f64::from_bits(sign | (0x7ff_u64 << 52)));
    }
    let fraction = if shift > 52 {
        significant >> (shift - 52)
    } else {
        significant << (52 - shift)
    };
    Some(f64::from_bits(
        sign | (((unbiased + 1023) as u64) << 52) | (fraction & ((1_u64 << 52) - 1)),
    ))
}
fn component(value: &Value, path: &str) -> Result<f64, ConfigError> {
    let text = match value {
        Value::String(value) => value.trim_matches(|c: char| c <= '\u{20}').to_owned(),
        Value::Number(value) => value.to_string(),
        _ => return Err(ConfigError::new(path, "expected double or numeric string")),
    };
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(&text);
    if unsigned == "NaN" {
        return Ok(f64::NAN);
    }
    if unsigned == "Infinity" {
        return Ok(if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    let text = text.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(&text);
    let parsed = if text.contains(['x', 'X']) {
        hex_double(text)
    } else {
        // Rust also accepts inf/nan spellings that Java rejects.
        if text
            .bytes()
            .all(|c| c.is_ascii_digit() || matches!(c, b'+' | b'-' | b'.' | b'e' | b'E'))
        {
            text.parse().ok()
        } else {
            None
        }
    };
    parsed.ok_or_else(|| ConfigError::new(path, "invalid double"))
}
fn vector(value: &Value, path: &str) -> Result<[f64; 3], ConfigError> {
    let object = value
        .as_object()
        .ok_or_else(|| ConfigError::new(path, "expected vector object"))?;
    let mut output = [0.0; 3];
    for (index, name) in ["x", "y", "z"].into_iter().enumerate() {
        if let Some(value) = object.get(name) {
            output[index] = component(value, path)?;
        }
    }
    Ok(output)
}
pub(super) fn parse_extended_firework(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    let Some(mode) = root.get("Mode") else {
        return;
    };
    let path = "ExtendedFirework.Mode";
    match mode
        .get("active")
        .and_then(scalar_string)
        .and_then(|tag| Mode::from_tag(&tag))
    {
        Some(value) => patch.changes.push(SettingChange::ExtendedFirework(
            ExtendedFireworkSetting::Mode(value),
        )),
        None => patch.issues.push(ConfigError::new(
            path,
            "missing or unknown case-sensitive mode",
        )),
    }
    let Some(choices) = mode.get("choices").and_then(Value::as_object) else {
        patch
            .issues
            .push(ConfigError::new(path, "expected choices object"));
        return;
    };
    for name in ["Normal", "Custom"] {
        let choice_path = format!("{path}.{name}");
        match choices.get(name) {
            Some(choice) => match nodes(choice, name, &choice_path) {
                Ok(choice) if name == "Custom" => leaf(
                    patch,
                    &choice,
                    "VelocityMultiplier",
                    &format!("{choice_path}.VelocityMultiplier"),
                    |value, path| {
                        vector(value, path).map(|value| {
                            SettingChange::ExtendedFirework(
                                ExtendedFireworkSetting::VelocityMultiplier(value),
                            )
                        })
                    },
                ),
                Ok(_) => {}
                Err(error) => patch.issues.push(error),
            },
            None => patch
                .issues
                .push(ConfigError::new(&choice_path, "choice not found")),
        }
    }
    if let Err(error) = nodes(mode, "Mode", path) {
        patch.issues.push(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_modes_keep_inactive_custom_vector_and_original_metadata() {
        let common = CommonSettings {
            enabled: true,
            hidden: false,
            key_bind: json!({"key":"key.keyboard.unknown","action":"Toggle"}),
        };
        for mode in [Mode::Normal, Mode::Custom] {
            for custom in [
                [0.1, 1.65, 0.5],
                [-3.0, 0.0, 50.125],
                [f64::MAX, f64::MIN_POSITIVE, -0.0],
            ] {
                for format in [JsonFormat::Interop, JsonFormat::File] {
                    let source = ExtendedFirework { mode, custom };
                    let value =
                        serialize_extended_firework(&source, &common, format, &str::to_owned)
                            .unwrap();
                    let patch = parse_settings(ModuleKind::ExtendedFirework, &value).unwrap();
                    assert!(patch.issues.is_empty(), "{:?}", patch.issues);
                    let mut restored = ExtendedFirework::default();
                    for change in patch.changes {
                        if let SettingChange::ExtendedFirework(value) = change {
                            value.apply(&mut restored);
                        }
                    }
                    assert_eq!(restored, source);
                    if format == JsonFormat::Interop {
                        assert_eq!(
                            value["value"][2]["choices"]["Custom"]["value"][0]["valueType"],
                            "VECTOR3_D"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn vector_defaults_missing_components_and_rejects_invalid_whole_leaf() {
        assert_eq!(
            vector(&json!({"x":"0x1.8p2","ignored":[false]}), "v").unwrap(),
            [6.0, 0.0, 0.0]
        );
        for value in [
            json!(null),
            json!([1, 2, 3]),
            json!({"x":1,"y":false}),
            json!({"x":"nan"}),
            json!({"z":"Infinityf"}),
        ] {
            assert!(vector(&value, "v").is_err());
        }
        for (text, bits) in [
            ("0x1p-1074", 1),
            ("0x1p-1075", 0),
            ("0x1.8p-1074", 2),
            ("-0x0p0", 1_u64 << 63),
            ("0x1.fffffffffffffp1023", f64::MAX.to_bits()),
        ] {
            assert_eq!(hex_double(text).unwrap().to_bits(), bits, "{text}");
        }
    }
}
