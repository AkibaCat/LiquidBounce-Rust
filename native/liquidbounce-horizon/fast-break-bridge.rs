// SPDX-License-Identifier: GPL-3.0-or-later
//! Current real main-hand tags at ordinary Game.send. No Game is held by a callback.
use std::sync::{OnceLock, RwLock};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub session: u64,
    pub mining_tool: bool,
}
fn slot() -> &'static RwLock<Option<Snapshot>> {
    static SLOT: OnceLock<RwLock<Option<Snapshot>>> = OnceLock::new();
    SLOT.get_or_init(|| RwLock::new(None))
}
pub fn publish(snapshot: Option<Snapshot>) {
    *slot().write().expect("FastBreak snapshot poisoned") = snapshot;
}
pub fn snapshot(session: u64) -> Option<Snapshot> {
    slot()
        .read()
        .expect("FastBreak snapshot poisoned")
        .filter(|value| value.session == session && session != 0)
}
