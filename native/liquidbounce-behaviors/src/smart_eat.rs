// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSmartEat, LiquidBounce ae0eb396. Effects run outside the registry lock.
use std::cmp::Ordering;

pub const SILENT_OFFHAND_REQUESTER: u64 = 0x534d_4541_544f_4646;
pub const AUTO_EAT_REQUESTER: u64 = 0x534d_4541_5441_5554;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Offhand,
    Hotbar(u8),
}
impl Slot {
    /// Slot encoding for the native render helper. Selection must instead use
    /// hotbar_index: offhand is a successful no-op in the original overload.
    pub fn native(self) -> i32 {
        match self {
            Self::Offhand => -1,
            Self::Hotbar(slot) => i32::from(slot),
        }
    }
    pub fn hotbar_index(self) -> Option<i32> {
        match self {
            Self::Offhand => None,
            Self::Hotbar(slot) => Some(i32::from(slot)),
        }
    }
    fn nearby(self, selected: u8) -> i32 {
        match self {
            Self::Offhand => i32::MIN + 1,
            Self::Hotbar(slot) if slot == selected => i32::MIN,
            Self::Hotbar(slot) => (i32::from(selected) - i32::from(slot)).abs(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Food {
    pub nutrition: i32,
    pub can_always_eat: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack {
    pub slot: Slot,
    pub item: String,
    pub food: Option<Food>,
    pub instant_health: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub health: f32,
    pub hunger: i32,
    pub can_eat: bool,
    pub server_slot: u8,
    pub main_mining_tool: bool,
    pub using_food_or_drink: bool,
    /// Exactly Slots.OffhandWithHotbar: offhand, hotbar 0 through 8.
    pub stacks: Vec<Stack>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub requester: u64,
    pub slot: Slot,
    pub ticks: i32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    /// None means no pause call. Some(0) still calls the original service.
    pub pause_combat: Option<i32>,
    pub selection: Option<Selection>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoodError {
    /// Original emergency-apple branch uses foodComponent!! and aborts its
    /// handler if a command-created stack removed the FOOD component.
    EmergencyAppleWithoutFood,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SmartEat {
    pub swap_back_delay: i32,
    pub prefer_gapple_health: f32,
    pub prefer_notch_health: f32,
    pub prefer_health_pot_health: f32,
    pub combat_pause_time: i32,
    pub not_during_combat: bool,
    pub silent_offhand: bool,
    pub render_slot: bool,
    pub render_offset: i32,
    pub auto_eat: bool,
    pub min_hunger: i32,
    force_use_key: bool,
    wait_callback: Option<u64>,
    callback_counter: u64,
    wait_registrations: Vec<u64>,
}
impl Default for SmartEat {
    fn default() -> Self {
        Self {
            swap_back_delay: 5,
            prefer_gapple_health: 9.0,
            prefer_notch_health: 2.0,
            prefer_health_pot_health: 12.0,
            combat_pause_time: 0,
            not_during_combat: false,
            silent_offhand: true,
            render_slot: true,
            render_offset: 40,
            auto_eat: true,
            min_hunger: 15,
            force_use_key: false,
            wait_callback: None,
            callback_counter: 0,
            wait_registrations: Vec::new(),
        }
    }
}
impl SmartEat {
    fn estimate(&self, context: &Context, stack: &Stack) -> Result<Option<(i32, i32)>, FoodError> {
        // Preserve branch order: a non-healing POTION at the health threshold
        // is rejected even if command-created FOOD exists on the same stack.
        if context.health <= self.prefer_health_pot_health && stack.item == "minecraft:potion" {
            return Ok(stack
                .instant_health
                .then_some((self.prefer_health_pot_health as i32, 0)));
        }
        if context.health <= self.prefer_gapple_health && stack.item == "minecraft:golden_apple" {
            return Ok(Some((
                self.prefer_gapple_health as i32,
                stack
                    .food
                    .ok_or(FoodError::EmergencyAppleWithoutFood)?
                    .nutrition,
            )));
        }
        if context.health <= self.prefer_notch_health
            && stack.item == "minecraft:enchanted_golden_apple"
        {
            return Ok(Some((
                self.prefer_notch_health as i32,
                stack
                    .food
                    .ok_or(FoodError::EmergencyAppleWithoutFood)?
                    .nutrition,
            )));
        }
        Ok(stack.food.map(|food| (20, food.nutrition)))
    }
    pub fn try_best_food<'a>(&self, context: &'a Context) -> Result<Option<&'a Stack>, FoodError> {
        let mut best: Option<(&Stack, (i32, i32))> = None;
        for stack in &context.stacks {
            let Some(score) = self.estimate(context, stack)? else {
                continue;
            };
            let better = best.is_none_or(|(current, current_score)| {
                // Kotlin maxWithOrNull compares healthThreshold.inv(), hunger,
                // then PREFER_NEARBY in ascending order. The last comparator
                // compares the same serversideSlot twice. Equal values retain
                // the first candidate; an equal meal therefore prefers the
                // farther slot despite the source comment saying "closest".
                (!score.0)
                    .cmp(&(!current_score.0))
                    .then_with(|| score.1.cmp(&current_score.1))
                    .then_with(|| {
                        stack
                            .slot
                            .nearby(context.server_slot)
                            .cmp(&current.slot.nearby(context.server_slot))
                    })
                    == Ordering::Greater
            });
            if better {
                best = Some((stack, score));
            }
        }
        Ok(best.map(|(stack, _)| stack))
    }
    pub fn best_food<'a>(&self, context: &'a Context) -> Option<&'a Stack> {
        self.try_best_food(context).ok().flatten()
    }
    /// Called at MultiPlayerGameMode.useItem RETURN, for PASS only. A block's
    /// PASS and a failed Item.use must never be routed to this method.
    pub fn interacted_pass(&self, running: bool, combat: bool, context: &Context) -> Effects {
        if !running || !self.silent_offhand {
            return Effects::default();
        }
        let Some(food) = self.best_food(context) else {
            return Effects::default();
        };
        let needs_hunger = food.food.is_some_and(|food| !food.can_always_eat);
        if (!context.can_eat && needs_hunger)
            || (self.not_during_combat && combat)
            || !context.main_mining_tool
        {
            return Effects::default();
        }
        Effects {
            pause_combat: Some(self.combat_pause_time),
            selection: Some(Selection {
                requester: SILENT_OFFHAND_REQUESTER,
                slot: food.slot,
                ticks: self.swap_back_delay.max(5),
            }),
        }
    }
    /// CoroutineTicker runs existing waiters before ordinary GameTick hooks.
    /// Explicit listener cancellation calls cancel_auto_eat. A world change
    /// alone preserves it; a subsequent tick with no player aborts the wait.
    pub fn advance_waiter(&mut self, context: &Context) -> Effects {
        let Some(callback) = self.wait_callback else {
            return Effects::default();
        };
        self.advance_callback(callback, true, Some(context)).1
    }
    /// Register these immediately after start_run, before any Host effect or
    /// another module continuation. A cancelled registration stays observable.
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }
    /// Called only for the callback at this position in the global FIFO. The
    /// callback keeps its identity until tickUntil completes; a cancelled old
    /// callback cannot advance a replacement. running is checked on resume,
    /// not before stopAt, because the original AutoEat has no requires guard.
    pub fn advance_callback(
        &mut self,
        callback: u64,
        running: bool,
        context: Option<&Context>,
    ) -> (bool, Effects) {
        if self.wait_callback != Some(callback) {
            return (false, Effects::default());
        }
        let Some(context) = context else {
            self.wait_callback = None;
            return (false, Effects::default());
        };
        let food = match self.try_best_food(context) {
            Ok(food) => food,
            Err(_) => {
                self.wait_callback = None;
                return (false, Effects::default());
            }
        };
        let selection = food.map(|stack| Selection {
            requester: AUTO_EAT_REQUESTER,
            slot: stack.slot,
            ticks: self.swap_back_delay,
        });
        if selection.is_some() {
            self.force_use_key = true;
        }
        // eat() is called even on the terminating tick, and selection event
        // cancellation does not prevent forceUseKey becoming true.
        if context.hunger > self.min_hunger {
            self.wait_callback = None;
            if running && self.auto_eat {
                self.force_use_key = false;
            }
        }
        (
            self.wait_callback == Some(callback),
            Effects {
                pause_combat: None,
                selection,
            },
        )
    }
    pub fn silent_offhand_tick(
        &self,
        running: bool,
        owner: Option<u64>,
        context: &Context,
    ) -> Effects {
        if running
            && self.silent_offhand
            && context.using_food_or_drink
            && owner == Some(SILENT_OFFHAND_REQUESTER)
        {
            Effects {
                pause_combat: None,
                selection: Some(Selection {
                    requester: SILENT_OFFHAND_REQUESTER,
                    slot: Slot::Hotbar(context.server_slot),
                    ticks: self.swap_back_delay,
                }),
            }
        } else {
            Effects::default()
        }
    }
    /// Ordinary GameTick hook: Exclusive(DROP), with first eat next tick.
    pub fn start_run(&mut self, running: bool, combat: bool, context: &Context) -> Effects {
        if running
            && self.auto_eat
            && self.wait_callback.is_none()
            && context.hunger < self.min_hunger
            && !(self.not_during_combat && combat)
        {
            self.callback_counter = self
                .callback_counter
                .checked_add(1)
                .expect("SmartEat callback id exhausted");
            self.wait_callback = Some(self.callback_counter);
            self.wait_registrations.push(self.callback_counter);
            Effects {
                pause_combat: Some(self.combat_pause_time),
                selection: None,
            }
        } else {
            Effects::default()
        }
    }
    /// KeybindIsPressedEvent for Options.keyUse only. Each query pauses combat.
    pub fn use_key(&self, running: bool, pressed: &mut bool) -> Effects {
        if running && self.auto_eat && self.force_use_key {
            *pressed = true;
            Effects {
                pause_combat: Some(self.combat_pause_time),
                selection: None,
            }
        } else {
            Effects::default()
        }
    }
    pub fn render_food<'a>(&self, running: bool, context: &'a Context) -> Option<&'a Stack> {
        (running && self.silent_offhand && self.render_slot)
            .then(|| self.best_food(context))
            .flatten()
    }
    /// The original onDisabled resets its silent slot owner only; cancelling
    /// the child scope ends the wait, without clearing forceUseKey.
    pub fn cancel_auto_eat(&mut self) {
        self.wait_callback = None;
    }
    pub fn waiting(&self) -> bool {
        self.wait_callback.is_some()
    }
    pub fn force_use_key(&self) -> bool {
        self.force_use_key
    }
}

/// Original integer GUI coordinates and draw order: decorations, model, frame.
pub fn render_positions(width: i32, height: i32, offset: i32) -> ([i32; 2], [i32; 4]) {
    let center = width / 2;
    (
        [
            center.wrapping_sub(117).wrapping_sub(offset),
            height.wrapping_sub(19),
        ],
        [
            center.wrapping_sub(120).wrapping_sub(offset),
            height.wrapping_sub(23),
            29,
            24,
        ],
    )
}
