// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use liquidbounce_behaviors::sword_block::Settings;
pub const NAMES: [&str; 5] = [
    "OnlyVisual",
    "FakeOnPressing",
    "HideShieldSlot",
    "ApplyToThirdPersonView",
    "AlwaysHideShield",
];
#[derive(Clone, Debug, PartialEq)]
pub struct SwordBlockSetting(pub String, pub bool);
impl SwordBlockSetting {
    pub fn apply(&self, s: &mut Settings) {
        match self.0.as_str() {
            "OnlyVisual" => s.only_visual = self.1,
            "FakeOnPressing" => s.fake_on_pressing = self.1,
            "HideShieldSlot" => s.hide_shield_slot = self.1,
            "ApplyToThirdPersonView" => s.third_person = self.1,
            "AlwaysHideShield" => s.always_hide_shield = self.1,
            _ => {}
        }
    }
}
pub fn serialize_sword_block(
    s: &Settings,
    c: &CommonSettings,
    f: JsonFormat,
    d: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree {
        format: f,
        describe: d,
    };
    let mut values = t.common(ModuleKind::SwordBlock, c);
    for (name, value) in NAMES.into_iter().zip([
        s.only_visual,
        s.fake_on_pressing,
        s.hide_shield_slot,
        s.third_person,
        s.always_hide_shield,
    ]) {
        let key = format!(
            "liquidbounce.module.swordBlock.{}{}",
            name[..1].to_ascii_lowercase(),
            &name[1..]
        );
        values.push(t.leaf(name, "BOOLEAN", json!(value), &key, None));
    }
    Ok(t.group("SwordBlock", "TOGGLEABLE", values))
}
pub(super) fn parse_sword_block(p: &mut SettingsPatch, root: &Nodes<'_>) {
    for name in NAMES {
        leaf(p, root, name, &format!("SwordBlock.{name}"), |v, path| {
            Ok(SettingChange::SwordBlock(SwordBlockSetting(
                name.into(),
                parse_bool(v, path)?,
            )))
        });
    }
}
