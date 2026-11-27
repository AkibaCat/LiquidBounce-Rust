// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared Timer state, original module modes, and render-thread accounting.
use crate::packets::{PacketEvent, PacketListener};
use horizon_client::mod_api::{ClientboundPlayPacket, Host};
use liquidbounce_behaviors::timer::{Context, Mode, ModuleTimer, Provider, Settings, Timer};
use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::ThreadId,
};

pub const TIMER_PROVIDER: Provider = u64::MAX;
#[derive(Default)]
pub struct TimerState {
    pub settings: Settings,
    pub module: ModuleTimer,
    pub timer: Timer,
}
#[derive(Default)]
pub struct TimerService {
    pub state: Mutex<TimerState>,
    running: AtomicBool,
    providers: Mutex<BTreeMap<Provider, bool>>,
    render_thread: Mutex<Option<ThreadId>>,
    pending_reset: AtomicBool,
    #[cfg(feature = "webview")]
    server_time: Mutex<(u64, u64, i64, i64)>,
}
impl TimerService {
    pub fn running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
        let mut state = self.state.lock().expect("Timer poisoned");
        let TimerState {
            settings,
            module,
            timer,
        } = &mut *state;
        module.synchronize(running, settings.mode, timer, TIMER_PROVIDER);
    }
    /// Publish the actual EventListener.running for other shared Timer providers.
    pub fn set_provider_running(&self, provider: Provider, running: bool) {
        assert_ne!(provider, TIMER_PROVIDER, "reserved Timer provider");
        self.providers
            .lock()
            .expect("Timer providers poisoned")
            .insert(provider, running);
    }
    fn providers(&self) -> BTreeMap<Provider, bool> {
        let mut providers = self
            .providers
            .lock()
            .expect("Timer providers poisoned")
            .clone();
        providers.insert(TIMER_PROVIDER, self.running());
        providers
    }
    fn render_actions(&self) {
        *self
            .render_thread
            .lock()
            .expect("Timer render owner poisoned") = Some(std::thread::current().id());
        if self.pending_reset.swap(false, Ordering::AcqRel) {
            self.state
                .lock()
                .expect("Timer poisoned")
                .timer
                .reset_balance();
        }
    }
    /// Apply config on the actor thread so changing modes cancels its old scope now.
    pub fn update_settings(&self, update: impl FnOnce(&mut Settings)) {
        let mut state = self.state.lock().expect("Timer poisoned");
        update(&mut state.settings);
        let TimerState {
            settings,
            module,
            timer,
        } = &mut *state;
        module.synchronize(self.running(), settings.mode, timer, TIMER_PROVIDER);
    }
    pub fn speed(&self)->f32 {
        self.render_actions();
        let providers=self.providers();
        self.state.lock().expect("Timer poisoned").timer.speed(true,&mut |id|providers.get(&id).copied().unwrap_or(false))
    }
    pub fn scale(&self, delta: f32) -> f32 {
        self.render_actions();
        let providers = self.providers();
        self.state
            .lock()
            .expect("Timer poisoned")
            .timer
            .scale_frame(delta, true, &mut |id| {
                providers.get(&id).copied().unwrap_or(false)
            })
    }
    pub fn frame(&self, ticks: i32, index: i32) -> (i32, bool) {
        self.render_actions();
        let providers = self.providers();
        let mut state = self.state.lock().expect("Timer poisoned");
        let end = state.timer.frame_ticks(ticks, index, true, &mut |id| {
            providers.get(&id).copied().unwrap_or(false)
        });
        (end, state.timer.is_held())
    }
    /// FIRST_PRIORITY GameTick, after CoroutineTicker and before module handlers.
    pub fn first_game_tick(&self) {
        self.render_actions();
        self.state.lock().expect("Timer poisoned").timer.game_tick();
    }
    pub fn player_tick_final(&self, cancelled: bool) {
        self.render_actions();
        self.state
            .lock()
            .expect("Timer poisoned")
            .timer
            .player_tick_final(cancelled);
    }
    pub fn world_change(&self) {
        self.render_actions();
        self.state
            .lock()
            .expect("Timer poisoned")
            .timer
            .world_change();
    }
    pub fn game_tick(&self, running: bool, context: Context) -> Vec<u64> {
        self.set_running(running);
        let mut state = self.state.lock().expect("Timer poisoned");
        let TimerState {
            settings,
            module,
            timer,
        } = &mut *state;
        if let Err(error) = module.game_tick(running, *settings, context, timer, TIMER_PROVIDER) {
            eprintln!("[LiquidBounce] Timer: {error:?}");
        }
        module.take_wait_registrations()
    }
    pub fn callback(&self, callback: u64, running: bool) -> bool {
        self.set_running(running);
        let mut state = self.state.lock().expect("Timer poisoned");
        let mode = state.settings.mode;
        state.module.advance_callback(callback, running, mode)
    }
}
impl PacketListener for TimerService {
    fn incoming(&self, event: &mut PacketEvent<'_, ClientboundPlayPacket>) {
        #[cfg(feature = "webview")]
        if let ClientboundPlayPacket::SetTime(packet) = event.packet {
            let mut sample = self
                .server_time
                .lock()
                .expect("Timer server-time probe poisoned");
            *sample = (
                sample.0.wrapping_add(1),
                event.context.session,
                packet.game_time,
                crate::behaviors::current_time_millis(),
            );
        }
        if !self.running() || !matches!(event.packet, ClientboundPlayPacket::PlayerPosition(_)) {
            return;
        }
        let mut state = self.state.lock().expect("Timer poisoned");
        if state.settings.mode != Mode::Boost || !state.settings.reset_balance_on_flag {
            return;
        }
        // The source observes cancelled PacketEvents. mc.execute is synchronous
        // on the render thread and queues an off-thread reset onto that thread.
        let same_thread = self
            .render_thread
            .lock()
            .expect("Timer render owner poisoned")
            .is_some_and(|id| id == std::thread::current().id());
        if same_thread {
            state.timer.reset_balance();
        } else {
            self.pending_reset.store(true, Ordering::Release);
        }
    }
}

impl crate::runtime::LiquidBounceHooks {
    pub(crate) fn timer_tick(&self, host: &dyn Host) {
        let context = host.client_timer_context();
        #[allow(unused_mut)]
        let mut state = self.state();
        let modules = state.behaviors.borrow();
        let running = context.is_some() && modules.query_running(&state.registry, "Timer");
        let combat = state.combat.in_combat();
        let [moving, inventory_or_container] = context.unwrap_or([false; 2]);
        for callback in modules.timer.game_tick(
            running,
            Context {
                moving,
                combat,
                inventory_or_container,
            },
        ) {
            modules
                .coroutine_ticker
                .register(crate::coroutine_ticker::WaitOwner::Timer, callback);
        }
        #[cfg(feature = "webview")]
        {
            let output = {
                let timer = modules.timer.state.lock().expect("Timer poisoned");
                let server = *modules
                    .timer
                    .server_time
                    .lock()
                    .expect("Timer server-time probe poisoned");
                serde_json::json!({"mode":timer.settings.mode.tag(),"balance":timer.timer.balance,
                    "serverTime":{"count":server.0,"session":server.1,"gameTime":server.2,"receivedAtMs":server.3}})
            };
            drop(modules);
            state.probe(
                "clientTimerContext",
                serde_json::json!({"running":running,"moving":moving,"combat":combat,"inventoryOrContainer":inventory_or_container}),
                output,
            );
        }
    }
    pub(crate) fn timer_callback(&self, host: &dyn Host, in_game: bool, callback: u64) -> bool {
        let owner = host.client_timer_context().is_some();
        let state = self.state();
        let modules = state.behaviors.borrow();
        modules.timer.callback(
            callback,
            in_game && owner && modules.query_running(&state.registry, "Timer"),
        )
    }
}
