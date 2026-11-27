// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces only auto_walk_runtime's original ownership/event state section.
use super::navigation_owners::{Holds, Owner, Slots};
thread_local! {
 static RUNTIME:RefCell<Option<Runtime>>=const{RefCell::new(None)};
 static CONFIG:RefCell<Value>=const{RefCell::new(Value::Null)};
 static RESULT:RefCell<Option<String>>=const{RefCell::new(None)};
 static EVENTS:RefCell<Vec<Event>>=const{RefCell::new(Vec::new())};
 static RETIRED:Cell<bool>=const{Cell::new(false)};
 static FIGHT_RUNTIME:RefCell<Option<FightRuntime>>=const{RefCell::new(None)};
 static FIGHT_CONFIG:RefCell<Value>=const{RefCell::new(Value::Null)};
 static FIGHT_RESULT:RefCell<Option<String>>=const{RefCell::new(None)};
 static FIGHT_EVENTS:RefCell<Vec<Event>>=const{RefCell::new(Vec::new())};
 static FIGHT_RETIRED:Cell<bool>=const{Cell::new(false)};
 static ACTIVE_OWNER:Cell<Owner>=const{Cell::new(Owner::AutoWalk)};
 static DESTROYING_OWNERS:Cell<Holds>=const{Cell::new(Holds::empty())};
 static USING_OWNERS:Cell<Holds>=const{Cell::new(Holds::empty())};
 static SAFE_OWNERS:Cell<Holds>=const{Cell::new(Holds::empty())};
 static SLOT_OWNERS:RefCell<Slots>=RefCell::new(Slots::default());
}
struct OwnerScope(Owner);
impl OwnerScope {
    fn enter(owner: Owner) -> Self {
        Self(ACTIVE_OWNER.replace(owner))
    }
}
impl Drop for OwnerScope {
    fn drop(&mut self) {
        ACTIVE_OWNER.set(self.0);
    }
}
fn set_hold(slot: &'static std::thread::LocalKey<Cell<Holds>>, value: bool) {
    slot.with(|s| {
        let mut holds = s.get();
        holds.set(ACTIVE_OWNER.get(), value);
        s.set(holds);
    });
}
pub(super) fn keeps_destroying() -> bool {
    DESTROYING_OWNERS.get().any()
}
pub(super) fn releasing_destroying() -> bool {
    DESTROYING_OWNERS.get().releasing_last(ACTIVE_OWNER.get())
}
pub(super) fn keep_destroying(value: bool) {
    set_hold(&DESTROYING_OWNERS, value);
}
pub(super) fn keeps_using() -> bool {
    USING_OWNERS.get().any()
}
pub(super) fn keep_using(value: bool) {
    set_hold(&USING_OWNERS, value);
}
pub(super) fn navigation_safe_walk(value: bool) {
    set_hold(&SAFE_OWNERS, value);
    horizon_client::navigation_physics::set_safe_walk(SAFE_OWNERS.get().any());
}
pub(super) fn claim_slot(slot: usize, current: usize) {
    SLOT_OWNERS.with(|s| s.borrow_mut().claim(ACTIVE_OWNER.get(), slot, current));
}
pub(super) fn release_slot() -> Option<usize> {
    SLOT_OWNERS.with(|s| s.borrow_mut().release(ACTIVE_OWNER.get()))
}
fn push_event(event: Event) {
    EVENTS.with(|s| s.borrow_mut().push(event));
    FIGHT_EVENTS.with(|s| s.borrow_mut().push(event));
}
pub(super) fn changed(epoch: u64, pos: [i32; 3]) {
    push_event(Event::Changed(epoch, BlockPos::new(pos[0], pos[1], pos[2])));
}
pub(super) fn confirmed(epoch: u64, pos: [i32; 3], state: u32) {
    push_event(Event::Confirmed(
        epoch,
        BlockPos::new(pos[0], pos[1], pos[2]),
        state,
    ));
}
pub(super) fn corrected(epoch: u64) {
    push_event(Event::Corrected(epoch));
}
pub(super) fn level_changed() {
    // Old worlds are retired without using a previous world's player or inventory.
    RETIRED.set(true);
    FIGHT_RETIRED.set(true);
    EVENTS.with(|s| s.borrow_mut().clear());
    FIGHT_EVENTS.with(|s| s.borrow_mut().clear());
    RESULT.with(|s| s.borrow_mut().take());
    FIGHT_RESULT.with(|s| s.borrow_mut().take());
    DESTROYING_OWNERS.set(Holds::empty());
    USING_OWNERS.set(Holds::empty());
    SAFE_OWNERS.set(Holds::empty());
    SLOT_OWNERS.with(|s| *s.borrow_mut() = Slots::default());
    horizon_client::navigation_physics::set_safe_walk(false);
}
