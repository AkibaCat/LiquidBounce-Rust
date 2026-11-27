// SPDX-License-Identifier: GPL-3.0-or-later
//! Timer holds and packet delays remain live while the player's ticks are held.
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::{LiquidBounceHooks, raw_options},
    timer_runtime::TimerService,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::{
    freeze::{Freeze, Mode, REASONS, Settings},
    random::with_thread_random,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
pub const TIMER_PROVIDER: u64 = 0x4652_4545_5a45;
pub enum Queued {
    Incoming(u64, C),
    Outgoing(u64, S),
}
#[derive(Default)]
struct State {
    settings: Settings,
    behavior: Freeze,
    queue: Vec<Queued>,
    incoming_flush: bool,
    pending_flag: bool,
    actual: [f32; 2],
    rotation: [f32; 2],
    flags: u8,
    hurt_sound: i32,
}
pub struct FreezeService {
    timer: Arc<TimerService>,
    running: AtomicBool,
    state: Mutex<State>,
}
fn random(r: [i32; 2]) -> i32 {
    with_thread_random(|v| v.next_i32_inclusive(r[0], r[1]))
}
impl FreezeService {
    pub fn new(timer: Arc<TimerService>) -> Self {
        Self {
            timer,
            running: AtomicBool::new(false),
            state: Mutex::default(),
        }
    }
    pub fn settings(&self) -> Settings {
        self.state.lock().expect("Freeze").settings
    }
    pub fn update(&self, change: impl FnOnce(&mut Settings)) {
        let mut s = self.state.lock().expect("Freeze");
        change(&mut s.settings);
        let settings = s.settings;
        s.behavior
            .synchronize(self.running.load(Ordering::Acquire), settings, random);
        drop(s);
        self.hold();
    }
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
        let mut s = self.state.lock().expect("Freeze");
        let settings = s.settings;
        s.behavior.synchronize(running, settings, random);
        drop(s);
        self.hold();
    }
    fn hold(&self) {
        let held = self.state.lock().expect("Freeze").behavior.holds();
        self.timer.set_provider_running(TIMER_PROVIDER, held);
        if held {
            self.timer
                .state
                .lock()
                .expect("Timer")
                .timer
                .hold(TIMER_PROVIDER, i32::MAX);
        }
    }
    pub fn cancel_movement(&self) -> bool {
        let mut s = self.state.lock().expect("Freeze");
        let settings = s.settings;
        s.behavior.cancel_movement(settings, random)
    }
    pub(crate) fn blink_queue(&self,outgoing:bool)->bool{
        let s=self.state.lock().expect("Freeze");self.running.load(Ordering::Acquire)&&s.settings.mode==Mode::Queue&&s.settings.queue_origins&(if outgoing{2}else{1})!=0
    }
    pub(crate) fn blink_own_hurt(&self,packet:&C)->bool{
        let C::Sound(p)=packet else{return false;};let s=self.state.lock().expect("Freeze");p.sound.1.as_ref().is_some_and(|v|v.0=="minecraft:entity.player.hurt")||p.sound.1.is_none()&&p.sound.0==s.hurt_sound
    }
    pub fn clear(&self) {
        let mut s = self.state.lock().expect("Freeze");
        s.queue.clear();
        s.pending_flag = false;
        s.incoming_flush = false;
    }
    fn frame(&self, context: [f64; 9], rotation: [f32; 2]) -> Option<usize> {
        let mut s = self.state.lock().expect("Freeze");
        s.actual = [context[5] as f32, context[6] as f32];
        s.rotation = rotation;
        s.flags = u8::from(context[0] != 0.) | u8::from(context[7] != 0.) << 1;
        s.hurt_sound = context[8] as i32;
        let flag = std::mem::take(&mut s.pending_flag);
        let reason = self
            .running
            .load(Ordering::Acquire)
            .then(|| {
                Freeze::disable_reason(s.settings, flag, std::array::from_fn(|i| context[i] != 0.))
            })
            .flatten();
        drop(s);
        self.hold();
        reason
    }
    fn take_ready(&self) -> Vec<Queued> {
        let mut s = self.state.lock().expect("Freeze");
        let running = self.running.load(Ordering::Acquire);
        let settings = s.settings;
        let flush_in = std::mem::take(&mut s.incoming_flush);
        let mut keep = Vec::new();
        let mut ready = Vec::new();
        for packet in s.queue.drain(..) {
            let should_keep = running
                && settings.mode == Mode::Queue
                && match &packet {
                    Queued::Incoming(..) => settings.queue_origins & 1 != 0 && !flush_in,
                    Queued::Outgoing(..) => settings.queue_origins & 2 != 0,
                };
            if should_keep {
                keep.push(packet);
            } else {
                ready.push(packet);
            }
        }
        s.queue = keep;
        ready
    }
    fn offsets(s: &mut State) -> [f32; 2] {
        with_thread_random(|r| s.behavior.offsets(|| r.next_unit_f64()))
    }
}
impl PacketListener for FreezeService {
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if !event.context.original() || !self.running.load(Ordering::Acquire) {
            return;
        }
        let mut s = self.state.lock().expect("Freeze");
        let settings = s.settings;
        if matches!(event.packet, C::PlayerPosition(_)) {
            s.pending_flag = true;
        }
        match settings.mode {
            Mode::Cancel if settings.cancel_origins & 1 != 0 => event.cancel(),
            Mode::Stationary => {
                Self::offsets(&mut s);
            }
            _ => {}
        }
    }
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        if !event.context.original() || !self.running.load(Ordering::Acquire) {
            return;
        }
        let mut s = self.state.lock().expect("Freeze");
        let settings = s.settings;
        match settings.mode {
            Mode::Cancel if settings.cancel_origins & 2 != 0 => event.cancel(),
            Mode::Stationary => {
                let offset = Self::offsets(&mut s);
                let yaw = s.rotation[0] + offset[0];
                let pitch = s.rotation[1] + offset[1];
                let flags = s.flags;
                let mut replacement = event.packet.clone();
                let movement = match &mut replacement {
                    S::Pong(_) if settings.cancel_c0b => {
                        event.cancel();
                        return;
                    }
                    S::UseItem(p) => {
                        p.yaw = yaw;
                        p.pitch = pitch;
                        Some([s.actual[0] + offset[0], s.actual[1] + offset[1]])
                    }
                    S::Interact(_) | S::Attack(_) | S::SpectatorAction(_) | S::UseItemOn(_) => {
                        Some([yaw, pitch])
                    }
                    _ => None,
                };
                drop(s);
                if let Some([yaw, pitch]) = movement {
                    event.cancel();
                    event.send_silently(horizon_client::freeze_bridge::rotation(yaw, pitch, flags));
                    event.send_silently(replacement);
                }
            }
            _ => {}
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn freeze_drain(&self, host: &mut dyn Host) {
        let service = self.state().behaviors.borrow().freeze.clone();
        let packets = service.take_ready();
        for packet in packets {
            let result = match packet {
                Queued::Incoming(session, packet) => host.replay_client_packet(session, packet),
                Queued::Outgoing(session, packet) => host.send_packet_silently(session, packet),
            };
            if let Err(error) = result {
                eprintln!("[LiquidBounce] Freeze replay: {error}");
            }
        }
    }
    pub(crate) fn freeze_frame(&self, host: &mut dyn Host) {
        let service = self.state().behaviors.borrow().freeze.clone();
        let reason = if let Some(context) = host.client_freeze_context() {
            let rotation = {
                let state = self.state();
                let m = state.behaviors.borrow();
                let r = m.rotations.lock().expect("RotationManager");
                r.current
                    .map_or([context[5] as f32, context[6] as f32], |r| [r.yaw, r.pitch])
            };
            service.frame(context, rotation)
        } else {
            service.clear();
            None
        };
        if let Some(reason) = reason {
            let notify = service.settings().notification;
            let options = raw_options(host);
            let effects = {
                let mut state = self.state();
                let id = state.behaviors.borrow().query_ids.get("Freeze").copied();
                if let Some(id) = id {
                    if let Err(error) = state.with_registry(options, |registry, callbacks| {
                        registry.set_enabled_sync(id, false, callbacks)
                    }) {
                        eprintln!("[LiquidBounce] Freeze disable: {error:?}");
                    }
                }
                state.take_effects()
            };
            crate::runtime::apply_effects(self, host, effects);
            if notify {
                let key = "liquidbounce.module.freeze.messages.disabled";
                #[cfg(feature = "webview")]
                let (title, message) = {
                    let language = horizon_client::lang::code();
                    let cause = liquidbounce_interop::localization::display_label(
                        &language,
                        REASONS[reason],
                    );
                    (
                        liquidbounce_interop::localization::display_module_name(
                            &language, "Freeze",
                        ),
                        liquidbounce_interop::localization::translate(&language, key)
                            .unwrap_or("Module disabled due to %s.")
                            .replace("%s", &cause),
                    )
                };
                #[cfg(not(feature = "webview"))]
                let message = {
                    let _ = key;
                    format!("Disabled due to {}", REASONS[reason])
                };
                #[cfg(feature = "webview")]
                self.state().events.borrow_mut().push((
                    "notification".into(),
                    serde_json::json!({"title":title,"message":message,"severity":"INFO"}),
                ));
                #[cfg(not(feature = "webview"))]
                host.chat(&message);
            }
        }
        self.freeze_drain(host);
    }
}
