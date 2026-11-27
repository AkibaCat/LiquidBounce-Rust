// SPDX-License-Identifier: GPL-3.0-or-later
// FriendManager / ListValue / ConfigSystem at LiquidBounce ae0eb396.
//! Persistent, case-sensitive FriendManager state shared by target consumers.
use liquidbounce_behaviors::targeting::FriendBook;
use serde_json::{Value, json};
use std::path::Path;

pub fn file(book: &FriendBook) -> Value {
    let mut entries = book.names.iter().collect::<Vec<_>>();
    // Java TreeSet<Friend> compares UTF-16 String code units.
    entries.sort_by(|(left, _), (right, _)| left.encode_utf16().cmp(right.encode_utf16()));
    let entries = entries.into_iter().map(|(name, alias)| {
        let mut value = json!({"name":name});
        if let Some(alias) = alias { value["alias"] = json!(alias); }
        value
    }).collect::<Vec<_>>();
    json!({"name":"Friends","value":[
        {"name":"Friends","value":entries},
        {"name":"CancelAttack","value":book.cancel_attack}
    ]})
}

fn gson_string(value: &Value) -> Result<Option<String>, String> {
    match value {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value.clone())),
        Value::Bool(_) | Value::Number(_) => Ok(Some(value.to_string())),
        _ => Err("Expected Gson string primitive".into()),
    }
}

fn decode_friend(value: &Value) -> Result<Option<(Option<String>, Option<String>)>, String> {
    if value.is_null() { return Ok(None); }
    let object = value.as_object().ok_or("Friend must be an object")?;
    let name = gson_string(object.get("name").unwrap_or(&Value::Null))?;
    let alias = gson_string(object.get("alias").unwrap_or(&Value::Null))?;
    Ok(Some((name, alias)))
}

/// ListValue decodes every array item before clearing the existing TreeSet.
/// A TreeSet insertion failure after decoding retains the inserted prefix.
fn set_friends(book: &mut FriendBook, value: &Value) -> Result<(), String> {
    let decoded = if let Value::Array(values) = value {
        values.iter().map(decode_friend).collect::<Result<Vec<_>,_>>()?
    } else {
        vec![decode_friend(value)?]
    };
    book.names.clear();
    for entry in decoded {
        let (name, alias) = entry.ok_or("TreeSet cannot contain null Friend")?;
        let name = name.ok_or("Friend.name cannot be null when compared by TreeSet")?;
        book.add(name, alias);
    }
    Ok(())
}

/// Root errors reject the document. Independent leaf failures are logged and
/// later leaves still apply, matching ConfigSystem.deserializeValue.
pub fn apply(book: &mut FriendBook, value: &Value) -> Result<Vec<String>, String> {
    if value["name"] != "Friends" { return Err("Friends config name mismatch".into()); }
    let values = value["value"].as_array().ok_or("Friends.value must be an array")?;
    for value in values {
        if !value.is_object() || !value["name"].is_string() {
            return Err("Friends config entries require names".into());
        }
    }
    let mut issues = Vec::new();
    if let Some(node) = values.iter().find(|node| node["name"] == "Friends") {
        if let Err(error) = set_friends(book, node.get("value").unwrap_or(&Value::Null)) {
            issues.push(format!("Friends.Friends: {error}"));
        }
    }
    if let Some(node) = values.iter().find(|node| node["name"] == "CancelAttack") {
        match node.get("value") {
            Some(Value::Bool(value)) => book.cancel_attack = *value,
            Some(Value::String(value)) => book.cancel_attack = value.eq_ignore_ascii_case("true"),
            _ => issues.push("Friends.CancelAttack: Expected Gson Boolean".into()),
        }
    }
    Ok(issues)
}

pub fn load(book: &mut FriendBook, path: &Path) -> Result<(), String> {
    if !path.is_file() { return Ok(()); }
    let value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    for issue in apply(book, &value)? { eprintln!("[LiquidBounce] friend config leaf skipped: {issue}"); }
    Ok(())
}

pub fn save(book: &FriendBook, path: &Path) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(&file(book)).map_err(|e| e.to_string())?;
    std::fs::write(&temporary, bytes).and_then(|()| std::fs::rename(&temporary, path))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn friends_file_uses_exact_names_first_duplicate_and_utf16_order() {
        let mut book = FriendBook::default();
        let issues = apply(&mut book, &json!({"name":"Friends","value":[
            {"name":"CancelAttack","value":true},
            {"name":"Friends","value":[{"name":"z","alias":"中文"},{"name":"z","alias":"ignored"},
                {"name":"Z"},{"name":"\u{e000}"},{"name":"\u{10000}"}]},
            {"name":"Friends","value":[]}
        ]})).unwrap();
        assert!(issues.is_empty());
        assert_eq!(book.names.len(), 4);
        assert!(book.cancel_attack);
        assert_eq!(book.names["z"].as_deref(), Some("中文"));
        let saved = file(&book);
        assert_eq!(saved["value"][0]["value"].as_array().unwrap().iter().map(|v|v["name"].as_str().unwrap()).collect::<Vec<_>>(),
            ["Z","z","\u{10000}","\u{e000}"]);
        assert!(saved["value"][0]["value"][0].get("alias").is_none());
    }
    #[test]
    fn list_decoding_failure_keeps_old_set_but_tree_insertion_failure_keeps_new_prefix() {
        let mut book = FriendBook::default();
        book.add("old".into(), None);
        assert!(set_friends(&mut book, &json!([{"name":"new"},42])).is_err());
        assert!(book.contains("old"));
        assert!(set_friends(&mut book, &json!([{"name":"new"},null,{"name":"late"}])).is_err());
        assert_eq!(book.names.keys().map(String::as_str).collect::<Vec<_>>(),["new"]);
        assert!(set_friends(&mut book, &json!([{"alias":"no name"}])).is_err());
        assert!(book.names.is_empty());
    }
    #[test]
    fn singleton_primitives_and_later_boolean_leaf_match_gson() {
        let mut book = FriendBook::default();
        apply(&mut book, &json!({"name":"Friends","value":[
            {"name":"Friends","value":{"name":123,"alias":false}},
            {"name":"CancelAttack","value":"TRUE"}
        ]})).unwrap();
        assert_eq!(book.names["123"].as_deref(),Some("false"));
        assert!(book.cancel_attack);
        let issues=apply(&mut book,&json!({"name":"Friends","value":[
            {"name":"Friends","value":null},{"name":"CancelAttack","value":false}
        ]})).unwrap();
        assert_eq!(issues.len(),1);
        assert!(book.names.is_empty());
        assert!(!book.cancel_attack);
        let before=book.clone();
        assert!(apply(&mut book,&json!({"name":"Other","value":[]})).is_err());
        assert_eq!(book,before);
    }
}
