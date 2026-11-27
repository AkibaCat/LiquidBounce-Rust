// Generated from exact transformed transport and actor slot sources.
// Source SHA256: 95d4c343a4a62dbc4ee869f78fa0b12aba0fd3a7907d2141c455f207ba6cccf8
extern crate liquidbounce_horizon_packet_tests as horizon_client;
pub use liquidbounce_horizon_packet_tests::packet_bridge;
pub use liquidbounce_horizon_packet_tests::slot_bridge;

pub mod lod_net {
    pub const CHANNEL: &str = "horizon:fixture_far_terrain";
    pub struct Inbox;
    impl Inbox { pub fn deliver(&self, _: Vec<u8>) {} }
}

#[allow(dead_code)]
mod transport_fixture {
    use horizon_protocol::{connection::{ConnReader, ConnWriter, Connection, Pipe}, packets::{Empty, configuration as cfg, play}, registry::Dynamic, text::Language};
    use liquidbounce_horizon_packet_tests::packets::*;
    use std::{collections::HashMap, path::Path, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{Sender, channel}}};

    struct Session { writer: Arc<Mutex<Option<ConnWriter>>>, configuring: Arc<AtomicBool>, serial: u64, debug: bool }
    impl Session {
            pub fn send(&self, packet: play::Serverbound) {
        self.send_with_path(packet, false);
    }
    pub fn send_silently(&self, packet: play::Serverbound) {
        self.send_with_path(packet, true);
    }
    fn send_with_path(&self, mut packet: play::Serverbound, silent: bool) {
        if self.configuring.load(Ordering::Relaxed) {
            return;
        }
        // No writer/registry lock may be held while callbacks run. A callback
        // can emit another packet (PositionPacketSeparator does exactly that).
        if crate::packet_bridge::outgoing(self.serial, &mut packet, silent) {
            return;
        }
        if self.debug
            && !matches!(
                packet,
                play::Serverbound::ClientTickEnd(_)
                    | play::Serverbound::MovePlayerPos(_)
                    | play::Serverbound::MovePlayerPosRot(_)
            )
        {
            eprintln!("[net] -> {packet:?}");
        }
        let sent = {
            let mut writer = self.writer.lock().unwrap();
            if self.configuring.load(Ordering::Relaxed) {
                return;
            }
            if let Some(w) = writer.as_mut() {
                if w.send(&packet.encode()).is_err() {
                    w.shutdown();
                    false
                } else {
                    true
                }
            } else {
                false
            }
        };
        // The observation receives the final, actually written packet and can
        // re-enter Session.send without retaining the writer lock.
        if sent {
            crate::slot_bridge::outgoing_sent(self.serial, &packet, silent);
        }
    }
    }
    type Cookies = Arc<Mutex<HashMap<String, Vec<u8>>>>;
    trait PacketRecorder: Send { fn is_active(&self) -> bool; fn received(&self, _: &play::Clientbound, _: i32, _: &[u8]); }
    type RecorderSlot = Arc<Mutex<Option<Box<dyn PacketRecorder>>>>;
    #[derive(Debug)]
    enum Event { Packet(Box<play::Clientbound>), DecodeError(String), PackPush(cfg::ResourcePackPush), PackPop(Option<u128>), Reconfiguring, Reconfigured(Arc<Dynamic>), ServerLinks(Vec<cfg::ServerLink>), Transfer { host: String, port: i32 }, Brand(String) }
    #[derive(Debug)]
    struct Reason(String);
    impl Reason {
        fn plain(value: impl Into<String>) -> Self { Self(value.into()) }
        fn keyed(value: &str, _: &Language) -> Self { Self(value.into()) }
        fn tag<T>(_: &T, _: &Language) -> Self { Self("disconnect".into()) }
    }
    fn connection_lost(error: impl std::fmt::Display, _: &Language) -> String { error.to_string() }
    #[allow(clippy::too_many_arguments)]
    fn configure(_: &mut ConnReader, _: &Mutex<Option<ConnWriter>>, _: &AtomicBool, _: &Language, _: &Sender<Event>, _: &Cookies, _: &Path) -> Result<Dynamic, Reason> { Err(Reason::plain("configuration is outside this transport fixture")) }
    fn receive_brand(payload: &cfg::CustomPayload, send: &Sender<Event>) {
        let brand = horizon_protocol::buf::Reader::new(&payload.data).string(32767).unwrap();
        send.send(Event::Brand(brand)).unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    fn receive_play(
    reader: &mut ConnReader,
    shared_writer: &Mutex<Option<ConnWriter>>,
    configuring: &AtomicBool,
    lang: &Language,
    send: &Sender<Event>,
    cookies: &Cookies,
    far: &crate::lod_net::Inbox,
    resources: &Path,
    recorder: &RecorderSlot,
) -> Result<Reason, Reason> {
    loop {
        let (id, payload) = reader
            .recv()
            .map_err(|e| Reason::plain(connection_lost(e, lang)))?;
        let mut packet = match play::Clientbound::decode(id, &payload) {
            Ok(p) => p,
            Err(err) => {
                let name = horizon_protocol::ids::name(
                    horizon_protocol::State::Play,
                    horizon_protocol::Direction::Clientbound,
                    id,
                )
                .unwrap_or("?");
                let _ = send.send(Event::DecodeError(format!("{name}: {err}")));
                continue;
            }
        };
        if let Some(rec) = recorder.lock().unwrap().as_ref()
            && rec.is_active()
        {
            rec.received(&packet, id, &payload);
        }
        // Before every play handler, including transport-managed keep-alives,
        // cookies, brand and transfer. Recorder above retains original wire data.
        if crate::packet_bridge::incoming(crate::packet_bridge::reader_session(), &mut packet) {
            continue;
        }
        let reply = |mut p: play::Serverbound| {
            if crate::packet_bridge::outgoing(crate::packet_bridge::reader_session(), &mut p, false) {
                return;
            }
            if let Some(w) = shared_writer.lock().unwrap().as_mut() {
                let _ = w.send(&p.encode());
            }
        };
        match packet {
            play::Clientbound::KeepAlive(k) => reply(play::Serverbound::KeepAlive(k)),
            play::Clientbound::BundleDelimiter(_) => {}
            play::Clientbound::ResourcePackPush(p) => {
                let _ = send.send(Event::PackPush(p));
            }
            play::Clientbound::ResourcePackPop(p) => {
                let _ = send.send(Event::PackPop(p.id));
            }
            play::Clientbound::StartConfiguration(_) => {
                let _ = send.send(Event::Reconfiguring);
                let mut acknowledgement = play::Serverbound::ConfigurationAcknowledged(Empty);
                let cancelled = crate::packet_bridge::outgoing(crate::packet_bridge::reader_session(), &mut acknowledgement, false);
                if let Some(w) = shared_writer.lock().unwrap().as_mut() {
                    if !cancelled { let _ = w.send(&acknowledgement.encode()); }
                    configuring.store(true, Ordering::Relaxed);
                }
                let registries = configure(
                    reader,
                    shared_writer,
                    configuring,
                    lang,
                    send,
                    cookies,
                    resources,
                )?;
                let _ = send.send(Event::Reconfigured(Arc::new(registries)));
            }
            // ClientCommonPacketListenerImpl.handleRequestCookie: the stored
            // cookie (or none) goes straight back.
            play::Clientbound::CookieRequest(q) => {
                let payload = cookies.lock().unwrap().get(&q.key).cloned();
                reply(play::Serverbound::CookieResponse(cfg::CookieResponse {
                    key: q.key,
                    payload,
                }));
            }
            play::Clientbound::StoreCookie(p) => {
                cookies.lock().unwrap().insert(p.key, p.payload);
            }
            play::Clientbound::ServerLinks(p) => {
                let _ = send.send(Event::ServerLinks(p.links));
            }
            // ClientCommonPacketListenerImpl.handleTransfer: the connection
            // ends and the game connects to the other address.
            play::Clientbound::Transfer(p) => {
                let _ = send.send(Event::Transfer {
                    host: p.host,
                    port: p.port,
                });
                return Ok(Reason::keyed("disconnect.transfer", lang));
            }
            play::Clientbound::Disconnect(d) => return Ok(Reason::tag(&d.reason, lang)),
            play::Clientbound::CustomPayload(p) if p.channel == "minecraft:brand" => {
                receive_brand(&p, send);
            }
            // A far tile, straight to the sampler waiting for it.
            play::Clientbound::CustomPayload(p) if p.channel == crate::lod_net::CHANNEL => {
                far.deliver(p.data);
            }
            other => {
                if send.send(Event::Packet(Box::new(other))).is_err() {
                    return Ok(Reason::plain("Disconnected"));
                }
            }
        }
    }
}


use std::cell::Cell;
use play::{Clientbound as C, Serverbound as S, Clientbound as ClientboundPlayPacket};
use liquidbounce_horizon_packet_tests::{slots::*, slot_bridge::SlotPolicy};
type Result<T, E = String> = std::result::Result<T, E>;
struct Remote { session: Session, carried: Cell<Option<usize>> }
struct Player { server_copy: bool, replaying: bool }
impl Player { fn native_client_queries_allowed(&self) -> bool { !self.server_copy && !self.replaying } }
struct Game { selected: usize, player: Option<Player>, lan_acting_uuid: Option<u128>, window: bool, remote: Option<Remote>, read_only_calls: Cell<usize> }
impl Game {
    fn native_window(&self) -> bool { self.window }
    fn native_client_packet(&self, _: &C) { self.read_only_calls.set(self.read_only_calls.get() + 1); }
    pub(super) fn native_client_slot_owner(&self) -> bool {
        self.native_window() && self.lan_acting_uuid.is_none()
            && self.player.as_ref().is_some_and(|p| p.native_client_queries_allowed())
    }
    pub(super) fn native_client_real_selected_slot(&self) -> Option<i32> {
        self.native_client_slot_owner().then_some(self.selected as i32)
    }
    pub(super) fn client_hotbar_slot(&self) -> usize {
        if self.native_client_slot_owner() {
            horizon_client::slot_bridge::serverside_slot(Some(self.selected as u8)) as usize
        } else {
            self.selected
        }
    }
    pub(super) fn native_client_serverside_slot(&self) -> Option<i32> {
        self.native_client_real_selected_slot().map(|_| self.client_hotbar_slot() as i32)
    }
    pub(super) fn native_client_clientside_slot(&self) -> Option<i32> {
        self.native_client_real_selected_slot().map(|slot| horizon_client::slot_bridge::clientside_slot(Some(slot as u8)) as i32)
    }
    pub(super) fn native_client_select_slot_silently(&self, requester: u64, slot: i32, ticks: i32) -> Result<bool, String> {
        let real = self.native_client_real_selected_slot().ok_or("local player is unavailable")?;
        horizon_client::slot_bridge::select_slot(requester, slot, ticks, Some(real as u8))
    }
    pub(super) fn native_client_reset_silent_slot(&self, requester: u64) -> bool {
        self.native_window() && horizon_client::slot_bridge::reset_slot(requester)
    }
    pub(super) fn native_client_silent_slot_modified(&self) -> bool {
        self.native_window() && horizon_client::slot_bridge::modified()
    }
    pub(super) fn native_client_silent_slot_modified_by(&self, requester: u64) -> bool {
        self.native_window() && horizon_client::slot_bridge::modified_by(requester)
    }
    pub(super) fn native_reject_slot_packet(&self, packet: &ClientboundPlayPacket) -> bool {
        if !self.native_client_slot_owner()
            || !horizon_client::slot_bridge::no_slot_set_running()
            || !matches!(packet, horizon_protocol::packets::play::Clientbound::SetHeldSlot(_)) {
            return false;
        }
        let slot = self.client_hotbar_slot() as i16;
        self.send(horizon_protocol::packets::play::Serverbound::SetCarriedItem(
            horizon_protocol::packets::play::SetCarriedItem { slot }
        ));
        true
    }
    pub(super) fn send(&self, packet: S) {
        if let Some(r) = &self.remote {
            // MultiPlayerGameMode's attack, interact, useItem, useItemOn, releaseUsingItem, dropItem and
            // piercingAttack run ensureHasSentCarriedItem before their packet: a slot the number keys
            // chose this tick goes out first. A miss, a dig's start or a swap waits for the next tick.
            if let S::Attack(_) | S::Interact(_) | S::UseItem(_) | S::UseItemOn(_) = &packet {
                self.remote_send_carried();
            } else if let S::PlayerAction(a) = &packet
                && matches!(a.action, 4 | 5 | 6 | 8)
            {
                self.remote_send_carried();
            }
            r.session.send(packet);
        }
    }
    fn remote_send_carried(&self) {
        let selected = self.client_hotbar_slot();
        if let Some(r) = &self.remote
            && r.carried.get() != Some(selected)
        {
            r.carried.set(Some(selected));
            r.session.send(S::SetCarriedItem(play::SetCarriedItem {
                slot: selected as i16,
            }));
        }
    }
    pub(super) fn handle_packet(&mut self, packet: C) -> Result<()> {
        if self.native_reject_slot_packet(&packet) { return Ok(()); }
        self.native_client_packet(&packet);
        self.handle_packet_direct(packet)
    }
    #[allow(clippy::single_match, clippy::collapsible_match)]
    fn handle_packet_direct(&mut self, packet: C) -> Result<()> {
        match packet {
            C::SetHeldSlot(p) => {
                if (0..9).contains(&p.slot) {
                    self.selected = p.slot as usize;
                    if self.native_window() { horizon_client::slot_bridge::server_slot_applied(p.slot as u8); }
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub(super) fn native_client_disconnecting(&self) {
        if self.native_window() { horizon_client::slot_bridge::world_change(); }
        mod_api::dispatch_client_disconnecting(self, mod_api::mods());
    }
    pub(super) fn native_client_level_load(&mut self, dimension: &str) {
        if self.native_window() { horizon_client::slot_bridge::world_change(); }
        mod_api::dispatch_client_level_load(self, mod_api::mods(), dimension);
    }
    pub(super) fn native_client_level_unload(&mut self, dimension: &str) {
        if self.native_window() { horizon_client::slot_bridge::world_change(); }
        mod_api::dispatch_client_level_unload(self, mod_api::mods(), dimension);
    }
    pub(super) fn native_client_tick_pre(&mut self) {
        if self.native_window() { horizon_client::slot_bridge::tick(); }
        mod_api::dispatch_client_tick_pre(self, mod_api::mods());
    }
}

mod mod_api {
    use super::Game;
    pub fn mods() -> &'static [()] { &[] }
    pub fn dispatch_client_disconnecting(_: &Game, _: &[()]) {}
    pub fn dispatch_client_level_load(_: &Game, _: &[()], _: &str) {}
    pub fn dispatch_client_level_unload(_: &Game, _: &[()], _: &str) {}
    pub fn dispatch_client_tick_pre(_: &Game, _: &[()]) {}
}
    #[test]
    fn exact_actor_and_transport_preserve_slot_lifecycle_and_ordinary_reply() {
        let service = Arc::new(SlotService::default());
        let previous_policy = crate::slot_bridge::set_policy(Some(service.clone()));
        let pipeline = Arc::new(PacketPipeline::default());
        let previous_packets = crate::packet_bridge::set_interceptor(Some(pipeline.clone()));
        let (client, server) = Pipe::pair();
        let (mut reader, writer) = Connection::local(client).split();
        let mut server = Connection::local(server);
        let session = Session { writer: Arc::new(Mutex::new(Some(writer))), configuring: Arc::new(AtomicBool::new(false)), serial: 73, debug: false };
        let writer = session.writer.clone();
        let configuring = session.configuring.clone();
        struct AuditedPolicy { inner: Arc<SlotService>, writer: Arc<Mutex<Option<ConnWriter>>>, callbacks: Arc<AtomicUsize> }
        impl SlotPolicy for AuditedPolicy {
            fn serverside_slot(&self, real: Option<u8>) -> u8 { self.inner.serverside_slot(real) }
            fn clientside_slot(&self, real: Option<u8>) -> u8 { self.inner.clientside_slot(real) }
            fn no_slot_set_running(&self) -> bool { self.inner.no_slot_set_running() }
            fn select_slot(&self, requester: u64, slot: i32, ticks: i32, real: Option<u8>) -> Result<bool> { self.inner.select_slot(requester,slot,ticks,real) }
            fn reset_slot(&self, requester: u64) -> bool { self.inner.reset_slot(requester) }
            fn modified(&self) -> bool { self.inner.modified() }
            fn modified_by(&self, requester: u64) -> bool { self.inner.modified_by(requester) }
            fn tick(&self) { self.inner.tick(); }
            fn world_change(&self) { self.inner.world_change(); }
            fn outgoing_slot_sent(&self, session: u64, slot: i16, silent: bool) {
                assert!(self.writer.try_lock().is_ok(), "wire observer retained Session.writer");
                // A write on the bridge here would deadlock if snapshot() kept
                // its read lock during this callback.
                let policy = crate::slot_bridge::set_policy(None);
                assert!(policy.is_some());
                crate::slot_bridge::set_policy(policy);
                self.inner.outgoing_slot_sent(session,slot,silent);
                self.callbacks.fetch_add(1, Ordering::Relaxed);
            }
            fn server_slot_applied(&self, slot: u8) { self.inner.server_slot_applied(slot); }
        }
        let wire_callbacks = Arc::new(AtomicUsize::new(0));
        crate::slot_bridge::set_policy(Some(Arc::new(AuditedPolicy {inner:service.clone(),writer:writer.clone(),callbacks:wire_callbacks.clone()})));
        let mut game = Game { selected: 2, player: Some(Player {server_copy:false,replaying:false}), lan_acting_uuid:None, window: true, remote: Some(Remote {session, carried: Cell::new(None)}), read_only_calls: Cell::new(0) };
        let next = |server: &mut Connection| { let (id, data) = server.recv().unwrap(); play::Serverbound::decode(id, &data).unwrap() };
        let (send, events) = channel();
        let thread = std::thread::spawn(move || {
            let _scope = crate::packet_bridge::session_scope(73);
            receive_play(&mut reader, &writer, &configuring, &Language::new(), &send, &Cookies::default(), &crate::lod_net::Inbox, Path::new("unused"), &Arc::new(Mutex::new(None)))
        });
        let deliver = |server: &mut Connection, slot: i32, game: &mut Game| {
            server.send(&C::SetHeldSlot(play::SetHeldSlot {slot}).encode()).unwrap();
            let Event::Packet(packet) = events.recv_timeout(std::time::Duration::from_secs(3)).unwrap() else { panic!("expected actor packet") };
            game.handle_packet(*packet).unwrap();
        };
        // Disabled: a valid server selection updates the real field. Carried
        // cache changes only when ensureHasSentCarriedItem actually runs.
        assert!(!service.no_slot_set.running());
        deliver(&mut server, 5, &mut game);
        assert_eq!(game.selected, 5);
        assert_eq!(service.last_server_applied_slot(), Some(5));
        assert_eq!(game.remote.as_ref().unwrap().carried.get(), None);
        game.remote_send_carried();
        assert_eq!(next(&mut server), S::SetCarriedItem(play::SetCarriedItem {slot:5}));
        assert_eq!(game.remote.as_ref().unwrap().carried.get(), Some(5));
        assert_eq!(game.native_client_real_selected_slot(), Some(5));
        assert_eq!(game.native_client_serverside_slot(), Some(5));
        assert_eq!(game.native_client_clientside_slot(), Some(5));
        let owner = Requester::allocate();
        assert!(game.native_client_select_slot_silently(owner.raw(), 7, 1).unwrap());
        assert!(game.native_client_silent_slot_modified());
        assert!(game.native_client_silent_slot_modified_by(owner.raw()));
        assert_eq!(game.native_client_serverside_slot(), Some(7));
        assert_eq!(game.native_client_clientside_slot(), Some(5));
        deliver(&mut server, 3, &mut game);
        assert_eq!(game.selected, 3);
        assert_eq!(game.client_hotbar_slot(), 7);
        game.remote_send_carried();
        assert_eq!(next(&mut server), S::SetCarriedItem(play::SetCarriedItem {slot:7}));
        // Arbitrary ordinary and silent sends are audit observations, never
        // a substitute for SilentHotbar.serversideSlot or its carried cache.
        game.send(S::SetCarriedItem(play::SetCarriedItem {slot:1}));
        assert_eq!(next(&mut server), S::SetCarriedItem(play::SetCarriedItem {slot:1}));
        assert_eq!(service.wire_slot(), Some(WireSlot {session:73,slot:1,silent:false}));
        game.remote.as_ref().unwrap().session.send_silently(S::SetCarriedItem(play::SetCarriedItem {slot:4}));
        assert_eq!(next(&mut server), S::SetCarriedItem(play::SetCarriedItem {slot:4}));
        assert_eq!(service.wire_slot(), Some(WireSlot {session:73,slot:4,silent:true}));
        assert_eq!(game.client_hotbar_slot(), 7);
        assert_eq!(game.remote.as_ref().unwrap().carried.get(), Some(7));
        // NoSlotSet replies through the exact ordinary Game.send/Session.send.
        // An invalid inbound slot is also cancelled and answered.
        service.no_slot_set.set_running(true);
        let readonly_before = game.read_only_calls.get();
        deliver(&mut server, -123, &mut game);
        assert_eq!(next(&mut server), S::SetCarriedItem(play::SetCarriedItem {slot:7}));
        assert_eq!(game.selected, 3);
        assert_eq!(game.read_only_calls.get(), readonly_before);
        assert_eq!(service.wire_slot(), Some(WireSlot {session:73,slot:7,silent:false}));
        struct ReplyListener { mode: Arc<AtomicUsize>, count: Arc<AtomicUsize>, service: Arc<SlotService> }
        impl PacketListener for ReplyListener {
            fn outgoing(&self, event: &mut PacketEvent<'_, S>) {
                if let S::SetCarriedItem(packet) = event.packet {
                    assert!(event.context.original());
                    assert_eq!(self.service.serverside_slot(Some(3)), 7);
                    self.count.fetch_add(1, Ordering::Relaxed);
                    if self.mode.load(Ordering::Relaxed) == 0 { packet.slot = 8; } else { event.cancel(); }
                }
            }
        }
        let mode = Arc::new(AtomicUsize::new(0));
        let count = Arc::new(AtomicUsize::new(0));
        let listener = pipeline.add_listener(0, Arc::new(ReplyListener {mode:mode.clone(),count:count.clone(),service:service.clone()}));
        deliver(&mut server, 1, &mut game);
        assert_eq!(next(&mut server), S::SetCarriedItem(play::SetCarriedItem {slot:8}));
        assert_eq!(service.wire_slot(), Some(WireSlot {session:73,slot:8,silent:false}));
        mode.store(1, Ordering::Relaxed);
        deliver(&mut server, 1, &mut game);
        game.send(S::ClientTickEnd(Empty));
        assert_eq!(next(&mut server), S::ClientTickEnd(Empty));
        assert_eq!(service.wire_slot(), Some(WireSlot {session:73,slot:8,silent:false}));
        assert_eq!(count.load(Ordering::Relaxed), 2);
        pipeline.remove_listener(listener);
        // Reader cancellation never schedules the actor reply.
        struct CancelIncoming;
        impl PacketListener for CancelIncoming {
            fn incoming(&self, event: &mut PacketEvent<'_, C>) { if matches!(event.packet, C::SetHeldSlot(_)) {event.cancel();} }
        }
        let listener = pipeline.add_listener(0, Arc::new(CancelIncoming));
        server.send(&C::SetHeldSlot(play::SetHeldSlot {slot:6}).encode()).unwrap();
        server.send(&C::Ping(cfg::Ping {id:92}).encode()).unwrap();
        let Event::Packet(packet) = events.recv_timeout(std::time::Duration::from_secs(3)).unwrap() else {panic!("ping sentinel")};
        assert_eq!(*packet, C::Ping(cfg::Ping {id:92}));
        assert!(events.try_recv().is_err());
        game.send(S::ClientTickEnd(Empty));
        assert_eq!(next(&mut server), S::ClientTickEnd(Empty));
        pipeline.remove_listener(listener);
        // Direct replay enters the direct handler, bypassing this original hook.
        crate::packet_bridge::IncomingReplay::new(73, C::SetHeldSlot(play::SetHeldSlot {slot:6})).apply(73, |packet| game.handle_packet_direct(packet)).unwrap();
        assert_eq!(game.selected, 6);
        assert_eq!(service.last_server_applied_slot(), Some(6));
        assert_eq!(game.client_hotbar_slot(), 7);
        game.native_client_tick_pre();
        assert_eq!(game.client_hotbar_slot(), 7);
        game.native_client_tick_pre();
        assert_eq!(game.client_hotbar_slot(), 6);
        assert!(!game.native_client_silent_slot_modified());
        assert!(game.native_client_select_slot_silently(owner.raw(), 4, i32::MAX).unwrap());
        assert!(!game.native_client_reset_silent_slot(Requester::allocate().raw()));
        assert!(game.native_client_reset_silent_slot(owner.raw()));
        for world_event in 0..3 {
            game.native_client_select_slot_silently(owner.raw(), 4, i32::MAX).unwrap();
            match world_event {0 => game.native_client_level_load("new"),1=>game.native_client_level_unload("old"),_=>game.native_client_disconnecting()}
            assert!(!game.native_client_silent_slot_modified());
            assert_eq!(service.wire_slot(), None);
        }
        // Server/guest Games always keep their own selected slot and accept
        // incoming changes even while the window's service is active.
        game.native_client_select_slot_silently(owner.raw(), 4, i32::MAX).unwrap();
        game.lan_acting_uuid = Some(21);
        assert_eq!(game.client_hotbar_slot(), 6);
        assert_eq!(game.native_client_real_selected_slot(), None);
        assert!(!game.native_reject_slot_packet(&C::SetHeldSlot(play::SetHeldSlot {slot:2})));
        game.lan_acting_uuid = None;
        for replaying in [false, true] {
            let player = game.player.as_mut().unwrap();
            player.server_copy = !replaying;
            player.replaying = replaying;
            assert_eq!(game.client_hotbar_slot(), 6);
            assert_eq!(game.native_client_real_selected_slot(), None);
            assert!(!game.native_reject_slot_packet(&C::SetHeldSlot(play::SetHeldSlot {slot:2})));
        }
        let player = game.player.as_mut().unwrap();
        player.server_copy = false;
        player.replaying = false;
        assert_eq!(game.client_hotbar_slot(), 4);
        game.window = false;
        assert_eq!(game.client_hotbar_slot(), 6);
        game.handle_packet(C::SetHeldSlot(play::SetHeldSlot {slot:2})).unwrap();
        assert_eq!(game.selected, 2);
        assert_eq!(service.state().unwrap().enforced_slot.index(), 4);
        game.window = true;
        game.player = None;
        assert_eq!(game.native_client_real_selected_slot(), None);
        assert_eq!(game.client_hotbar_slot(), 2);
        assert!(game.native_client_select_slot_silently(owner.raw(), 1, 1).is_err());
        let wire_before = service.wire_slot();
        let callbacks_before = wire_callbacks.load(Ordering::Relaxed);
        let session = &game.remote.as_ref().unwrap().session;
        session.configuring.store(true, Ordering::Relaxed);
        session.send(S::SetCarriedItem(play::SetCarriedItem {slot:8}));
        session.configuring.store(false, Ordering::Relaxed);
        let saved_writer = session.writer.lock().unwrap().take();
        session.send(S::SetCarriedItem(play::SetCarriedItem {slot:8}));
        *session.writer.lock().unwrap() = saved_writer;
        assert_eq!(service.wire_slot(), wire_before);
        assert_eq!(wire_callbacks.load(Ordering::Relaxed), callbacks_before);
        assert!(callbacks_before > 0);
        drop(server);
        assert!(thread.join().unwrap().is_err());
        crate::packet_bridge::set_interceptor(previous_packets);
        crate::slot_bridge::set_policy(previous_policy);
    }

}
