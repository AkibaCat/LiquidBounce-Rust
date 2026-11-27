// SPDX-License-Identifier: GPL-3.0-or-later
//! True setter events, separate from client entity-data packets. Shared by server workers.
use horizon_core::geometry::Vec3;
use std::{
    cell::RefCell,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub own: bool,
    pub ticked: bool,
    pub center: [f64; 3],
    pub movement: [f64; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Update {
    pub now: i64,
    pub context: Context,
    pub old: f32,
    pub new: f32,
    pub max: f32,
}
static EVENTS: Mutex<Vec<Update>> = Mutex::new(Vec::new());
thread_local! {static PLAYER:RefCell<Option<Context>>=const{RefCell::new(None)};static CLIENT:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};static KNOWN_MOVEMENT:std::cell::Cell<Option<Vec3>>=const{std::cell::Cell::new(None)};}
pub fn actor_movement(movement: Option<Vec3>) {
    KNOWN_MOVEMENT.set(movement);
}
pub fn mark_client() {
    CLIENT.set(true);
    install();
}
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}
pub fn context(own: bool, ticked: bool, center: Vec3, movement: Vec3) -> Context {
    Context {
        own,
        ticked,
        center: [center.x, center.y, center.z],
        movement: [movement.x, movement.y, movement.z],
    }
}
pub fn record(context: Context, old: f32, new: f32, max: f32) {
    if old != new && !context.own && context.ticked {
        EVENTS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(Update {
                now: now(),
                context,
                old,
                new,
                max,
            });
    }
}
pub fn drain() -> Vec<Update> {
    std::mem::take(&mut *EVENTS.lock().unwrap_or_else(|e| e.into_inner()))
}
pub fn clear() {
    EVENTS.lock().unwrap_or_else(|e| e.into_inner()).clear();
}
pub struct PlayerScope(Option<Context>);
impl Drop for PlayerScope {
    fn drop(&mut self) {
        PLAYER.with_borrow_mut(|p| *p = self.0);
    }
}
pub fn player(p: &crate::motion::PlayerMotion) -> PlayerScope {
    install();
    let c = context(
        CLIENT.get(),
        p.health.tick_count != 0,
        p.bounds().center(),
        KNOWN_MOVEMENT.get().unwrap_or(p.velocity),
    );
    PlayerScope(PLAYER.with_borrow_mut(|v| v.replace(c)))
}
pub fn install() {
    let _ = horizon_world::living_health::HEALTH_SET_OBSERVER.set(|old, new, max| {
        PLAYER.with_borrow(|p| {
            if let Some(c) = *p {
                record(c, old, new, max);
            }
        });
    });
}
pub fn mob(m: &crate::mobs::Mob, new: f32) {
    record(
        context(false, m.age != 0, m.bounds().center(), m.velocity),
        m.health.max(0.),
        new.clamp(0., crate::mobs::finalize::max_health(m)),
        crate::mobs::finalize::max_health(m),
    );
}
pub fn stand(s: &crate::entities::ArmorStand, new: f32) {
    record(
        context(false, !s.first_tick, s.bounds().center(), s.velocity),
        s.health.max(0.),
        new.clamp(0., 20.),
        20.,
    );
}
impl crate::mobs::Mob {
    pub fn client_lb_set_health(&mut self, value: f32) {
        mob(self, value);
        self.health = value;
    }
}
impl crate::entities::ArmorStand {
    pub fn client_lb_set_health(&mut self, value: f32) {
        stand(self, value);
        self.health = value;
    }
}
