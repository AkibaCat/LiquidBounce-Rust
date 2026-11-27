// SPDX-License-Identifier: GPL-3.0-or-later
//! InventoryPlan/InventoryAction/InventoryManager at ae0eb396. The caller owns
//! real menu operations and synchronous continuation tails; one shared Manager
//! must serve every module. Never invoke Host while this state is borrowed.

use crate::{random::ThreadRandom, vomit::Constraints};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stack {
    /// Item identity plus resolved default/data components; count excluded.
    pub components: String,
    pub count: i32,
    /// Exact native ItemStack codec compound, retained for Creative(copy).
    /// Matching and server-lock identity use resolved components, not this patch.
    pub wire: String,
}
impl Stack {
    pub fn empty(&self) -> bool {
        self.count <= 0
    }
    pub fn same_item(&self, other: &Self) -> bool {
        !self.empty() && !other.empty() && self.components == other.components
    }
    fn matches(&self, other: &Self) -> bool {
        (self.empty() && other.empty()) || (self.count == other.count && self.same_item(other))
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuState {
    pub container_id: i32,
    pub stacks: Vec<Stack>,
    pub carried: Stack,
}
impl MenuState {
    pub fn matches(&self, other: &Self) -> bool {
        self.container_id == other.container_id
            && self.carried.matches(&other.carried)
            && self.stacks.len() == other.stacks.len()
            && self
                .stacks
                .iter()
                .zip(&other.stacks)
                .all(|(a, b)| a.matches(b))
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    fn distance(self, other: Self) -> f32 {
        ((self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)).sqrt()
    }
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// Stable actual AbstractContainerMenu object identity, not container id.
    pub object: u64,
    pub slots: Vec<Position>,
}
impl Geometry {
    pub fn center(&self) -> Option<Position> {
        let first = *self.slots.first()?;
        let mut min = first;
        let mut max = first;
        for p in &self.slots {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Some(Position {
            x: (min.x + max.x) / 2.0 + 8.0,
            y: (min.y + max.y) / 2.0 + 8.0,
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Screen {
    /// Stable actual Screen object identity (different from menu identity).
    pub object: u64,
    pub container_id: i32,
    pub geometry: Geometry,
    pub non_player_slots: Vec<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot {
    /// Concrete slot in a captured container menu.
    Container(i32),
    /// Native player inventory id 0..35, armor 36..39, offhand 40.
    Player(u8),
}
impl Slot {
    pub fn server_id(self, screen: Option<&Screen>) -> Option<i32> {
        match self {
            Self::Container(id) => Some(id),
            Self::Player(slot) if slot < 9 => {
                Some(screen.map_or(36, |s| s.geometry.slots.len() as i32 - 9) + i32::from(slot))
            }
            Self::Player(slot) if slot < 36 => {
                Some(screen.map_or(9, |s| s.geometry.slots.len() as i32 - 36) + i32::from(slot) - 9)
            }
            Self::Player(slot) if (36..40).contains(&slot) => {
                screen.is_none().then_some(44 - i32::from(slot))
            }
            Self::Player(40) => screen.is_none().then_some(45),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreativeStack {
    Existing(Stack),
    /// Resolve Block.asItem (including air/no-item blocks) and construct a fresh
    /// default ItemStack with this count. The chain shares this chosen block.
    Block {
        registry_index: i32,
        count: i32,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Click {
        screen: Option<Screen>,
        slot: Slot,
        button: i32,
        input: Input,
    },
    Creative {
        stack: CreativeStack,
        slot: Option<Slot>,
    },
    UseItem {
        slot: u8,
    },
    CloseScreen {
        container_id: i32,
    },
}
impl Action {
    pub fn requires_player_inventory(&self) -> bool {
        matches!(self, Self::Click { screen: None, .. })
    }
    fn click(&self) -> Option<(Option<&Screen>, Slot, i32, Input)> {
        if let Self::Click {
            screen,
            slot,
            button,
            input,
        } = self
        {
            Some((screen.as_ref(), *slot, *button, *input))
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Requirements {
    pub movement: [f32; 2],
    pub jumping: bool,
    pub rotation_matches: bool,
    pub using_item: bool,
    pub breaking: Option<bool>,
    pub combat: bool,
}
impl Requirements {
    fn no_movement(self) -> bool {
        (self.movement[0] * self.movement[0] + self.movement[1] * self.movement[1]).abs() < 1e-5
            && !self.jumping
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Context {
    pub now: i64,
    pub in_game: bool,
    pub creative: bool,
    pub menu: MenuState,
    pub inventory_menu: Geometry,
    pub screen: Option<Screen>,
    pub inventory_screen: bool,
    pub container_screen: bool,
    pub no_screen: bool,
    pub server_controlled_inventory: bool,
    pub requirements: Requirements,
    /// BetterInventory move-cursor enabled, mouse released, screen not hidden.
    pub move_visible_cursor: bool,
    pub visible_cursor: Position,
    /// Player inventory indexed 0..40; includes all actual components/counts.
    pub player_stacks: Vec<Stack>,
}
impl Context {
    fn stack(&self, slot: Slot) -> Stack {
        match slot {
            Slot::Player(index) => self.player_stacks.get(usize::from(index)),
            Slot::Container(index) => usize::try_from(index)
                .ok()
                .and_then(|i| self.menu.stacks.get(i)),
        }
        .cloned()
        .unwrap_or_default()
    }
}
#[derive(Clone, Debug)]
pub struct OwnerState {
    pub running: bool,
    /// Other original `requires` guards; manager adds the exact expected-menu
    /// and locked-count guard. Checked only while this request is pending.
    pub guards_pass: bool,
    pub constraints: Constraints,
}
#[derive(Clone, Debug)]
pub struct Request {
    pub id: u64,
    pub owner: u64,
    pub priority: i32,
    pub actions: Vec<Action>,
    expected: MenuState,
    locked_count: usize,
}
impl Request {
    pub fn new(
        id: u64,
        owner: u64,
        priority: i32,
        actions: Vec<Action>,
        context: &Context,
        manager: &Manager,
    ) -> Self {
        Self {
            id,
            owner,
            priority,
            actions,
            expected: context.menu.clone(),
            locked_count: manager.locked_count(),
        }
    }
    fn requires_player_inventory(&self) -> bool {
        self.actions.iter().any(Action::requires_player_inventory)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Completed { request: u64 },
    Cancelled { request: u64 },
    Performed { request: u64, action: Action },
}
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// For native 26.3 without ViaFabricPlus this is a no-op. Its associated
    /// source start delay/wait still happens. Do not set open-server-side here.
    OpenInventory,
    CloseInventory,
    MoveCursor(Position),
    Action(Action),
    Outcome(Outcome),
}
#[derive(Clone, Debug)]
pub struct Emission {
    pub ticket: u64,
    pub effect: Effect,
}
#[derive(Clone, Debug)]
struct SentClick {
    id: u64,
    container: i32,
    slot: i32,
    input: Input,
    stack: Stack,
    moving: bool,
    restored: bool,
    until: i64,
}
#[derive(Clone, Debug)]
struct Chain {
    request: Request,
    index: usize,
    completed: bool,
}
#[derive(Clone, Debug)]
struct PlanState {
    expected: MenuState,
    locked_count: usize,
    moved: std::collections::BTreeMap<Slot, Slot>,
}
#[derive(Clone, Debug)]
struct Travel {
    screen: u64,
    from: Position,
    to: Position,
    start: i64,
    millis: i32,
    woke: bool,
}
#[derive(Clone, Debug)]
enum Phase {
    Idle,
    Recover,
    RecoverPickup(Slot),
    Loop,
    Recent,
    Open,
    AfterOpenDelay,
    Miss,
    MissPause,
    CloseScreen,
    TravelStart,
    Travel(Travel),
    Ready,
    AfterAction { success: bool },
    AfterActionPause,
    FinishChain,
    CloseAfterPause,
    Done,
    TickWait { id: u64, resume: Box<Phase> },
}
#[derive(Clone, Debug)]
struct Inflight {
    emission: Emission,
    action: Option<Action>,
    record: Option<SentClick>,
    recovery: bool,
}
#[derive(Clone, Debug)]
pub struct Manager {
    requests: Vec<Request>,
    plans: std::collections::BTreeMap<u64, PlanState>,
    chain: Option<Chain>,
    phase: Phase,
    pause_until: Option<i64>,
    inflight: Option<Inflight>,
    outcomes: std::collections::VecDeque<Outcome>,
    pub cursor: Option<Position>,
    pub inventory_open_server_side: bool,
    recent_open: bool,
    pub last_clicked_slot: i32,
    maximum_close_delay: i32,
    container_wait: Option<i64>,
    use_wait: Option<i64>,
    revert_wait: Option<i64>,
    recent_clicks: Vec<SentClick>,
    rejections: Vec<(Stack, i32)>,
    locked: Vec<Stack>,
    pub response_time: i32,
    pub lock_after_rejections: i32,
    serial: u64,
    wait_registrations: Vec<u64>,
}
impl Default for Manager {
    fn default() -> Self {
        Self {
            requests: vec![],
            plans: Default::default(),
            chain: None,
            phase: Phase::Idle,
            pause_until: None,
            inflight: None,
            outcomes: Default::default(),
            cursor: None,
            inventory_open_server_side: false,
            recent_open: false,
            last_clicked_slot: -1,
            maximum_close_delay: 0,
            container_wait: None,
            use_wait: None,
            revert_wait: None,
            recent_clicks: vec![],
            rejections: vec![],
            locked: vec![],
            response_time: 400,
            lock_after_rejections: 3,
            serial: 0,
            wait_registrations: vec![],
        }
    }
}
impl Manager {
    fn serial(&mut self) -> u64 {
        self.serial = self.serial.wrapping_add(1);
        self.serial
    }
    pub fn busy(&self) -> bool {
        !matches!(self.phase, Phase::Idle)
    }
    pub fn pending(&self) -> usize {
        self.requests.len()
    }
    /// Earliest render-thread delay/timeout still scheduled after `now`.
    /// The host must resume on its executor between ticks, not round to ticks.
    pub fn next_wakeup(&self, now: i64) -> Option<i64> {
        self.pause_until
            .into_iter()
            .chain(self.container_wait)
            .chain(self.use_wait)
            .chain(self.revert_wait)
            .chain(self.recent_clicks.iter().map(|c| c.until))
            .filter(|at| *at > now)
            .min()
    }
    pub fn locked_count(&self) -> usize {
        self.locked.len()
    }
    pub fn is_locked(&self, stack: &Stack) -> bool {
        self.locked.iter().any(|old| old.same_item(stack))
    }
    /// Original nearestFirst: stable greedy order, using inventory-menu slot
    /// origins (not randomized click points), and removing one equal entry.
    pub fn nearest_first<T: Clone>(
        &self,
        items: &[T],
        geometry: &Geometry,
        slot: impl Fn(&T) -> Slot,
    ) -> Result<Vec<T>, &'static str> {
        let mut from = self
            .cursor
            .or_else(|| geometry.center())
            .ok_or("empty inventory geometry")?;
        let mut left: Vec<_> = items.iter().collect();
        let mut ordered = Vec::with_capacity(left.len());
        while !left.is_empty() {
            let position = |item: &T| {
                slot(item)
                    .server_id(None)
                    .and_then(|id| usize::try_from(id).ok())
                    .and_then(|id| geometry.slots.get(id))
                    .copied()
            };
            let distance_from = |item: &T| {
                position(item).map_or(f32::MAX, |p| {
                    let dx = p.x - from.x;
                    let dy = p.y - from.y;
                    dx * dx + dy * dy
                })
            };
            let mut nearest = 0;
            // Kotlin minBy starts with the first item, then Float.compare
            // orders finite unresolved-slot MAX_VALUE before an overflowed
            // infinite distance and treats NaN as larger than every number.
            let mut distance = distance_from(left[0]);
            for (index, item) in left.iter().enumerate().skip(1) {
                let d = distance_from(item);
                if !d.is_nan() && (distance.is_nan() || d < distance) {
                    nearest = index;
                    distance = d;
                }
            }
            let next = left.remove(nearest);
            if let Some(p) = position(next) {
                from = p;
            }
            ordered.push(next.clone());
        }
        Ok(ordered)
    }
    pub fn inventory_open(&self, c: &Context) -> bool {
        c.inventory_screen || self.inventory_open_server_side
    }
    fn can_close(&self, c: &Context) -> bool {
        !c.inventory_screen && c.menu.container_id == 0 && self.inventory_open(c)
    }
    pub fn opened(&mut self) {
        self.recent_open = true;
    }
    pub fn set_open_server_side(&mut self, open: bool) {
        if open && !self.inventory_open_server_side {
            self.opened();
        }
        if !open {
            self.cursor = None;
        }
        self.inventory_open_server_side = open;
    }
    pub fn sent_click(&mut self, container: i32) {
        if container == 0 {
            self.set_open_server_side(true);
        }
    }
    pub fn may_open_container(&mut self, now: i64) {
        self.container_wait = Some(now + i64::from(self.response_time));
    }
    pub fn container_opened(&mut self) {
        self.container_wait = None;
        self.set_open_server_side(false);
    }
    /// READ_FINAL_STATE close handler cancels an already-closed player close.
    /// Call before packet acceptance; return false to cancel that normal event.
    pub fn outgoing_close(&mut self, container: i32) -> bool {
        self.recent_clicks.retain(|click| click.container == 0);
        if container == 0 && !self.inventory_open_server_side {
            return false;
        }
        self.set_open_server_side(false);
        true
    }
    pub fn incoming_close(&mut self) {
        self.recent_clicks.retain(|click| click.container == 0);
        self.set_open_server_side(false);
    }
    pub fn item_used(&mut self, now: i64) {
        self.use_wait = Some(now + i64::from(self.response_time));
    }
    fn wait_active(wait: Option<i64>, now: i64) -> bool {
        wait.is_some_and(|until| now < until)
    }
    pub fn container_open_pending(&self, now: i64) -> bool {
        Self::wait_active(self.container_wait, now)
    }
    pub fn use_item_pending(&self, now: i64) -> bool {
        Self::wait_active(self.use_wait, now)
    }
    pub fn revert_pending(&self, now: i64) -> bool {
        Self::wait_active(self.revert_wait, now) || self.recent_clicks.iter().any(|c| c.restored)
    }
    pub fn can_click_player_inventory(&self, c: &Context) -> bool {
        c.menu.container_id == 0
            && !c.server_controlled_inventory
            && !Self::wait_active(self.container_wait, c.now)
            && !Self::wait_active(self.use_wait, c.now)
            && !self.revert_pending(c.now)
    }
    fn passes(&self, action: &Action, c: &Context, constraints: &Constraints) -> bool {
        let r = c.requirements;
        let values = [
            r.no_movement(),
            r.rotation_matches,
            !r.using_item,
            r.breaking == Some(false),
            !r.combat,
            !action.requires_player_inventory() || self.inventory_open(c),
        ];
        values
            .iter()
            .enumerate()
            .all(|(i, v)| constraints.requirements & (1 << i) == 0 || *v)
    }
    pub fn can_action(&self, action: &Action, c: &Context, constraints: &Constraints) -> bool {
        match action {
            Action::Click { screen, input, .. } => {
                if !self.passes(action, c, constraints) {
                    return false;
                }
                if !matches!(input, Input::Pickup | Input::PickupAll) && !c.menu.carried.empty() {
                    return false;
                }
                match screen {
                    None => self.can_click_player_inventory(c),
                    Some(screen) => c
                        .screen
                        .as_ref()
                        .is_some_and(|s| s.container_id == screen.container_id),
                }
            }
            Action::Creative { .. } => self.passes(action, c, constraints) && c.creative,
            Action::UseItem { .. } => {
                !self.inventory_open(c) && !c.container_screen && !c.inventory_screen
            }
            Action::CloseScreen { container_id } => c.menu.container_id == *container_id,
        }
    }
    fn can_chain(&self, request: &Request, c: &Context, constraints: &Constraints) -> bool {
        request
            .actions
            .iter()
            .all(|a| self.can_action(a, c, constraints))
    }
    /// Reuse the same id for consecutive perform() calls in one plan. End the
    /// plan explicitly once its coroutine body returns, after handling its
    /// Completed outcome, so expected stacks and slotOf mappings survive.
    pub fn submit(&mut self, mut request: Request) {
        let plan = self.plans.entry(request.id).or_insert_with(|| PlanState {
            expected: request.expected.clone(),
            locked_count: request.locked_count,
            moved: Default::default(),
        });
        request.expected = plan.expected.clone();
        request.locked_count = plan.locked_count;
        if request.actions.is_empty() {
            self.outcomes.push_back(Outcome::Completed {
                request: request.id,
            });
        } else {
            self.requests.push(request);
        }
    }
    pub fn end_plan(&mut self, id: u64) {
        self.plans.remove(&id);
    }
    pub fn slot_of(&self, id: u64, slot: Slot) -> Slot {
        self.plans
            .get(&id)
            .and_then(|p| p.moved.get(&slot))
            .copied()
            .unwrap_or(slot)
    }
    pub fn cancel_pending(&mut self, owner: u64) {
        let mut cancelled = vec![];
        self.requests.retain(|r| {
            if r.owner == owner {
                cancelled.push(r.id);
                false
            } else {
                true
            }
        });
        self.outcomes.extend(
            cancelled
                .into_iter()
                .map(|request| Outcome::Cancelled { request }),
        );
    }
    pub fn world_change(&mut self) {
        let ids = self.requests.iter().map(|r| r.id).collect::<Vec<_>>();
        self.requests.clear();
        self.outcomes.extend(
            ids.into_iter()
                .map(|request| Outcome::Cancelled { request }),
        );
        self.set_open_server_side(false);
        self.container_wait = None;
        self.use_wait = None;
        self.revert_wait = None;
        self.recent_clicks.clear();
        self.rejections.clear();
        self.locked.clear();
        // The source world handler ends pending plan continuations; it does
        // not cancel InventoryManager's own already executing atomic chain.
    }
    pub fn begin_tick(&mut self, c: &Context) {
        if self.busy() {
            return;
        }
        if !c.in_game {
            let ids = self.requests.iter().map(|r| r.id).collect::<Vec<_>>();
            self.requests.clear();
            self.outcomes.extend(
                ids.into_iter()
                    .map(|request| Outcome::Cancelled { request }),
            );
            return;
        }
        self.maximum_close_delay = 0;
        self.phase = Phase::Recover;
    }
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }
    fn wait_tick(&mut self, resume: Phase) {
        let id = self.serial();
        self.wait_registrations.push(id);
        self.phase = Phase::TickWait {
            id,
            resume: Box::new(resume),
        };
    }
    pub fn advance_callback(&mut self, id: u64) -> bool {
        let phase = std::mem::replace(&mut self.phase, Phase::Idle);
        match phase {
            Phase::TickWait { id: active, resume } if id == active => {
                self.phase = *resume;
                false
            }
            other => {
                self.phase = other;
                false
            }
        }
    }
    fn pause(&mut self, millis: i32, now: i64) {
        if millis > 0 {
            self.pause_until = Some(now + i64::from(millis));
        }
    }
    fn sample(random: &mut ThreadRandom, range: [i32; 2]) -> Result<i32, &'static str> {
        if range[0] > range[1] {
            return Err("empty inventory delay range");
        }
        Ok(random.next_i32_inclusive(range[0], range[1]))
    }
    fn emit(&mut self, effect: Effect, record: Option<SentClick>, recovery: bool) -> Emission {
        let emission = Emission {
            ticket: self.serial(),
            effect,
        };
        let action = if let Effect::Action(action) = &emission.effect {
            Some(action.clone())
        } else {
            None
        };
        self.inflight = Some(Inflight {
            emission: emission.clone(),
            action,
            record,
            recovery,
        });
        emission
    }
    fn current_action(&self) -> Option<Action> {
        self.chain
            .as_ref()
            .and_then(|ch| ch.request.actions.get(ch.index))
            .cloned()
    }
    fn record_click(&mut self, action: &Action, c: &Context) -> Option<SentClick> {
        let (screen, slot, _, input) = action.click()?;
        let stack = c.stack(slot);
        if stack.empty() {
            return None;
        }
        Some(SentClick {
            id: self.serial(),
            container: screen.map_or(0, |s| s.container_id),
            slot: slot.server_id(screen)?,
            input,
            stack,
            moving: !c.requirements.no_movement(),
            restored: false,
            until: 0,
        })
    }
    fn finish_current(&mut self, cancelled: bool) {
        if let Some(chain) = self.chain.take()
            && cancelled
        {
            self.outcomes.push_back(Outcome::Cancelled {
                request: chain.request.id,
            });
        }
        self.phase = Phase::Loop;
    }
    pub fn resync_slot(&mut self, container: i32, slot: i32, stack: &Stack, now: i64) {
        let mut removed = vec![];
        for click in &mut self.recent_clicks {
            if click.container != container || click.slot != slot {
                continue;
            }
            let restored = matches!(click.input, Input::Throw | Input::Swap | Input::QuickMove)
                && stack.same_item(&click.stack)
                && stack.count >= click.stack.count;
            if restored && !click.restored {
                click.restored = true;
                click.until = now + i64::from(self.response_time);
            }
            if !restored {
                removed.push(click.id);
            }
        }
        self.recent_clicks.retain(|c| !removed.contains(&c.id));
    }
    pub fn resync_content(&mut self, container: i32, stacks: &[Stack], now: i64) {
        let slots = self
            .recent_clicks
            .iter()
            .filter(|c| c.container == container)
            .map(|c| c.slot)
            .collect::<Vec<_>>();
        for slot in slots {
            let stack = usize::try_from(slot)
                .ok()
                .and_then(|i| stacks.get(i))
                .cloned()
                .unwrap_or_default();
            self.resync_slot(container, slot, &stack, now);
        }
    }
    pub fn poll_timers(&mut self, now: i64) {
        // Re-armed clicks append logically at their new deadline, preserving
        // creation id for equal timeouts on this same render thread.
        while let Some(index) = self
            .recent_clicks
            .iter()
            .enumerate()
            .filter(|(_, c)| c.until <= now)
            .min_by_key(|(_, c)| (c.until, c.id))
            .map(|(i, _)| i)
        {
            let click = self.recent_clicks.remove(index);
            if !click.restored {
                continue;
            }
            self.revert_wait = Some(now + i64::from(self.response_time));
            if click.moving || self.is_locked(&click.stack) {
                continue;
            }
            let count = if let Some((_, count)) = self
                .rejections
                .iter_mut()
                .find(|(s, _)| s.same_item(&click.stack))
            {
                count
            } else {
                self.rejections.push((click.stack.clone(), 0));
                &mut self.rejections.last_mut().unwrap().1
            };
            *count = count.wrapping_add(1);
            if *count >= self.lock_after_rejections {
                self.locked.push(click.stack);
            }
        }
    }
    fn take_request(
        &mut self,
        c: &Context,
        owner: &mut impl FnMut(u64) -> OwnerState,
    ) -> Option<Request> {
        let locked = self.locked_count();
        let mut cancelled = vec![];
        self.requests.retain(|r| {
            let o = owner(r.owner);
            let valid = o.running
                && o.guards_pass
                && r.locked_count == locked
                && r.expected.matches(&c.menu);
            if !valid {
                cancelled.push(r.id);
            }
            valid
        });
        self.outcomes.extend(
            cancelled
                .into_iter()
                .map(|request| Outcome::Cancelled { request }),
        );
        // endRequests resumes cancelled plans synchronously before selecting
        // another chain. Their tails may submit replacement requests.
        if !self.outcomes.is_empty() {
            return None;
        }
        let index = self
            .requests
            .iter()
            .enumerate()
            .filter(|(_, r)| self.can_chain(r, c, &owner(r.owner).constraints))
            .min_by_key(|(i, r)| {
                (
                    r.requires_player_inventory(),
                    std::cmp::Reverse(r.priority),
                    *i,
                )
            })
            .map(|(i, _)| i)?;
        Some(self.requests.remove(index))
    }
    /// Returns one synchronous native effect, or None at an actual wait/end.
    /// The caller must acknowledge each effect before asking for another, so
    /// reentrant frame/tick calls cannot duplicate any in-flight action.
    pub fn step(
        &mut self,
        c: &Context,
        owner: &mut impl FnMut(u64) -> OwnerState,
        random: &mut ThreadRandom,
    ) -> Result<Option<Emission>, &'static str> {
        let result = self.step_inner(c, owner, random);
        if result.is_err() {
            if let Some(chain) = self.chain.take() {
                self.outcomes.push_back(Outcome::Cancelled {
                    request: chain.request.id,
                });
            }
            self.phase = Phase::Idle;
            self.pause_until = None;
        }
        result
    }
    fn step_inner(
        &mut self,
        c: &Context,
        owner: &mut impl FnMut(u64) -> OwnerState,
        random: &mut ThreadRandom,
    ) -> Result<Option<Emission>, &'static str> {
        if self.inflight.is_some() {
            return Ok(None);
        }
        self.poll_timers(c.now);
        if let Some(outcome) = self.outcomes.pop_front() {
            return Ok(Some(self.emit(Effect::Outcome(outcome), None, false)));
        }
        if Self::wait_active(self.pause_until, c.now) {
            return Ok(None);
        }
        self.pause_until = None;
        loop {
            let constraints = self
                .chain
                .as_ref()
                .map(|ch| owner(ch.request.owner).constraints);
            let phase = std::mem::replace(&mut self.phase, Phase::Idle);
            match phase {
                Phase::Idle => {
                    self.phase = Phase::Idle;
                    return Ok(None);
                }
                Phase::TickWait { .. } => {
                    self.phase = phase;
                    return Ok(None);
                }
                Phase::Recover => {
                    self.phase = Phase::Loop;
                    if c.no_screen && !c.menu.carried.empty() && self.can_click_player_inventory(c)
                    {
                        let empty = (9u8..36)
                            .chain(0..9)
                            .map(Slot::Player)
                            .find(|s| c.stack(*s).empty());
                        if let Some(slot) = empty {
                            if !self.inventory_open(c) {
                                self.wait_tick(Phase::RecoverPickup(slot));
                                return Ok(Some(self.emit(Effect::OpenInventory, None, false)));
                            }
                            self.phase = Phase::RecoverPickup(slot);
                        }
                    }
                }
                Phase::RecoverPickup(slot) => {
                    let action = Action::Click {
                        screen: None,
                        slot,
                        button: 0,
                        input: Input::Pickup,
                    };
                    self.phase = Phase::Loop;
                    return Ok(Some(self.emit(Effect::Action(action), None, true)));
                }
                Phase::Loop => {
                    let request = self.take_request(c, owner);
                    if let Some(outcome) = self.outcomes.pop_front() {
                        self.phase = Phase::Loop;
                        return Ok(Some(self.emit(Effect::Outcome(outcome), None, false)));
                    }
                    if let Some(request) = request {
                        self.chain = Some(Chain {
                            request,
                            index: 0,
                            completed: false,
                        });
                        self.phase = Phase::Recent;
                    } else if !self.inventory_open(c) || !self.can_close(c) {
                        self.phase = Phase::Done;
                    } else if self.maximum_close_delay > 0 {
                        let delay = self.maximum_close_delay;
                        self.maximum_close_delay = 0;
                        self.phase = Phase::Loop;
                        self.pause(delay, c.now);
                    } else {
                        self.phase = Phase::Done;
                        return Ok(Some(self.emit(Effect::CloseInventory, None, false)));
                    }
                }
                Phase::Recent => {
                    self.phase = Phase::Open;
                    if self.recent_open {
                        self.recent_open = false;
                        self.pause(
                            Self::sample(random, constraints.as_ref().unwrap().start_delay)?,
                            c.now,
                        );
                    }
                }
                Phase::Open => {
                    self.phase = Phase::Miss;
                    let action = self.current_action().unwrap();
                    let constraints = constraints.unwrap();
                    if action.requires_player_inventory() {
                        if !self.inventory_open(c) {
                            self.phase = Phase::AfterOpenDelay;
                            return Ok(Some(self.emit(Effect::OpenInventory, None, false)));
                        }
                    } else if self.can_close(c) && self.inventory_open(c) {
                        self.phase = Phase::CloseAfterPause;
                        self.pause(Self::sample(random, constraints.close_delay)?, c.now);
                    }
                }
                Phase::AfterOpenDelay => {
                    self.phase = Phase::Miss;
                    self.pause(
                        Self::sample(random, constraints.unwrap().start_delay)?,
                        c.now,
                    );
                }
                Phase::CloseAfterPause => {
                    self.phase = Phase::Miss;
                    return Ok(Some(self.emit(Effect::CloseInventory, None, false)));
                }
                Phase::Miss => {
                    self.phase = Phase::CloseScreen;
                    let current_action = self.current_action().unwrap();
                    if self.chain.as_ref().unwrap().index == 0
                        && let Some((screen, slot, _, input)) = current_action.click()
                    {
                        let constraints = constraints.unwrap();
                        let miss = Self::sample(random, constraints.miss_chance)?
                            > random.next_i32_bound(100);
                        // The RNG is sampled even for THROW; it cannot actually miss.
                        if miss
                            && input != Input::Throw
                            && self.can_chain(
                                &self.chain.as_ref().unwrap().request,
                                c,
                                &constraints,
                            )
                            && let (Some(screen), Slot::Container(clicked)) = (screen, slot)
                        {
                            let closest = screen
                                .non_player_slots
                                .iter()
                                .copied()
                                .filter(|id| {
                                    usize::try_from(*id)
                                        .ok()
                                        .and_then(|id| c.menu.stacks.get(id))
                                        .is_some_and(Stack::empty)
                                })
                                .min_by_key(|id| {
                                    let dx = clicked % 9 - id % 9;
                                    let dy = clicked / 9 - id / 9;
                                    dx * dx + dy * dy
                                });
                            if let Some(id) = closest {
                                let action = Action::Click {
                                    screen: Some(screen.clone()),
                                    slot: Slot::Container(id),
                                    button: 0,
                                    input: Input::Pickup,
                                };
                                self.phase = Phase::MissPause;
                                return Ok(Some(self.emit(Effect::Action(action), None, false)));
                            }
                        }
                    }
                }
                Phase::MissPause => {
                    self.phase = Phase::CloseScreen;
                    self.pause(
                        Self::sample(random, constraints.unwrap().click_delay)?,
                        c.now,
                    );
                }
                Phase::CloseScreen => {
                    self.phase = Phase::TravelStart;
                    if matches!(self.current_action(), Some(Action::CloseScreen { .. })) {
                        self.pause(
                            Self::sample(random, constraints.unwrap().close_delay)?,
                            c.now,
                        );
                    }
                }
                Phase::TravelStart => {
                    self.phase = Phase::Ready;
                    let current_action = self.current_action().unwrap();
                    if let Some((screen, slot, _, _)) = current_action.click() {
                        let geometry = screen.map_or(&c.inventory_menu, |s| &s.geometry);
                        if let Some(position) = slot
                            .server_id(screen)
                            .and_then(|id| usize::try_from(id).ok())
                            .and_then(|id| geometry.slots.get(id))
                            .copied()
                        {
                            let visible = c.screen.as_ref().filter(|s| {
                                s.geometry.object == geometry.object && c.move_visible_cursor
                            });
                            let from = visible.map_or_else(
                                || {
                                    self.cursor
                                        .or_else(|| geometry.center())
                                        .unwrap_or_default()
                                },
                                |_| c.visible_cursor,
                            );
                            let to = Position {
                                x: position.x + 2.0 + random.next_unit() * 12.0,
                                y: position.y + 2.0 + random.next_unit() * 12.0,
                            };
                            let constraints = constraints.unwrap();
                            let delay = if self.cursor.is_none() {
                                constraints.start_delay
                            } else {
                                constraints.click_delay
                            };
                            let millis = (Self::sample(random, delay)? as f32
                                * (from.distance(to) / 18.0)
                                * constraints.slot_multiplier)
                                as i32;
                            self.cursor = Some(to);
                            if let Some(visible) = visible {
                                self.phase = Phase::Travel(Travel {
                                    screen: visible.object,
                                    from,
                                    to,
                                    start: c.now,
                                    millis,
                                    woke: false,
                                });
                            } else {
                                self.pause(millis, c.now);
                            }
                        }
                    }
                }
                Phase::Travel(mut travel) => {
                    if travel.woke && c.screen.as_ref().is_none_or(|s| s.object != travel.screen) {
                        self.phase = Phase::Ready;
                        continue;
                    }
                    let progress = if travel.millis > 0 {
                        ((c.now - travel.start) as f32 / travel.millis as f32).min(1.0)
                    } else {
                        1.0
                    };
                    let position = travel
                        .from
                        .lerp(travel.to, progress * progress * (3.0 - 2.0 * progress));
                    if progress < 1.0 {
                        travel.woke = true;
                        self.phase = Phase::Travel(travel);
                        self.pause(5, c.now);
                    } else {
                        self.phase = Phase::Ready;
                    }
                    return Ok(Some(self.emit(Effect::MoveCursor(position), None, false)));
                }
                Phase::Ready => {
                    let constraints = constraints.unwrap();
                    if !self.can_chain(&self.chain.as_ref().unwrap().request, c, &constraints) {
                        self.finish_current(true);
                        continue;
                    }
                    let action = self.current_action().unwrap();
                    let record = self.record_click(&action, c);
                    self.phase = Phase::AfterAction { success: false };
                    return Ok(Some(self.emit(Effect::Action(action), record, false)));
                }
                Phase::AfterAction { success } => {
                    if success {
                        let action = self.current_action().unwrap();
                        self.phase = Phase::AfterActionPause;
                        if matches!(action, Action::CloseScreen { .. }) {
                            self.wait_tick(Phase::AfterActionPause);
                            return Ok(None);
                        }
                        self.pause(
                            Self::sample(random, constraints.unwrap().click_delay)?,
                            c.now,
                        );
                    } else {
                        self.advance_action();
                    }
                }
                Phase::AfterActionPause => self.advance_action(),
                Phase::FinishChain => {
                    let chain = self.chain.as_mut().unwrap();
                    if !chain.completed {
                        chain.completed = true;
                        self.phase = Phase::FinishChain;
                        let outcome = Outcome::Completed {
                            request: chain.request.id,
                        };
                        return Ok(Some(self.emit(Effect::Outcome(outcome), None, false)));
                    }
                    let delay = Self::sample(random, constraints.unwrap().close_delay)?;
                    self.maximum_close_delay = self.maximum_close_delay.max(delay);
                    self.finish_current(false);
                }
                Phase::Done => {
                    self.last_clicked_slot = -1;
                    self.phase = Phase::Idle;
                    return Ok(None);
                }
            }
            if Self::wait_active(self.pause_until, c.now) {
                return Ok(None);
            }
            self.pause_until = None;
        }
    }
    fn advance_action(&mut self) {
        let chain = self.chain.as_mut().unwrap();
        chain.index += 1;
        self.phase = if chain.index >= chain.request.actions.len() {
            Phase::FinishChain
        } else {
            Phase::Recent
        };
    }
    pub fn acknowledge(&mut self, ticket: u64, succeeded: bool, c: &Context) -> bool {
        if self
            .inflight
            .as_ref()
            .is_none_or(|e| e.emission.ticket != ticket)
        {
            return false;
        }
        let mut inflight = self.inflight.take().unwrap();
        let Some(action) = inflight.action else {
            return true;
        };
        if inflight.recovery {
            if succeeded {
                if let Some((screen, slot, _, _)) = action.click()
                    && let Some(id) = slot.server_id(screen)
                {
                    self.last_clicked_slot = id;
                }
                self.wait_tick(Phase::Loop);
            }
            return true;
        }
        // Miss-clicks are emitted while phase already points at CloseScreen and
        // do not enter the chain's normal onPerformed/delay continuation.
        if !matches!(self.phase, Phase::AfterAction { .. }) {
            if succeeded
                && let Some((screen, slot, _, _)) = action.click()
                && let Some(id) = slot.server_id(screen)
            {
                self.last_clicked_slot = id;
            }
            return true;
        }
        self.phase = Phase::AfterAction { success: succeeded };
        if !succeeded {
            return true;
        }
        if let Some((screen, slot, _, _)) = action.click()
            && let Some(id) = slot.server_id(screen)
        {
            self.last_clicked_slot = id;
        }
        if let Action::Creative {
            slot: Some(slot), ..
        } = &action
            && let Some(id) = slot.server_id(None)
        {
            self.last_clicked_slot = id;
        }
        if let Some(mut record) = inflight.record.take() {
            record.until = c.now + i64::from(self.response_time);
            self.recent_clicks.push(record);
        }
        let chain = self.chain.as_mut().unwrap();
        chain.request.expected = c.menu.clone();
        chain.request.locked_count = self.locked.len();
        if let Some(plan) = self.plans.get_mut(&chain.request.id) {
            plan.expected = c.menu.clone();
            plan.locked_count = self.locked.len();
            if let Action::Click {
                slot,
                button,
                input: Input::Swap,
                ..
            } = &action
                && ((0..9).contains(button) || *button == 40)
            {
                let target = Slot::Player(*button as u8);
                let origin = |slot| {
                    plan.moved
                        .iter()
                        .find_map(|(key, value)| (*value == slot).then_some(*key))
                        .unwrap_or(slot)
                };
                let first = origin(*slot);
                let second = origin(target);
                plan.moved.insert(first, target);
                plan.moved.insert(second, *slot);
            }
        }
        self.outcomes.push_back(Outcome::Performed {
            request: chain.request.id,
            action: action.clone(),
        });
        if chain.index + 1 == chain.request.actions.len() {
            chain.completed = true;
            self.outcomes.push_back(Outcome::Completed {
                request: chain.request.id,
            });
            if matches!(action, Action::UseItem { .. }) {
                self.item_used(c.now);
            }
        }
        true
    }
}
