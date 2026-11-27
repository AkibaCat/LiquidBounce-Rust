// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::ping_spoof_runtime::PingSpoofService;
use std::sync::atomic::Ordering;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PingSpoofSetting {
    Delay(i32),
}
impl PingSpoofSetting {
    pub fn apply(self, s: &PingSpoofService) {
        match self {
            Self::Delay(v) => s.delay.store(v, Ordering::Release),
        }
    }
}
pub fn serialize_ping_spoof(
    s: &PingSpoofService,
    c: &CommonSettings,
    format: JsonFormat,
    describe: &dyn Fn(&str) -> String,
) -> Result<Value, ConfigError> {
    let t = Tree { format, describe };
    let mut v = t.common(ModuleKind::PingSpoof, c);
    v.push(t.ranged(
        "Delay",
        "INT",
        json!(s.delay.load(Ordering::Acquire)),
        json!({"from":0,"to":25000}),
        "ms",
        "liquidbounce.module.pingSpoof.delay",
    ));
    Ok(t.group("PingSpoof", "TOGGLEABLE", v))
}
pub(super) fn parse_ping_spoof(patch: &mut SettingsPatch, root: &Nodes<'_>) {
    leaf(patch, root, "Delay", "PingSpoof.Delay", |v, p| {
        parse_int(v, p).map(|v| SettingChange::PingSpoof(PingSpoofSetting::Delay(v)))
    });
}
