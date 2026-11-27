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
        /* GENERATED_SESSION_METHODS */
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
    /* GENERATED_RECEIVE_PLAY */

    #[test]
    fn exact_patched_send_and_receive_bodies_change_real_framed_pipe_packets() {
        struct Handler { seen: Arc<AtomicUsize> }
        impl PacketListener for Handler {
            fn incoming(&self, event: &mut PacketEvent<'_, play::Clientbound>) {
                assert_eq!(event.context.session, 73);
                match event.packet {
                    play::Clientbound::KeepAlive(packet) if packet.id == 7 => packet.id = 70,
                    play::Clientbound::KeepAlive(packet) if packet.id == 8 => event.cancel(),
                    _ => {}
                }
                self.seen.fetch_add(1, Ordering::Relaxed);
            }
            fn outgoing(&self, event: &mut PacketEvent<'_, play::Serverbound>) {
                assert_eq!(event.context.session, 73);
                match event.packet {
                    play::Serverbound::SetCarriedItem(packet) => packet.slot = 9,
                    play::Serverbound::KeepAlive(packet) if packet.id == 70 => packet.id = 71,
                    _ => {}
                }
                self.seen.fetch_add(1, Ordering::Relaxed);
            }
        }
        let seen = Arc::new(AtomicUsize::new(0));
        let pipeline = Arc::new(PacketPipeline::default());
        pipeline.add_listener(0, Arc::new(Handler { seen: seen.clone() }));
        pipeline.no_swing.set_running(true);
        let previous = crate::packet_bridge::set_interceptor(Some(pipeline));
        let (client, server) = Pipe::pair();
        let (mut reader, writer) = Connection::local(client).split();
        let mut server = Connection::local(server);
        let session = Session { writer: Arc::new(Mutex::new(Some(writer))), configuring: Arc::new(AtomicBool::new(false)), serial: 73, debug: false };
        let next = |server: &mut Connection| { let (id, data) = server.recv().unwrap(); play::Serverbound::decode(id, &data).unwrap() };
        session.send(play::Serverbound::SetCarriedItem(play::SetCarriedItem { slot: 2 }));
        assert_eq!(next(&mut server), play::Serverbound::SetCarriedItem(play::SetCarriedItem { slot: 9 }));
        session.send(play::Serverbound::Punch(Empty));
        session.send(play::Serverbound::ClientTickEnd(Empty));
        assert_eq!(next(&mut server), play::Serverbound::ClientTickEnd(Empty));
        let before_silent = seen.load(Ordering::Relaxed);
        session.send_silently(play::Serverbound::Punch(Empty));
        assert_eq!(next(&mut server), play::Serverbound::Punch(Empty));
        assert_eq!(seen.load(Ordering::Relaxed), before_silent);
        let writer = session.writer.clone();
        let configuring = session.configuring.clone();
        let (send, events) = channel();
        let thread = std::thread::spawn(move || {
            let _scope = crate::packet_bridge::session_scope(73);
            receive_play(&mut reader, &writer, &configuring, &Language::new(), &send, &Cookies::default(), &crate::lod_net::Inbox, Path::new("unused"), &Arc::new(Mutex::new(None)))
        });
        server.send(&play::Clientbound::KeepAlive(cfg::KeepAlive { id: 7 }).encode()).unwrap();
        assert_eq!(next(&mut server), play::Serverbound::KeepAlive(cfg::KeepAlive { id: 71 }));
        server.send(&play::Clientbound::KeepAlive(cfg::KeepAlive { id: 8 }).encode()).unwrap();
        server.send(&play::Clientbound::Ping(cfg::Ping { id: 3 }).encode()).unwrap();
        assert!(matches!(*match events.recv_timeout(std::time::Duration::from_secs(3)).unwrap() { Event::Packet(packet) => packet, other => panic!("{other:?}") }, play::Clientbound::Ping(cfg::Ping { id: 3 })));
        // An extra keepalive reply here would prove cancelled inbound data ran.
        session.send(play::Serverbound::ClientTickEnd(Empty));
        assert_eq!(next(&mut server), play::Serverbound::ClientTickEnd(Empty));
        drop(server);
        assert!(thread.join().unwrap().is_err());
        crate::packet_bridge::set_interceptor(previous);
    }
}
