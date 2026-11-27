// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoTool and its anchored inventory swap controller, from ae0eb396.
//! Game facts and inventory action scheduling are supplied by the client actor.
use std::{cmp::Ordering, collections::BTreeSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolSelector {
    #[default]
    Dynamic,
    Static,
}
impl ToolSelector {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Dynamic => "Dynamic",
            Self::Static => "Static",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    Whitelist,
    #[default]
    Blacklist,
}
impl Filter {
    pub fn accepts(self, member: bool) -> bool {
        (self == Self::Whitelist) == member
    }
    pub fn tag(self) -> &'static str {
        match self {
            Self::Whitelist => "Whitelist",
            Self::Blacklist => "Blacklist",
        }
    }
}

pub const REQUIREMENTS: [&str; 5] = [
    "NoMovement",
    "NoRotation",
    "NotUsingItem",
    "NotBreaking",
    "NotDuringCombat",
];
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryConstraints {
    pub start_delay: [i32; 2],
    pub click_delay: [i32; 2],
    pub close_delay: [i32; 2],
    pub miss_chance: [i32; 2],
    pub slot_multiplier: f32,
    pub requirements: u8,
    pub millisecond_delays: bool,
}
impl Default for InventoryConstraints {
    fn default() -> Self {
        Self {
            start_delay: [50, 100],
            click_delay: [100, 200],
            close_delay: [50, 100],
            miss_chance: [0, 0],
            slot_multiplier: 0.2,
            requirements: 0,
            millisecond_delays: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RequirementState {
    pub movement: [f32; 2],
    pub jumping: bool,
    pub rotation_matches: bool,
    pub using_item: bool,
    pub breaking: bool,
    pub in_combat: bool,
}
impl InventoryConstraints {
    pub fn passes(&self, state: RequirementState) -> bool {
        let checks = [
            (state.movement[0] * state.movement[0] + state.movement[1] * state.movement[1]).abs()
                < 1e-5
                && !state.jumping,
            state.rotation_matches,
            !state.using_item,
            !state.breaking,
            !state.in_combat,
        ];
        checks
            .into_iter()
            .enumerate()
            .all(|(i, passes)| self.requirements & (1 << i) == 0 || passes)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolStack {
    /// Inventory index: hotbar 0..8, inventory 9..35. Offhand is not searched.
    pub slot: u8,
    pub destroy_speed: f32,
    pub efficiency: i32,
    pub silk_touch: i32,
    pub max_damage: i32,
    pub damage: i32,
    pub empty: bool,
    /// ItemStack.matches identity: count, item and every component including damage.
    pub identity: String,
}
impl ToolStack {
    pub fn mining_speed(&self) -> f32 {
        let mut speed = self.destroy_speed;
        if speed > 1.0 && self.efficiency != 0 {
            // Kotlin Int.sq intentionally overflows before its conversion to Float.
            let addition = self.efficiency.wrapping_mul(self.efficiency) as f32 + 1.0;
            speed += addition.clamp(0.0, 1024.0);
        }
        speed
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningContext<'a> {
    pub position: [i32; 3],
    pub block: &'a str,
    pub tick: i32,
    pub creative: bool,
    pub sneaking: bool,
    pub in_combat: bool,
    /// Distances are to head-block centers, from the player's eyes.
    pub bed_head_distances_squared: &'a [f64],
    pub serverside_slot: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AutoTool {
    pub tool_selector: ToolSelector,
    pub ignore_durability: bool,
    pub consider_inventory: bool,
    pub constraints: InventoryConstraints,
    pub slot: i32,
    pub filter: Filter,
    pub blocks: BTreeSet<String>,
    pub silk_touch_handler: bool,
    pub silk_touch_filter: Filter,
    pub silk_touch_blocks: BTreeSet<String>,
    pub swap_back_delay: i32,
    pub switch_delay: i32,
    pub require_sneaking: bool,
    pub not_during_combat: bool,
    pub require_near_bed: bool,
    pub bed_distance: f32,
    breaking_pos: Option<[i32; 3]>,
    breaking_started_tick: i32,
    pub swaps: AnchoredSwapController,
}
impl Default for AutoTool {
    fn default() -> Self {
        Self {
            tool_selector: ToolSelector::Dynamic,
            ignore_durability: false,
            consider_inventory: false,
            constraints: InventoryConstraints::default(),
            slot: 0,
            filter: Filter::Blacklist,
            blocks: BTreeSet::new(),
            silk_touch_handler: false,
            silk_touch_filter: Filter::Whitelist,
            silk_touch_blocks: [
                "minecraft:ender_chest",
                "minecraft:glowstone",
                "minecraft:sea_lantern",
                "minecraft:turtle_egg",
            ]
            .map(str::to_owned)
            .into(),
            swap_back_delay: 20,
            switch_delay: 0,
            require_sneaking: false,
            not_during_combat: false,
            require_near_bed: false,
            bed_distance: 10.0,
            breaking_pos: None,
            breaking_started_tick: 0,
            swaps: AnchoredSwapController::default(),
        }
    }
}
impl AutoTool {
    pub fn is_inventory_considered(&self, running: bool) -> bool {
        running && self.tool_selector == ToolSelector::Dynamic && self.consider_inventory
    }
    /// Module disable clears breaking state; mode changes preserve that state.
    /// ModuleAutoTool.onDisabled itself resets only its own silent slot/session.
    pub fn disable(&mut self) {
        self.breaking_pos = None;
    }
    pub fn inventory_child_disabled(&mut self) {
        self.swaps.reset();
    }
    pub fn cancel_breaking(&mut self, running: bool) {
        if self.is_inventory_considered(running) {
            self.swaps.clear_requested();
        }
    }
    pub fn silk_accepts(&self, block: &str, stack: &ToolStack, running: bool) -> bool {
        !running
            || !self.silk_touch_handler
            || !self.silk_touch_blocks.contains(block)
            || (self.silk_touch_filter == Filter::Blacklist) == (stack.silk_touch == 0)
    }
    /// BlockBreakingProgressEvent, before vanilla guards and carried-slot sync.
    pub fn progress(
        &mut self,
        running: bool,
        context: MiningContext<'_>,
        stacks: &[ToolStack],
    ) -> Option<u8> {
        if !running {
            return None;
        }
        if self.switch_delay > 0 {
            if self.breaking_pos != Some(context.position) {
                self.breaking_pos = Some(context.position);
                self.breaking_started_tick = context.tick;
            }
            if context.tick.wrapping_sub(self.breaking_started_tick) < self.switch_delay {
                return None;
            }
        }
        let inventory = self.is_inventory_considered(running);
        let bed_distance_squared = self.bed_distance * self.bed_distance;
        if (self.not_during_combat && context.in_combat)
            || (self.require_sneaking && !context.sneaking)
            || (self.require_near_bed
                && !context
                    .bed_head_distances_squared
                    .iter()
                    .any(|distance| *distance <= f64::from(bed_distance_squared)))
        {
            if inventory {
                self.swaps.clear_requested();
            }
            return None;
        }
        self.tool_slot(running, context, stacks)
    }
    /// ToolSelectorMode.getTool, also used by PacketMine while AutoTool is
    /// disabled. The caller's mining event requirements are deliberately
    /// separate from the shared selector's filter and inventory controller.
    pub fn tool_slot(
        &mut self,
        running: bool,
        context: MiningContext<'_>,
        stacks: &[ToolStack],
    ) -> Option<u8> {
        let inventory = self.is_inventory_considered(running);
        // Filtering the block is outside getToolSlot: it does not clear a
        // previous inventory request, nor does the initial switch delay.
        if !self.filter.accepts(self.blocks.contains(context.block)) {
            return None;
        }
        if self.tool_selector == ToolSelector::Static {
            return u8::try_from(self.slot).ok().filter(|slot| *slot < 9);
        }
        let best = stacks
            .iter()
            .filter(|stack| {
                stack.slot < if inventory { 36 } else { 9 }
                    && !context.creative
                    && (self.ignore_durability
                        || stack.max_damage <= 0
                        || stack.max_damage.wrapping_sub(stack.damage) > 2)
                    && self.silk_accepts(context.block, stack, running)
            })
            .max_by(|a, b| compare_tools(a, b, context.serverside_slot));
        // Kotlin maxWith returns the first equal maximum. Rust max_by returns
        // the last: break complete ties using source iteration order (slot).
        let best = best.map(|stack| stack.slot);
        match best {
            Some(slot @ 0..9) => {
                if inventory {
                    self.swaps.clear_requested();
                    self.swaps.touch();
                }
                Some(slot)
            }
            Some(slot) => {
                if inventory {
                    self.swaps.request(slot);
                }
                None
            }
            None => {
                if inventory {
                    self.swaps.clear_requested();
                }
                None
            }
        }
    }
}
fn compare_tools(a: &ToolStack, b: &ToolStack, selected: u8) -> Ordering {
    // java.lang.Double.compare distinguishes signed zero and ranks NaN last.
    let java_compare = |a: f32, b: f32| {
        if a.is_nan() {
            if b.is_nan() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else if b.is_nan() {
            Ordering::Less
        } else {
            a.total_cmp(&b)
        }
    };
    java_compare(a.mining_speed(), b.mining_speed())
        .then_with(|| nearby(b.slot, selected).cmp(&nearby(a.slot, selected)))
        .then_with(|| b.slot.cmp(&a.slot))
}
fn nearby(slot: u8, selected: u8) -> i32 {
    if slot >= 9 {
        i32::MAX
    } else if slot == selected {
        i32::MIN
    } else {
        (i32::from(selected) - i32::from(slot)).abs()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwapAction {
    pub source: u8,
    pub hotbar: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwapPlan {
    pub action: SwapAction,
    pub restoring: bool,
    sampled_stack: String,
    expected: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnchoredSwapController {
    pub requested: Option<u8>,
    pub anchor_action: Option<SwapAction>,
    pub anchor_hotbar: Option<u8>,
    pub idle_ticks: i32,
    pub active: Option<SwapPlan>,
}
impl AnchoredSwapController {
    pub fn request(&mut self, source: u8) {
        self.requested = Some(source);
        self.touch();
    }
    pub fn clear_requested(&mut self) {
        self.requested = None;
    }
    pub fn touch(&mut self) {
        self.idle_ticks = 0;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// One GameTickEvent when the inventory plan has no suspended run.
    pub fn tick(&mut self, stacks: &[ToolStack], server_slot: u8, delay: i32) -> Option<SwapPlan> {
        if self.active.is_some() {
            return None;
        }
        let (action, restoring) = if let Some(source) = self.requested {
            let hotbar = self.anchor_hotbar.unwrap_or_else(|| {
                stacks
                    .iter()
                    .find(|s| s.slot < 9 && s.empty)
                    .map_or(server_slot, |s| s.slot)
            });
            (SwapAction { source, hotbar }, false)
        } else {
            let action = self.anchor_action?;
            self.anchor_hotbar?;
            self.idle_ticks = self.idle_ticks.wrapping_add(1);
            if self.idle_ticks <= delay {
                return None;
            }
            (action, true)
        };
        let sample_slot = if restoring {
            action.hotbar
        } else {
            action.source
        };
        let sampled_stack = stacks
            .iter()
            .find(|s| s.slot == sample_slot)?
            .identity
            .clone();
        let plan = SwapPlan {
            action,
            restoring,
            sampled_stack,
            expected: stacks.iter().map(|s| s.identity.clone()).collect(),
        };
        self.active = Some(plan.clone());
        Some(plan)
    }
    /// Both the plan's own `requires` and InventoryPlan.matchesExpected guards.
    pub fn guards_hold(&self, stacks: &[ToolStack], delay: i32) -> bool {
        let Some(plan) = &self.active else {
            return false;
        };
        let sample_slot = if plan.restoring {
            plan.action.hotbar
        } else {
            plan.action.source
        };
        let changed = stacks
            .iter()
            .find(|s| s.slot == sample_slot)
            .is_none_or(|s| s.identity != plan.sampled_stack);
        let own = if plan.restoring {
            (self.idle_ticks > delay && self.requested.is_none()) || changed
        } else {
            self.requested == Some(plan.action.source) || changed
        };
        own && stacks.iter().map(|s| &s.identity).eq(plan.expected.iter())
    }
    pub fn performed(&mut self, succeeded: bool) {
        let Some(plan) = self.active.take() else {
            return;
        };
        if !succeeded {
            return;
        }
        if plan.restoring {
            self.reset();
            return;
        }
        self.anchor_hotbar = Some(plan.action.hotbar);
        self.anchor_action.get_or_insert(plan.action);
        self.idle_ticks = 0;
        self.requested = None;
    }
    pub fn cancel_plan(&mut self) {
        self.active = None;
    }
}

#[path = "auto_tool_inventory.rs"]
pub mod inventory;
