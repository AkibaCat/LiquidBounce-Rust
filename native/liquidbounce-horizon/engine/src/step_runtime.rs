// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
    timer_runtime::TimerService,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::{
    random::with_thread_random,
    step::{Effects, Mode, Settings, Step},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
pub const TIMER_PROVIDER: u64 = 0x5354_4550;
#[derive(Default)]
struct State {
    behavior: Step,
    queue: Vec<(u64, S)>,
    flush: bool,
}
pub struct StepService {
    state: Mutex<State>,
    timer: Arc<TimerService>,
    running: AtomicBool,
    speed: AtomicBool,
}
fn effects(e: Effects) -> String {
    serde_json::json!({"setY":e.set_y,"addY":e.add_y,"strafe":e.strafe,"subtractFraction":e.subtract_y_fraction,"ground":e.ground}).to_string()
}
impl StepService {
    pub fn new(timer: Arc<TimerService>) -> Self {
        Self {
            state: Mutex::default(),
            timer,
            running: AtomicBool::new(false),
            speed: AtomicBool::new(false),
        }
    }
    pub fn settings(&self) -> Settings {
        self.state.lock().expect("Step").behavior.settings
    }
    pub fn update(&self, change: impl FnOnce(&mut Settings)) {
        let mut s = self.state.lock().expect("Step");
        let before = s.behavior.settings.mode;
        change(&mut s.behavior.settings);
        let mode = s.behavior.settings.mode;
        if mode != before {
            s.behavior.settings.mode = before;
            s.behavior.set_mode(mode);
        }
        drop(s);
        self.publish();
    }
    pub fn set_running(&self, running: bool, speed: bool) {
        let old = self.running.swap(running, Ordering::AcqRel);
        self.speed.store(speed, Ordering::Release);
        if old && !running {
            self.disabled();
        }
        self.timer.set_provider_running(TIMER_PROVIDER, running);
        self.publish();
    }
    fn publish(&self) {
        let s = self.settings();
        let r = self.running.load(Ordering::Acquire);
        horizon_client::step_bridge::set_flags(
            r && s.mode == Mode::Legit,
            r && s.mode == Mode::Hypixel && s.spoof,
        );
    }
    pub fn disabled(&self) {
        self.state.lock().expect("Step").behavior.disabled();
    }
    fn apply_timer(&self, e: Effects) {
        if let Some((speed, ticks)) = e.timer {
            self.timer.state.lock().expect("Timer").timer.request_speed(
                speed,
                20,
                TIMER_PROVIDER,
                ticks,
            );
        }
    }
    fn input(&self, can: [bool; 3]) -> Effects {
        let e = self.state.lock().expect("Step").behavior.input(
            self.running.load(Ordering::Acquire),
            self.speed.load(Ordering::Acquire),
            can,
        );
        self.apply_timer(e);
        e
    }
    fn callback(&self, id: u64, in_game: bool) -> (bool, Effects) {
        let result = self.state.lock().expect("Step").behavior.callback(
            id,
            in_game && self.running.load(Ordering::Acquire),
            self.speed.load(Ordering::Acquire),
        );
        self.apply_timer(result.1);
        result
    }
    pub fn height(&self, height: &mut f32) {
        *height = self
            .state
            .lock()
            .expect("Step")
            .behavior
            .height(self.running.load(Ordering::Acquire), *height);
    }
    pub fn success(&self, dy: f64, y: f64) -> String {
        let s = self.state.lock().expect("Step").behavior.success(
            self.running.load(Ordering::Acquire),
            dy,
            y,
            |r| with_thread_random(|random| random.next_i32_inclusive(r[0], r[1])),
        );
        serde_json::json!({"destinations":s.destinations,"awardJump":s.award_jump,"fullPacket":s.full_packet}).to_string()
    }
    fn tick(&self) {
        self.state
            .lock()
            .expect("Step")
            .behavior
            .game_tick(self.running.load(Ordering::Acquire));
    }
    fn drain(&self) -> Vec<(u64, S)> {
        let mut s = self.state.lock().expect("Step");
        if std::mem::take(&mut s.flush)
            || !s.behavior.queues_outgoing(
                self.running.load(Ordering::Acquire),
                self.speed.load(Ordering::Acquire),
            )
        {
            std::mem::take(&mut s.queue)
        } else {
            Vec::new()
        }
    }
    pub fn clear(&self) {
        let mut s = self.state.lock().expect("Step");
        s.queue.clear();
        s.flush = false;
        s.behavior.disabled();
        horizon_client::step_bridge::clear();
    }
}
impl PacketListener for StepService {
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if event.context.original()
            && matches!(
                event.packet,
                C::PlayerPosition(_) | C::Disconnect(_) | C::Respawn(_) | C::Login(_)
            )
        {
            self.state.lock().expect("Step").flush = true;
        }
    }
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        if !event.context.original()
            || event.is_cancelled()
            || matches!(event.packet, S::Chat(_) | S::ChatCommand(_))
        {
            return;
        }
        let mut s = self.state.lock().expect("Step");
        if s.behavior.queues_outgoing(
            self.running.load(Ordering::Acquire),
            self.speed.load(Ordering::Acquire),
        ) {
            s.queue.push((event.context.session, event.packet.clone()));
            event.cancel();
        }
    }
}
impl LiquidBounceHooks {
    fn step_wait(&self, e: Effects) {
        if let Some(id) = e.wait {
            self.state()
                .behaviors
                .borrow()
                .coroutine_ticker
                .register(WaitOwner::Step, id);
        }
    }
    pub(crate) fn step_input(&self, can: [bool; 3], jump: &mut bool, result: &mut String) {
        let service = self.state().behaviors.borrow().step.clone();
        let e = service.input(can);
        *jump |= e.jump;
        self.step_wait(e);
        *result = effects(e);
    }
    pub(crate) fn step_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let service = self.state().behaviors.borrow().step.clone();
        let (keep, e) = service.callback(id, in_game);
        self.step_wait(e);
        host.client_step_effects(&effects(e));
        self.step_drain(host);
        keep
    }
    pub(crate) fn step_drain(&self, host: &mut dyn Host) {
        let service = self.state().behaviors.borrow().step.clone();
        for (session, packet) in service.drain() {
            if let Err(error) = host.send_packet_silently(session, packet) {
                eprintln!("[LiquidBounce] Step replay: {error}");
            }
        }
    }
    pub(crate) fn step_tick(&self, host: &mut dyn Host) {
        self.state().behaviors.borrow().step.tick();
        self.step_drain(host);
    }
}
