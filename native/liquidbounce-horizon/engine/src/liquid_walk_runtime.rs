// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
    timer_runtime::TimerService,
};
use horizon_client::mod_api::{Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::liquid_walk::{LiquidWalk, Settings};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
pub const TIMER_PROVIDER: u64 = 0x4c49_5155_4944;
pub struct LiquidWalkService {
    behavior: Mutex<LiquidWalk>,
    running: AtomicBool,
    timer: Arc<TimerService>,
}
impl LiquidWalkService {
    pub fn new(timer: Arc<TimerService>) -> Self {
        Self {
            behavior: Mutex::default(),
            running: AtomicBool::new(false),
            timer,
        }
    }
    pub fn settings(&self) -> Settings {
        self.behavior.lock().expect("LiquidWalk").settings
    }
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.behavior.lock().expect("LiquidWalk").settings);
        self.publish();
    }
    fn publish(&self) {
        horizon_client::liquid_walk_native::set_mode(if self.running.load(Ordering::Acquire) {
            self.settings().mode.code()
        } else {
            0
        });
    }
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::Release);
        self.timer.set_provider_running(TIMER_PROVIDER, running);
        self.publish();
    }
    fn tick(&self, context: [bool; 5]) -> Option<f64> {
        let effect = self.behavior.lock().expect("LiquidWalk").tick(
            self.running.load(Ordering::Acquire),
            context[4],
            [context[0], context[1], context[2], context[3]],
        );
        if let Some(speed) = effect.timer {
            self.timer.state.lock().expect("Timer").timer.request_speed(
                speed,
                20,
                TIMER_PROVIDER,
                1,
            );
        }
        effect.velocity_y
    }
    pub fn clear(&self) {
        horizon_client::liquid_walk_native::clear();
    }
}
impl PacketListener for LiquidWalkService {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        if !event.context.original() || !self.running.load(Ordering::Acquire) {
            return;
        }
        if !matches!(
            event.packet,
            S::MovePlayerPos(_)
                | S::MovePlayerPosRot(_)
                | S::MovePlayerRot(_)
                | S::MovePlayerStatus(_)
        ) {
            return;
        }
        let (context, in_water) = event
            .client_player
            .and_then(|p| p.liquid_walk_context())
            .unwrap_or_else(horizon_client::liquid_walk_native::packet_context);
        let effect = self.behavior.lock().expect("LiquidWalk").packet(
            true,
            horizon_client::liquid_walk_native::shift(),
            in_water,
            context,
        );
        let flags = match event.packet {
            S::MovePlayerPos(p) => {
                if effect.subtract_y {
                    p.y -= 0.001;
                }
                &mut p.flags
            }
            S::MovePlayerPosRot(p) => {
                if effect.subtract_y {
                    p.y -= 0.001;
                }
                &mut p.flags
            }
            S::MovePlayerRot(p) => &mut p.flags,
            S::MovePlayerStatus(p) => &mut p.flags,
            _ => unreachable!(),
        };
        if let Some(ground) = effect.ground {
            *flags = (*flags & !1) | u8::from(ground);
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn liquid_walk_tick(&self, host: &mut dyn Host) {
        let service = self.state().behaviors.borrow().liquid_walk.clone();
        service.publish();
        if let Some(context) = host.client_liquid_walk_context() {
            if let Some(y) = service.tick(context) {
                host.client_high_jump_vertical(y);
            }
        } else {
            service.clear();
        }
    }
}
