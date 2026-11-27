// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::packet_mine_runtime::PacketMineRuntime;
use liquidbounce_behaviors::packet_mine::{Mode, Rotate, Settings, SwitchMethod, ToolMode};

#[derive(Clone, Debug, PartialEq)]
pub enum PacketMineSetting {
    Value(String, Value),
}

fn choice<T: Copy>(name: &str, values: &[(&str, T)]) -> Option<T> {
    values
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| *v)
}

impl PacketMineSetting {
    pub fn apply(self, r: &mut PacketMineRuntime) -> Result<(), ConfigError> {
        let Self::Value(path, value) = self;
        let p = format!("PacketMine.{path}");
        let s = &mut r.behavior.settings;
        match path.as_str() {
            "Mode" => {
                s.mode = choice(
                    &scalar_string(&value)
                        .ok_or_else(|| ConfigError::new(&p, "expected choice"))?,
                    &[
                        ("Normal", Mode::Normal),
                        ("Immediate", Mode::Immediate),
                        ("Civ", Mode::Civ),
                    ],
                )
                .ok_or_else(|| ConfigError::new(&p, "unknown mode"))?
            }
            "Switch" => {
                s.tool = choice(
                    &scalar_string(&value)
                        .ok_or_else(|| ConfigError::new(&p, "expected choice"))?,
                    &[
                        ("Always", ToolMode::Always),
                        ("PostStart", ToolMode::PostStart),
                        ("OnStop", ToolMode::OnStop),
                        ("Never", ToolMode::Never),
                    ],
                )
                .ok_or_else(|| ConfigError::new(&p, "unknown switch"))?
            }
            "SwitchMethod" => {
                s.method = choice(
                    &scalar_string(&value)
                        .ok_or_else(|| ConfigError::new(&p, "expected choice"))?,
                    &[
                        ("Normal", SwitchMethod::Normal),
                        ("Swap", SwitchMethod::Swap),
                        ("Pick", SwitchMethod::Pick),
                    ],
                )
                .ok_or_else(|| ConfigError::new(&p, "unknown switch method"))?
            }
            "Rotate" => {
                s.rotate = choice(
                    &scalar_string(&value)
                        .ok_or_else(|| ConfigError::new(&p, "expected choice"))?,
                    &[
                        ("OnStart", Rotate::OnStart),
                        ("OnStop", Rotate::OnStop),
                        ("Both", Rotate::Both),
                        ("Always", Rotate::Always),
                        ("Never", Rotate::Never),
                    ],
                )
                .ok_or_else(|| ConfigError::new(&p, "unknown rotate mode"))?
            }
            "Range" => s.range = parse_float(&value, &p)?,
            "WallsRange" => s.walls = parse_float(&value, &p)?,
            "KeepRange" => s.keep = parse_float(&value, &p)?,
            "BreakDamage" => s.break_damage = parse_float(&value, &p)?,
            "PostBreakDelay" => s.post_break_delay = parse_int(&value, &p)?,
            "SelectDelay" => s.select_delay = parse_int(&value, &p)?,
            "NormalClientSet" => s.normal_client_set = parse_bool(&value, &p)?,
            "NormalConfirm" => s.normal_confirm = parse_bool(&value, &p)?,
            "ImmediateConfirm" => s.immediate_confirm = parse_bool(&value, &p)?,
            "CivSwitch" => s.civ_switch = parse_bool(&value, &p)?,
            "AbortOnSwitch" => s.abort_on_switch = parse_bool(&value, &p)?,
            "CancelAutomatic" => s.cancel_automatic = parse_bool(&value, &p)?,
            "IgnoreOpenInventory" => s.ignore_inventory = parse_bool(&value, &p)?,
            "AbortAlwaysDown" => s.abort_down = parse_bool(&value, &p)?,
            _ => return Err(ConfigError::new(&p, "unknown PacketMine setting")),
        }
        Ok(())
    }
}

pub fn serialize_packet_mine(
    r: &PacketMineRuntime,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let s = &r.behavior.settings;
    let key = |p: &str| format!("liquidbounce.module.packetMine.{}", p);
    let mut values = t.common(ModuleKind::PacketMine, c);
    values.extend(
        [
            t.leaf(
                "Mode",
                "CHOOSE",
                json!(match s.mode {
                    Mode::Normal => "Normal",
                    Mode::Immediate => "Immediate",
                    Mode::Civ => "Civ",
                }),
                &key("mode"),
                None,
            ),
            t.leaf(
                "Switch",
                "CHOOSE",
                json!(match s.tool {
                    ToolMode::Always => "Always",
                    ToolMode::PostStart => "PostStart",
                    ToolMode::OnStop => "OnStop",
                    ToolMode::Never => "Never",
                }),
                &key("switch"),
                None,
            ),
            t.leaf(
                "SwitchMethod",
                "CHOOSE",
                json!(match s.method {
                    SwitchMethod::Normal => "Normal",
                    SwitchMethod::Swap => "Swap",
                    SwitchMethod::Pick => "Pick",
                }),
                &key("switchMethod"),
                None,
            ),
            t.leaf(
                "Rotate",
                "CHOOSE",
                json!(match s.rotate {
                    Rotate::OnStart => "OnStart",
                    Rotate::OnStop => "OnStop",
                    Rotate::Both => "Both",
                    Rotate::Always => "Always",
                    Rotate::Never => "Never",
                }),
                &key("rotate"),
                None,
            ),
            t.ranged(
                "Range",
                "FLOAT",
                json!(s.range),
                json!({"from":1.,"to":6.}),
                "m",
                &key("range"),
            ),
            t.ranged(
                "WallsRange",
                "FLOAT",
                json!(s.walls),
                json!({"from":0.,"to":6.}),
                "m",
                &key("wallsRange"),
            ),
            t.ranged(
                "KeepRange",
                "FLOAT",
                json!(s.keep),
                json!({"from":0.,"to":200.}),
                "m",
                &key("keepRange"),
            ),
            t.ranged(
                "BreakDamage",
                "FLOAT",
                json!(s.break_damage),
                json!({"from":0.,"to":2.}),
                "",
                &key("breakDamage"),
            ),
            t.ranged(
                "PostBreakDelay",
                "INT",
                json!(s.post_break_delay),
                json!({"from":0,"to":10}),
                "ticks",
                &key("postBreakDelay"),
            ),
            t.ranged(
                "SelectDelay",
                "INT",
                json!(s.select_delay),
                json!({"from":0,"to":400}),
                "ms",
                &key("selectDelay"),
            ),
        ]
        .into_iter(),
    );
    for (name, val, path) in [
        ("NormalClientSet", s.normal_client_set, "normalClientSet"),
        ("NormalConfirm", s.normal_confirm, "normalConfirm"),
        ("ImmediateConfirm", s.immediate_confirm, "immediateConfirm"),
        ("CivSwitch", s.civ_switch, "civSwitch"),
        ("AbortOnSwitch", s.abort_on_switch, "abortOnSwitch"),
        ("CancelAutomatic", s.cancel_automatic, "cancelAutomatic"),
        (
            "IgnoreOpenInventory",
            s.ignore_inventory,
            "ignoreOpenInventory",
        ),
        ("AbortAlwaysDown", s.abort_down, "abortAlwaysDown"),
    ] {
        values.push(t.leaf(name, "BOOLEAN", json!(val), &key(path), None));
    }
    Ok(t.group("PacketMine", "TOGGLEABLE", values))
}

pub fn parse_packet_mine(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for (name, path) in [
        ("Mode", "Mode"),
        ("Switch", "Switch"),
        ("SwitchMethod", "SwitchMethod"),
        ("Rotate", "Rotate"),
        ("Range", "Range"),
        ("WallsRange", "WallsRange"),
        ("KeepRange", "KeepRange"),
        ("BreakDamage", "BreakDamage"),
        ("PostBreakDelay", "PostBreakDelay"),
        ("SelectDelay", "SelectDelay"),
        ("NormalClientSet", "NormalClientSet"),
        ("NormalConfirm", "NormalConfirm"),
        ("ImmediateConfirm", "ImmediateConfirm"),
        ("CivSwitch", "CivSwitch"),
        ("AbortOnSwitch", "AbortOnSwitch"),
        ("CancelAutomatic", "CancelAutomatic"),
        ("IgnoreOpenInventory", "IgnoreOpenInventory"),
        ("AbortAlwaysDown", "AbortAlwaysDown"),
    ] {
        leaf(p, root, name, &format!("PacketMine.{path}"), |v, _| {
            Ok(SettingChange::PacketMine(PacketMineSetting::Value(
                path.into(),
                v.clone(),
            )))
        });
    }
}
