use crate::{
    anti_hunger_bridge::{ClientPacketPlayer, Snapshot},
    anti_hunger_packets::AntiHungerService,
    packets::{PacketEvent, PacketListener, PacketPipeline},
};
use horizon_protocol::packets::play::{self, Serverbound};
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};
thread_local! {static DONE:Cell<bool>=const{Cell::new(false)};static TRACE:RefCell<Vec<&'static str>>=const{RefCell::new(vec![])};}
struct Player {
    body: RefCell<crate::MotionWorld>,
    snapshot: Cell<Snapshot>,
    service: Arc<AntiHungerService>,
    notifications: RefCell<Vec<serde_json::Value>>,
}
impl ClientPacketPlayer for Player {
    fn snapshot(&self) -> Snapshot {
        self.snapshot.get()
    }
    fn jump_from_ground(&self) {
        TRACE.with(|v| v.borrow_mut().push("jump"));
        assert!(self.service.behavior.lock().unwrap().last_notification > 0);
        let notifications = self.service.notifications.drain(&|_, key| key.into());
        assert_eq!(notifications.len(), 1);
        self.notifications
            .borrow_mut()
            .extend(notifications.into_iter().map(|(_, v)| v));
        self.body.borrow_mut().jump_from_ground().unwrap();
        DONE.set(true);
    }
}
struct Cancel;
impl PacketListener for Cancel {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if matches!(event.packet, Serverbound::PlayerCommand(_)) {
            event.cancel();
        }
    }
}
struct Observe;
impl PacketListener for Observe {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if matches!(event.packet, Serverbound::PlayerCommand(_)) {
            assert!(DONE.get());
            TRACE.with(|v| v.borrow_mut().push("later"));
        }
    }
}
struct Nested;
impl PacketListener for Nested {
    fn outgoing(&self, event: &mut PacketEvent<'_, Serverbound>) {
        if matches!(event.packet, Serverbound::Punch(_)) {
            assert!(event.send(Serverbound::PlayerCommand(play::PlayerCommand {
                id: 1,
                action: 1,
                data: 0
            })));
            assert!(DONE.get());
            TRACE.with(|v| v.borrow_mut().push("nested-return"));
        }
    }
}
#[test]
fn actual_pipeline_applies_jump_before_later_cancelled_and_nested_listeners() {
    crate::high_jump_bridge::install(crate::jump_callback);
    crate::POLICY.with(|p| p.borrow_mut().settings.motion = 0.9375);
    let service = Arc::new(AntiHungerService::default());
    service.set_running(true);
    let pipeline = Arc::new(PacketPipeline::default());
    pipeline.add_listener(10, Arc::new(Cancel));
    pipeline.add_listener(0, service.clone());
    pipeline.add_listener(-10, Arc::new(Observe));
    pipeline.add_listener(20, Arc::new(Nested));
    pipeline.install();
    let player = Player {
        body: RefCell::new(crate::world()),
        snapshot: Cell::new(Snapshot::default()),
        service: service.clone(),
        notifications: RefCell::new(vec![]),
    };
    let mut command = Serverbound::PlayerCommand(play::PlayerCommand {
        id: 1,
        action: 1,
        data: 0,
    });
    let cancelled = crate::packet_bridge::outgoing_with_client_player(
        73,
        &mut command,
        false,
        &|_| panic!("no nested send"),
        Some(&player),
    );
    assert!(cancelled);
    assert_eq!(player.body.borrow().player.velocity.y, 0.9375);
    assert_eq!(player.body.borrow().power_calls, 2);
    assert_eq!(
        TRACE.with(|v| std::mem::take(&mut *v.borrow_mut())),
        vec!["jump", "later"]
    );
    assert_eq!(
        player.notifications.borrow()[0],
        serde_json::json!({"title":"AntiHunger","message":"liquidbounce.module.antiHunger.messages.warnSprint","severity":"INFO"})
    );
    DONE.set(false);
    service.behavior.lock().unwrap().last_notification = 0;
    let mut punch = Serverbound::Punch(horizon_protocol::packets::Empty);
    let sender = |mut packet| {
        assert!(crate::packet_bridge::outgoing_with_client_player(
            73,
            &mut packet,
            false,
            &|_| panic!("second nesting"),
            Some(&player)
        ));
    };
    assert!(!crate::packet_bridge::outgoing_with_client_player(
        73,
        &mut punch,
        false,
        &sender,
        Some(&player)
    ));
    assert_eq!(
        TRACE.with(|v| std::mem::take(&mut *v.borrow_mut())),
        vec!["jump", "later", "nested-return"]
    );
    for flags in 0..32 {
        player.snapshot.set(Snapshot {
            swimming: flags & 1 != 0,
            passenger: flags & 2 != 0,
            destroying: flags & 4 != 0,
            in_water: flags & 8 != 0,
            under_water: flags & 16 != 0,
            fall_distance: -0.0,
        });
        let mut variants = [
            Serverbound::MovePlayerStatus(play::MovePlayerStatus { flags: 0b1011 }),
            Serverbound::MovePlayerRot(play::MovePlayerRot {
                yaw: 13.0,
                pitch: -22.0,
                flags: 0b1011,
            }),
            Serverbound::MovePlayerPos(play::MovePlayerPos {
                x: 1.,
                y: 2.,
                z: 3.,
                flags: 0b1011,
            }),
            Serverbound::MovePlayerPosRot(play::MovePlayerPosRot {
                x: 1.,
                y: 2.,
                z: 3.,
                yaw: 13.,
                pitch: -22.,
                flags: 0b1011,
            }),
        ];
        for packet in &mut variants {
            assert!(!crate::packet_bridge::outgoing_with_client_player(
                73,
                packet,
                false,
                &|_| panic!("move nested"),
                Some(&player)
            ));
            let actual = match packet {
                Serverbound::MovePlayerStatus(p) => p.flags,
                Serverbound::MovePlayerRot(p) => p.flags,
                Serverbound::MovePlayerPos(p) => p.flags,
                Serverbound::MovePlayerPosRot(p) => p.flags,
                _ => unreachable!(),
            };
            assert_eq!(actual, if flags == 0 { 0b1010 } else { 0b1011 });
        }
    }
    service.behavior.lock().unwrap().last_notification = 0;
    let mut breaking = Serverbound::PlayerAction(play::PlayerAction {
        action: 0,
        pos: [1, 2, 3],
        direction: 2,
        sequence: 7,
    });
    DONE.set(false);
    assert!(!crate::packet_bridge::outgoing_with_client_player(
        73,
        &mut breaking,
        false,
        &|_| panic!("break nested"),
        Some(&player)
    ));
    assert!(!DONE.get());
    assert_eq!(
        service.notifications.drain(&|_, key| key.into())[0].1["message"],
        "liquidbounce.module.antiHunger.messages.warnBreak"
    );
    // A silent send bypasses ordinary listeners, and a send with no owner scope
    // does not touch an unrelated player's cooldown or motion.
    service.behavior.lock().unwrap().last_notification = 0;
    player.snapshot.set(Snapshot::default());
    let mut start = Serverbound::PlayerCommand(play::PlayerCommand {
        id: 1,
        action: 1,
        data: 0,
    });
    assert!(!crate::packet_bridge::outgoing_with_client_player(
        73,
        &mut start,
        true,
        &|_| panic!("silent nested"),
        Some(&player)
    ));
    let mut ownerless = Serverbound::MovePlayerStatus(play::MovePlayerStatus { flags: 0b1011 });
    assert!(!crate::packet_bridge::outgoing_with_client_player(
        73,
        &mut ownerless,
        false,
        &|_| panic!("ownerless nested"),
        None
    ));
    assert!(matches!(ownerless, Serverbound::MovePlayerStatus(p) if p.flags == 0b1011));
    assert!(!DONE.get());
    assert_eq!(service.behavior.lock().unwrap().last_notification, 0);
    assert!(
        service
            .notifications
            .drain(&|_, _| panic!("no notification"))
            .is_empty()
    );
    // Real enable/disable toggles preserve the singleton Chronometer.
    service.behavior.lock().unwrap().last_notification = 12345;
    service.set_running(false);
    let mut status = Serverbound::MovePlayerStatus(play::MovePlayerStatus { flags: 0b1011 });
    assert!(!crate::packet_bridge::outgoing_with_client_player(
        73,
        &mut status,
        false,
        &|_| panic!("disabled nested"),
        Some(&player)
    ));
    assert!(matches!(status, Serverbound::MovePlayerStatus(p) if p.flags == 0b1011));
    service.set_running(true);
    assert_eq!(service.behavior.lock().unwrap().last_notification, 12345);
    crate::packet_bridge::set_interceptor(None);
}
