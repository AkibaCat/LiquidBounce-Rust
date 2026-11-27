// SPDX-License-Identifier: GPL-3.0-or-later
//! Criticals uses the shared combat clock, real movement packets and Blink queue.
use crate::{
    freeze_runtime::FreezeService,
    packets::{PacketEvent, PacketListener},
    runtime::LiquidBounceHooks,
    super_knockback_runtime::CombatCooldown,
    timer_runtime::TimerService,
};
use horizon_client::mod_api::{ClientboundPlayPacket as C, Host, ServerboundPlayPacket as S};
use liquidbounce_behaviors::{
    clicking::Cooldown,
    criticals::{self as model, Blink, BlinkAction, Criticals, Player, Settings},
};
use std::sync::{Arc, Mutex};
pub const TIMER_PROVIDER: u64 = 0x4352_4954_4943;
enum Packet {
    Incoming(C),
    Outgoing(S),
}
struct State {
    behavior: Criticals,
    running: bool,
    player: Option<(Player, Cooldown)>,
    blink: Blink<Packet>,
    ready: Vec<model::Queued<Packet>>,
}
pub struct CriticalsService {
    state: Mutex<State>,
    timer: Arc<TimerService>,
    cooldown: Arc<CombatCooldown>,
    freeze: Arc<FreezeService>,
    blink_module:Arc<crate::blink_runtime::BlinkService>,
    no_slow:Arc<crate::no_slow_runtime::NoSlowService>,
    anti_void:Arc<crate::anti_void_runtime::AntiVoidService>,
    fake_lag:Arc<crate::fake_lag_runtime::FakeLagService>,
    kill_aura:Arc<crate::kill_aura_runtime::KillAuraBlocking>,
    scaffold:Arc<crate::scaffold_runtime::ScaffoldBlink>,
}
fn sample(r: [i32; 2]) -> i32 {
    liquidbounce_behaviors::random::random_i32_inclusive(r[0], r[1])
}
fn player(v: &serde_json::Value) -> Option<Player> {
    Some(Player {
        allows_ignoring_ground: v["allowsCritical"].as_bool()?,
        on_ground: v["onGround"].as_bool()?,
        sprinting: v["sprinting"].as_bool()?,
        was_sprinting: v["wasSprinting"].as_bool()?,
        legacy: v["legacy"].as_bool()?,
        fall_distance: v["fallDistance"].as_f64()?,
        y: v["y"].as_f64()?,
        old_y: v["oldY"].as_f64()?,
        tick_count: v["tickCount"].as_i64()? as i32,
    })
}
fn movement(p: &S) -> bool {
    matches!(
        p,
        S::MovePlayerPos(_) | S::MovePlayerPosRot(_) | S::MovePlayerRot(_) | S::MovePlayerStatus(_)
    )
}
fn change_movement(p: &mut S, y: Option<f64>) {
    match p {
        S::MovePlayerPos(p) => {
            if let Some(y) = y {
                p.y += y;
            }
            p.flags &= !1;
        }
        S::MovePlayerPosRot(p) => {
            if let Some(y) = y {
                p.y += y;
            }
            p.flags &= !1;
        }
        S::MovePlayerRot(p) => p.flags &= !1,
        S::MovePlayerStatus(p) => p.flags &= !1,
        _ => {}
    }
}
fn pass(p: &S) -> bool {
    matches!(
        p,
        S::UseItemOn(_)
            | S::PlayerAction(_)
            | S::SignUpdate(_)
            | S::Interact(_)
            | S::Attack(_)
            | S::SpectatorAction(_)
            | S::Punch(_)
            | S::ResourcePack(_)
    )
}
impl CriticalsService {
    pub fn new(
        timer: Arc<TimerService>,
        cooldown: Arc<CombatCooldown>,
        freeze: Arc<FreezeService>,
        blink_module:Arc<crate::blink_runtime::BlinkService>,
        no_slow:Arc<crate::no_slow_runtime::NoSlowService>,
        anti_void:Arc<crate::anti_void_runtime::AntiVoidService>,
    fake_lag:Arc<crate::fake_lag_runtime::FakeLagService>,
    kill_aura:Arc<crate::kill_aura_runtime::KillAuraBlocking>,
    scaffold:Arc<crate::scaffold_runtime::ScaffoldBlink>,
    ) -> Self {
        Self {
            state: Mutex::new(State {
                behavior: Criticals::default(),
                running: false,
                player: None,
                blink: Blink::new(sample(Settings::default().blink_delay)),
                ready: vec![],
            }),
            timer,
            cooldown,
            freeze,
            blink_module,
            no_slow,
            anti_void,
            fake_lag,
            kill_aura,
            scaffold,
        }
    }
    pub fn settings(&self) -> Settings {
        self.state.lock().expect("Criticals").behavior.settings
    }
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        let mut s = self.state.lock().expect("Criticals");
        let mode = s.behavior.settings.mode;
        f(&mut s.behavior.settings);
        let running = s.running;
        s.behavior.transition(running, running, mode);
        if mode == 4 && s.behavior.settings.mode != 4 {
            s.blink.in_state = false;
        }
    }
    pub fn set_running(&self, running: bool) {
        let mut s = self.state.lock().expect("Criticals");
        let old = s.running;
        let mode = s.behavior.settings.mode;
        s.behavior.transition(old, running, mode);
        s.running = running;
        if !running {
            s.blink.in_state = false;
        }
        drop(s);
        self.timer.set_provider_running(TIMER_PROVIDER, running);
    }
    pub fn disconnect(&self) {
        let mut s = self.state.lock().expect("Criticals");
        s.blink.disconnect();
        s.ready.clear();
        s.player = None;
    }
    pub fn selection(&self) -> (f32, bool, bool) {
        let s = self.state.lock().expect("Criticals");
        (
            s.behavior.settings.jump_height,
            s.behavior.adjust_next_jump,
            s.running && s.behavior.settings.mode == 4 && s.blink.in_state,
        )
    }
    fn snapshot(&self, v: &serde_json::Value) -> Option<(Player, Cooldown)> {
        Some((
            player(v)?,
            self.cooldown.snapshot(
                v["tickRate"].as_f64()? as f32,
                v["attackDelay"].as_f64()? as f32,
                v["hasCooldown"].as_bool()?,
            ),
        ))
    }
    fn refresh(&self, v: Option<serde_json::Value>) -> Option<(Player, Cooldown)> {
        let p = v.as_ref().and_then(|v| self.snapshot(v));
        self.state.lock().expect("Criticals").player = p;
        p
    }
    fn live<P>(&self, event: &PacketEvent<'_, P>) -> Option<(Player, Cooldown)> {
        if let Some(v) = event.client_player.and_then(|p| p.criticals_snapshot()) {
            let p = self.snapshot(&v);
            self.state.lock().expect("Criticals").player = p;
            p
        } else {
            self.state.lock().expect("Criticals").player
        }
    }
    fn take_origin(s: &mut State, outgoing: bool) -> Vec<model::Queued<Packet>> {
        let mut ready = vec![];
        let mut keep = std::collections::VecDeque::new();
        for p in s.blink.queue.drain(..) {
            if matches!(p.packet, Packet::Outgoing(_)) == outgoing {
                ready.push(p);
            } else {
                keep.push_back(p);
            }
        }
        s.blink.queue = keep;
        ready
    }
    fn action(&self, s: &mut State, outgoing: bool, pass: bool, moving:Option<bool>, container:bool, fake_packet:liquidbounce_behaviors::fake_lag::PacketKind, use_item:bool) -> BlinkAction {
        // Criticals' normal BlinkPacketEvent listener runs before Freeze's
        // SAFETY_FEATURE (-50), whose Queue action takes final precedence.
        let crit = s.player.is_some_and(|(p, c)| p.would_crit(c, true));
        let running = s.running && s.behavior.settings.mode == 4;
        let action = if outgoing {
            let delay = s.behavior.settings.blink_delay;
            s.blink.action(
                running,
                crit,
                crate::behaviors::current_time_millis(),
                pass,
                || sample(delay),
            )
        } else {
            if running {
                s.blink.in_state = false;
            }
            BlinkAction::Flush
        };
        let fake_lag=self.fake_lag.queues(outgoing,fake_packet,s.blink.queue.front().map(|p|p.millis));
        let inventory=if outgoing{horizon_client::inventory_move::blink(container)}else{0};
        if inventory==2{return BlinkAction::Queue;}
        // BlinkPacketEvent.action is monotonic: QUEUE > PASS > FLUSH.
        if self.scaffold.queues(outgoing,use_item)||fake_lag||self.blink_module.queues(outgoing)||self.freeze.blink_queue(outgoing)||self.anti_void.queues(outgoing)||self.kill_aura.queues(outgoing,use_item){return BlinkAction::Queue;}
        if outgoing&&self.no_slow.blink_active(){
            if moving==Some(true){return BlinkAction::Queue;}
            if action==BlinkAction::Flush{return BlinkAction::Pass;}
        }
        if inventory==1&&action==BlinkAction::Flush{BlinkAction::Pass}else{action}
    }
    fn probe(&self, outgoing: bool) -> Vec<model::Queued<Packet>> {
        let mut s = self.state.lock().expect("Criticals");
        if self.action(&mut s, outgoing, false,None,false,Default::default(),false) == BlinkAction::Flush {
            let packets = Self::take_origin(&mut s, outgoing);
            s.ready.extend(packets);
        }
        let mut ready = vec![];
        let mut keep = vec![];
        for p in s.ready.drain(..) {
            if matches!(p.packet, Packet::Outgoing(_)) == outgoing {
                ready.push(p);
            } else {
                keep.push(p);
            }
        }
        s.ready = keep;
        ready
    }
    pub(crate) fn is_lagging(&self)->bool{!self.state.lock().expect("BlinkManager").blink.queue.is_empty()}
    pub(crate) fn positions(&self)->Vec<[f64;3]>{
        self.state.lock().expect("BlinkManager").blink.queue.iter().filter_map(|s|Self::packet_position(&s.packet)).collect()
    }
    fn packet_position(p:&Packet)->Option<[f64;3]>{match p{
        Packet::Outgoing(S::MovePlayerPos(p))=>Some([p.x,p.y,p.z]),
        Packet::Outgoing(S::MovePlayerPosRot(p))=>Some([p.x,p.y,p.z]),_=>None,
    }}
    pub(crate) fn flush_origins(&self,mask:u32){
        let mut s=self.state.lock().expect("BlinkManager");
        for outgoing in [false,true]{if mask&(if outgoing{2}else{1})!=0{let ready=Self::take_origin(&mut s,outgoing);s.ready.extend(ready);}}
    }
    fn operation_queue(s:&mut State)->liquidbounce_behaviors::blink::Queue<model::Queued<Packet>>{
        let mut q=liquidbounce_behaviors::blink::Queue::default();
        for item in s.blink.queue.drain(..){let position=Self::packet_position(&item.packet);let outgoing=matches!(item.packet,Packet::Outgoing(_));let movement=matches!(&item.packet,Packet::Outgoing(p)if movement(p));
            q.0.push_back(liquidbounce_behaviors::blink::Snapshot{outgoing,millis:item.millis,position,movement,packet:item});}
        q
    }
    pub(crate) fn flush_count(&self,count:i32){
        let mut s=self.state.lock().expect("BlinkManager");let mut q=Self::operation_queue(&mut s);
        s.ready.extend(q.flush_count(count).into_iter().map(|s|s.packet));s.blink.queue=q.0.into_iter().map(|s|s.packet).collect();
    }
    pub(crate) fn cancel_shared(&self)->Option<[f64;3]>{
        let mut s=self.state.lock().expect("BlinkManager");let mut q=Self::operation_queue(&mut s);let(at,ready)=q.cancel();s.ready.extend(ready.into_iter().map(|s|s.packet));at
    }
    pub(crate) fn flush_outgoing_event(&self,event:&mut PacketEvent<'_,S>){
        let mut s=self.state.lock().expect("BlinkManager");let mut keep=Vec::new();let mut ready=Vec::new();
        for p in s.ready.drain(..){if matches!(p.packet,Packet::Outgoing(_)){ready.push(p);}else{keep.push(p);}}
        s.ready=keep;drop(s);
        let mut pending=Vec::new();
        for p in ready {if p.session==event.context.session {if let Packet::Outgoing(packet)=&p.packet {if !event.send_silently(packet.clone()){pending.push(p);}}}}
        self.state.lock().expect("BlinkManager").ready.extend(pending);
    }

    pub fn jump(&self, motion: &mut f32) {
        let mut s = self.state.lock().expect("Criticals");
        if s.running && s.behavior.settings.mode == 3 {
            s.behavior.jump(motion);
        }
    }
}
pub struct NoGroundListener(pub Arc<CriticalsService>);
impl PacketListener for NoGroundListener {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        let s = self.0.state.lock().expect("Criticals");
        if s.running && s.behavior.settings.mode == 2 {
            change_movement(event.packet, None);
        }
    }
}
pub struct RewoundListener(pub Arc<CriticalsService>);
impl PacketListener for RewoundListener {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        if !movement(event.packet) {
            return;
        }
        let Some((p, _)) = self.0.live(event) else {
            return;
        };
        let mut s = self.0.state.lock().expect("Criticals");
        if s.running && s.behavior.settings.mode == 6 {
            if let Some(y) = s.behavior.rewound(p) {
                change_movement(event.packet, Some(y));
            }
        }
    }
}
pub struct BlinkListener(pub Arc<CriticalsService>);
impl PacketListener for BlinkListener {
    fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
        self.0.kill_aura.outgoing(event.packet);
        if event.is_cancelled() {
            return;
        }
        if let Some(dead)=event.client_player.and_then(|p|p.anti_void_dead()){self.0.anti_void.observe_dead(dead);}
        self.0.live(event);
        let mut s = self.0.state.lock().expect("Criticals");
        let ready = match self.0.action(&mut s, true, pass(event.packet),Some(movement(event.packet)),horizon_client::inventory_move::container_packet(event.packet),crate::fake_lag_runtime::packet(event.packet),matches!(event.packet,S::UseItem(_))) {
            BlinkAction::Flush => CriticalsService::take_origin(&mut s, true),
            BlinkAction::Queue => {
                if !matches!(event.packet, S::Chat(_) | S::ChatCommand(_)) {
                    s.blink.enqueue(
                        event.context.session,
                        crate::behaviors::current_time_millis(),
                        Packet::Outgoing(event.packet.clone()),
                    );
                    event.cancel();
                }
                vec![]
            }
            BlinkAction::Pass => vec![],
        };
        drop(s);
        let mut deferred = vec![];
        for p in ready {
            if p.session == event.context.session {
                if let Packet::Outgoing(packet) = p.packet {
                    if !event.send_silently(packet.clone()) {
                        deferred.push(model::Queued {
                            session: p.session,
                            millis: p.millis,
                            packet: Packet::Outgoing(packet),
                        });
                    }
                }
            }
        }
        self.0
            .state
            .lock()
            .expect("Criticals")
            .ready
            .extend(deferred);
    }
    fn incoming(&self, event: &mut PacketEvent<'_, C>) {
        if event.is_cancelled() {
            return;
        }
        let mut s = self.0.state.lock().expect("Criticals");
        let action = self.0.action(&mut s, false, false,None,false,Default::default(),false);
        let flush = matches!(
            event.packet,
            C::PlayerPosition(_) | C::Disconnect(_) | C::Respawn(_) | C::Login(_)
        ) || matches!(event.packet,C::SetHealth(p) if p.health<=0.);
        if action == BlinkAction::Flush || flush {
            let ready = CriticalsService::take_origin(&mut s, false);
            drop(s);
            let mut deferred=Vec::new();
            for p in ready {if p.session==event.context.session {
                if let Packet::Incoming(packet)=&p.packet {if !event.replay_before(packet.clone()){deferred.push(p);}}
            }}
            self.0.state.lock().expect("BlinkManager").ready.extend(deferred);
            return;
        }
        if action == BlinkAction::Pass
            || matches!(
                event.packet,
                C::SystemChat(_) | C::DisguisedChat(_) | C::StartConfiguration(_) | C::Transfer(_)
            )
        {
            return;
        }
        if self.0.freeze.blink_own_hurt(event.packet) {
            return;
        }
        s.blink.enqueue(
            event.context.session,
            crate::behaviors::current_time_millis(),
            Packet::Incoming(event.packet.clone()),
        );
        event.cancel();
    }
}
impl LiquidBounceHooks {
    fn criticals_service(&self) -> Arc<CriticalsService> {
        self.state().behaviors.borrow().criticals.clone()
    }
    fn criticals_enemy(&self, host: &dyn Host, range: f32, visible: bool) -> bool {
        // Copy native entities first: both target facts and Host callbacks can
        // dispatch module hooks, so the registry is never held across them.
        let candidates = host.client_criticals_candidates(range, visible);
        let mut state = self.state();
        candidates.into_iter().any(|v| {
            let Some(id) = v["id"].as_i64() else {
                return false;
            };
            let Some(target) = crate::targeting::snapshot(&v["target"].to_string()) else {
                return false;
            };
            let info = state.target_info(id as i32, &target);
            state
                .global_targets
                .combat
                .should_be_attacked(&target.entity, info)
        })
    }
    pub(crate) fn criticals_stop_sprint(&self, host: &dyn Host, source: u8) -> bool {
        let service = self.criticals_service();
        let settings = service.settings();
        if !settings.sprint_enabled {
            return false;
        }
        let Some((p, c)) = service.refresh(host.client_criticals_context()) else {
            return false;
        };
        let enemy =
            p.would_crit(c, true) && self.criticals_enemy(host, settings.sprint_range, false);
        let s = service.state.lock().expect("Criticals");
        let running = s.behavior.sprint_running(s.running, p, c, enemy);
        s.behavior.stop_sprint(source, running)
    }
    pub(crate) fn criticals_attack_sprint(&self, host: &mut dyn Host) {
        let service = self.criticals_service();
        let settings = service.settings();
        let Some((p, c)) = service.refresh(host.client_criticals_context()) else {
            return;
        };
        let enemy = settings.sprint_enabled
            && p.would_crit(c, true)
            && self.criticals_enemy(host, settings.sprint_range, false);
        let when = {
            let s = service.state.lock().expect("Criticals");
            s.behavior.sprint_running(s.running, p, c, enemy)
        };
        if when && settings.stop_sprinting == 3 && p.was_sprinting {
            host.client_criticals_stop();
        }
    }
    pub(crate) fn criticals_attack(&self, host: &mut dyn Host, target: u32, living: bool) {
        let service = self.criticals_service();
        let settings = service.settings();
        // AutoWeapon's ordinary handler has already selected and synchronized
        // its slot. Read its resulting body/cooldown for Packet and Visuals.
        let Some((p, c)) = service.refresh(host.client_criticals_context()) else {
            return;
        };
        let enemy = settings.sprint_enabled
            && p.would_crit(c, true)
            && self.criticals_enemy(host, settings.sprint_range, false);
        let (running, when) = {
            let s = service.state.lock().expect("Criticals");
            (s.running, s.behavior.sprint_running(s.running, p, c, enemy))
        };
        let offsets = service
            .state
            .lock()
            .expect("Criticals")
            .behavior
            .offsets(running, living, p, c, when);
        let packet_visual = !offsets.is_empty();
        for (y, ground) in offsets {
            host.client_criticals_packet(settings.packet_type, y, ground);
        }
        if running && living && settings.visuals && packet_visual {
            host.client_criticals_visual(target as i32, settings.critical, settings.magic);
        }
        if running && living && settings.visuals && (settings.fake || p.would_crit(c, false)) {
            host.client_criticals_visual(target as i32, settings.critical, settings.magic);
        }
    }
    pub(crate) fn criticals_tick(&self, host: &dyn Host) {
        let service = self.criticals_service();
        let settings = service.settings();
        let p = service.refresh(host.client_criticals_context());
        let running = service.state.lock().expect("Criticals").running;
        if running && settings.mode == 4 {
            let enemy = self.criticals_enemy(host, settings.blink_range, false);
            service.state.lock().expect("Criticals").blink.enemy = enemy;
        }
        if running
            && settings.mode == 5
            && self.criticals_enemy(host, settings.timer_range, false)
            && p.is_some_and(|(p, c)| p.would_crit(c, true))
        {
            service
                .timer
                .state
                .lock()
                .expect("Timer")
                .timer
                .request_speed(settings.timer_speed, 20, TIMER_PROVIDER, 1);
        }
    }
    pub(crate) fn criticals_input(&self, host: &mut dyn Host, jump: &mut bool) {
        let service = self.criticals_service();
        let (active, settings) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let s = service.state.lock().expect("Criticals");
            (
                s.behavior.jump_active(
                    s.running,
                    [
                        false,
                        b.query_running(&state.registry, "AutoClicker"),
                        b.query_running(&state.registry, "TriggerBot"),
                    ],
                ),
                s.behavior.settings,
            )
        };
        if !active {
            return;
        }
        let Some((p, c)) = service.refresh(host.client_criticals_context()) else {
            return;
        };
        if !p.allows_ignoring_ground {
            return;
        }
        let wait = if settings.optimize {
            let initial = 0.42f32;
            let ticks = ((initial / 0.08f32) * 3.) as i32;
            match host.client_crosshair_clicker_falling(ticks, f64::from(initial)) {
                Ok(collision) => model::wait_for_jump(c, initial, collision),
                Err(error) => {
                    eprintln!("[LiquidBounce] Criticals falling: {error}");
                    return;
                }
            }
        } else {
            false
        };
        if wait {
            return;
        }
        let enemy = self.criticals_enemy(host, settings.jump_range, settings.can_be_seen);
        service
            .state
            .lock()
            .expect("Criticals")
            .behavior
            .jump_input(active, p, enemy, wait, jump);
    }
    fn criticals_replay(&self, host: &mut dyn Host, packets: Vec<model::Queued<Packet>>) {
        for p in packets {
            let result = match p.packet {
                Packet::Incoming(packet) => host.replay_client_packet(p.session, packet),
                Packet::Outgoing(packet) => host.send_packet_silently(p.session, packet),
            };
            if let Err(error) = result {
                eprintln!("[LiquidBounce] Blink replay: {error}");
            }
        }
    }
    pub(crate) fn criticals_flush_ready(&self,host:&mut dyn Host){
        let service=self.criticals_service();let ready=std::mem::take(&mut service.state.lock().expect("BlinkManager").ready);self.criticals_replay(host,ready);
    }
    pub(crate) fn criticals_frame(&self, host: &mut dyn Host) {
        let service = self.criticals_service();
        if host.client_criticals_context().is_none() {
            service.disconnect();
            return;
        }
        service.refresh(host.client_criticals_context());
        let packets = service.probe(true);
        self.criticals_replay(host, packets);
    }
    pub(crate) fn criticals_receive(&self, host: &mut dyn Host) {
        let service = self.criticals_service();
        if host.client_criticals_context().is_none() {
            service.disconnect();
            return;
        }
        let packets = service.probe(false);
        self.criticals_replay(host, packets);
    }
}
