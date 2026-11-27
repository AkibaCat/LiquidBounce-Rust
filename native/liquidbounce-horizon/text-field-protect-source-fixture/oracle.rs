use crate::text_field_protect::{Pattern, ProtectError, TextFieldProtect};
use serde_json::{Value, json};

#[test]
fn original_kotlin_protect_and_repeat() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("inputs.json")).unwrap();
    let expected: Vec<Value> = serde_json::from_str(include_str!("expected.json")).unwrap();
    for (index, (row, expected)) in rows.iter().zip(&expected).enumerate() {
        let patterns = row["patterns"]
            .as_array()
            .map(|v| {
                v.iter()
                    .map(|p| p.as_str().map(Pattern::new).transpose())
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose();
        let actual = match patterns {
            Err(_) => json!({"error":"PatternSyntaxException"}),
            Ok(patterns) => {
                let state = patterns.map_or_else(TextFieldProtect::default, |patterns| {
                    TextFieldProtect { patterns }
                });
                match state.protect(
                    row["running"].as_bool().unwrap(),
                    row["input"].as_str().unwrap(),
                    row["index"].as_i64().unwrap() as i32,
                ) {
                    Ok(text) => json!({"text":text}),
                    Err(ProtectError::NullPattern) => json!({"error":"NullPointerException"}),
                    Err(ProtectError::NegativeCount) => {
                        json!({"error":"NegativeArraySizeException"})
                    }
                }
            }
        };
        assert_eq!(&actual, expected, "row {index}: {row}");
    }
    assert_eq!(rows.len(), expected.len());
}
