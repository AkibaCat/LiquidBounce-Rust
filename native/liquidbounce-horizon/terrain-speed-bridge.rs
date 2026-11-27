// SPDX-License-Identifier: GPL-3.0-or-later
//! Window-thread scope for global Block.getFriction callbacks.
//!
//! Non-player entity solvers do not hold the local player's storage. Each
//! callback queues its individual multiplication, which Game drains before
//! returning from that solver or entering another player-observing callback.
//! Never combine multipliers: doing so changes rounding and signed zero.
use crate::terrain_speed::{FastClimb, TerrainSpeed};
use crate::terrain_speed_order::Selection;
use horizon_core::geometry::Vec3;
use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub settings: TerrainSpeed,
    pub forward: bool,
    pub sprint: bool,
    pub jump: bool,
    pub sneak: bool,
    /// Last installed untransformed ClientInput, before this tick reads keys.
    pub input: u8,
}
#[derive(Default)]
struct Context {
    snapshot: Option<Snapshot>,
    pending: Vec<f64>,
    selection: Selection,
    driven: HashSet<u32>,
    old_position: Option<Vec3>,
}
thread_local! {static CONTEXT:RefCell<Context>=RefCell::new(Context::default());}
pub struct Scope(Option<Context>);
pub fn enter(snapshot: Option<Snapshot>) -> Scope {
    Scope(Some(CONTEXT.with(|c| {
        c.replace(Context {
            snapshot,
            ..Default::default()
        })
    })))
}
impl Drop for Scope {
    fn drop(&mut self) {
        CONTEXT.with(|c| {
            let previous = self.0.take().unwrap();
            let current = c.replace(previous);
            debug_assert!(
                current.pending.is_empty(),
                "unflushed global TerrainSpeed friction callbacks"
            );
            debug_assert!(
                current.old_position.is_none(),
                "unflushed TerrainSpeed absSnapTo history"
            );
        });
    }
}
pub fn snapshot() -> Option<Snapshot> {
    CONTEXT.with(|c| c.borrow().snapshot)
}
pub fn selection() -> Selection {
    CONTEXT.with(|c|c.borrow().selection)
}
pub fn selected(id:u32)->bool {selection().includes(id)}
pub fn ancillary()->bool {selection().ancillary()}
pub fn ordered()->bool {snapshot().is_some()}
pub struct SelectionScope(Selection);
pub fn select(selection:Selection)->SelectionScope {
    SelectionScope(CONTEXT.with(|c|std::mem::replace(&mut c.borrow_mut().selection,selection)))
}
impl Drop for SelectionScope {
    fn drop(&mut self){CONTEXT.with(|c|c.borrow_mut().selection=self.0);}
}
pub fn mark_driven(id:u32){CONTEXT.with(|c|{c.borrow_mut().driven.insert(id);});}
pub fn was_driven(id:u32)->bool{CONTEXT.with(|c|c.borrow().driven.contains(&id))}
pub fn climb() -> Option<(FastClimb, bool)> {
    snapshot().map(|s| (s.settings.fast_climb, s.forward))
}
pub fn record_snap(position: Vec3) {
    CONTEXT.with(|c| c.borrow_mut().old_position = Some(position));
}
pub fn take_snap() -> Option<Vec3> {
    CONTEXT.with(|c| c.borrow_mut().old_position.take())
}
pub fn flush(velocity: &mut Vec3) {
    CONTEXT.with(|c| {
        for factor in c.borrow_mut().pending.drain(..) {
            velocity.x *= factor;
            velocity.y *= 1.0;
            velocity.z *= factor;
        }
    });
}
pub fn discard_pending() {
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.pending.clear();
    });
}
/// Exact event point for an arbitrary queried block, including non-player bodies.
pub fn friction(block: &str, base: f32) -> f32 {
    query_friction(block, base)
}
pub fn friction_for(_entity: u32, block: &str, base: f32) -> f32 {
    query_friction(block, base)
}
fn query_friction(block: &str, base: f32) -> f32 {
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        let Some(s) = c.snapshot else { return base };
        let ice = s.settings.ice_speed;
        if !ice.enabled
            || !matches!(
                block,
                "minecraft:ice"
                    | "minecraft:blue_ice"
                    | "minecraft:frosted_ice"
                    | "minecraft:packed_ice"
            )
        {
            return base;
        }
        if ice.motion_enabled {
            c.pending.push(f64::from(ice.horizontal_motion));
        }
        ice.slipperiness
    })
}
/// A local-player solver already owns its velocity, so apply synchronously here.
pub fn player_friction(block: &str, base: f32, velocity: &mut Vec3) -> f32 {
    flush(velocity);
    let Some(s) = snapshot() else { return base };
    let mut raw = [velocity.x, velocity.y, velocity.z];
    let friction = s
        .settings
        .ice_speed
        .block_friction(true, block, base, &mut raw);
    *velocity = Vec3::new(raw[0], raw[1], raw[2]);
    friction
}
