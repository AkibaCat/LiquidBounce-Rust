// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::vehicle_control::Settings;
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleControlSetting {
    pub path: String,
    pub value: Value,
}
impl VehicleControlSetting {
    pub fn apply(self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.path.as_str();
        let v = &self.value;
        match p {
            "BaseSpeed.Horizontal" => s.horizontal = parse_float(v, p)?,
            "BaseSpeed.Vertical" => s.vertical = parse_float(v, p)?,
            "SprintSpeed.Enabled" => s.sprint_enabled = parse_bool(v, p)?,
            "SprintSpeed.Horizontal" => s.sprint_horizontal = parse_float(v, p)?,
            "SprintSpeed.Vertical" => s.sprint_vertical = parse_float(v, p)?,
            "Glide" => s.glide = parse_float(v, p)?,
            "MouseControl" => s.mouse_control = parse_bool(v, p)?,
            "NoGlideOnSprint" => s.no_glide_on_sprint = parse_bool(v, p)?,
            "Rehook.Enabled" => s.rehook = parse_bool(v, p)?,
            "Rehook.UnhookAfter" => s.unhook_after = parse_int(v, p)?,
            "Rehook.HookAfter" => s.hook_after = parse_int(v, p)?,
            _ => return Err(ConfigError::new(p, "unknown VehicleControl setting")),
        };
        Ok(())
    }
}
fn key(p: &str) -> String {
    format!("liquidbounce.module.vehicleControl.{p}")
}
pub fn serialize_vehicle_control(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::VehicleControl, c);
    let float =
        |name: &str, value: f32, p: &str, from: f32, to: f32| -> Result<Value, ConfigError> {
            Ok(t.ranged(
                name,
                "FLOAT",
                float_json(value, p)?,
                json!({"from":from,"to":to}),
                "",
                &key(p),
            ))
        };
    let boolean =
        |name: &str, value: bool, p: &str| t.leaf(name, "BOOLEAN", json!(value), &key(p), None);
    values.push(float("Glide", s.glide, "glide", -0.3, 0.3)?);
    values.push(boolean("MouseControl", s.mouse_control, "mouseControl"));
    let mut no_glide = boolean("NoGlideOnSprint", s.no_glide_on_sprint, "noGlideOnSprint");
    if f == JsonFormat::Interop {
        no_glide["aliases"] = json!(["NoGlideOnSpring"]);
    }
    values.push(no_glide);
    values.push(t.group(
        "BaseSpeed",
        "CONFIGURABLE",
        vec![
            float("Horizontal", s.horizontal, "baseSpeed.horizontal", 0.1, 10.)?,
            float("Vertical", s.vertical, "baseSpeed.vertical", 0.1, 10.)?,
        ],
    ));
    values.push(t.group(
        "SprintSpeed",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.sprint_enabled, "sprintSpeed.enabled"),
            float(
                "Horizontal",
                s.sprint_horizontal,
                "sprintSpeed.horizontal",
                0.1,
                10.,
            )?,
            float(
                "Vertical",
                s.sprint_vertical,
                "sprintSpeed.vertical",
                0.1,
                10.,
            )?,
        ],
    ));
    values.push(t.group(
        "Rehook",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.rehook, "rehook.enabled"),
            t.ranged(
                "UnhookAfter",
                "INT",
                json!(s.unhook_after),
                json!({"from":1,"to":10}),
                "ticks",
                &key("rehook.unhookAfter"),
            ),
            t.ranged(
                "HookAfter",
                "INT",
                json!(s.hook_after),
                json!({"from":1,"to":10}),
                "ticks",
                &key("rehook.hookAfter"),
            ),
        ],
    ));
    Ok(t.group("VehicleControl", "TOGGLEABLE", values))
}
pub(super) fn parse_vehicle_control(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn leaves(p: &mut SettingsPatch, n: &Nodes<'_>, group: &str, names: &[&str]) {
        for name in names {
            let path = if group.is_empty() {
                (*name).to_owned()
            } else {
                format!("{group}.{name}")
            };
            let actual = if *name == "NoGlideOnSprint" && n.get(*name).is_none() {
                "NoGlideOnSpring"
            } else {
                name
            };
            leaf(p, n, actual, &format!("VehicleControl.{path}"), |v, _| {
                let value = VehicleControlSetting {
                    path: path.clone(),
                    value: v.clone(),
                };
                value.clone().apply(&mut Settings::default())?;
                Ok(SettingChange::VehicleControl(value))
            });
        }
    }
    leaves(p, root, "", &["Glide", "MouseControl", "NoGlideOnSprint"]);
    for (group, names) in [
        ("BaseSpeed", vec!["Horizontal", "Vertical"]),
        ("SprintSpeed", vec!["Enabled", "Horizontal", "Vertical"]),
        ("Rehook", vec!["Enabled", "UnhookAfter", "HookAfter"]),
    ] {
        if let Some(n) = root.get(group) {
            match nodes(n, group, &format!("VehicleControl.{group}")) {
                Ok(n) => leaves(p, &n, group, &names),
                Err(e) => p.issues.push(e),
            }
        }
    }
}
