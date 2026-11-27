use liquidbounce_horizon_packet_tests::{slot_bridge::SlotPolicy, slots::*};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

fn slot(index: i32) -> HotbarSlot {
    HotbarSlot::new(index).unwrap()
}

#[test]
fn original_nullable_identity_visible_capture_reset_and_tick_boundaries() {
    let mut hotbar = SilentHotbar::default();
    let first = Requester::allocate();
    let second = Requester::allocate();
    assert_ne!(first, second);
    assert_eq!(Requester::from_raw(first.raw()), first);
    assert!(!hotbar.is_slot_modified());
    // Kotlin null === null is true even when hotbarState is absent.
    assert!(hotbar.is_slot_modified_by(Requester::NONE));
    assert!(!hotbar.is_slot_modified_by(first));
    assert_eq!(hotbar.serverside_slot(None).index(), 0);
    assert_eq!(hotbar.clientside_slot(Some(slot(2))).index(), 2);
    hotbar.select_slot(first, slot(7), 3, Some(slot(2)));
    assert_eq!(hotbar.serverside_slot(Some(slot(5))).index(), 7);
    assert_eq!(hotbar.clientside_slot(Some(slot(5))).index(), 2);
    hotbar.tick();
    hotbar.select_slot(second, slot(8), 1, Some(slot(5)));
    assert_eq!(hotbar.ticks_since_last_update(), 0);
    assert_eq!(hotbar.clientside_slot(Some(slot(5))).index(), 2);
    assert!(!hotbar.reset_slot(first));
    hotbar.tick();
    assert!(hotbar.is_slot_modified());
    assert_eq!(hotbar.ticks_since_last_update(), 1);
    hotbar.tick();
    assert!(!hotbar.is_slot_modified());
    assert_eq!(hotbar.serverside_slot(Some(slot(5))).index(), 5);
    assert_eq!(hotbar.clientside_slot(Some(slot(5))).index(), 5);
    for duration in [0, -1, i32::MIN] {
        hotbar.select_slot(first, slot(8), duration, None);
        hotbar.tick();
        assert!(!hotbar.is_slot_modified());
    }
    hotbar.select_slot(Requester::NONE, slot(4), i32::MAX, Some(slot(1)));
    for _ in 0..128 {
        hotbar.tick();
    }
    assert_eq!(hotbar.ticks_since_last_update(), 128);
    assert!(hotbar.is_slot_modified_by(Requester::NONE));
    assert!(hotbar.reset_slot(Requester::NONE));
    assert_eq!(hotbar.ticks_since_last_update(), 128);
    hotbar.world_change();
    assert_eq!(hotbar.ticks_since_last_update(), 0);
    assert!(!hotbar.is_slot_modified());
    assert!(HotbarSlot::new(-1).is_err());
    assert!(HotbarSlot::new(9).is_err());
}

#[test]
fn selection_validation_cancellation_priority_reentry_and_post_event_real_slot() {
    struct Listener {
        name: &'static str,
        events: Arc<Mutex<Vec<&'static str>>>,
        service: Arc<SlotService>,
        real: Arc<AtomicU8>,
        cancel: bool,
        reenter: AtomicBool,
    }
    impl SelectSlotListener for Listener {
        fn selecting(&self, event: &mut SelectSlotEvent) {
            self.events.lock().unwrap().push(self.name);
            if self.cancel {
                event.cancel();
            }
            assert!(event.slot().index() < 9);
            let _ = self.service.modified_by(event.requester().raw());
            let id = self.service.add_listener(-999, Arc::new(Noop));
            assert!(self.service.remove_listener(id));
            // Original selection event can re-enter selection and state reads.
            if self.reenter.swap(false, Ordering::Relaxed) {
                self.service
                    .select_slot_silently(Requester::NONE, 6, 9, || Some(1))
                    .unwrap();
                self.service.world_change();
            }
            self.real.store(4, Ordering::Relaxed);
        }
    }
    struct Noop;
    impl SelectSlotListener for Noop {
        fn selecting(&self, _: &mut SelectSlotEvent) {}
    }
    let service = Arc::new(SlotService::default());
    let events = Arc::new(Mutex::new(Vec::new()));
    let real = Arc::new(AtomicU8::new(2));
    let owner = Requester::allocate();
    let listener = |name, cancel, reenter| {
        Arc::new(Listener {
            name,
            events: events.clone(),
            service: service.clone(),
            real: real.clone(),
            cancel,
            reenter: AtomicBool::new(reenter),
        })
    };
    let late = service.add_listener(0, listener("late", false, false));
    let high = service.add_listener(100, listener("high", false, true));
    let peer = service.add_listener(100, listener("peer", false, false));
    assert!(
        service
            .select_slot_silently(owner, 7, 1, || {
                assert!(!service.modified());
                Some(real.load(Ordering::Relaxed))
            })
            .unwrap()
    );
    assert_eq!(service.clientside_slot(Some(2)), 4);
    assert_eq!(
        *events.lock().unwrap(),
        ["high", "high", "peer", "late", "peer", "late"]
    );
    for id in [late, high, peer] {
        assert!(service.remove_listener(id));
    }
    service.tick();
    let before = service.state();
    events.lock().unwrap().clear();
    let cancel = service.add_listener(10, listener("cancel", true, false));
    let after = service.add_listener(0, listener("after", false, false));
    assert!(
        !service
            .select_slot_silently(owner, 1, 999, || panic!(
                "cancelled selection never reads real slot"
            ))
            .unwrap()
    );
    assert_eq!(service.state(), before);
    assert_eq!(*events.lock().unwrap(), ["cancel", "after"]);
    events.lock().unwrap().clear();
    assert!(service.select_slot_silently(owner, 99, 1, || None).is_err());
    assert!(events.lock().unwrap().is_empty());
    assert!(
        service
            .select_inventory_slot_silently(owner, None, 1, || panic!("offhand ignored"))
            .unwrap()
    );
    assert!(events.lock().unwrap().is_empty());
    assert_eq!(service.state(), before);
    service.tick();
    assert!(
        !service.modified(),
        "cancelled request must not refresh original timer"
    );
    assert!(service.remove_listener(cancel));
    assert!(service.remove_listener(after));
}

#[test]
fn real_slot_fallback_and_wire_audit_stay_independent() {
    let service = SlotService::default();
    assert!(!service.no_slot_set.running());
    assert!(service.no_slot_set.set_running(true));
    assert!(!service.no_slot_set.set_running(true));
    assert_eq!(service.serverside_slot(Some(2)), 2);
    service.outgoing_slot_sent(10, 7, false);
    assert_eq!(service.serverside_slot(Some(3)), 3);
    assert_eq!(
        service.wire_slot(),
        Some(WireSlot {
            session: 10,
            slot: 7,
            silent: false
        })
    );
    service
        .select_slot_silently(Requester::NONE, 5, 100, || Some(3))
        .unwrap();
    service.outgoing_slot_sent(10, 8, true);
    service.server_slot_applied(4);
    assert_eq!(service.serverside_slot(Some(4)), 5);
    assert_eq!(service.clientside_slot(Some(4)), 3);
    assert_eq!(service.last_real_slot(), Some(4));
    assert_eq!(service.last_server_applied_slot(), Some(4));
    assert!(service.reset_slot(0));
    assert_eq!(service.serverside_slot(Some(6)), 6);
    assert_eq!(service.clientside_slot(Some(6)), 6);
    assert_eq!(
        service.wire_slot(),
        Some(WireSlot {
            session: 10,
            slot: 8,
            silent: true
        })
    );
    service.world_change();
    assert_eq!(service.wire_slot(), None);
    assert_eq!(service.last_real_slot(), None);
    assert_eq!(service.last_server_applied_slot(), None);
    assert_eq!(service.serverside_slot(None), 0);
    assert!(service.no_slot_set.running());
}
