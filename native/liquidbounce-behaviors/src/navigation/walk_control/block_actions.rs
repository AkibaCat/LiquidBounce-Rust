// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete common BlockActions state machine. The pending placement map's
//! iteration is an explicit Java-map contract: native integration must supply
//! the original bucket/resize/tree identity order rather than Rust HashMap order.
use super::{
    block_targets::{self, BlockHit, BlockTargets, Hand, Plan, TargetHost},
    turning::LookPurpose,
    view_rotation::ViewRotation,
};
use crate::navigation::{
    aabb::{Aabb, Direction},
    geometry::{BlockPos, Vec3},
    terrain::TerrainState,
    walk::WalkState,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

/// Mirrors a JDK HashMap's key iteration, including its retained capacity after
/// clear, replacement keeping the original key, resize and tree-bin identity.
/// `inserted` is called only when a new equal key enters the pending map.
pub trait PlacementOrder {
    fn inserted(&mut self, pos: BlockPos);
    fn removed(&mut self, pos: BlockPos);
    fn clear(&mut self);
    fn keys(&self) -> Vec<BlockPos>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoubleHalf {
    Lower,
    Upper,
}
pub trait ActionHost: TargetHost {
    fn collision_empty(&self, state: &Self::State, pos: BlockPos) -> bool;
    fn double_half(&self, state: &Self::State) -> Option<DoubleHalf>;
    fn player_box(&self) -> Aabb;
    fn look(&mut self, rotation: ViewRotation, purpose: LookPurpose);
    fn select_hotbar_slot(&mut self, slot: i32);
    fn release_hotbar_slot(&mut self);
    fn start_destroy(&mut self, pos: BlockPos, face: Direction) -> bool;
    fn continue_destroy(&mut self, pos: BlockPos, face: Direction) -> bool;
    fn stop_destroy(&mut self);
    fn use_item_on(&mut self, hand: Hand, hit: BlockHit) -> bool;
    fn latency_millis(&self) -> i32;
    fn block_placed(&mut self, placed: &Arc<PlacedBlock>);
    fn block_kept(&mut self, placed: &Arc<PlacedBlock>);
    fn block_rejected(&mut self, placed: &Arc<PlacedBlock>);
    fn server_put_back_broken(&mut self, pos: BlockPos);
    fn server_took_back_placed(&mut self, pos: BlockPos);
    fn pick_face(&self, pos: BlockPos, face: Direction) -> Option<BlockHit> {
        let hit = self.pick(self.sent_rotation(), self.reach() + 1.0, None);
        (hit.is_block && hit.pos == pos && hit.direction == face).then_some(hit)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Verdict {
    Pending = 0,
    Kept = 1,
    Rejected = 2,
}
impl Verdict {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Kept => "kept",
            Self::Rejected => "rejected",
        }
    }
}
#[derive(Debug)]
pub struct PlacedBlock {
    pub pos: BlockPos,
    pub tick: i32,
    verdict: AtomicU8,
}
impl PlacedBlock {
    pub fn new(pos: BlockPos, tick: i32) -> Self {
        Self {
            pos,
            tick,
            verdict: AtomicU8::new(Verdict::Pending as u8),
        }
    }
    pub fn verdict(&self) -> Verdict {
        match self.verdict.load(Ordering::Relaxed) {
            0 => Verdict::Pending,
            1 => Verdict::Kept,
            2 => Verdict::Rejected,
            _ => unreachable!("valid placement verdict"),
        }
    }
}
#[derive(Clone, Debug)]
pub enum Action {
    Mine(BlockPos),
    Open(BlockPos),
    Place {
        pos: BlockPos,
        ready: bool,
        plan: Option<Plan>,
        held: Option<ViewRotation>,
    },
}
impl Action {
    pub fn pos(&self) -> BlockPos {
        match *self {
            Self::Mine(pos) | Self::Open(pos) | Self::Place { pos, .. } => pos,
        }
    }
}
pub struct BlockActions<O: PlacementOrder> {
    action: Option<Action>,
    pub rotation: Option<ViewRotation>,
    use_cooldown: i32,
    ticks: i32,
    digging: Option<BlockPos>,
    dug: HashMap<BlockPos, i32>,
    pub unbreakable: HashSet<i64>,
    unconfirmed: HashMap<BlockPos, Arc<PlacedBlock>>,
    unconfirmed_order: O,
    unconfirmed_opens: HashMap<BlockPos, i32>,
    pub placement: Option<Arc<PlacedBlock>>,
    pub rejected_in_row: i32,
    pub rejections: i32,
}
impl<O: PlacementOrder> BlockActions<O> {
    pub fn new(order: O) -> Self {
        Self {
            action: None,
            rotation: None,
            use_cooldown: 0,
            ticks: 0,
            digging: None,
            dug: HashMap::new(),
            unbreakable: HashSet::new(),
            unconfirmed: HashMap::new(),
            unconfirmed_order: order,
            unconfirmed_opens: HashMap::new(),
            placement: None,
            rejected_in_row: 0,
            rejections: 0,
        }
    }
    pub fn action(&self) -> Option<&Action> {
        self.action.as_ref()
    }
    pub fn is_active(&self) -> bool {
        self.action.is_some()
    }
    pub fn is_unconfirmed(&self, pos: BlockPos) -> bool {
        self.unconfirmed.contains_key(&pos)
    }
    pub fn is_open_unconfirmed(&self, pos: BlockPos) -> bool {
        self.unconfirmed_opens.contains_key(&pos)
    }
    pub fn forget_rejections(&mut self) {
        self.rejected_in_row = 0;
    }
    pub fn mine(&mut self, host: &mut impl ActionHost, pos: BlockPos) {
        if !matches!(self.action,Some(Action::Mine(current))if current==pos) {
            self.idle(host);
            self.action = Some(Action::Mine(pos));
        }
    }
    pub fn place(
        &mut self,
        host: &mut impl ActionHost,
        pos: BlockPos,
        ready: bool,
        held: Option<ViewRotation>,
    ) {
        if let Some(Action::Place {
            pos: current,
            ready: current_ready,
            held: current_held,
            ..
        }) = &mut self.action
            && *current == pos
        {
            *current_ready = ready;
            *current_held = held;
            return;
        }
        if !matches!(self.action, Some(Action::Place { .. })) {
            self.idle(host);
        }
        self.action = Some(Action::Place {
            pos,
            ready,
            plan: None,
            held,
        });
    }
    pub fn open(&mut self, host: &mut impl ActionHost, pos: BlockPos) {
        if !matches!(self.action, Some(Action::Open(_))) {
            self.idle(host);
        }
        self.action = Some(Action::Open(pos));
    }
    pub fn idle(&mut self, host: &mut impl ActionHost) {
        let Some(stopped) = self.action.take() else {
            return;
        };
        self.rotation = None;
        if matches!(stopped, Action::Mine(_)) && self.digging.is_some() {
            self.digging = None;
            host.stop_destroy();
        }
        host.release_hotbar_slot();
    }
    pub fn aim(&mut self, host: &mut impl ActionHost) {
        self.rotation = None;
        let Some(action) = self.action.as_mut() else {
            return;
        };
        let aim = match action {
            Action::Place {
                pos, plan, held, ..
            } => {
                if let Some(held) = held {
                    *held
                } else {
                    *plan = BlockTargets::placeable_slot(host)
                        .and_then(|slot| BlockTargets::plan_placement(host, *pos, slot, false));
                    let Some(plan) = plan else {
                        return;
                    };
                    plan.rotation
                }
            }
            Action::Open(pos) => {
                if host.state(*pos).open().unwrap_or(true) {
                    return;
                }
                let Some(aim) = BlockTargets::aim_at(host, *pos) else {
                    return;
                };
                aim
            }
            Action::Mine(pos) => {
                let Some(aim) = BlockTargets::aim_at(host, *pos) else {
                    return;
                };
                aim
            }
        };
        self.rotation = Some(aim);
        host.look(aim, LookPurpose::Aim);
    }
    pub fn tick(&mut self, host: &mut impl ActionHost) {
        self.ticks = self.ticks.wrapping_add(1);
        if self.use_cooldown > 0 {
            self.use_cooldown -= 1;
        }
        self.expire_verdicts(host);
        match self.action.clone() {
            Some(Action::Mine(pos)) => self.break_tick(host, pos),
            Some(Action::Open(pos)) => self.open_tick(host, pos),
            Some(Action::Place {
                pos,
                ready: true,
                plan,
                held,
            }) => {
                let plan = if held.is_some() {
                    BlockTargets::held_click(host, pos, host.sent_rotation())
                } else {
                    plan
                };
                if let Some(plan) = plan {
                    self.place_tick(host, plan);
                }
            }
            _ => {}
        }
    }
    fn break_tick(&mut self, host: &mut impl ActionHost, pos: BlockPos) {
        let state = host.state(pos);
        if host.collision_empty(&state, pos) {
            if self.digging == Some(pos) {
                self.dug.entry(pos).or_insert(self.ticks);
            }
            return;
        }
        if let Some(slot) = BlockTargets::digging_slot(host, &state) {
            host.select_hotbar_slot(slot);
        }
        let reach = host.reach() + 1.0;
        let hit = BlockTargets::raycast(host, host.sent_rotation(), reach);
        if !hit.is_block || hit.pos != pos {
            return;
        }
        if self.digging == Some(pos) {
            host.continue_destroy(pos, hit.direction);
        } else {
            self.digging = Some(pos);
            host.start_destroy(pos, hit.direction);
        }
    }
    fn open_tick(&mut self, host: &mut impl ActionHost, pos: BlockPos) {
        let state = host.state(pos);
        if self.use_cooldown > 0 || state.open().unwrap_or(true) {
            return;
        }
        let hit = BlockTargets::raycast(host, host.sent_rotation(), host.reach());
        if hit.is_block && Self::is_part_of(host, hit.pos, pos, &state) {
            if host.use_item_on(Hand::Main, hit) {
                self.unconfirmed_opens.insert(pos, self.ticks);
            }
            self.use_cooldown = 10;
        }
    }
    fn is_part_of<H: ActionHost>(host: &H, hit: BlockPos, pos: BlockPos, state: &H::State) -> bool {
        if hit == pos {
            true
        } else if hit == pos.above() {
            host.double_half(state) == Some(DoubleHalf::Lower)
        } else if hit == pos.offset(0, -1, 0) {
            host.double_half(state) == Some(DoubleHalf::Upper)
        } else {
            false
        }
    }
    fn place_tick(&mut self, host: &mut impl ActionHost, plan: Plan) {
        let block_box = Aabb::new(
            Vec3::new(
                f64::from(plan.pos.x),
                f64::from(plan.pos.y),
                f64::from(plan.pos.z),
            ),
            Vec3::new(
                f64::from(plan.pos.x) + 1.0,
                f64::from(plan.pos.y) + 1.0,
                f64::from(plan.pos.z) + 1.0,
            ),
        );
        if host.player_box().intersects(block_box) {
            return;
        }
        let Some(hit) = host.pick_face(plan.against, plan.face) else {
            return;
        };
        if plan.slot >= 0 {
            host.select_hotbar_slot(plan.slot);
        }
        if host.use_item_on(plan.hand(), hit) {
            let placed = Arc::new(PlacedBlock::new(
                block_targets::relative(hit.pos, hit.direction),
                self.ticks,
            ));
            if self
                .unconfirmed
                .insert(placed.pos, placed.clone())
                .is_none()
            {
                self.unconfirmed_order.inserted(placed.pos);
            }
            self.placement = Some(placed.clone());
            host.block_placed(&placed);
        }
    }
    pub fn answer<H: ActionHost>(&mut self, host: &mut H, pos: BlockPos, state: &H::State) -> bool {
        if let Some(placed) = self.unconfirmed.remove(&pos) {
            self.unconfirmed_order.removed(pos);
            self.judge(host, &placed, !state.can_be_replaced());
        }
        self.unconfirmed_opens.remove(&pos);
        if self.dug.remove(&pos).is_none() || state.can_be_replaced() {
            return false;
        }
        self.unbreakable.insert(pos.packed());
        host.server_put_back_broken(pos);
        true
    }
    fn expire_verdicts(&mut self, host: &mut impl ActionHost) {
        if self.unconfirmed.is_empty() && self.unconfirmed_opens.is_empty() && self.dug.is_empty() {
            return;
        }
        let timeout = 10_i32.wrapping_add((host.latency_millis() / 50).wrapping_mul(2));
        self.unconfirmed_opens
            .retain(|_, tick| self.ticks.wrapping_sub(*tick) <= timeout);
        self.dug
            .retain(|_, tick| self.ticks.wrapping_sub(*tick) <= timeout);
        for pos in self.unconfirmed_order.keys() {
            let placed = self
                .unconfirmed
                .get(&pos)
                .expect("placement order must match pending map");
            if self.ticks.wrapping_sub(placed.tick) > timeout {
                let placed = self.unconfirmed.remove(&pos).unwrap();
                self.unconfirmed_order.removed(pos);
                let kept = !host.state(placed.pos).can_be_replaced();
                self.judge(host, &placed, kept);
            }
        }
    }
    fn judge(&mut self, host: &mut impl ActionHost, placed: &Arc<PlacedBlock>, kept: bool) {
        placed.verdict.store(
            if kept {
                Verdict::Kept
            } else {
                Verdict::Rejected
            } as u8,
            Ordering::Relaxed,
        );
        self.rejected_in_row = if kept {
            0
        } else {
            self.rejected_in_row.wrapping_add(1)
        };
        if kept {
            host.block_kept(placed);
        } else {
            self.rejections = self.rejections.wrapping_add(1);
            host.server_took_back_placed(placed.pos);
            host.block_rejected(placed);
        }
    }
    pub fn level_changed(&mut self) {
        self.action = None;
        self.rotation = None;
        self.digging = None;
        self.unconfirmed.clear();
        self.unconfirmed_order.clear();
        self.unconfirmed_opens.clear();
        self.dug.clear();
        self.unbreakable.clear();
        self.placement = None;
        self.rejected_in_row = 0;
    }
    pub fn timing(&self) -> (i32, i32, Option<BlockPos>) {
        (self.ticks, self.use_cooldown, self.digging)
    }
}
impl<O: PlacementOrder> super::step_check::ActionConfirmation for BlockActions<O> {
    fn is_unconfirmed(&self, pos: BlockPos) -> bool {
        self.is_unconfirmed(pos)
    }
}
