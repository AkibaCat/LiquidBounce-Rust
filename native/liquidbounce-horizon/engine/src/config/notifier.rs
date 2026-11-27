// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::notifier::{FORMATS, SWITCHES, Settings};
#[derive(Clone, Debug, PartialEq)]
pub struct NotifierSetting(pub String, pub Value);
impl NotifierSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        if let Some(i) = SWITCHES.iter().position(|n| *n == p) {
            s.switches[i] = parse_bool(&self.1, p)?;
        } else if let Some(i) = FORMATS.iter().position(|n| *n == p) {
            s.formats[i] =
                scalar_string(&self.1).ok_or_else(|| ConfigError::new(p, "expected text"))?;
        } else if p == "UseNotification" {
            s.notification = parse_bool(&self.1, p)?;
        } else if p == "HeldItems" {
            let mut items = vec![];
            for v in self
                .1
                .as_array()
                .ok_or_else(|| ConfigError::new(p, "expected items"))?
            {
                let id = scalar_string(v)
                    .ok_or_else(|| ConfigError::new(p, "expected item identifier"))?;
                items.push(if id.contains(':') {
                    id
                } else {
                    format!("minecraft:{id}")
                });
            }
            items.sort();
            items.dedup();
            s.held_items = items;
        } else {
            return Err(ConfigError::new(p, "unknown Notifier setting"));
        }
        Ok(())
    }
}
fn key(n: &str) -> String {
    format!(
        "liquidbounce.module.notifier.{}{}",
        n[..1].to_ascii_lowercase(),
        &n[1..]
    )
}
pub fn serialize_notifier(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::Notifier, c);
    for i in 0..6 {
        values.push(t.leaf(
            SWITCHES[i],
            "BOOLEAN",
            json!(s.switches[i]),
            &key(SWITCHES[i]),
            None,
        ));
        values.push(t.leaf(
            FORMATS[i],
            "TEXT",
            json!(s.formats[i]),
            &key(FORMATS[i]),
            None,
        ));
        if i == 4 {
            let mut v = t.leaf(
                "HeldItems",
                "REGISTRY_LIST",
                json!(s.held_items),
                &key("HeldItems"),
                None,
            );
            if f == JsonFormat::Interop {
                v["registry"] = json!("item");
            }
            values.push(v);
        }
    }
    values.push(t.leaf(
        "UseNotification",
        "BOOLEAN",
        json!(s.notification),
        &key("UseNotification"),
        None,
    ));
    Ok(t.group("Notifier", "TOGGLEABLE", values))
}
pub(super) fn parse_notifier(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in SWITCHES
        .into_iter()
        .chain(FORMATS)
        .chain(["HeldItems", "UseNotification"])
    {
        let path = format!("Notifier.{name}");
        leaf(patch, root, name, &path, |v, _| {
            let setting = NotifierSetting(name.into(), v.clone());
            setting.apply(&mut Settings::default())?;
            Ok(SettingChange::Notifier(setting))
        });
    }
}
