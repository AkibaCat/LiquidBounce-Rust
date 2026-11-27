    use liquidbounce_horizon_packet_tests::rotation_packets::{RotationListener, RotationManager};
    use liquidbounce_behaviors::rotations::Rotation;
    static TRANSPORT_TESTS: Mutex<()> = Mutex::new(());

    fn next_serverbound(server: &mut Connection) -> play::Serverbound {
        let (id, data) = server.recv().unwrap();
        play::Serverbound::decode(id, &data).unwrap()
    }
    fn pair(manager: Arc<Mutex<RotationManager>>) -> (Arc<PacketPipeline>, Arc<RotationListener>) {
        let pipeline = Arc::new(PacketPipeline::default());
        let listener = Arc::new(RotationListener(manager));
        pipeline.add_listener(i32::MIN, listener.clone());
        (pipeline, listener)
    }
    fn assert_state(manager: &Mutex<RotationManager>, actual: [f32;2], theoretical: [f32;2]) {
        let state = manager.lock().unwrap();
        assert_eq!([state.actual_server.yaw,state.actual_server.pitch],actual);
        assert_eq!([state.theoretical_server.yaw,state.theoretical_server.pitch],theoretical);
        assert!(state.actual_server.is_normalized);
        assert!(state.theoretical_server.is_normalized);
    }

    #[test]
    fn final_outgoing_packets_track_mutation_cancellation_and_actual_silent_writes() {
        let _serial = TRANSPORT_TESTS.lock().unwrap();
        struct Mutate { mode: Arc<AtomicUsize>, calls: Arc<AtomicUsize> }
        impl PacketListener for Mutate {
            fn outgoing(&self, event: &mut PacketEvent<'_, play::Serverbound>) {
                self.calls.fetch_add(1,Ordering::Relaxed);
                if !matches!(event.packet,play::Serverbound::MovePlayerRot(_)) {return;}
                match self.mode.load(Ordering::Relaxed) {
                    1 => *event.packet = play::Serverbound::UseItem(play::UseItem {hand:1,sequence:6,yaw:725.,pitch:275.}),
                    2 => {
                        *event.packet=play::Serverbound::MovePlayerRot(play::MovePlayerRot {yaw:450.,pitch:100.,flags:3});
                        event.cancel();
                    }
                    3 => *event.packet=play::Serverbound::MovePlayerPos(play::MovePlayerPos {x:1.,y:2.,z:3.,flags:1}),
                    _ => {}
                }
            }
        }
        struct CancelSilent { calls: Arc<AtomicUsize> }
        impl PacketListener for CancelSilent {
            fn outgoing(&self, event: &mut PacketEvent<'_, play::Serverbound>) {
                assert_eq!(event.context.delivery,Delivery::Silent);
                self.calls.fetch_add(1,Ordering::Relaxed);
                event.cancel();
            }
        }
        let manager=Arc::new(Mutex::new(RotationManager::default()));
        let (pipeline,listener)=pair(manager.clone());
        let mode=Arc::new(AtomicUsize::new(1));
        let ordinary_calls=Arc::new(AtomicUsize::new(0));
        pipeline.add_listener(0,Arc::new(Mutate {mode:mode.clone(),calls:ordinary_calls.clone()}));
        let silent_calls=Arc::new(AtomicUsize::new(0));
        pipeline.set_silent_sinks(SilentSinks {rotation:Some(listener.clone()),position_separator:Some(Arc::new(CancelSilent {calls:silent_calls.clone()})),packet_logger:None});
        let previous=crate::packet_bridge::set_interceptor(Some(pipeline));
        let (client,server)=Pipe::pair();
        let (_,writer)=Connection::local(client).split();
        let mut server=Connection::local(server);
        let session=Session {writer:Arc::new(Mutex::new(Some(writer))),configuring:Arc::new(AtomicBool::new(false)),serial:73,debug:false};
        let rot=||play::Serverbound::MovePlayerRot(play::MovePlayerRot {yaw:1.,pitch:2.,flags:1});
        session.send(rot());
        assert_eq!(next_serverbound(&mut server),play::Serverbound::UseItem(play::UseItem {hand:1,sequence:6,yaw:725.,pitch:275.}));
        assert_state(&manager,[5.,-85.],[5.,-85.]);
        mode.store(2,Ordering::Relaxed);
        session.send(rot());
        session.send(play::Serverbound::ClientTickEnd(Empty));
        assert_eq!(next_serverbound(&mut server),play::Serverbound::ClientTickEnd(Empty));
        assert_state(&manager,[5.,-85.],[90.,90.]);
        assert_eq!(manager.lock().unwrap().server_rotation(false,false),Rotation {yaw:5.,pitch:-85.,is_normalized:true});
        assert_eq!(manager.lock().unwrap().server_rotation(true,false),Rotation {yaw:90.,pitch:90.,is_normalized:true});
        assert_eq!(manager.lock().unwrap().server_rotation(false,true),Rotation {yaw:90.,pitch:90.,is_normalized:true});
        mode.store(3,Ordering::Relaxed);
        session.send(rot());
        assert!(matches!(next_serverbound(&mut server),play::Serverbound::MovePlayerPos(_)));
        assert_state(&manager,[5.,-85.],[90.,90.]);
        mode.store(0,Ordering::Relaxed);
        for packet in [
            play::Serverbound::MovePlayerPosRot(play::MovePlayerPosRot {x:1.,y:2.,z:3.,yaw:-725.,pitch:-270.,flags:2}),
            play::Serverbound::UseItem(play::UseItem {hand:0,sequence:7,yaw:-725.,pitch:-270.}),
        ] {
            session.send(packet.clone());
            assert_eq!(next_serverbound(&mut server),packet);
            assert_state(&manager,[-5.,90.],[-5.,90.]);
        }
        let before=ordinary_calls.load(Ordering::Relaxed);
        // Silent rotation runs first. Cancellation by the next silent service
        // is ignored by the actual source Session and bridge send path.
        session.send_silently(play::Serverbound::MovePlayerRot(play::MovePlayerRot {yaw:450.,pitch:540.,flags:0}));
        assert_eq!(next_serverbound(&mut server),play::Serverbound::MovePlayerRot(play::MovePlayerRot {yaw:450.,pitch:540.,flags:0}));
        assert_state(&manager,[90.,-90.],[90.,-90.]);
        assert_eq!(ordinary_calls.load(Ordering::Relaxed),before);
        assert_eq!(silent_calls.load(Ordering::Relaxed),1);
        let mut packet=rot();
        let mut event=PacketEvent::new(PacketContext {session:73,origin:TransferOrigin::Outgoing,delivery:Delivery::Silent},&mut packet);
        event.cancel();
        listener.outgoing(&mut event);
        assert_state(&manager,[1.,2.],[1.,2.]);
        for packet in [
            play::Serverbound::MovePlayerStatus(play::MovePlayerStatus {flags:3}),
            play::Serverbound::MovePlayerPos(play::MovePlayerPos {x:9.,y:8.,z:7.,flags:0}),
            play::Serverbound::UseItemOn(play::UseItemOn {hand:0,pos:[1,2,3],face:2,cursor:[0.5;3],inside:false,world_border:false,sequence:1}),
        ] {
            session.send(packet.clone());
            assert_eq!(next_serverbound(&mut server),packet);
            assert_state(&manager,[1.,2.],[1.,2.]);
        }
        crate::packet_bridge::set_interceptor(previous);
    }

    #[test]
    fn final_incoming_corrections_resolve_actual_rotation_even_when_cancelled() {
        let _serial=TRANSPORT_TESTS.lock().unwrap();
        struct MutateIncoming {mode:Arc<AtomicUsize>}
        impl PacketListener for MutateIncoming {
            fn incoming(&self,event:&mut PacketEvent<'_,play::Clientbound>) {
                if !matches!(event.packet,play::Clientbound::PlayerPosition(_)|play::Clientbound::PlayerRotation(_)) {return;}
                match self.mode.load(Ordering::Relaxed) {
                    1=>event.cancel(),
                    2=>*event.packet=play::Clientbound::PlayerRotation(play::PlayerRotation {yaw:15.,pitch:200.,relative_yaw:true,relative_pitch:false}),
                    _=>{}
                }
            }
        }
        let manager=Arc::new(Mutex::new(RotationManager::default()));
        let (pipeline,listener)=pair(manager.clone());
        let mode=Arc::new(AtomicUsize::new(0));
        pipeline.add_listener(0,Arc::new(MutateIncoming {mode:mode.clone()}));
        let previous=crate::packet_bridge::set_interceptor(Some(pipeline));
        let (client,server)=Pipe::pair();
        let (mut reader,writer)=Connection::local(client).split();
        let mut server=Connection::local(server);
        let writer=Arc::new(Mutex::new(Some(writer)));
        let (send,events)=channel();
        let thread=std::thread::spawn(move||{
            let _scope=crate::packet_bridge::session_scope(73);
            receive_play(&mut reader,&writer,&AtomicBool::new(false),&Language::new(),&send,&Cookies::default(),&crate::lod_net::Inbox,Path::new("unused"),&Arc::new(Mutex::new(None)))
        });
        let receive=|events:&std::sync::mpsc::Receiver<Event>| {
            let Event::Packet(packet)=events.recv_timeout(std::time::Duration::from_secs(3)).unwrap() else {panic!("expected actor packet")};
            *packet
        };
        let position=|yaw,pitch,relatives|play::Clientbound::PlayerPosition(play::PlayerPosition {teleport:8,position:[1.,2.,3.],delta:[0.;3],yaw,pitch,relatives});
        let packet=position(370.,120.,0);
        server.send(&packet.encode()).unwrap();
        assert_eq!(receive(&events),packet);
        assert_state(&manager,[370.,90.],[370.,90.]);
        // An unsent outgoing rotation creates a different theoretical value.
        let mut packet=play::Serverbound::MovePlayerRot(play::MovePlayerRot {yaw:90.,pitch:20.,flags:0});
        let mut event=PacketEvent::new(PacketContext {session:73,origin:TransferOrigin::Outgoing,delivery:Delivery::Original},&mut packet);
        event.cancel();
        listener.outgoing(&mut event);
        assert_state(&manager,[370.,90.],[90.,20.]);
        mode.store(1,Ordering::Relaxed);
        server.send(&position(20.,-30.,play::relative::Y_ROT|play::relative::X_ROT).encode()).unwrap();
        server.send(&play::Clientbound::Ping(cfg::Ping {id:91}).encode()).unwrap();
        assert_eq!(receive(&events),play::Clientbound::Ping(cfg::Ping {id:91}));
        assert_state(&manager,[390.,60.],[390.,60.]);
        mode.store(2,Ordering::Relaxed);
        server.send(&position(1.,1.,0).encode()).unwrap();
        assert_eq!(receive(&events),play::Clientbound::PlayerRotation(play::PlayerRotation {yaw:15.,pitch:200.,relative_yaw:true,relative_pitch:false}));
        assert_state(&manager,[405.,90.],[405.,90.]);
        mode.store(0,Ordering::Relaxed);
        for flags in 0..4 {
            for kind in 0..2 {
                manager.lock().unwrap().inbound_rotation(500.,70.,false,false);
                let relative_yaw=flags&1!=0;
                let relative_pitch=flags&2!=0;
                let packet=if kind==0 {position(-20.,-100.,7|if relative_yaw {play::relative::Y_ROT}else{0}|if relative_pitch {play::relative::X_ROT}else{0})}
                    else {play::Clientbound::PlayerRotation(play::PlayerRotation {yaw:-20.,pitch:-100.,relative_yaw,relative_pitch})};
                server.send(&packet.encode()).unwrap();
                assert_eq!(receive(&events),packet);
                let expected=[if relative_yaw {480.} else {-20.},if relative_pitch {-30.} else {-90.}];
                assert_state(&manager,expected,expected);
            }
        }
        let before=manager.lock().unwrap().clone();
        crate::packet_bridge::IncomingReplay::new(73,position(1.,2.,0)).apply(73,|_|Ok::<_,()>(())).unwrap();
        assert_eq!(*manager.lock().unwrap(),before);
        drop(server);
        assert!(thread.join().unwrap().is_err());
        crate::packet_bridge::set_interceptor(previous);
    }

    #[test]
    fn actual_codec_rotation_variants_keep_raw_payloads_and_normalize_only_tracking() {
        let manager=Arc::new(Mutex::new(RotationManager::default()));
        let (_,listener)=pair(manager.clone());
        for (yaw,pitch,expected) in [
            (180.,180.,[-180.,-90.]),(540.,-270.,[-180.,90.]),
            (-540.,540.,[-180.,-90.]),(725.,275.,[5.,-85.]),
        ] {
            for kind in 0..3 {
                let packet=match kind {
                    0=>play::Serverbound::MovePlayerRot(play::MovePlayerRot {yaw,pitch,flags:3}),
                    1=>play::Serverbound::MovePlayerPosRot(play::MovePlayerPosRot {x:1.,y:2.,z:3.,yaw,pitch,flags:1}),
                    _=>play::Serverbound::UseItem(play::UseItem {hand:0,sequence:17,yaw,pitch}),
                };
                let bytes=packet.encode();
                let mut reader=horizon_protocol::buf::Reader::new(&bytes);
                let id=reader.varint().unwrap();
                let mut decoded=play::Serverbound::decode(id,reader.rest()).unwrap();
                assert_eq!(decoded,packet);
                let mut event=PacketEvent::new(PacketContext {session:1,origin:TransferOrigin::Outgoing,delivery:Delivery::Original},&mut decoded);
                listener.outgoing(&mut event);
                assert_eq!(decoded,packet,"tracker must not rewrite the actual packet");
                assert_state(&manager,expected,expected);
            }
        }
        assert_eq!(play::relative::Y_ROT,8);
        assert_eq!(play::relative::X_ROT,16);
    }
