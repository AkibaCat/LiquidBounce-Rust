// SPDX-License-Identifier: GPL-3.0-or-later
//! The single-chain portion of InventoryManager used by AnchoredHotbarSwapController.
//! Time progresses on frames as well as ticks; delays are never rounded to ticks.
use super::{InventoryConstraints, RequirementState, SwapAction};

pub trait Random {
    fn int(&mut self, range: [i32; 2]) -> Result<i32, String>;
    fn float(&mut self) -> f32;
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Context {
    pub now: i64,
    pub player_inventory: bool,
    pub server_controlled: bool,
    pub cursor_empty: bool,
    pub inventory_screen: bool,
    pub inventory_open: bool,
    pub requirements: RequirementState,
    /// All 46 inventory-menu slots plus the carried stack, not only hotbar.
    pub menu_identity: String,
    /// Cursor relative to the inventory menu while a visible screen owns it.
    pub visible_cursor: Option<[f32; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    Swap(SwapAction),
    Close,
    MoveCursor([f32; 2]),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Waiting,
    Start,
    Travel,
    Click,
}
#[derive(Clone, Debug, PartialEq)]
struct Pending {
    action: SwapAction,
    menu_identity: String,
    phase: Phase,
    until: i64,
    travel_start: i64,
    travel_from: [f32; 2],
    travel_to: [f32; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackRecord {
    pub identity: String,
    pub components: String,
    pub count: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct SentClick {
    slot: i32,
    stack: StackRecord,
    moving: bool,
    restored: bool,
    until: i64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Executor {
    pending: Option<Pending>,
    pub cursor: Option<[f32; 2]>,
    pub inventory_open_server_side: bool,
    recent_open: bool,
    next_chain: i64,
    close_after: Option<i64>,
    maximum_close_delay: i32,
    finish_after_click: Option<InventoryConstraints>,
    container_wait: i64,
    use_wait: i64,
    revert_wait: i64,
    recent_clicks: Vec<SentClick>,
    rejections: Vec<(String, i32)>,
    locked: Vec<String>,
    pub response_time: i32,
    pub lock_after_rejections: i32,
}
impl Default for Executor {
    fn default() -> Self {
        Self {
            pending: None,
            cursor: None,
            inventory_open_server_side: false,
            recent_open: false,
            next_chain: 0,
            close_after: None,
            maximum_close_delay: 0,
            finish_after_click: None,
            container_wait: 0,
            use_wait: 0,
            revert_wait: 0,
            recent_clicks: Vec::new(),
            rejections: Vec::new(),
            locked: Vec::new(),
            response_time: 400,
            lock_after_rejections: 3,
        }
    }
}
impl Executor {
    pub fn active(&self) -> bool {
        self.pending.is_some()
    }
    pub fn locked_count(&self) -> usize {
        self.locked.len()
    }
    pub fn is_locked(&self, stack: &StackRecord) -> bool {
        stack.count > 0 && self.locked.contains(&stack.components)
    }
    pub fn cancel(&mut self) {
        self.pending = None;
    }
    pub fn world_change(&mut self) {
        *self = Self::default();
    }
    pub fn opened(&mut self) {
        self.recent_open = true;
    }
    pub fn container_opened(&mut self) {
        self.container_wait = 0;
    }
    pub fn may_open_container(&mut self, now: i64) {
        self.container_wait = now + i64::from(self.response_time);
    }
    pub fn item_used(&mut self, now: i64) {
        self.use_wait = now + i64::from(self.response_time);
    }
    pub fn closed(&mut self) {
        self.inventory_open_server_side = false;
        self.cursor = None;
    }
    pub fn click_sent(&mut self) {
        if !self.inventory_open_server_side {
            self.recent_open = true;
        }
        self.inventory_open_server_side = true;
    }
    pub fn poll(&mut self, now: i64) {
        self.expire(now);
    }
    pub fn submit(&mut self, action: SwapAction, context: &Context) {
        self.pending = Some(Pending {
            action,
            menu_identity: context.menu_identity.clone(),
            phase: Phase::Waiting,
            until: 0,
            travel_start: 0,
            travel_from: [0.0; 2],
            travel_to: [0.0; 2],
        });
    }
    pub fn can_perform(&self, context: &Context, constraints: &InventoryConstraints) -> bool {
        context.player_inventory
            && !context.server_controlled
            && context.cursor_empty
            && context.now >= self.container_wait
            && context.now >= self.use_wait
            && context.now >= self.revert_wait
            && !self.recent_clicks.iter().any(|click| click.restored)
            && constraints.passes(context.requirements)
    }
    /// Inventory menu coordinates, shared by visible and silent inventory clicks.
    pub fn slot_position(slot: u8) -> [f32; 2] {
        [
            8.0 + f32::from(slot % 9) * 18.0,
            if slot < 9 {
                142.0
            } else {
                84.0 + f32::from((slot - 9) / 9) * 18.0
            },
        ]
    }
    /// Bounds of all 46 native InventoryMenu slots, including the result slot.
    pub fn menu_center() -> [f32; 2] {
        [89.0, 83.0]
    }
    pub fn frame(
        &mut self,
        context: &Context,
        constraints: &InventoryConstraints,
        guards: bool,
        random: &mut impl Random,
    ) -> Result<Option<Operation>, String> {
        self.expire(context.now);
        // perform() samples CloseDelay after its suspended ClickDelay, even
        // if another chain has been requested in the meantime.
        if context.now >= self.next_chain
            && let Some(finished) = self.finish_after_click.take()
        {
            self.maximum_close_delay = self
                .maximum_close_delay
                .max(random.int(finished.close_delay)?.max(0));
        }
        if self.pending.is_some()
            && (!guards
                || self
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.menu_identity != context.menu_identity))
        {
            self.cancel();
        }
        if context.now < self.next_chain {
            return Ok(None);
        }
        // The coroutine's close pause continues even if a new request arrives.
        if let Some(until) = self.close_after {
            if context.now < until {
                return Ok(None);
            }
            self.close_after = None;
        }
        let can = self.can_perform(context, constraints);
        if self.pending.is_none()
            || (self
                .pending
                .as_ref()
                .is_some_and(|p| p.phase == Phase::Waiting)
                && !can)
        {
            if self.inventory_open_server_side
                && !context.inventory_screen
                && context.player_inventory
            {
                if self.maximum_close_delay > 0 {
                    self.close_after = Some(context.now + i64::from(self.maximum_close_delay));
                    self.maximum_close_delay = 0;
                    return Ok(None);
                }
                self.closed();
                return Ok(Some(Operation::Close));
            }
            self.maximum_close_delay = 0;
            return Ok(None);
        }
        let pending = self.pending.as_mut().expect("pending checked");
        match pending.phase {
            Phase::Waiting => {
                if !can {
                    return Ok(None);
                }
                // A real InventoryScreen open takes its start delay too.
                let start = if self.recent_open
                    || !(context.inventory_open || self.inventory_open_server_side)
                {
                    self.recent_open = false;
                    random.int(constraints.start_delay)?.max(0)
                } else {
                    0
                };
                pending.until = context.now + i64::from(start);
                pending.phase = Phase::Start;
            }
            Phase::Start | Phase::Travel | Phase::Click => {}
        }
        let pending = self.pending.as_mut().expect("pending checked");
        if context.now < pending.until && pending.phase != Phase::Travel {
            return Ok(None);
        }
        if pending.phase == Phase::Start {
            // performMissClick deliberately has no effect on InventoryItemSlot.
            // Kotlin still samples its probability and Random.nextInt(100).
            let _miss = random.int(constraints.miss_chance)? > random.int([0, 99])?;
            let from = context
                .visible_cursor
                .or(self.cursor)
                .unwrap_or_else(Self::menu_center);
            let position = Self::slot_position(pending.action.source);
            let to = [
                position[0] + 2.0 + random.float() * 12.0,
                position[1] + 2.0 + random.float() * 12.0,
            ];
            let distance = ((from[0] - to[0]).powi(2) + (from[1] - to[1]).powi(2)).sqrt() / 18.0;
            let delay = if self.cursor.is_none() {
                constraints.start_delay
            } else {
                constraints.click_delay
            };
            let millis =
                (random.int(delay)? as f32 * distance * constraints.slot_multiplier) as i32;
            self.cursor = Some(to);
            pending.travel_start = context.now;
            pending.travel_from = from;
            pending.travel_to = to;
            pending.until = context.now + i64::from(millis.max(0));
            pending.phase = Phase::Travel;
        }
        if pending.phase == Phase::Travel && context.now < pending.until {
            if context.visible_cursor.is_some() {
                let progress = ((context.now - pending.travel_start) as f32
                    / (pending.until - pending.travel_start) as f32)
                    .min(1.0);
                let progress = progress * progress * (3.0 - 2.0 * progress);
                return Ok(Some(Operation::MoveCursor(std::array::from_fn(|i| {
                    pending.travel_from[i]
                        + (pending.travel_to[i] - pending.travel_from[i]) * progress
                }))));
            }
            return Ok(None);
        }
        if !can {
            self.cancel();
            return Ok(None);
        }
        pending.phase = Phase::Click;
        Ok(Some(Operation::Swap(pending.action)))
    }
    /// Invoked immediately after the actual menu click is predicted/sent.
    pub fn performed(
        &mut self,
        context: &Context,
        constraints: &InventoryConstraints,
        source: StackRecord,
        succeeded: bool,
        random: &mut impl Random,
    ) -> Result<(), String> {
        let pending = self.pending.take();
        if !succeeded {
            return Ok(());
        }
        self.click_sent();
        if source.count > 0
            && let Some(pending) = pending
        {
            self.recent_clicks.push(SentClick {
                slot: i32::from(pending.action.source),
                stack: source,
                moving: !(context.requirements.movement[0] * context.requirements.movement[0]
                    + context.requirements.movement[1] * context.requirements.movement[1])
                    .abs()
                    .lt(&1e-5)
                    || context.requirements.jumping,
                restored: false,
                until: context.now + i64::from(self.response_time),
            });
        }
        self.next_chain = context.now + i64::from(random.int(constraints.click_delay)?.max(0));
        self.finish_after_click = Some(constraints.clone());
        Ok(())
    }
    /// A ContainerSetContent/ContainerSetSlot result, converted through to_stack.
    pub fn resync(&mut self, slot: i32, stack: &StackRecord, now: i64) {
        for click in &mut self.recent_clicks {
            if click.slot != slot {
                continue;
            }
            let restored =
                stack.components == click.stack.components && stack.count >= click.stack.count;
            if restored && !click.restored {
                click.restored = true;
                click.until = now + i64::from(self.response_time);
            }
        }
        self.recent_clicks.retain(|click| {
            click.slot != slot
                || (stack.components == click.stack.components && stack.count >= click.stack.count)
        });
    }
    pub fn resync_content(&mut self, slots: &[(i32, StackRecord)], now: i64) {
        let expected: Vec<_> = self.recent_clicks.iter().map(|click| click.slot).collect();
        let empty = StackRecord {
            identity: "empty".to_owned(),
            components: "empty".to_owned(),
            count: 0,
        };
        for slot in expected {
            let stack = slots
                .iter()
                .find(|(index, _)| *index == slot)
                .map_or(&empty, |(_, stack)| stack);
            self.resync(slot, stack, now);
        }
    }
    fn expire(&mut self, now: i64) {
        let mut rejected = Vec::new();
        self.recent_clicks.retain(|click| {
            if now < click.until {
                return true;
            }
            if click.restored {
                rejected.push((click.stack.components.clone(), click.moving));
            }
            false
        });
        for (components, moving) in rejected {
            self.revert_wait = now + i64::from(self.response_time);
            if moving || self.locked.contains(&components) {
                continue;
            }
            let row =
                if let Some(index) = self.rejections.iter().position(|(id, _)| *id == components) {
                    &mut self.rejections[index]
                } else {
                    self.rejections.push((components.clone(), 0));
                    self.rejections.last_mut().expect("pushed")
                };
            row.1 += 1;
            if row.1 >= self.lock_after_rejections && !self.locked.contains(&components) {
                self.locked.push(components);
            }
        }
    }
}
