// SPDX-License-Identifier: GPL-3.0-or-later
//! Policy shared with the world crate, active only inside real local-player calls.
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::RwLock;
static POLICY: RwLock<[f64; 12]> = RwLock::new([-1.; 12]);
thread_local! {static OWNER:Cell<bool>=const{Cell::new(false)};}
/// The scope is thread-bound and restores the enclosing owner even on early return.
/// A nested server/replay call explicitly installs false instead of inheriting true.
pub struct OwnerScope {
    previous: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for OwnerScope {
    fn drop(&mut self) {
        OWNER.with(|owner| owner.set(self.previous));
    }
}
pub fn owner_scope(active: bool) -> OwnerScope {
    OwnerScope {
        previous: OWNER.with(|owner| owner.replace(active)),
        _thread: PhantomData,
    }
}
fn owned() -> bool {
    OWNER.with(Cell::get)
}
pub fn configure(values: [f64; 12]) {
    *POLICY.write().expect("NoSlow policy") = values;
}
pub fn get() -> [f64; 12] {
    *POLICY.read().expect("NoSlow policy")
}
pub fn running() -> bool {
    owned() && get()[0] == 1.
}
pub fn factor(index: usize) -> Option<f32> {
    let p = get();
    (owned() && p[0] == 1. && p[index] >= 0.).then_some(p[index] as f32)
}
pub fn block_factor(name: &str, original: f32) -> f32 {
    match name {
        "minecraft:soul_sand" => factor(2),
        "minecraft:slime_block" => factor(3),
        "minecraft:honey_block" => factor(4),
        _ => None,
    }
    .unwrap_or(original)
}
pub fn friction(name: &str, original: f32) -> f32 {
    if name == "minecraft:slime_block" && factor(3).is_some() {
        0.6
    } else {
        original
    }
}
pub fn fluid_push() -> bool {
    let p = get();
    !owned() || p[0] != 1. || p[6] != 1.
}
pub fn fluid_collision() -> bool {
    let p = get();
    !owned() || p[0] != 1. || p[6] != 1. || p[7] == 1.
}
