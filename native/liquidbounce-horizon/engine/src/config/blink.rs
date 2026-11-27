// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::blink_runtime::BlinkService;
use liquidbounce_behaviors::blink::Settings;

#[derive(Clone, Debug, PartialEq)]
pub struct BlinkSetting(pub String, pub Value);
impl BlinkSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        match p {
            "Directions" => s.directions = parse_multi_mask(v, p, &["Incoming", "Outgoing"])?,
            "Dummy" => s.dummy = parse_bool(v, p)?,
            "Ambush" => s.ambush = parse_bool(v, p)?,
            "AutoDisable" => s.auto_disable = parse_bool(v, p)?,
            "AutoReset.Enabled" => s.auto_reset = parse_bool(v, p)?,
            "AutoReset.ResetAfter" => s.reset_after = parse_int(v, p)?,
            "AutoReset.ResetAction" => {
                s.reset_action = match scalar_string(v).as_deref() {
                    Some(t) if t.eq_ignore_ascii_case("Reset") => 0,
                    Some(t) if t.eq_ignore_ascii_case("Blink") => 1,
                    _ => return Err(ConfigError::new(p, "expected Reset or Blink")),
                }
            }
            _ => return Err(ConfigError::new(p, "unknown Blink setting")),
        };
        Ok(())
    }
}
pub fn serialize_blink(
    s: &BlinkService,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = s.settings();
    let mut values = t.common(ModuleKind::Blink, c);
    let k = |p: &str| format!("liquidbounce.module.blink.{p}");
    let mut dirs = t.multi(
        "Directions",
        s.directions,
        &["Incoming", "Outgoing"],
        &k("directions"),
    );
    if f == JsonFormat::Interop {
        dirs["canBeNone"] = json!(false);
    }
    values.push(dirs);
    for (name, p, v) in [
        ("Dummy", "dummy", s.dummy),
        ("Ambush", "ambush", s.ambush),
        ("AutoDisable", "autoDisable", s.auto_disable),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(v), &k(p), None));
    }
    let mut action = t.leaf(
        "ResetAction",
        "CHOOSE",
        json!(if s.reset_action == 0 {
            "Reset"
        } else {
            "Blink"
        }),
        &k("autoReset.resetAction"),
        None,
    );
    if f == JsonFormat::Interop {
        action["choices"] = json!(["Reset", "Blink"]);
    }
    values.push(t.group(
        "AutoReset",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.auto_reset),
                &k("autoReset.enabled"),
                None,
            ),
            t.ranged(
                "ResetAfter",
                "INT",
                json!(s.reset_after),
                json!({"from":1,"to":1000}),
                "",
                &k("autoReset.resetAfter"),
            ),
            action,
        ],
    ));
    Ok(t.group("Blink", "TOGGLEABLE", values))
}
pub(super) fn parse_blink(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
        leaf(p, n, name, &format!("Blink.{path}"), |v, _| {
            let s = BlinkSetting(path.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::Blink(s))
        });
    }
    for name in ["Directions", "Dummy", "Ambush", "AutoDisable"] {
        add(p, root, name, name);
    }
    if let Some(v) = root.get("AutoReset") {
        match nodes(v, "AutoReset", "Blink.AutoReset") {
            Ok(n) => {
                for name in ["Enabled", "ResetAfter", "ResetAction"] {
                    add(p, &n, name, &format!("AutoReset.{name}"));
                }
            }
            Err(e) => p.issues.push(e),
        }
    }
}
