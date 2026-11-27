// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
    timer_runtime::TimerService,
};
use horizon_client::mod_api::{
    ClientboundPlayPacket as Clientbound, Host, ServerboundPlayPacket as Serverbound,
};
use liquidbounce_behaviors::super_knockback::{Player, Random, Step, Target, cooldown::Cooldown};
use std::sync::{Arc, Mutex};
pub struct CombatCooldown {
    timer: Arc<TimerService>,
    state: Mutex<Cooldown>,
}
impl CombatCooldown {
    pub fn new(timer: Arc<TimerService>) -> Self {
        Self {
            timer,
            state: Mutex::new(Cooldown::default()),
        }
    }
    fn now(&self) -> f64 {
        self.timer
            .state
            .lock()
            .expect("Timer poisoned")
            .timer
            .real_ticks
    }
    pub fn drain(&self) {
        let now = self.now();
        self.state
            .lock()
            .expect("CombatCooldown poisoned")
            .drain(now);
    }
    pub fn snapshot(&self, rate:f32, attack_delay:f32, has_cooldown:bool)->liquidbounce_behaviors::clicking::Cooldown {
        let real_ticks=self.now();let mut state=self.state.lock().expect("CombatCooldown poisoned");state.drain(real_ticks);
        liquidbounce_behaviors::clicking::Cooldown {real_ticks,reset_ticks:state.reset_ticks,server_tick_speed:if state.tps.is_nan(){1.0}else{state.tps/f64::from(rate)},attack_delay,has_cooldown}
    }
    pub fn progress(&self, rate: f32, delay: f32) -> f32 {
        let now = self.now();
        let mut state = self.state.lock().expect("CombatCooldown poisoned");
        state.drain(now);
        state.progress(now, rate, delay)
    }
    pub fn world_change(&self) {
        let now = self.now();
        self.state
            .lock()
            .expect("CombatCooldown poisoned")
            .world_change(now);
    }
    pub fn disconnect(&self) {
        let now = self.now();
        self.state
            .lock()
            .expect("CombatCooldown poisoned")
            .disconnect(now);
    }
}
impl PacketListener for CombatCooldown {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if matches!(
            event.packet,
            Serverbound::Attack(_) | Serverbound::Punch(_) | Serverbound::SetCarriedItem(_)
        ) {
            let now = self.now();
            self.state
                .lock()
                .expect("CombatCooldown poisoned")
                .outgoing(now);
        }
    }
    fn incoming(&self, event: &mut PacketEvent<'_, Clientbound>) {
        match event.packet {
            Clientbound::Respawn(_) => self
                .state
                .lock()
                .expect("CombatCooldown poisoned")
                .respawn(),
            Clientbound::SetTime(_) => self
                .state
                .lock()
                .expect("CombatCooldown poisoned")
                .set_time(crate::behaviors::current_time_millis()),
            _ => {}
        }
    }
}
struct LiveRandom;
impl Random for LiveRandom {
    fn unit(&mut self) -> f32 {
        liquidbounce_behaviors::random::random_unit_f32()
    }
    fn ticks(&mut self, r: [i32; 2]) -> Result<i32, String> {
        if r[0] > r[1] {
            return Err("empty SuperKnockback tick range".into());
        }
        Ok(liquidbounce_behaviors::random::random_i32_inclusive(
            r[0], r[1],
        ))
    }
}
fn player(c: &serde_json::Value) -> Player {
    Player {
        forward: c["forward"].as_f64().unwrap_or(0.0) as f32,
        sideways: c["sideways"].as_f64().unwrap_or(0.0) as f32,
        sprinting: c["sprinting"].as_bool().unwrap_or(false),
        was_sprinting: c["wasSprinting"].as_bool().unwrap_or(false),
        on_ground: c["onGround"].as_bool().unwrap_or(false),
        water_or_bubble: c["waterOrBubble"].as_bool().unwrap_or(false),
    }
}
impl LiquidBounceHooks {
    fn super_knockback_step(&self, host: &mut dyn Host, step: Step) {
        for start in step.packets {
            host.client_super_knockback_packet(start);
        }
        if step.mark_sprint {
            host.client_super_knockback_mark_sprint();
        }
        if let Some(id) = step.wait {
            self.state()
                .behaviors
                .borrow()
                .coroutine_ticker
                .register(WaitOwner::SuperKnockback, id);
        }
    }
    pub(crate) fn super_knockback_attack(&self, host: &mut dyn Host, target: u32) {
        let (running, cooldown) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.query_running(&s.registry, "SuperKnockback"),
                b.combat_cooldown.clone(),
            )
        };
        if !running {
            return;
        }
        let Some(c) = host.client_super_knockback_context(target) else {
            return;
        };
        let progress = cooldown.progress(
            c["tickRate"].as_f64().unwrap_or(20.0) as f32,
            c["attackDelay"].as_f64().unwrap_or(5.0) as f32,
        );
        let p = player(&c);
        let t = Target {
            living: c["living"].as_bool().unwrap_or(false),
            hurt_time: c["hurtTime"].as_i64().unwrap_or(0) as i32,
            facing: c["facing"].as_bool().unwrap_or(false),
            would_critical: c["criticalAllowed"].as_bool().unwrap_or(false)
                && progress > 0.9
                && (!p.sprinting || c["legacy"].as_bool().unwrap_or(false))
                && c["fallDistance"].as_f64().unwrap_or(0.0) > 0.0,
        };
        let step = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            b.super_knockback.attack(running, p, t, &mut LiveRandom)
        };
        match step {
            Ok(step) => self.super_knockback_step(host, step),
            Err(e) => {
                self.state().behaviors.borrow_mut().super_knockback.cancel();
                eprintln!("[LiquidBounce] SuperKnockback: {e}");
            }
        }
    }
    pub(crate) fn super_knockback_callback(
        &self,
        host: &mut dyn Host,
        in_game: bool,
        id: u64,
    ) -> bool {
        let c = host.client_super_knockback_context(u32::MAX);
        let p = c.as_ref().map(player);
        let result = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let running = in_game && b.query_running(&s.registry, "SuperKnockback");
            b.super_knockback.callback(id, running, p, &mut LiveRandom)
        };
        match result {
            Ok((keep, step)) => {
                self.super_knockback_step(host, step);
                keep
            }
            Err(e) => {
                self.state().behaviors.borrow_mut().super_knockback.cancel();
                eprintln!("[LiquidBounce] SuperKnockback: {e}");
                false
            }
        }
    }
}
