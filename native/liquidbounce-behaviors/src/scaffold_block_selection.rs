// SPDX-License-Identifier: GPL-3.0-or-later
//! ScaffoldBlockItemSelection and ModuleScaffold's hotbar comparator.
//! Native facts use the BlockItem's default state at BlockPos.ZERO.
use std::{cmp::Ordering, collections::BTreeSet};

pub const DEFAULT_DISALLOWED: [&str; 3] = [
    "minecraft:tnt",
    "minecraft:cobweb",
    "minecraft:nether_portal",
];
pub const DEFAULT_UNFAVORABLE: [&str; 7] = [
    "minecraft:crafting_table",
    "minecraft:jigsaw",
    "minecraft:smithing_table",
    "minecraft:fletching_table",
    "minecraft:enchanting_table",
    "minecraft:cauldron",
    "minecraft:magma_block",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub disallowed: BTreeSet<String>,
    pub unfavorable: BTreeSet<String>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            disallowed: DEFAULT_DISALLOWED.into_iter().map(str::to_owned).collect(),
            unfavorable: DEFAULT_UNFAVORABLE.into_iter().map(str::to_owned).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoBlock {
    pub enabled: bool,
    pub always: bool,
    pub slot_reset_delay: i32,
    pub do_not_use_below_count: i32,
}
impl Default for AutoBlock {
    fn default() -> Self {
        Self {
            enabled: true,
            always: false,
            slot_reset_delay: 5,
            do_not_use_below_count: 1,
        }
    }
}

/// Only actual BlockItems produce this snapshot. Non-block items are None.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockStack {
    pub block: String,
    pub count: i32,
    pub player_can_stand_up: bool,
    pub falling: bool,
    pub base_entity: bool,
    pub full_cube: bool,
    pub redstone_conductor: bool,
    pub friction: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
    pub hardness: f32,
}
impl BlockStack {
    pub fn valid(&self, policy: &Policy) -> bool {
        self.player_can_stand_up && !self.falling && !policy.disallowed.contains(&self.block)
    }
    pub fn unfavorable(&self, policy: &Policy) -> bool {
        self.friction > 0.6f32
            || self.speed_factor < 1.0f32
            || self.jump_factor < 1.0f32
            || self.base_entity
            || !self.full_cube
            || policy.unfavorable.contains(&self.block)
    }
    pub fn valid_count(&self, policy: &Policy) -> i32 {
        if self.valid(policy) { self.count } else { 0 }
    }
    fn hardness_distance(&self, neutral: bool) -> f64 {
        let hardness = f64::from(self.hardness);
        if neutral && (0.8..=2.0).contains(&hardness) {
            0.0
        } else {
            (1.7 - hardness).abs()
        }
    }
}

/// Java Double.compare, including equal NaNs and signed zero.
fn double_compare(a: f64, b: f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    let bits = |v: f64| {
        if v.is_nan() {
            0x7ff8_0000_0000_0000i64
        } else {
            v.to_bits() as i64
        }
    };
    bits(a).cmp(&bits(b))
}

/// maxWith uses this exact chain. The walkability and reversed stack-count
/// directions intentionally preserve the frozen source, including its quirks.
pub fn compare(a: &BlockStack, b: &BlockStack, policy: &Policy) -> Ordering {
    (!a.unfavorable(policy))
        .cmp(&!b.unfavorable(policy))
        .then_with(|| a.redstone_conductor.cmp(&b.redstone_conductor))
        .then_with(|| a.full_cube.cmp(&b.full_cube))
        .then_with(|| double_compare(f64::from(a.friction), f64::from(b.friction)))
        .then_with(|| {
            double_compare(
                (f64::from(a.jump_factor) - 1.0).abs(),
                (f64::from(b.jump_factor) - 1.0).abs(),
            )
        })
        .then_with(|| {
            double_compare(
                (f64::from(a.speed_factor) - 1.0).abs(),
                (f64::from(b.speed_factor) - 1.0).abs(),
            )
        })
        .then_with(|| double_compare(b.hardness_distance(true), a.hardness_distance(true)))
        .then_with(|| b.count.cmp(&a.count))
        .then_with(|| double_compare(b.hardness_distance(false), a.hardness_distance(false)))
}

pub fn best_slot(
    hotbar: &[Option<BlockStack>; 9],
    threshold: i32,
    policy: &Policy,
) -> Option<usize> {
    let choose = |above_threshold: bool| {
        let mut best: Option<usize> = None;
        for (slot, stack) in hotbar.iter().enumerate() {
            let Some(stack) = stack.as_ref().filter(|s| s.valid(policy)) else {
                continue;
            };
            if above_threshold && stack.count <= threshold {
                continue;
            }
            if best.is_none_or(|previous| {
                compare(
                    stack,
                    hotbar[previous].as_ref().expect("chosen block"),
                    policy,
                ) == Ordering::Greater
            }) {
                best = Some(slot);
            }
        }
        best
    };
    choose(true).or_else(|| choose(false))
}

/// ModuleScaffold.blockCount reads the raw selected slot when AutoBlock is off.
/// The effective SilentHotbar main hand is separate and only affects collision
/// context and whether a suitable block is already held.
pub fn block_count(
    hotbar: &[Option<BlockStack>; 9],
    offhand: Option<&BlockStack>,
    selected: usize,
    auto_block: bool,
    policy: &Policy,
) -> i32 {
    let offhand = offhand.map_or(0, |s| s.valid_count(policy));
    let main = if auto_block {
        hotbar
            .iter()
            .flatten()
            .fold(0i32, |sum, s| sum.wrapping_add(s.valid_count(policy)))
    } else {
        hotbar
            .get(selected)
            .and_then(Option::as_ref)
            .map_or(0, |s| s.valid_count(policy))
    };
    offhand.wrapping_add(main)
}
