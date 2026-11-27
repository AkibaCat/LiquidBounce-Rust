// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::flag_check::Settings;
#[derive(Clone, Debug, PartialEq)]
pub struct FlagCheckSetting(pub String, pub Value);
fn color(v: &Value, p: &str) -> Result<i32, ConfigError> {
    if let Some(s) = v.as_str() {
        let h = s.strip_prefix('#').unwrap_or(s);
        if !matches!(h.len(), 6 | 8) {
            return Err(ConfigError::new(p, "expected 6 or 8 hexadecimal digits"));
        }
        let n =
            i64::from_str_radix(h, 16).map_err(|_| ConfigError::new(p, "invalid color"))? as i32;
        Ok(n | if h.len() == 6 {
            0xff000000u32 as i32
        } else {
            0
        })
    } else {
        parse_int(v, p)
    }
}
impl FlagCheckSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = self.0.as_str();
        let v = &self.1;
        match p {
            "ChatMessage" => s.chat = parse_bool(v, p)?,
            "Notification" => s.notification = parse_bool(v, p)?,
            "InvalidAttributes" => s.invalid = parse_bool(v, p)?,
            "ResetFlags.Enabled" => s.reset = parse_bool(v, p)?,
            "ResetFlags.After" => s.after = parse_int(v, p)?,
            "Render.Enabled" => s.render = parse_bool(v, p)?,
            "Render.NotInFirstPerson" => s.not_first = parse_bool(v, p)?,
            "Render.Alive" => s.alive = parse_int(v, p)?,
            "Render.OutTime" => s.out_time = parse_int(v, p)?,
            "Render.Color" => s.color = color(v, p)?,
            "Render.OutlineColor" => s.outline = color(v, p)?,
            "Render.FadeOut" => {
                let tag = scalar_string(v).ok_or_else(|| ConfigError::new(p, "expected easing"))?;
                s.fade = Easing::ALL
                    .into_iter()
                    .find(|e| e.tag().eq_ignore_ascii_case(&tag))
                    .ok_or_else(|| ConfigError::new(p, "unknown easing"))?;
            }
            _ => return Err(ConfigError::new(p, "unknown FlagCheck setting")),
        };
        Ok(())
    }
}
pub fn serialize_flag_check(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let k = |p: &str| format!("liquidbounce.module.flagCheck.{p}");
    let boolean = |n: &str, v: bool, p: &str| t.leaf(n, "BOOLEAN", json!(v), &k(p), None);
    let mut values = t.common(ModuleKind::FlagCheck, c);
    values.extend([
        boolean("ChatMessage", s.chat, "chatMessage"),
        boolean("Notification", s.notification, "notification"),
        boolean("InvalidAttributes", s.invalid, "invalidAttributes"),
    ]);
    let mut reset = t.group(
        "ResetFlags",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.reset, "resetFlags.enabled"),
            t.ranged(
                "After",
                "INT",
                json!(s.after),
                json!({"from":1,"to":300}),
                "s",
                &k("resetFlags.after"),
            ),
        ],
    );
    let mut fade = t.leaf(
        "FadeOut",
        "CHOICE",
        json!(s.fade.tag()),
        &k("render.fadeOut"),
        None,
    );
    if f == JsonFormat::Interop {
        fade["choices"] = json!(Easing::ALL.map(Easing::tag));
    }
    let mut render = t.group(
        "Render",
        "TOGGLEABLE",
        vec![
            boolean("Enabled", s.render, "render.enabled"),
            boolean("NotInFirstPerson", s.not_first, "render.notInFirstPerson"),
            t.ranged(
                "Alive",
                "INT",
                json!(s.alive),
                json!({"from":0,"to":3000}),
                "ms",
                &k("render.alive"),
            ),
            fade,
            t.ranged(
                "OutTime",
                "INT",
                json!(s.out_time),
                json!({"from":0,"to":2000}),
                "ms",
                &k("render.outTime"),
            ),
            t.leaf("Color", "COLOR", json!(s.color), &k("render.color"), None),
            t.leaf(
                "OutlineColor",
                "COLOR",
                json!(s.outline),
                &k("render.outlineColor"),
                None,
            ),
        ],
    );
    if f == JsonFormat::Interop {
        for (v, p) in [(&mut reset, "resetFlags"), (&mut render, "render")] {
            v["key"] = json!(k(p));
            v["description"] = json!(d(&format!("{}.description", k(p))));
        }
    }
    values.extend([reset, render]);
    Ok(t.group("FlagCheck", "TOGGLEABLE", values))
}
pub(super) fn parse_flag_check(p: &mut SettingsPatch, root: &Nodes<'_>) {
    fn add(p: &mut SettingsPatch, n: &Nodes<'_>, name: &str, path: &str) {
        leaf(p, n, name, &format!("FlagCheck.{path}"), |v, _| {
            let s = FlagCheckSetting(path.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::FlagCheck(s))
        });
    }
    for name in ["ChatMessage", "Notification", "InvalidAttributes"] {
        add(p, root, name, name);
    }
    for (group, names) in [
        ("ResetFlags", &["Enabled", "After"][..]),
        (
            "Render",
            &[
                "Enabled",
                "NotInFirstPerson",
                "Alive",
                "FadeOut",
                "OutTime",
                "Color",
                "OutlineColor",
            ][..],
        ),
    ] {
        if let Some(v) = root.get(group) {
            match nodes(v, group, &format!("FlagCheck.{group}")) {
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
