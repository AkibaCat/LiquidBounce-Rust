// SPDX-License-Identifier: GPL-3.0-or-later
//! RegistryListValue + IdentifierWithRegistryAdapter.SOUND_EVENT semantics.

use serde_json::Value;
use std::collections::BTreeSet;

/// Resolve the complete new value before replacing the live selection. Gson's
/// registry adapter accepts string/number tokens; an unknown/invalid identifier
/// returns null, which the original registry-comparator TreeSet retains.
pub fn deserialize_sounds(
    value: &Value,
    registry: &[String],
) -> Result<BTreeSet<Option<String>>, &'static str> {
    let resolve = |value: &Value| {
        let text = match value {
            Value::String(text) => text.clone(),
            Value::Number(number) => number.to_string(),
            _ => return Err("registry entries must be string or number tokens"),
        };
        let name = match text.find(':') {
            None => format!("minecraft:{text}"),
            Some(0) => format!("minecraft{text}"),
            Some(_) => text,
        };
        Ok(registry.contains(&name).then_some(name))
    };
    if let Value::Array(values) = value {
        values.iter().map(resolve).collect()
    } else {
        Ok(BTreeSet::from([resolve(value)?]))
    }
}

/// IdentifierWithRegistryAdapter.write emits null for a null registry key.
pub fn serialize_sounds(sounds: &BTreeSet<Option<String>>) -> Value {
    let mut ordered: Vec<_> = sounds.iter().collect();
    // Identifier.compareTo orders path before namespace; compareBy(registry's
    // getKey) puts null first. BTreeSet's ordinary string order is irrelevant
    // to membership, but must not leak into the original config/UI order.
    ordered.sort_by(|a, b| match (a, b) {
        (Some(a), Some(b)) => {
            let (an, ap) = a.split_once(':').unwrap_or(("minecraft", a));
            let (bn, bp) = b.split_once(':').unwrap_or(("minecraft", b));
            ap.cmp(bp).then_with(|| an.cmp(bn))
        }
        _ => a.cmp(b),
    });
    Value::Array(
        ordered
            .into_iter()
            .map(|name| name.clone().map_or(Value::Null, Value::String))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn original_registry_import_keeps_unknown_null_keys_and_rejects_bad_token_types() {
        let registry = vec![
            "minecraft:entity.fishing_bobber.splash".into(),
            "custom:fish".into(),
        ];
        let selection = deserialize_sounds(
            &json!(["entity.fishing_bobber.splash", "custom:fish", "missing", 12]),
            &registry,
        )
        .unwrap();
        assert_eq!(
            selection,
            BTreeSet::from([None, Some(registry[0].clone()), Some(registry[1].clone())])
        );
        assert_eq!(
            deserialize_sounds(&json!("entity.fishing_bobber.splash"), &registry).unwrap(),
            BTreeSet::from([Some(registry[0].clone())])
        );
        assert_eq!(
            deserialize_sounds(&json!([]), &registry).unwrap(),
            BTreeSet::new()
        );
        for invalid in [
            json!(true),
            Value::Null,
            json!({}),
            json!(["custom:fish", false]),
        ] {
            assert!(deserialize_sounds(&invalid, &registry).is_err());
        }
        assert_eq!(
            serialize_sounds(&selection),
            json!([
                null,
                "minecraft:entity.fishing_bobber.splash",
                "custom:fish"
            ])
        );
    }
}
