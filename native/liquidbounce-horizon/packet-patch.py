#!/usr/bin/env python3
"""Cumulative exact-source packet transforms; importing never edits upstream."""
from pathlib import Path
import json

HERE = Path(__file__).resolve().parent

HELPERS = '''    /// Current transport generation; never a control socket id or REST token.
    pub(super) fn native_client_session(&self) -> Option<u64> {
        self.remote.as_ref().filter(|remote| remote.joined && remote.reconfiguring.is_none()).map(|remote| remote.session.serial())
    }
    /// NetworkUtils.sendPacketSilently: tracking sinks only, no ordinary event.
    pub(super) fn native_send_packet_silently(&mut self, session: u64, packet: ServerboundPlayPacket) -> Result<(), String> {
        let remote = self.remote.as_ref().ok_or("not connected to a server")?;
        if !remote.joined || remote.reconfiguring.is_some() || remote.session.serial() != session || session == 0 {
            return Err("stale or non-play packet session".into());
        }
        if let horizon_protocol::packets::play::Serverbound::CustomPayload(payload) = &packet
            && payload.data.len() > horizon_protocol::quiet::MAX_PAYLOAD {
            return Err("custom payload exceeds the protocol limit".into());
        }
        remote.session.send_silently(packet);
        Ok(())
    }
    /// Replay an incoming packet directly on the game actor. It never returns
    /// through the general packet interceptor or the read-only original hook.
    /// Configuration renegotiation needs the reader's live handshake and is
    /// deliberately rejected instead of simulating that transaction.
    pub(super) fn native_replay_client_packet(&mut self, session: u64, packet: ClientboundPlayPacket) -> Result<(), String> {
        use horizon_protocol::packets::{configuration as cfg, play::{Clientbound as C, Serverbound as S}};
        let current = self.native_client_session().ok_or("not in a joined play session")?;
        let replay = horizon_client::packet_bridge::IncomingReplay::new(session, packet);
        replay.apply(current, |packet| -> Result<(), String> {
            match packet {
                C::KeepAlive(packet) => self.send(S::KeepAlive(packet)),
                C::BundleDelimiter(_) => {},
                C::CookieRequest(packet) => {
                    let payload = self.remote.as_ref().ok_or("disconnected during replay")?.session.cookies.lock().unwrap().get(&packet.key).cloned();
                    self.send(S::CookieResponse(cfg::CookieResponse { key: packet.key, payload }));
                }
                C::StoreCookie(packet) => {
                    self.remote.as_ref().ok_or("disconnected during replay")?.session.cookies.lock().unwrap().insert(packet.key, packet.payload);
                }
                C::ResourcePackPush(packet) => self.handle_pack_push(packet),
                C::ResourcePackPop(packet) => self.handle_pack_pop(packet.id),
                C::ServerLinks(packet) => {
                    self.remote.as_mut().ok_or("disconnected during replay")?.server_links = super::server_links::trusted(packet.links);
                }
                C::CustomPayload(packet) if packet.channel == "minecraft:brand" => {
                    let brand = horizon_protocol::buf::Reader::new(&packet.data).string(32767).map_err(|error| error.to_string())?;
                    self.remote.as_mut().ok_or("disconnected during replay")?.brand = brand;
                }
                C::CustomPayload(packet) if packet.channel == horizon_client::lod_net::CHANNEL => {
                    return Err("far-terrain transport replay is unsupported".into());
                }
                C::StartConfiguration(_) | C::Transfer(_) | C::Disconnect(_) => {
                    return Err("connection-state transport replay is unsupported".into());
                }
                packet => self.handle_packet_direct(packet).map_err(|error| error.to_string())?,
            }
            Ok(())
        }).map_err(|error| match error {
            horizon_client::packet_bridge::ReplayError::StaleSession => "stale packet session".into(),
            horizon_client::packet_bridge::ReplayError::Handler(error) => error,
        })
    }

'''


def apply(edit, replace_once) -> dict[str, str]:
    """Apply existing-file transforms and return independent added-file texts.

    Caller edit(path, transform) must accumulate edits of the same file. Add the
    returned files as ("", text) entries when producing a unified Git patch.
    """
    def network(text: str) -> str:
        text = replace_once(text, '''        let (send, events) = channel();
        let writer = Arc::new(Mutex::new(None));
''', '''        let serial = next_serial();
        let (send, events) = channel();
        let writer = Arc::new(Mutex::new(None));
''')
        text = replace_once(text, '''                horizon_threads::favour_current();
                let result = run(
''', '''                horizon_threads::favour_current();
                let _packet_session = crate::packet_bridge::session_scope(serial);
                let result = run(
''')
        text = replace_once(text, '''            far,
            recorder,
            serial: next_serial(),
''', '''            far,
            recorder,
            serial,
''')
        text = replace_once(text, '''    pub fn send(&self, packet: play::Serverbound) {
        if self.debug
''', '''    pub fn send(&self, packet: play::Serverbound) {
        self.send_with_path(packet, false);
    }
    /// Explicit upstream silent-send path, bypassing general packet listeners.
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
''')
        text = replace_once(text, '''        let packet = match play::Clientbound::decode(id, &payload) {
''', '''        let mut packet = match play::Clientbound::decode(id, &payload) {
''')
        text = replace_once(text, '''        let reply = |p: play::Serverbound| {
            if let Some(w) = shared_writer.lock().unwrap().as_mut() {
''', '''        // Before every play handler, including transport-managed keep-alives,
        // cookies, brand and transfer. Recorder above retains original wire data.
        if crate::packet_bridge::incoming(crate::packet_bridge::reader_session(), &mut packet) {
            continue;
        }
        let reply = |mut p: play::Serverbound| {
            if crate::packet_bridge::outgoing(crate::packet_bridge::reader_session(), &mut p, false) {
                return;
            }
            if let Some(w) = shared_writer.lock().unwrap().as_mut() {
''')
        text = replace_once(text, '''                if let Some(w) = shared_writer.lock().unwrap().as_mut() {
                    let _ = w.send(&play::Serverbound::ConfigurationAcknowledged(Empty).encode());
                    configuring.store(true, Ordering::Relaxed);
''', '''                let mut acknowledgement = play::Serverbound::ConfigurationAcknowledged(Empty);
                let cancelled = crate::packet_bridge::outgoing(crate::packet_bridge::reader_session(), &mut acknowledgement, false);
                if let Some(w) = shared_writer.lock().unwrap().as_mut() {
                    if !cancelled { let _ = w.send(&acknowledgement.encode()); }
                    configuring.store(true, Ordering::Relaxed);
''')
        text = replace_once(text, '''        let body = if self.configuring.load(Ordering::Relaxed) {
            cfg::Serverbound::ResourcePack(status).encode()
        } else {
            play::Serverbound::ResourcePack(status).encode()
        };
''', '''        if !self.configuring.load(Ordering::Relaxed) {
            drop(writer);
            self.send(play::Serverbound::ResourcePack(status));
            return;
        }
        let body = cfg::Serverbound::ResourcePack(status).encode();
''')
        return replace_once(text, '''        let body = if self.configuring.load(Ordering::Relaxed) {
            cfg::Serverbound::CustomClickAction(click).encode()
        } else {
            play::Serverbound::CustomClickAction(click).encode()
        };
''', '''        if !self.configuring.load(Ordering::Relaxed) {
            drop(writer);
            self.send(play::Serverbound::CustomClickAction(click));
            return;
        }
        let body = cfg::Serverbound::CustomClickAction(click).encode();
''')

    edit("crates/horizon-client/src/net.rs", network)
    edit("crates/horizon-client/src/client_lib.rs", lambda text: replace_once(text, "pub mod net;\n", "pub mod net;\npub mod packet_bridge;\n"))
    def native_helpers(text: str) -> str:
        text = replace_once(text, "    ClientboundPlayPacket, NativeBlockRim, NativeColoredLight, NativeLootDrops, NativeMinimap,\n", "    ClientboundPlayPacket, ServerboundPlayPacket, NativeBlockRim, NativeColoredLight, NativeLootDrops, NativeMinimap,\n")
        return replace_once(text, "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n", HELPERS + "    /// Player FOV scale, after vanilla effects and before zoom and easing.\n")
    edit("crates/horizon-client/src/game/native_hooks.rs", native_helpers)
    edit("crates/horizon-client/src/game/remote.rs", lambda text: replace_once(text, '''    pub(super) fn handle_packet(&mut self, packet: C) -> Result<()> {
        self.native_client_packet(&packet);
        match packet {
''', '''    pub(super) fn handle_packet(&mut self, packet: C) -> Result<()> {
        self.native_client_packet(&packet);
        self.handle_packet_direct(packet)
    }
    /// Direct incoming replay bypasses Connection's original-packet callbacks.
    pub(super) fn handle_packet_direct(&mut self, packet: C) -> Result<()> {
        match packet {
'''))
    edit("crates/horizon-client/src/game/hand.rs", lambda text: replace_once(text, '''    pub(super) fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {
        let effects = &self.potions.effects;
''', '''    pub(super) fn swing_hand(&mut self, hand: Hand, animation: SwingAnimation) {
        if self.native_window() && horizon_client::packet_bridge::cancel_local_swing() {
            return;
        }
        let effects = &self.potions.effects;
'''))
    helpers = ["client_session", "send_packet_silently", "replay_client_packet"]
    fragment = {
        "helpers": [{"name": name, "target": f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in helpers],
        "hooks": [],
        "rust_summary": "Process-wide typed play-packet interception before transport handling/writing, explicit silent send and direct game replay; local-client NoSwing cancellation.",
        "doc_summary": "Local source extension. Transport callbacks have no Game/registry/writer lock. Incoming connection-state/far-terrain replay is explicitly unsupported.",
    }
    return {
        "crates/horizon-client/src/packet_bridge.rs": (HERE / "packet-bridge.rs").read_text(encoding="utf-8"),
        "tools/native-hook-fragments/liquidbounce_packets.json": json.dumps(fragment, indent=2) + "\n",
    }
