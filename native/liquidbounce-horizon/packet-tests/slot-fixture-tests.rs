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
