// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::packet_logger::Settings;
#[derive(Clone, Debug, PartialEq)]
pub struct PacketLoggerSetting(pub String, pub Value);
impl PacketLoggerSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        match p {
            "Filter" => {
                s.whitelist = match scalar_string(v).as_deref() {
                    Some("Whitelist") => true,
                    Some("Blacklist") => false,
                    _ => return Err(ConfigError::new(p, "expected Whitelist or Blacklist")),
                }
            }
            "C2SPackets" | "S2CPackets" => {
                let a = v
                    .as_array()
                    .ok_or_else(|| ConfigError::new(p, "expected packet list"))?;
                let mut items = Vec::new();
                for v in a {
                    let mut id = scalar_string(v)
                        .ok_or_else(|| ConfigError::new(p, "expected packet identifier"))?;
                    if !id.contains(':') {
                        id = format!("minecraft:{id}");
                    }
                    items.push(id);
                }
                items.sort();
                items.dedup();
                if p == "C2SPackets" {
                    s.c2s = items
                } else {
                    s.s2c = items
                }
            }
            "ShowFieldType" => s.show_type = parse_bool(v, p)?,
            "OutputTarget" => {
                let a = v
                    .as_array()
                    .ok_or_else(|| ConfigError::new(p, "expected output targets"))?;
                if a.is_empty() {
                    return Err(ConfigError::new(p, "at least one output target required"));
                }
                let mut chat = false;
                let mut file = false;
                for v in a {
                    match scalar_string(v).as_deref() {
                        Some("Chat") => chat = true,
                        Some("File") => file = true,
                        _ => return Err(ConfigError::new(p, "unknown output target")),
                    }
                }
                s.chat = chat;
                s.file = file;
            }
            _ => return Err(ConfigError::new(p, "unknown PacketLogger setting")),
        };
        Ok(())
    }
}
pub fn serialize_packet_logger(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::PacketLogger, c);
    let k = |p: &str| format!("liquidbounce.module.packetLogger.{p}");
    let mut filter = t.leaf(
        "Filter",
        "CHOOSE",
        json!(if s.whitelist {
            "Whitelist"
        } else {
            "Blacklist"
        }),
        &k("filter"),
        None,
    );
    if f == JsonFormat::Interop {
        filter["choices"] = json!(["Whitelist", "Blacklist"]);
    }
    values.push(filter);
    for (name, key, kind, items) in [
        ("C2SPackets", "c2SPackets", "c2s_packet", &s.c2s),
        ("S2CPackets", "s2CPackets", "s2c_packet", &s.s2c),
    ] {
        let mut v = t.leaf(name, "REGISTRY_LIST", json!(items), &k(key), None);
        if f == JsonFormat::Interop {
            v["registry"] = json!(kind);
        }
        values.push(v);
    }
    values.push(t.leaf(
        "ShowFieldType",
        "BOOLEAN",
        json!(s.show_type),
        &k("showFieldType"),
        None,
    ));
    let mut output = t.leaf(
        "OutputTarget",
        "MULTI_CHOOSE",
        json!(
            [
                if s.chat { Some("Chat") } else { None },
                if s.file { Some("File") } else { None }
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
        ),
        &k("outputTarget"),
        None,
    );
    if f == JsonFormat::Interop {
        output["choices"] = json!(["Chat", "File"]);
        output["canBeNone"] = json!(false);
    }
    values.push(output);
    Ok(t.group("PacketLogger", "TOGGLEABLE", values))
}
pub(super) fn parse_packet_logger(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in [
        "Filter",
        "C2SPackets",
        "S2CPackets",
        "ShowFieldType",
        "OutputTarget",
    ] {
        leaf(p, root, name, &format!("PacketLogger.{name}"), |v, _| {
            let setting = PacketLoggerSetting(name.into(), v.clone());
            setting.apply(&mut Settings::default())?;
            Ok(SettingChange::PacketLogger(setting))
        });
    }
}
