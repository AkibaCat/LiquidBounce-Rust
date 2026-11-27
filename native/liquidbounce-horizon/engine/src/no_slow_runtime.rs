// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    coroutine_ticker::WaitOwner,
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
};
use horizon_client::{
    mod_api::{Host, ServerboundPlayPacket as S},
    no_slow_bridge as bridge,
};
use liquidbounce_behaviors::no_slow::{
    self as model, Action, Command, Context, Mode, NoSlow, Send,
};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
#[derive(Default)]
struct State {
    model: NoSlow,
    tracked: Option<i32>,
    context: Context,
    callbacks: [u64; 3],
    waiting: [bool; 3],
    phase: [u8; 3],
    queue: Vec<(u64, S)>,
    slowness_applied: bool,
    warned: Option<Mode>,
}
#[derive(Default)]
pub struct NoSlowService {
    state: Mutex<State>,
    running: AtomicBool,
}
fn context(v: [f64; 15], tracked: Option<i32>) -> Context {
    Context {
        using: v[0] != 0.,
        consumable: v[14] != 0.,
        action: Action::from_id(v[1] as u8),
        hand: v[2] as i32,
        ticks: v[3] as i32,
        remaining: v[4] as i32,
        blocking: v[5] != 0.,
        moving: v[6] != 0.,
        selected: v[7] as i32,
        sneaking: v[8] != 0.,
        tracked_hand: tracked,
    }
}
fn wire(commands: Vec<Send>) -> String {
    Value::Array(
        commands
            .into_iter()
            .map(|s| {
                let mut v = match s.command {
                    Command::Carried(slot) => json!({"kind":"carried","slot":slot}),
                    Command::Use { hand, predict } => {
                        json!({"kind":"use","hand":hand,"predict":predict})
                    }
                    Command::Release { at_player } => json!({"kind":"release","player":at_player}),
                    Command::StopUsing => json!({"kind":"stop"}),
                    Command::LegacySneak(start) => json!({"kind":"legacySneak","start":start}),
                };
                v["untracked"] = json!(s.untracked);
                v
            })
            .collect(),
    )
    .to_string()
}
impl NoSlowService {
    pub(crate) fn blink_active(&self)->bool{
        let raw=bridge::context();let s=self.state.lock().expect("NoSlow");
        Self::blink(&s,self.running.load(Ordering::Acquire),raw)
    }
    pub fn settings(&self) -> NoSlow {
        self.state.lock().expect("NoSlow").model
    }
    pub fn update(&self, f: impl FnOnce(&mut NoSlow)) {
        let mut s = self.state.lock().expect("NoSlow");
        let old = s.model;
        f(&mut s.model);
        if old.settings.uses.map(|g| g.mode) != s.model.settings.uses.map(|g| g.mode)
            || old.settings.uses.map(|g| g.enabled) != s.model.settings.uses.map(|g| g.enabled)
        {
            s.model.prevent = false;
            for i in 0..3 {
                s.callbacks[i] = s.callbacks[i].wrapping_add(1);
                s.waiting[i] = false;
                s.phase[i] = 0;
            }
        }
        self.publish(&s);
    }
    fn publish(&self, s: &State) {
        bridge::configure(
            s.model
                .policy(self.running.load(Ordering::Acquire), s.context),
        );
    }
    pub fn set_running(&self, running: bool) {
        if self.running.swap(running, Ordering::AcqRel) && !running {
            self.cancel();
        }
        let s = self.state.lock().expect("NoSlow");
        self.publish(&s);
    }
    pub fn cancel(&self) {
        let mut s = self.state.lock().expect("NoSlow");
        s.model.prevent = false;
        for i in 0..3 {
            s.callbacks[i] = s.callbacks[i].wrapping_add(1);
            s.waiting[i] = false;
            s.phase[i] = 0;
        }
        self.publish(&s);
    }
    pub fn world_change(&self) {
        self.cancel();
        let mut s = self.state.lock().expect("NoSlow");
        s.tracked = None;
        s.queue.clear();
        s.context = Context::default();
        self.publish(&s);
        bridge::set_context(None);
    }
    fn sync(&self, raw: Option<[f64; 15]>) {
        let mut s = self.state.lock().expect("NoSlow");
        s.context = raw.map(|v| context(v, s.tracked)).unwrap_or_default();
        self.publish(&s);
    }
    fn invalid(&self, c: Context) -> bool {
        let s = self.state.lock().expect("NoSlow");
        (0..3).any(|g| {
            s.model
                .group_running(self.running.load(Ordering::Acquire), c, g)
                && s.model.settings.uses[g].mode == Mode::InvalidHand
        })
    }
    fn blink(s: &State, running: bool, raw: Option<[f64; 15]>) -> bool {
        s.model.group_running(running, s.context, 0)
            && s.model.settings.uses[0].mode == Mode::Blink
            && raw.is_some_and(|v| v[11] != 0.)
    }
}
impl PacketListener for NoSlowService {
    fn outgoing(&self, e: &mut PacketEvent<'_, S>) {
        if !e.context.original() {
            return;
        }
        let raw = bridge::context();
        let Some(raw) = raw else {
            return;
        };
        let running = self.running.load(Ordering::Acquire);
        let (cancel, duplicate) = {
            let mut s = self.state.lock().expect("NoSlow");
            // InteractionTracker is active even while NoSlow itself is disabled.
            if !bridge::untracked() {
                match e.packet {
                    S::UseItem(p) => {
                        s.tracked =
                            if (0..2).contains(&p.hand) && raw[9 + p.hand as usize] as u8 == 3 {
                                Some(p.hand)
                            } else {
                                None
                            };
                    }
                    S::SetCarriedItem(_) => {
                        if s.tracked == Some(0) {
                            s.tracked = None;
                        }
                    }
                    S::PlayerAction(p) if matches!(p.action, 5 | 6) => s.tracked = None,
                    _ => {}
                }
            }
            s.context = context(raw, s.tracked);
            self.publish(&s);
            let c = s.context;
            let mut cancel = false;
            let mut duplicate = None;
            if let S::UseItemOn(p) = e.packet {
                let action = if p.hand == 1 {
                    Action::from_id(raw[10] as u8)
                } else {
                    Action::from_id(raw[9] as u8)
                };
                let air = e
                    .client_player
                    .and_then(|v| v.no_slow_block_air(p.pos))
                    .unwrap_or(false);
                cancel = air
                    && [1usize, 2].into_iter().any(|g| {
                        s.model.group_running(running, c, g)
                            && s.model.settings.no_block_interact[g - 1]
                            && action.group() == Some(g)
                    });
                if s.model.group_running(running, c, 0)
                    && s.model.settings.uses[0].mode == Mode::Intave14
                    && c.using
                    && c.moving
                {
                    let mut p = p.clone();
                    p.cursor = [0.5; 3];
                    duplicate = Some(S::UseItemOn(p));
                }
            }
            (cancel, duplicate)
        };
        if cancel {
            e.cancel();
        }
        if let Some(p) = duplicate {
            bridge::intave_nested(|| e.send(p));
        }
    }
}
pub struct NoSlowFinal(pub Arc<NoSlowService>);
impl PacketListener for NoSlowFinal {
    fn outgoing(&self, e: &mut PacketEvent<'_, S>) {
        if e.is_cancelled() || !e.context.original() {
            return;
        }
        let raw = bridge::context();
        let c = {
            let s = self.0.state.lock().expect("NoSlow");
            raw.map(|v| context(v, s.tracked)).unwrap_or(s.context)
        };
        if let S::UseItem(p) = e.packet
            && self.0.invalid(c)
        {
            let mut p = p.clone();
            p.hand = -1;
            e.cancel();
            e.send_silently(S::UseItem(p));
            return;
        }
    }
}
pub struct NoSlowBlink(pub Arc<NoSlowService>);
impl PacketListener for NoSlowBlink {
    fn outgoing(&self, e: &mut PacketEvent<'_, S>) {
        if e.is_cancelled() || !e.context.original() {
            return;
        }
        let raw = bridge::context();
        let mut s = self.0.state.lock().expect("NoSlow");
        let active = NoSlowService::blink(&s, self.0.running.load(Ordering::Acquire), raw);
        if active
            && matches!(
                e.packet,
                S::MovePlayerPos(_)
                    | S::MovePlayerPosRot(_)
                    | S::MovePlayerRot(_)
                    | S::MovePlayerStatus(_)
            )
        {
            s.queue.push((e.context.session, e.packet.clone()));
            e.cancel();
            return;
        }
        if !active && !s.queue.is_empty() {
            let packets = std::mem::take(&mut s.queue);
            drop(s);
            for (session, p) in packets {
                if session == e.context.session {
                    e.send_silently(p);
                }
            }
        }
    }
}
impl LiquidBounceHooks {
    fn no_slow_service(&self) -> Arc<NoSlowService> {
        self.state().behaviors.borrow().no_slow.clone()
    }
    pub(crate) fn no_slow_frame(&self, host: &mut dyn Host) {
        let service = self.no_slow_service();
        let raw = host.client_no_slow_context();
        service.sync(raw);
        let queue = {
            let mut s = service.state.lock().expect("NoSlow");
            if NoSlowService::blink(&s, service.running.load(Ordering::Acquire), raw) {
                Vec::new()
            } else {
                std::mem::take(&mut s.queue)
            }
        };
        for (session, p) in queue {
            let _ = host.send_packet_silently(session, p);
        }
    }
    pub(crate) fn no_slow_input(
        &self,
        host: &mut dyn Host,
        event: &mut horizon_client::mod_api::ClientDirectionalInputEvent,
    ) {
        let s = self.no_slow_service();
        s.sync(host.client_no_slow_context());
        let state = s.state.lock().expect("NoSlow");
        let mut c = state.context;
        c.moving = (event.forward != event.backward) || (event.left != event.right);
        c.sneaking = event.shift;
        bridge::configure(state.model.policy(s.running.load(Ordering::Acquire), c));
        if state.model.jump(s.running.load(Ordering::Acquire), c) {
            event.jump = true;
        }
    }
    pub(crate) fn no_slow_network(&self, host: &mut dyn Host, pre: bool) {
        let service = self.no_slow_service();
        service.sync(host.client_no_slow_context());
        let commands = {
            let s = service.state.lock().expect("NoSlow");
            s.model
                .network(service.running.load(Ordering::Acquire), s.context, pre)
        };
        host.client_no_slow_send(&wire(commands));
    }
    pub(crate) fn no_slow_tick(&self, host: &mut dyn Host) {
        let service = self.no_slow_service();
        let raw = host.client_no_slow_context();
        service.sync(raw);
        let running = service.running.load(Ordering::Acquire) && raw.is_some();
        let (registrations, slowness, warn, queue) = {
            let mut s = service.state.lock().expect("NoSlow");
            let mut ids = Vec::new();
            for g in 0..3 {
                let active = s.model.group_running(running, s.context, g)
                    && s.model.settings.uses[g].mode == Mode::Grim2371;
                if !active && s.waiting[g] {
                    s.waiting[g] = false;
                    s.callbacks[g] = s.callbacks[g].wrapping_add(1);
                    s.phase[g] = 0;
                    s.model.prevent = false;
                }
                if active && !s.waiting[g] {
                    s.waiting[g] = true;
                    s.phase[g] = 0;
                    s.callbacks[g] = s.callbacks[g].wrapping_add(1);
                    ids.push((s.callbacks[g] << 2) | g as u64);
                }
            }
            let slowness = if running && s.model.settings.enabled[6] {
                s.slowness_applied = true;
                Some(s.model.slowness)
            } else if std::mem::take(&mut s.slowness_applied) {
                Some(0.15)
            } else {
                None
            };
            let mode = s.model.settings.sneak_mode;
            let wants =
                running && s.model.settings.enabled[0] && matches!(mode, Mode::Switch | Mode::Aac5);
            let warn = wants && s.warned != Some(mode);
            s.warned = wants.then_some(mode);
            let queue = if !NoSlowService::blink(&s, running, raw) {
                std::mem::take(&mut s.queue)
            } else {
                Vec::new()
            };
            service.publish(&s);
            (ids, slowness, warn, queue)
        };
        for id in registrations {
            self.state()
                .behaviors
                .borrow()
                .coroutine_ticker
                .register(WaitOwner::NoSlow, id);
        }
        if let Some(value) = slowness {
            host.client_no_slow_slowness(value);
        }
        if warn {
            self.sneak_protocol_warning();
        }
        for (session, p) in queue {
            let _ = host.send_packet_silently(session, p);
        }
    }
    pub(crate) fn no_slow_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let service = self.no_slow_service();
        service.sync(host.client_no_slow_context());
        let g = (id & 3) as usize;
        if g >= 3 {
            return false;
        }
        let command = {
            let mut s = service.state.lock().expect("NoSlow");
            if id >> 2 != s.callbacks[g] || !s.waiting[g] {
                return false;
            }
            if !in_game
                || !s
                    .model
                    .group_running(service.running.load(Ordering::Acquire), s.context, g)
                || s.model.settings.uses[g].mode != Mode::Grim2371
            {
                s.model.prevent = false;
                s.waiting[g] = false;
                s.phase[g] = 0;
                service.publish(&s);
                return false;
            }
            s.model.prevent = false;
            let command = Send {
                command: Command::Use {
                    hand: s.context.hand,
                    predict: true,
                },
                untracked: false,
            };
            s.phase[g] += 1;
            service.publish(&s);
            command
        };
        host.client_no_slow_send(&wire(vec![command]));
        let mut s = service.state.lock().expect("NoSlow");
        if id >> 2 == s.callbacks[g] && s.waiting[g] && s.phase[g] >= 2 {
            s.phase[g] = 0;
            s.model.prevent = true;
            s.waiting[g] = false;
            service.publish(&s);
            return false;
        }
        true
    }
}
