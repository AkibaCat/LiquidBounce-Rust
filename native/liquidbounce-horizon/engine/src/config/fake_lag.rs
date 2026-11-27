// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::fake_lag_runtime::FakeLagService;
use liquidbounce_behaviors::fake_lag::Settings;
#[derive(Clone, Debug, PartialEq)]
pub struct FakeLagSetting(pub String, pub Value);
pub const FLUSH_ON: [&str; 3] = ["EntityInteract", "BlockInteract", "Action"];
impl FakeLagSetting {
    pub fn apply(&self, s: &mut Settings) -> Result<(), ConfigError> {
        let p = format!("FakeLag.{}", self.0);
        let v = &self.1;
        match self.0.as_str() {
            "Range" => {
                let r = parse_float_pair(v, &p)?;
                if r[0] > r[1] {
                    return Err(ConfigError::new(&p, "empty range"));
                }
                s.range = r;
            }
            "Delay" => {
                let r = parse_int_pair(v, &p)?;
                if r[0] > r[1] {
                    return Err(ConfigError::new(&p, "empty delay range"));
                }
                s.delay = r;
            }
            "RecoilTime" => s.recoil = parse_int(v, &p)?,
            "Mode" => {
                s.constant = match scalar_string(v).as_deref() {
                    Some(t) if t.eq_ignore_ascii_case("Constant") => true,
                    Some(t) if t.eq_ignore_ascii_case("Dynamic") => false,
                    _ => return Err(ConfigError::new(&p, "expected Constant or Dynamic")),
                }
            }
            "FlushOn" => s.flush_on = parse_multi_mask(v, &p, &FLUSH_ON)?,
            _ => return Err(ConfigError::new(&p, "unknown FakeLag setting")),
        }
        Ok(())
    }
}
pub fn serialize_fake_lag(
    service: &FakeLagService,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = service.settings();
    let k = |p: &str| format!("liquidbounce.module.fakeLag.{p}");
    let mut values = t.common(ModuleKind::FakeLag, c);
    values.push(t.ranged(
        "Range",
        "FLOAT_RANGE",
        json!({"from":s.range[0],"to":s.range[1]}),
        json!({"from":0,"to":10}),
        "",
        &k("range"),
    ));
    values.push(t.ranged(
        "Delay",
        "INT_RANGE",
        json!({"from":s.delay[0],"to":s.delay[1]}),
        json!({"from":0,"to":1000}),
        "ms",
        &k("delay"),
    ));
    values.push(t.ranged(
        "RecoilTime",
        "INT",
        json!(s.recoil),
        json!({"from":0,"to":1000}),
        "ms",
        &k("recoilTime"),
    ));
    let mut mode = t.leaf(
        "Mode",
        "CHOOSE",
        json!(if s.constant { "Constant" } else { "Dynamic" }),
        &k("mode"),
        None,
    );
    if f == JsonFormat::Interop {
        mode["choices"] = json!(["Constant", "Dynamic"]);
    }
    values.push(mode);
    values.push(t.multi("FlushOn", s.flush_on, &FLUSH_ON, &k("flushOn")));
    Ok(t.group("FakeLag", "TOGGLEABLE", values))
}
pub(super) fn parse_fake_lag(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in ["Range", "Delay", "RecoilTime", "Mode", "FlushOn"] {
        leaf(p, root, name, &format!("FakeLag.{name}"), |v, _| {
            let s = FakeLagSetting(name.into(), v.clone());
            s.apply(&mut Settings::default())?;
            Ok(SettingChange::FakeLag(s))
        });
    }
}
