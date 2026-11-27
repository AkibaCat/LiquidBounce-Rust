use std::{collections::BTreeMap,fmt};
use serde_json::{Value,Map,Number,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum ModuleKind{TargetStrafe}
impl ModuleKind{fn name(self)->&'static str{"TargetStrafe"}fn key(self)->&'static str{"liquidbounce.module.targetStrafe"}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum JsonFormat {
    Interop,
    File,
}
#[derive(Clone,Debug,PartialEq)]
pub struct CommonSettings {
    pub enabled: bool,
    pub hidden: bool,
    pub key_bind: Value,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ConfigError {
    pub path: String,
    pub message: String,
}
impl ConfigError {
    fn new(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: path.to_owned(),
            message: message.into(),
        }
    }
}
impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}
struct Tree<'a> {
    format: JsonFormat,
    describe: &'a dyn Fn(&str) -> String,
}
impl Tree<'_> {
    fn group(&self, name: &str, kind: &str, children: Vec<Value>) -> Value {
        let mut object = json!({"name":name,"value":children});
        if self.format == JsonFormat::Interop {
            object["valueType"] = json!(kind);
        }
        object
    }

    fn leaf(
        &self,
        name: &str,
        kind: &str,
        value: Value,
        key: &str,
        description_key: Option<&str>,
    ) -> Value {
        let mut object = json!({"name":name,"value":value});
        if self.format == JsonFormat::Interop {
            object["valueType"] = json!(kind);
            object["key"] = json!(key);
            let fallback = format!("{key}.description");
            object["description"] = json!((self.describe)(description_key.unwrap_or(&fallback)));
        }
        object
    }

    fn ranged(
        &self,
        name: &str,
        kind: &str,
        value: Value,
        range: Value,
        suffix: &str,
        key: &str,
    ) -> Value {
        let mut leaf = self.leaf(name, kind, value, key, None);
        if self.format == JsonFormat::Interop {
            leaf["range"] = range;
            leaf["suffix"] = json!(suffix);
        }
        leaf
    }

    fn mode(&self, active: &str, choices: Map<String, Value>) -> Value {
        let mut mode = self.group("Mode", "CHOICE", vec![]);
        mode["active"] = json!(active);
        mode["choices"] = Value::Object(choices);
        mode
    }

    fn multi(&self, name: &str, mask: u32, choices: &[&str], key: &str) -> Value {
        let selected: Vec<_> = choices
            .iter()
            .enumerate()
            .filter_map(|(index, tag)| (mask & (1 << index) != 0).then_some(*tag))
            .collect();
        let mut value = self.leaf(name, "MULTI_CHOOSE", json!(selected), key, None);
        if self.format == JsonFormat::Interop {
            value["choices"] = json!(choices);
            value["canBeNone"] = json!(true);
            value["isOrderSensitive"] = json!(false);
        }
        value
    }

    fn common(&self, module: ModuleKind, common: &CommonSettings) -> Vec<Value> {
        let key = module.key();
        let mut values = Vec::new();
        // ClientModule marks Enabled notAnOption, excluded by INTEROP only.
        if self.format == JsonFormat::File {
            values.push(self.leaf(
                "Enabled",
                "BOOLEAN",
                json!(common.enabled),
                &format!("{key}.enabled"),
                None,
            ));
        }
        values.push(self.leaf(
            "Bind",
            "BIND",
            common.key_bind.clone(),
            &format!("{key}.bind"),
            Some("liquidbounce.common.bind.description"),
        ));
        values.push(self.leaf(
            "Hidden",
            "BOOLEAN",
            json!(common.hidden),
            &format!("{key}.hidden"),
            Some("liquidbounce.common.hidden.description"),
        ));
        values
    }
}
fn float_json(value: f32, path: &str) -> Result<Value, ConfigError> {
    if !value.is_finite() {
        return Err(ConfigError::new(
            path,
            "non-finite float cannot be serialized by the original Gson",
        ));
    }
    // Java Float.toString emits shortest f32 text; promoting directly to f64
    // would expose binary rounding (0.1 -> 0.10000000149011612) in the UI.
    let mut text = value.to_string();
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    text.parse::<Number>()
        .map(Value::Number)
        .map_err(|error| ConfigError::new(path, error.to_string()))
}
fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(_) | Value::Bool(_) => Some(value.to_string()),
        _ => None,
    }
}
fn nodes<'a>(json: &'a Value, expected: &str, path: &str) -> Result<Nodes<'a>, ConfigError> {
    let object = json
        .as_object()
        .ok_or_else(|| ConfigError::new(path, "expected a group object"))?;
    let name = object
        .get("name")
        .and_then(scalar_string)
        .ok_or_else(|| ConfigError::new(path, "missing group name"))?;
    if name != expected {
        return Err(ConfigError::new(
            path,
            format!("config name {name:?} does not match {expected:?}"),
        ));
    }
    let values = object
        .get("value")
        .and_then(Value::as_array)
        .ok_or_else(|| ConfigError::new(path, "expected value array"))?;
    let mut by_name = BTreeMap::new();
    for node in values {
        let name = node
            .as_object()
            .and_then(|node| node.get("name"))
            .and_then(scalar_string)
            .ok_or_else(|| ConfigError::new(path, "stored value must be an object with name"))?;
        // The original queues duplicates and each declared value consumes once.
        by_name.entry(name).or_insert(node);
    }
    Ok(by_name)
}
fn value_of<'a>(node: &'a Value, path: &str) -> Result<&'a Value, ConfigError> {
    node.get("value")
        .filter(|value| !value.is_null())
        .ok_or_else(|| ConfigError::new(path, "missing value"))
}
fn parse_bool(value: &Value, path: &str) -> Result<bool, ConfigError> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::String(value) => Ok(value.eq_ignore_ascii_case("true")),
        _ => Err(ConfigError::new(path, "expected boolean or boolean string")),
    }
}
fn parse_float(value: &Value, path: &str) -> Result<f32, ConfigError> {
    let text = match value {
        Value::Number(value) => value.to_string(),
        Value::String(value) => value
            .trim_matches(|character: char| character <= '\u{20}')
            .to_owned(),
        _ => return Err(ConfigError::new(path, "expected float or numeric string")),
    };
    let text = text.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(&text);
    text.parse()
        .map_err(|_| ConfigError::new(path, "invalid float"))
}
fn parse_int(value: &Value, path: &str) -> Result<i32, ConfigError> {
    match value {
        Value::String(value) => value
            .parse()
            .map_err(|_| ConfigError::new(path, "invalid integer string")),
        Value::Number(value) => {
            decimal_int(&value.to_string()).ok_or_else(|| ConfigError::new(path, "invalid integer"))
        }
        _ => Err(ConfigError::new(path, "expected integer or numeric string")),
    }
}
fn decimal_int(text: &str) -> Option<i32> {
    let (negative, text) = text
        .strip_prefix('-')
        .map_or((false, text), |text| (true, text));
    let (mantissa, exponent) = text
        .split_once(['e', 'E'])
        .map_or(Some((text, 0_i64)), |(mantissa, exp)| {
            exp.parse::<i64>().ok().map(|exp| (mantissa, exp))
        })?;
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits: Vec<_> = whole.bytes().chain(fraction.bytes()).collect();
    if digits.is_empty() || digits.iter().any(|digit| !digit.is_ascii_digit()) {
        return None;
    }
    let integer_digits = (digits.len() as i64)
        .saturating_add(exponent)
        .saturating_sub(fraction.len() as i64);
    if integer_digits <= 0 {
        return Some(0);
    }
    let mut value = 0_u32;
    for digit in digits
        .iter()
        .take(integer_digits.min(digits.len() as i64) as usize)
    {
        value = value
            .wrapping_mul(10)
            .wrapping_add(u32::from(*digit - b'0'));
    }
    let zeros = integer_digits.saturating_sub(digits.len() as i64);
    if zeros >= 32 {
        value = 0;
    } else {
        for _ in 0..zeros {
            value = value.wrapping_mul(10);
        }
    }
    Some(if negative {
        value.wrapping_neg() as i32
    } else {
        value as i32
    })
}
fn parse_multi_mask(value: &Value, path: &str, choices: &[&str]) -> Result<u32, ConfigError> {
    let mut mask = 0;
    let mut include = |value: &Value| -> Result<(), ConfigError> {
        if !value.is_string() && !value.is_number() && !value.is_boolean() {
            return Err(ConfigError::new(
                path,
                "multichoice entries must be JSON primitives",
            ));
        }
        if let Some(tag) = value.as_str()
            && let Some(index) = choices
                .iter()
                .position(|choice| choice.eq_ignore_ascii_case(tag))
        {
            mask |= 1 << index;
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
    Ok(mask)
}
#[derive(Clone,Debug,PartialEq)]
pub struct SettingsPatch {
    pub module: ModuleKind,
    /// Declaration order, not input JSON order. Do not partition/reorder these.
    pub changes: Vec<SettingChange>,
    /// Original ConfigSystem catches these errors per leaf/group and continues.
    pub issues: Vec<ConfigError>,
}
type Nodes<'a>=BTreeMap<String,&'a Value>;
#[derive(Clone,Debug,PartialEq)]pub enum SettingChange{Enabled(bool),Bind(Value),Hidden(bool),TargetStrafe(TargetStrafeSetting)}
fn leaf(
    patch: &mut SettingsPatch,
    by_name: &Nodes<'_>,
    name: &str,
    path: &str,
    parse: impl FnOnce(&Value, &str) -> Result<SettingChange, ConfigError>,
) {
    if let Some(node) = by_name.get(name) {
        match value_of(node, path).and_then(|value| parse(value, path)) {
            Ok(change) => patch.changes.push(change),
            Err(error) => patch.issues.push(error),
        }
    }
}
pub fn parse_settings(module:ModuleKind,json:&Value)->Result<SettingsPatch,ConfigError>{
    let root = nodes(json,module.name(),module.name())?;
    let mut patch = SettingsPatch {
        module,
        changes: Vec::new(),
        issues: Vec::new(),
    };
    leaf(&mut patch, &root, "Enabled", "Enabled", |value, path| {
        parse_bool(value, path).map(SettingChange::Enabled)
    });
    leaf(&mut patch, &root, "Bind", "Bind", |value, path| {
        if value.is_array() {
            return Err(ConfigError::new(
                path,
                "expected InputBind object or legacy primitive",
            ));
        }
        // Preserve this request for the bridge's original key-name adapter.
        Ok(SettingChange::Bind(value.clone()))
    });
    leaf(&mut patch, &root, "Hidden", "Hidden", |value, path| {
        parse_bool(value, path).map(SettingChange::Hidden)
    });
target_strafe_config::parse_target_strafe(&mut patch,&root);Ok(patch)}
#[path="target-strafe-config.rs"]mod target_strafe_config;pub use target_strafe_config::*;
