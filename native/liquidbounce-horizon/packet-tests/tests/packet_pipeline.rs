use horizon_protocol::packets::{
    Empty,
    configuration::{KeepAlive, Ping},
    play::{Clientbound, Serverbound, SetCarriedItem},
};
use liquidbounce_horizon_packet_tests::{packet_bridge as bridge, packets::*};
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn final_packet_observers_see_late_changes_and_skip_cancelled_and_silent_packets() {
    struct Observer(Arc<Mutex<Vec<i16>>>);
    impl PacketListener for Observer {
        fn outgoing_accepted(&self, context: PacketContext, packet: &Serverbound) {
            assert_eq!(context.session, 19);
            assert_eq!(context.delivery, Delivery::Original);
            if let Serverbound::SetCarriedItem(packet) = packet {
                self.0.lock().unwrap().push(packet.slot);
            }
        }
    }
    struct Change;
    impl PacketListener for Change {
        fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
            if let Serverbound::SetCarriedItem(packet) = event.packet {
                packet.slot = 6;
            }
        }
    }
    struct Cancel;
    impl PacketListener for Cancel {
        fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
            event.cancel();
        }
    }
    let pipeline = PacketPipeline::default();
    let observed = Arc::new(Mutex::new(Vec::new()));
    pipeline.add_listener(100, Arc::new(Observer(observed.clone())));
    pipeline.add_listener(-100, Arc::new(Change));
    let mut packet = Serverbound::SetCarriedItem(SetCarriedItem { slot: 1 });
    let context = PacketContext {
        session: 19,
        origin: TransferOrigin::Outgoing,
        delivery: Delivery::Original,
    };
    pipeline.outgoing(&mut PacketEvent::new(context, &mut packet));
    assert_eq!(*observed.lock().unwrap(), [6]);
    pipeline.outgoing(&mut PacketEvent::new(
        PacketContext { delivery: Delivery::Silent, ..context },
        &mut packet,
    ));
    assert_eq!(*observed.lock().unwrap(), [6]);
    pipeline.add_listener(-200, Arc::new(Cancel));
    let mut event = PacketEvent::new(context, &mut packet);
    pipeline.outgoing(&mut event);
    assert!(event.is_cancelled());
    assert_eq!(*observed.lock().unwrap(), [6]);
}

#[test]
fn final_packet_observers_use_the_same_listener_snapshot_as_mutating_handlers() {
    struct Observer(Arc<AtomicUsize>);
    impl PacketListener for Observer {
        fn outgoing_accepted(&self, _: PacketContext, _: &Serverbound) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    struct InstallObserver(Arc<PacketPipeline>, Arc<AtomicUsize>);
    impl PacketListener for InstallObserver {
        fn outgoing(&self, _: &mut PacketEvent<'_, Serverbound>) {
            self.0.add_listener(0, Arc::new(Observer(self.1.clone())));
        }
    }
    let pipeline = Arc::new(PacketPipeline::default());
    let count = Arc::new(AtomicUsize::default());
    pipeline.add_listener(100, Arc::new(InstallObserver(pipeline.clone(), count.clone())));
    let mut packet = Serverbound::SetCarriedItem(SetCarriedItem { slot: 1 });
    let context = PacketContext {
        session: 1,
        origin: TransferOrigin::Outgoing,
        delivery: Delivery::Original,
    };
    pipeline.outgoing(&mut PacketEvent::new(context, &mut packet));
    assert_eq!(count.load(Ordering::Relaxed), 0);
    pipeline.outgoing(&mut PacketEvent::new(context, &mut packet));
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[test]
fn no_swing_defaults_settings_and_independent_hide_targets_match_upstream() {
    let module = NoSwing::default();
    assert_eq!(module.hide_for(), HideFor::BOTH);
    assert!(!module.should_hide_for_client());
    assert!(!module.should_hide_for_server());
    assert!(module.set_running(true));
    assert!(!module.set_running(true));
    assert!(module.should_hide_for_client());
    assert!(module.should_hide_for_server());
    module.set_hide_for(HideFor::CLIENT);
    assert!(module.should_hide_for_client());
    assert!(!module.should_hide_for_server());
    module.set_hide_for(HideFor::SERVER);
    assert!(!module.should_hide_for_client());
    assert!(module.should_hide_for_server());
    module.set_hide_for(HideFor::NONE);
    assert!(!module.should_hide_for_client());
    assert!(!module.should_hide_for_server());
    assert_eq!(
        parse_no_swing_hide_for(&json!(["server", "Client", "Server", "unknown"])).unwrap(),
        HideFor::BOTH
    );
    assert_eq!(
        parse_no_swing_hide_for(&json!("CLIENT")).unwrap(),
        HideFor::CLIENT
    );
    assert_eq!(parse_no_swing_hide_for(&json!([])).unwrap(), HideFor::NONE);
    assert_eq!(parse_no_swing_hide_for(&json!(42)).unwrap(), HideFor::NONE);
    assert_eq!(parse_no_swing_hide_for(&json!({})).unwrap(), HideFor::NONE);
    assert!(parse_no_swing_hide_for(&json!([null])).is_err());
    let file = serialize_no_swing_hide_for(HideFor::SERVER, false, "ignored");
    assert_eq!(file, json!({"name":"HideFor","value":["Server"]}));
    let interop = serialize_no_swing_hide_for(HideFor::BOTH, true, "description");
    assert_eq!(interop["valueType"], "MULTI_CHOOSE");
    assert_eq!(interop["value"], json!(["Client", "Server"]));
    assert_eq!(interop["choices"], json!(["Client", "Server"]));
    assert_eq!(interop["canBeNone"], true);
    assert_eq!(interop["isOrderSensitive"], false);
}

#[test]
fn actual_typed_packet_dispatch_cancel_mutation_silent_reentry_and_direct_replay() {
    struct General {
        count: Arc<AtomicUsize>,
    }
    impl PacketListener for General {
        fn incoming(&self, event: &mut PacketEvent<'_, Clientbound>) {
            assert!(event.context.original());
            assert_eq!(event.context.origin, TransferOrigin::Incoming);
            assert_eq!(event.context.session, 73);
            if let Clientbound::KeepAlive(packet) = event.packet {
                packet.id = 99;
            }
            event.cancel();
            self.count.fetch_add(1, Ordering::Relaxed);
        }
        fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
            assert_eq!(event.context.delivery, Delivery::Original);
            if let Serverbound::SetCarriedItem(packet) = event.packet {
                packet.slot = 7;
            }
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }
    struct Reentrant {
        pipeline: Arc<PacketPipeline>,
    }
    impl PacketListener for Reentrant {
        fn outgoing(&self, _: &mut PacketEvent<'_, Serverbound>) {
            // Acquiring the list's write lock from a callback would deadlock if
            // dispatcher retained its read lock. The new listener sees next event.
            self.pipeline.add_listener(
                -10,
                Arc::new(General {
                    count: Arc::default(),
                }),
            );
        }
    }
    struct Silent {
        name: &'static str,
        order: Arc<Mutex<Vec<&'static str>>>,
    }
    impl PacketListener for Silent {
        fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
            assert!(!event.context.original());
            assert_eq!(event.context.delivery, Delivery::Silent);
            self.order.lock().unwrap().push(self.name);
            // Upstream does not honor cancellation by the direct tracking sinks.
            event.cancel();
        }
    }
    let pipeline = Arc::new(PacketPipeline::default());
    let count = Arc::new(AtomicUsize::new(0));
    pipeline.add_listener(
        0,
        Arc::new(General {
            count: count.clone(),
        }),
    );
    let reentrant = pipeline.add_listener(
        100,
        Arc::new(Reentrant {
            pipeline: pipeline.clone(),
        }),
    );
    let previous = bridge::set_interceptor(Some(pipeline.clone()));
    let mut incoming = Clientbound::KeepAlive(KeepAlive { id: 4 });
    assert!(bridge::incoming(73, &mut incoming));
    assert_eq!(incoming, Clientbound::KeepAlive(KeepAlive { id: 99 }));
    let mut outgoing = Serverbound::SetCarriedItem(SetCarriedItem { slot: 2 });
    assert!(!bridge::outgoing(73, &mut outgoing, false));
    assert_eq!(
        outgoing,
        Serverbound::SetCarriedItem(SetCarriedItem { slot: 7 })
    );
    assert!(pipeline.remove_listener(reentrant));
    let before = count.load(Ordering::Relaxed);
    pipeline.no_swing.set_running(true);
    let mut punch = Serverbound::Punch(Empty);
    assert!(bridge::outgoing(73, &mut punch, false));
    assert_eq!(count.load(Ordering::Relaxed), before);
    assert!(bridge::cancel_local_swing());
    pipeline.no_swing.set_hide_for(HideFor::SERVER);
    assert!(!bridge::cancel_local_swing());
    let order = Arc::new(Mutex::new(Vec::new()));
    let sink = |name| {
        Some(Arc::new(Silent {
            name,
            order: order.clone(),
        }) as Arc<dyn PacketListener>)
    };
    pipeline.set_silent_sinks(SilentSinks {
        rotation: sink("rotation"),
        position_separator: sink("separator"),
        packet_logger: sink("logger"),
    });
    assert!(!bridge::outgoing(73, &mut punch, true));
    assert_eq!(
        *order.lock().unwrap(),
        vec!["rotation", "separator", "logger"]
    );
    assert_eq!(count.load(Ordering::Relaxed), before);
    pipeline.set_silent_sinks(SilentSinks::default());
    assert!(!bridge::outgoing(73, &mut punch, true));
    let replayed = Arc::new(AtomicUsize::new(0));
    let applied = replayed.clone();
    IncomingReplay::new(73, Clientbound::Ping(Ping { id: 5 }))
        .apply(73, move |packet| {
            assert_eq!(packet, Clientbound::Ping(Ping { id: 5 }));
            applied.fetch_add(1, Ordering::Relaxed);
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(replayed.load(Ordering::Relaxed), 1);
    assert_eq!(count.load(Ordering::Relaxed), before);
    assert_eq!(
        IncomingReplay::new(72, incoming).apply(73, |_| Ok::<_, ()>(())),
        Err(ReplayError::StaleSession)
    );
    assert_eq!(bridge::reader_session(), 0);
    {
        let _outer = bridge::session_scope(73);
        assert_eq!(bridge::reader_session(), 73);
        {
            let _inner = bridge::session_scope(74);
            assert_eq!(bridge::reader_session(), 74);
        }
        assert_eq!(bridge::reader_session(), 73);
        assert_eq!(
            std::thread::spawn(bridge::reader_session).join().unwrap(),
            0
        );
    }
    assert_eq!(bridge::reader_session(), 0);
    bridge::set_interceptor(previous);
}
