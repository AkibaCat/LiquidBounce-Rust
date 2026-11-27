// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoDeposit shares the original item/menu/title filters with ChestCleaner.
use crate::chest_cleaner::ChestCleaner;
use std::collections::BTreeSet;
pub const SWINGS: [&str; 4] = ["DoNotHide", "HideForBoth", "HideForClient", "HideForServer"];
pub const PAUSES: [&str; 2] = ["Combat", "UsingItem"];
#[derive(Clone, Debug)]
pub struct PunchSettings {
    pub enabled: bool,
    pub range: f32,
    pub wall_range: f32,
    pub delay: i32,
    pub swing: usize,
    pub pauses: u32,
    pub blocks: BTreeSet<String>,
    pub rotate: bool,
}
impl Default for PunchSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            range: 3.,
            wall_range: 0.,
            delay: 5,
            swing: 0,
            pauses: 0,
            blocks: ["minecraft:chest", "minecraft:ender_chest"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            rotate: false,
        }
    }
}
impl PunchSettings {
    pub fn running(&self, parent: bool, combat: bool, using: bool) -> bool {
        parent
            && self.enabled
            && !(self.pauses & 1 != 0 && combat)
            && !(self.pauses & 2 != 0 && using)
    }
}
#[derive(Clone, Debug, Default)]
pub struct AutoDeposit {
    pub inventory: ChestCleaner,
    pub punch: PunchSettings,
}
impl AutoDeposit {
    pub fn matching(&self, items: &[(String, bool)], inventory: bool) -> Vec<u8> {
        items
            .iter()
            .take(if inventory { 36 } else { 9 })
            .enumerate()
            .filter_map(|(i, (item, empty))| {
                self.inventory.selects(item, *empty).then_some(i as u8)
            })
            .collect()
    }
}
