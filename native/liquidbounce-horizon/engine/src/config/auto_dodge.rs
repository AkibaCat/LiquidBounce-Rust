// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::auto_dodge::Settings;
#[derive(Clone, Debug, PartialEq)]
pub struct AutoDodgeSetting(pub String, pub Value);
impl AutoDodgeSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        match p {
            "AllowRotationChange.Enabled" => s.rotation = parse_bool(v, p)?,
            "AllowRotationChange.AllowJump" => s.jump = parse_bool(v, p)?,
            "AllowTimer.Enabled" => s.timer = parse_bool(v, p)?,
            "AllowTimer.TimerSpeed" => s.speed = parse_float(v, p)?,
            "Ignore" => {
                s.ignore = parse_multi_mask(v, p, &["OpenInventory", "UsingItem", "UsingScaffold"])?
            }
            _ => return Err(ConfigError::new(p, "unknown AutoDodge setting")),
        };
        Ok(())
    }
}
pub fn serialize_auto_dodge(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let k = |p: &str| format!("liquidbounce.module.autoDodge.{p}");
    let mut values = t.common(ModuleKind::AutoDodge, c);
    values.push(t.multi(
        "Ignore",
        s.ignore,
        &["OpenInventory", "UsingItem", "UsingScaffold"],
        &k("ignore"),
    ));
    values.push(t.group(
        "AllowRotationChange",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.rotation),
                &k("allowRotationChange.enabled"),
                None,
            ),
            t.leaf(
                "AllowJump",
                "BOOLEAN",
                json!(s.jump),
                &k("allowRotationChange.allowJump"),
                None,
            ),
        ],
    ));
    values.push(t.group(
        "AllowTimer",
        "TOGGLEABLE",
        vec![
            t.leaf(
                "Enabled",
                "BOOLEAN",
                json!(s.timer),
                &k("allowTimer.enabled"),
                None,
            ),
            t.ranged(
                "TimerSpeed",
                "FLOAT",
                json!(s.speed),
                json!({"from":1.,"to":10.}),
                "x",
                &k("allowTimer.timerSpeed"),
            ),
        ],
    ));
    Ok(t.group("AutoDodge", "TOGGLEABLE", values))
}
pub(super) fn parse_auto_dodge(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
        leaf(p, n, name, &format!("AutoDodge.{path}"), |v, _| {
            let s = AutoDodgeSetting(path.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::AutoDodge(s))
        });
    }
    add(p, root, "Ignore", "Ignore");
    for (group, names) in [
        ("AllowRotationChange", &["Enabled", "AllowJump"][..]),
        ("AllowTimer", &["Enabled", "TimerSpeed"][..]),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("AutoDodge.{group}")) {
                Ok(n) => {
                    for name in names {
                        add(p, &n, name, &format!("{group}.{name}"));
                    }
                }
                Err(e) => p.issues.push(e),
            }
        }
    }
}
