// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent navigation/AbilityCapture.kt with live read order and copied tool data.
use super::{
    body::{WalkBody, WalkPlayer, capture_motion},
    break_times::{BreakStack, BreakState, BreakTimes},
    bridging::Bridging,
};
use crate::navigation::walk::{abilities::WalkAbilities, costs::Costs};
use std::{collections::HashSet, sync::Arc};

pub trait CaptureStack<B>: BreakStack<B> {
    fn count(&self) -> i32;
}
pub trait CaptureInventory {
    type Stack;
    fn item(&self, slot: i32) -> Self::Stack;
}
pub type PlacePolicy<T> = Arc<dyn Fn(&T) -> bool>;
pub type BreakPolicy<B> = Arc<dyn Fn(&B) -> bool + Send + Sync>;
pub struct CaptureOptions<B, T> {
    pub can_break: bool,
    pub can_place: bool,
    pub can_parkour: bool,
    pub risky_jumps: bool,
    pub max_fall: i32,
    pub fast_bridges: bool,
    pub bridging: Bridging,
    pub risky_bridges: bool,
    pub placeable: PlacePolicy<T>,
    pub breakable: BreakPolicy<B>,
    pub reserved: Arc<HashSet<i64>>,
}
pub trait CaptureHost: WalkBody {
    type Stack: CaptureStack<<Self::Player as WalkPlayer>::State>;
    type Inventory: CaptureInventory<Stack = Self::Stack>;
    fn capture_options(&self) -> CaptureOptions<<Self::Player as WalkPlayer>::State, Self::Stack>;
    fn capture_inventory(&self) -> &Self::Inventory;
    fn safe_fall_distance(&self) -> f64;
    fn health(&self) -> f32;
    fn jump_climbs_ladders(&self) -> bool;
}
pub fn hotbar<H: CaptureHost>(host: &H) -> Vec<H::Stack> {
    let inventory = host.capture_inventory();
    (0..9).map(|slot| inventory.item(slot)).collect()
}
pub fn safe_fall<H: CaptureHost>(host: &H) -> i32 {
    let distance = host.safe_fall_distance().floor() as i32;
    host.capture_options()
        .max_fall
        .wrapping_add(0.max(distance.wrapping_sub(3)))
}
pub fn count_placeable_blocks<H: CaptureHost>(host: &H) -> i32 {
    let blocks = host.capture_options().placeable;
    let offhand = host.capture_inventory().item(40);
    let mut stacks = vec![offhand];
    stacks.extend(hotbar(host));
    stacks.iter().fold(0_i32, |total, stack| {
        total.wrapping_add(if blocks(stack) { stack.count() } else { 0 })
    })
}
pub fn capture_walk<H: CaptureHost>(
    host: &H,
    avoid: &HashSet<i64>,
    unbreakable: &HashSet<i64>,
) -> WalkAbilities<<H::Player as WalkPlayer>::State>
where
    <H::Player as WalkPlayer>::State: BreakState + 'static,
{
    let options = host.capture_options();
    let placeable_blocks = if options.can_place {
        count_placeable_blocks(host)
    } else {
        0
    };
    let can_sprint = host.player().food_level() > 6;
    let safe_fall = safe_fall(host);
    let break_ticks: crate::navigation::walk::abilities::BreakTicks<
        <H::Player as WalkPlayer>::State,
    > = if options.can_break {
        let selected: Vec<_> = hotbar(host)
            .into_iter()
            .filter(|stack| !stack.is_empty())
            .collect();
        let times = BreakTimes::capture(&selected);
        let breakable = options.breakable;
        Arc::new(move |state| {
            if breakable(state) {
                times.ticks(state)
            } else {
                f64::INFINITY
            }
        })
    } else {
        Arc::new(|_| f64::INFINITY)
    };
    let avoid = Arc::new(avoid.clone());
    let air = host.player().air_supply();
    let motion = capture_motion(host.player());
    let jump_climbs = host.jump_climbs_ladders();
    let bridge_ticks = if options.fast_bridges {
        Costs::RUN_BRIDGE
    } else if matches!(options.bridging, Bridging::Godbridge | Bridging::Telly)
        || options.bridging == Bridging::Auto && options.risky_bridges
    {
        Costs::GODBRIDGE
    } else {
        Costs::NINJA_BRIDGE
    };
    let unbreakable = Arc::new(unbreakable.clone());
    let health = host.health();
    WalkAbilities {
        can_break: options.can_break,
        can_place: placeable_blocks > 0,
        placeable_blocks,
        can_parkour: options.can_parkour,
        risky_jumps: options.can_parkour && options.risky_jumps,
        can_sprint,
        safe_fall,
        water_fall: 23,
        break_ticks,
        avoid,
        air,
        motion,
        jump_climbs,
        bridge_ticks,
        unbreakable,
        health,
        reserved: options.reserved,
    }
}
