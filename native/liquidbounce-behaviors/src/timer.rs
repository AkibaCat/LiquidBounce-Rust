// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTimer and shared Timer/RequestHandler, LiquidBounce ae0eb396.
//! All arithmetic deliberately retains JVM Float/Int/Long evaluation order.

pub type Provider = u64;

#[derive(Clone, Copy, Debug)]
struct ActiveRequest<T> {
    expires_at: i64,
    priority: i32,
    provider: Provider,
    value: T,
}

/// Java PriorityBlockingQueue's comparator heap, including equal priorities.
/// Provider replacement compacts in array order and heapifies (bulkRemove).
#[derive(Clone, Debug)]
pub struct RequestHandler<T> {
    current_tick: i64,
    queue: Vec<ActiveRequest<T>>,
}
impl<T> Default for RequestHandler<T> {
    fn default() -> Self {
        Self {
            current_tick: 0,
            queue: Vec::new(),
        }
    }
}
impl<T: Copy> RequestHandler<T> {
    pub fn tick(&mut self, delta: i32) {
        self.current_tick = self.current_tick.wrapping_add(i64::from(delta));
    }
    pub fn clear(&mut self) {
        self.queue.clear();
        self.current_tick = 0;
    }
    pub fn current_tick(&self) -> i64 {
        self.current_tick
    }
    fn sift_down(&mut self, mut index: usize, value: ActiveRequest<T>) {
        let half = self.queue.len() >> 1;
        while index < half {
            let mut child = (index << 1) + 1;
            let right = child + 1;
            if right < self.queue.len()
                && (!self.queue[child].priority) > (!self.queue[right].priority)
            {
                child = right;
            }
            if (!value.priority) <= (!self.queue[child].priority) {
                break;
            }
            self.queue[index] = self.queue[child];
            index = child;
        }
        self.queue[index] = value;
    }
    pub fn request(&mut self, expires_in: i32, priority: i32, provider: Provider, value: T) {
        let old_len = self.queue.len();
        self.queue.retain(|entry| entry.provider != provider);
        if old_len != self.queue.len() {
            for index in (0..(self.queue.len() >> 1)).rev() {
                let value = self.queue[index];
                self.sift_down(index, value);
            }
        }
        let request = ActiveRequest {
            expires_at: self.current_tick.wrapping_add(i64::from(expires_in)),
            priority,
            provider,
            value,
        };
        let mut index = self.queue.len();
        self.queue.push(request);
        while index > 0 {
            let parent = (index - 1) >> 1;
            if (!request.priority) >= (!self.queue[parent].priority) {
                break;
            }
            self.queue[index] = self.queue[parent];
            index = parent;
        }
        self.queue[index] = request;
    }
    fn remove_root(&mut self) {
        let value = self.queue.pop().expect("nonempty heap");
        if !self.queue.is_empty() {
            self.sift_down(0, value);
        }
    }
    /// Off-thread queries return the current root without pruning, as upstream.
    pub fn active(
        &mut self,
        same_thread: bool,
        running: &mut impl FnMut(Provider) -> bool,
    ) -> Option<T> {
        loop {
            let top = *self.queue.first()?;
            if same_thread && (top.expires_at <= self.current_tick || !running(top.provider)) {
                self.remove_root();
                continue;
            }
            return Some(top.value);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Timer {
    pub requests: RequestHandler<f32>,
    holds: RequestHandler<()>,
    pub balance: f32,
    pub frame_speed: f32,
    pub burst_ticks: i32,
    pub bursting: bool,
    pub held_ticks: i32,
    pub real_ticks: f64,
    held_history: [i32; 20],
    held_history_end: usize,
    pub frame_end: i32,
}
impl Default for Timer {
    fn default() -> Self {
        Self {
            requests: Default::default(),
            holds: Default::default(),
            balance: 0.0,
            frame_speed: 1.0,
            burst_ticks: 0,
            bursting: false,
            held_ticks: 0,
            real_ticks: 0.0,
            held_history: [0; 20],
            held_history_end: 0,
            frame_end: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidBalanceRange;
impl Timer {
    pub fn speed(&mut self, same_thread: bool, running: &mut impl FnMut(Provider) -> bool) -> f32 {
        self.requests.active(same_thread, running).unwrap_or(1.0)
    }
    pub fn scale_frame(
        &mut self,
        delta: f32,
        same_thread: bool,
        running: &mut impl FnMut(Provider) -> bool,
    ) -> f32 {
        let speed = self.speed(same_thread, running);
        self.frame_speed = if speed > 0.0 { speed } else { 1.0 };
        delta * self.frame_speed
    }
    pub fn request_speed(
        &mut self,
        speed: f32,
        priority: i32,
        provider: Provider,
        reset_after_ticks: i32,
    ) {
        self.requests
            .request(reset_after_ticks.wrapping_add(1), priority, provider, speed);
    }
    pub fn balance_within(&mut self, range: [i32; 2]) -> Result<f32, InvalidBalanceRange> {
        let [start, end] = range.map(|v| v as f32);
        if start > end {
            return Err(InvalidBalanceRange);
        }
        if self.balance < start {
            self.balance = start;
        } else if self.balance > end {
            self.balance = end;
        }
        Ok(self.balance)
    }
    pub fn spendable_ticks(&mut self, range: [i32; 2]) -> Result<i32, InvalidBalanceRange> {
        Ok((self.balance_within(range)? - range[0] as f32) as i32)
    }
    pub fn request_balanced(
        &mut self,
        speed: f32,
        priority: i32,
        provider: Provider,
        range: [i32; 2],
    ) -> Result<bool, InvalidBalanceRange> {
        let balance = self.balance_within(range)?;
        let allowed = if speed > 1.0 {
            balance > range[0] as f32
        } else if speed < 1.0 {
            balance < range[1] as f32
        } else {
            true
        };
        if allowed {
            self.request_speed(speed, priority, provider, 1);
        }
        Ok(allowed)
    }
    pub fn hold(&mut self, provider: Provider, ticks: i32) {
        self.holds.request(ticks, 0, provider, ());
    }
    pub fn held_ticks_within(&self, ticks: i32) -> i32 {
        let mut held = self.held_ticks;
        for back in 1..=ticks.min(20) {
            held = held
                .wrapping_add(self.held_history[(self.held_history_end + 20 - back as usize) % 20]);
        }
        held
    }
    pub fn burst(
        &mut self,
        player: bool,
        ticks: i32,
        range: [i32; 2],
    ) -> Result<i32, InvalidBalanceRange> {
        if !player {
            return Ok(0);
        }
        let count = ticks.min(self.spendable_ticks(range)?.wrapping_sub(self.burst_ticks));
        if count <= 0 {
            return Ok(0);
        }
        self.burst_ticks = self.burst_ticks.wrapping_add(count);
        Ok(count)
    }
    pub fn frame_ticks(
        &mut self,
        ticks: i32,
        index: i32,
        same_thread: bool,
        running: &mut impl FnMut(Provider) -> bool,
    ) -> i32 {
        if index == 0 {
            self.frame_end = ticks;
            self.real_ticks += f64::from(ticks) / f64::from(self.frame_speed);
        }
        let mut held = 0i32;
        while index < self.frame_end && self.holds.active(same_thread, running).is_some() {
            self.holds.tick(1);
            self.frame_end = self.frame_end.wrapping_sub(1);
            held = held.wrapping_add(1);
        }
        if held > 0 {
            self.balance += held as f32 / self.frame_speed;
            self.held_ticks = self.held_ticks.wrapping_add(held);
        }
        self.bursting = self.burst_ticks > 0 && self.holds.active(same_thread, running).is_none();
        if self.bursting {
            self.burst_ticks = self.burst_ticks.wrapping_sub(1);
            self.frame_end = self.frame_end.wrapping_add(1);
        }
        if index < self.frame_end {
            self.held_history[self.held_history_end] = self.held_ticks;
            self.held_history_end = (self.held_history_end + 1) % 20;
            self.held_ticks = 0;
        }
        self.frame_end
    }
    pub fn is_held(&self) -> bool {
        self.held_ticks > 0
    }
    pub fn game_tick(&mut self) {
        self.requests.tick(1);
    }
    pub fn player_tick_final(&mut self, cancelled: bool) {
        if !self.bursting {
            self.balance += 1.0 / self.frame_speed;
        }
        if !cancelled {
            self.balance -= 1.0;
        }
    }
    /// Invoke on render executor, as mc.execute { balance = 0f }.
    pub fn reset_balance(&mut self) {
        self.balance = 0.0;
    }
    pub fn world_change(&mut self) {
        self.balance = 0.0;
        self.burst_ticks = 0;
        self.held_ticks = 0;
        self.held_history.fill(0);
        self.holds.clear();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Classic,
    Pulse,
    Boost,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Pulse => "Pulse",
            Self::Boost => "Boost",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "Classic" => Some(Self::Classic),
            "Pulse" => Some(Self::Pulse),
            "Boost" => Some(Self::Boost),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub classic_speed: f32,
    pub normal_speed: f32,
    pub normal_ticks: i32,
    pub pulse_boost_speed: f32,
    pub boost_ticks: i32,
    pub on_move: bool,
    pub boost_speed: f32,
    pub slow_speed: f32,
    pub normalize_combat: bool,
    pub balance: [i32; 2],
    pub reset_balance_on_flag: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Classic,
            classic_speed: 2.0,
            normal_speed: 0.5,
            normal_ticks: 20,
            pulse_boost_speed: 2.0,
            boost_ticks: 20,
            on_move: false,
            boost_speed: 1.3,
            slow_speed: 0.6,
            normalize_combat: true,
            balance: [0, 20],
            reset_balance_on_flag: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub moving: bool,
    pub combat: bool,
    pub inventory_or_container: bool,
}
#[derive(Clone, Copy, Debug)]
struct Wait {
    callback: u64,
    ticks: i32,
    elapsed: i32,
}
#[derive(Clone, Debug, Default)]
pub struct ModuleTimer {
    running: bool,
    mode: Mode,
    next_boost: bool,
    wait: Option<Wait>,
    next_callback: u64,
    registrations: Vec<u64>,
}
impl ModuleTimer {
    pub fn synchronize(
        &mut self,
        running: bool,
        mode: Mode,
        timer: &mut Timer,
        provider: Provider,
    ) {
        if self.running && (!running || self.mode != mode) {
            self.wait = None;
            self.registrations.clear();
        }
        if running && (!self.running || self.mode != mode) && mode == Mode::Pulse {
            self.next_boost = false;
        }
        if self.running && !running {
            timer.request_speed(1.0, -20, provider, 1);
        }
        self.running = running;
        self.mode = mode;
    }
    /// ModeValueGroup and ClientModule remove their coroutine scopes immediately.
    pub fn cancel_sequences(&mut self) {
        self.wait = None;
        self.registrations.clear();
    }
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.registrations)
    }
    /// Run only via the common CoroutineTicker, before FIRST_PRIORITY GameTick.
    pub fn advance_callback(&mut self, callback: u64, running: bool, mode: Mode) -> bool {
        let Some(wait) = self.wait.as_mut().filter(|wait| wait.callback == callback) else {
            return false;
        };
        if !running || mode != Mode::Pulse {
            self.wait = None;
            return false;
        }
        wait.elapsed = wait.elapsed.wrapping_add(1);
        if wait.elapsed >= wait.ticks {
            self.wait = None;
            false
        } else {
            true
        }
    }
    pub fn game_tick(
        &mut self,
        running: bool,
        settings: Settings,
        context: Context,
        timer: &mut Timer,
        provider: Provider,
    ) -> Result<(), InvalidBalanceRange> {
        self.synchronize(running, settings.mode, timer, provider);
        if !running {
            return Ok(());
        }
        match settings.mode {
            Mode::Classic => timer.request_speed(settings.classic_speed, 20, provider, 1),
            Mode::Pulse => {
                if self.wait.is_some() || settings.on_move && !context.moving {
                    return Ok(());
                }
                let (speed, ticks) = if self.next_boost {
                    (settings.pulse_boost_speed, settings.boost_ticks)
                } else {
                    (settings.normal_speed, settings.normal_ticks)
                };
                self.next_boost = !self.next_boost;
                timer.request_speed(speed, 20, provider, ticks);
                if ticks != 0 {
                    self.next_callback = self.next_callback.wrapping_add(1);
                    self.wait = Some(Wait {
                        callback: self.next_callback,
                        ticks,
                        elapsed: 0,
                    });
                    self.registrations.push(self.next_callback);
                }
            }
            Mode::Boost => {
                if settings.normalize_combat && context.combat {
                    timer.request_speed(1.0, 20, provider, 1);
                    return Ok(());
                }
                let speed = if context.moving {
                    settings.boost_speed
                } else if context.inventory_or_container {
                    return Ok(());
                } else {
                    settings.slow_speed
                };
                timer.request_balanced(speed, 20, provider, settings.balance)?;
            }
        }
        Ok(())
    }
}
