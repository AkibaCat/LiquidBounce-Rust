// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::report_helper::{MODES, State};
#[derive(Clone, Debug, PartialEq)]
pub struct ReportHelperSetting {
    pub path: String,
    pub value: Value,
}
impl ReportHelperSetting {
    pub fn apply(self, s: &mut State) -> Result<(), ConfigError> {
        let p = self.path.as_str();
        let v = &self.value;
        match p {
            "AutoReport.Enabled" => s.set_report(parse_bool(v, p)?),
            "AutoReport.Delay" => {
                let range = parse_int_pair(v, p)?;
                if range[0] > range[1] {
                    return Err(ConfigError::new(p, "empty delay range"));
                }
                s.settings.delay = range;
            }
            "AutoReport.Chance" => s.settings.chance = parse_int(v, p)?,
            "AutoReport.CommandPattern" => {
                s.settings.pattern =
                    scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected text"))?
            }
            "AutoConfirm.Enabled" => s.set_confirm(parse_bool(v, p)?),
            "AutoConfirm.Mode" => {
                let tag = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected choice"))?;
                let mode = MODES
                    .iter()
                    .position(|m| m.eq_ignore_ascii_case(&tag))
                    .ok_or_else(|| ConfigError::new(p, "unknown ReportHelper mode"))?;
                s.set_mode(mode);
            }
            _ => return Err(ConfigError::new(p, "unknown ReportHelper setting")),
        }
        Ok(())
    }
}
pub fn serialize_report_helper(
    s: &State,
    common: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let key = "liquidbounce.module.reportHelper";
    let mut values = t.common(ModuleKind::ReportHelper, common);
    let boolean = |name: &str, value: bool, path: &str| {
        t.leaf(
            name,
            "BOOLEAN",
            json!(value),
            &format!("{key}.{path}"),
            None,
        )
    };
    values.push(t.group(
        "AutoReport",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.settings.report, "autoReport.enabled"),
            t.ranged(
                "Delay",
                "INT_RANGE",
                json!({"from":s.settings.delay[0],"to":s.settings.delay[1]}),
                json!({"from":0,"to":20}),
                "ticks",
                &format!("{key}.autoReport.delay"),
            ),
            t.ranged(
                "Chance",
                "INT",
                json!(s.settings.chance),
                json!({"from":1,"to":100}),
                "%",
                &format!("{key}.autoReport.chance"),
            ),
            t.leaf(
                "CommandPattern",
                "TEXT",
                json!(s.settings.pattern),
                &format!("{key}.autoReport.commandPattern"),
                None,
            ),
        ],
    ));
    let choices = MODES
        .into_iter()
        .map(|name| (name.into(), t.group(name, "CONFIGURABLE", vec![])))
        .collect();
    let mut mode = t.mode(MODES[s.settings.mode], choices);
    if format == JsonFormat::Interop {
        mode["key"] = json!(format!("{key}.autoConfirm.mode"));
        mode["description"] = json!(describe(&format!("{key}.autoConfirm.mode.description")));
    }
    values.push(t.group(
        "AutoConfirm",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.settings.confirm, "autoConfirm.enabled"),
            mode,
        ],
    ));
    Ok(t.group("ReportHelper", "TOGGLEABLE", values))
}
fn collect(patch: &mut SettingsPatch, root: &Nodes<'_>, group: &str, names: &[&str]) {
    for name in names {
        if let Some(node) = root.get(*name) {
            match value_of(node, name) {
                Ok(value) => patch
                    .changes
                    .push(SettingChange::ReportHelper(ReportHelperSetting {
                        path: format!("{group}.{name}"),
                        value: value.clone(),
                    })),
                Err(error) => patch.issues.push(error),
            }
        }
    }
}
pub(super) fn parse_report_helper(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    for group in ["AutoReport", "AutoConfirm"] {
        let Some(node) = root.get(group) else {
            continue;
        };
        let path = format!("ReportHelper.{group}");
        let root = match nodes(node, group, &path) {
            Ok(v) => v,
            Err(e) => {
                patch.issues.push(e);
                continue;
            }
        };
        if group == "AutoReport" {
            collect(
                patch,
                &root,
                group,
                &["Enabled", "Delay", "Chance", "CommandPattern"],
            );
        } else {
            collect(patch, &root, group, &["Enabled"]);
            if let Some(mode) = root.get("Mode") {
                match mode.get("active") {
                    Some(v) => {
                        patch
                            .changes
                            .push(SettingChange::ReportHelper(ReportHelperSetting {
                                path: "AutoConfirm.Mode".into(),
                                value: v.clone(),
                            }))
                    }
                    None => patch.issues.push(ConfigError::new(
                        "ReportHelper.AutoConfirm.Mode.active",
                        "missing active choice",
                    )),
                }
                if let Err(e) = nodes(mode, "Mode", "ReportHelper.AutoConfirm.Mode") {
                    patch.issues.push(e);
                }
                if let Some(choices) = mode.get("choices").and_then(Value::as_object) {
                    for name in MODES {
                        match choices.get(name) {
                            Some(v) => {
                                if let Err(e) =
                                    nodes(v, name, &format!("ReportHelper.AutoConfirm.Mode.{name}"))
                                {
                                    patch.issues.push(e);
                                }
                            }
                            None => patch.issues.push(ConfigError::new(
                                "ReportHelper.AutoConfirm.Mode.choices",
                                "missing choice",
                            )),
                        }
                    }
                } else {
                    patch.issues.push(ConfigError::new(
                        "ReportHelper.AutoConfirm.Mode.choices",
                        "expected choices object",
                    ));
                }
            }
        }
    }
}
