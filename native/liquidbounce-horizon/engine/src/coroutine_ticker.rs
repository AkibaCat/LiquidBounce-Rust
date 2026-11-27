// SPDX-License-Identifier: GPL-3.0-or-later
//! CoroutineTicker pending/running callback FIFO from LiquidBounce ae0eb396.
//! Module callbacks and Host effects run outside the queue's short mutex lock.
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitOwner {
    Timer,
    HighJump,
    Scaffold,
    ScaffoldTower,
    AutoDeposit,
    Replenish,
    AutoCrafter,
    AutoArmor,
    AutoMobHeal,
    NoSlow,
    NoWeb,
    VehicleControl,
    Step,
    ResetVl,
    AutoLeave,
    AutoAccount,
    ReportHelperReport,
    ReportHelperConfirm,
    AutoQueue,
    FastUse,
    FastExp,
    AutoBuff,
    AutoPearl,
    AutoRod,
    Clip,
    Spider,
    SuperKnockback,
    AntiBotMatrix,
    TriggerBot,
    AutoClickerAttack,
    AutoClickerUse,
    Derp,
    HandDerp,
    AntiAfk,
    AutoFish,
    AutoRespawn,
    SmartEat,
    SkinDerp,
    InventoryManager,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Task {
    pub ticket: u64,
    pub owner: WaitOwner,
    pub callback: u64,
}
#[derive(Debug, Default)]
struct State {
    next: u64,
    running: Vec<Task>,
    pending: Vec<Task>,
    advancing: bool,
}
#[derive(Clone, Debug, Default)]
pub struct CoroutineTicker {
    state: Arc<Mutex<State>>,
}
impl CoroutineTicker {
    /// Call exactly when the module's real coroutine registers a waiter.
    /// An off-thread producer can enqueue safely; no callback runs here.
    pub fn register(&self, owner: WaitOwner, callback: u64) -> u64 {
        let mut state = self.state.lock().expect("CoroutineTicker poisoned");
        state.next = state
            .next
            .checked_add(1)
            .expect("CoroutineTicker ticket exhausted");
        let ticket = state.next;
        state.pending.push(Task {
            ticket,
            owner,
            callback,
        });
        ticket
    }
    /// Nested Minecraft.tick calls must not advance callbacks a second time.
    /// Taking a batch freezes this tick; registrations during it stay pending.
    pub fn begin_tick(&self) -> Option<TickBatch> {
        let mut state = self.state.lock().expect("CoroutineTicker poisoned");
        if state.advancing {
            return None;
        }
        state.advancing = true;
        let mut tasks = std::mem::take(&mut state.running);
        tasks.append(&mut state.pending);
        Some(TickBatch {
            ticker: self.clone(),
            remaining: tasks.into(),
            retained: Vec::new(),
            current: None,
        })
    }
}
/// An unfinished callback is retained in place. New callbacks never join this
/// batch, even if they resume synchronously while a previous callback executes.
pub struct TickBatch {
    ticker: CoroutineTicker,
    remaining: VecDeque<Task>,
    retained: Vec<Task>,
    current: Option<Task>,
}
impl TickBatch {
    pub fn next_task(&mut self) -> Option<Task> {
        self.finish_current(true);
        self.current = self.remaining.pop_front();
        self.current
    }
    /// keep=false also handles an already-cancelled/missing local callback.
    pub fn finish_current(&mut self, keep: bool) {
        if let Some(task) = self.current.take()
            && keep
        {
            self.retained.push(task);
        }
    }
}
impl Drop for TickBatch {
    fn drop(&mut self) {
        self.finish_current(true);
        self.retained.extend(self.remaining.drain(..));
        let mut state = self.ticker.state.lock().expect("CoroutineTicker poisoned");
        state.running = std::mem::take(&mut self.retained);
        state.advancing = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistent_running_precedes_new_pending_and_nested_ticks_do_nothing() {
        let ticker = CoroutineTicker::default();
        let old = ticker.register(WaitOwner::AntiAfk, 1);
        {
            let mut batch = ticker.begin_tick().unwrap();
            assert_eq!(batch.next_task().unwrap().ticket, old);
            assert!(ticker.begin_tick().is_none());
            batch.finish_current(true);
        }
        let derp = ticker.register(WaitOwner::Derp, 2);
        let hand = ticker.register(WaitOwner::HandDerp, 3);
        {
            let mut batch = ticker.begin_tick().unwrap();
            assert_eq!(batch.next_task().unwrap().ticket, old);
            batch.finish_current(false);
            let first = batch.next_task().unwrap();
            assert_eq!(first.ticket, derp);
            let next = ticker.register(WaitOwner::Derp, 4);
            assert!(next > hand);
            batch.finish_current(false);
            assert_eq!(batch.next_task().unwrap().ticket, hand);
            batch.finish_current(true);
            assert!(batch.next_task().is_none());
        }
        let mut batch = ticker.begin_tick().unwrap();
        assert_eq!(batch.next_task().unwrap().ticket, hand);
        batch.finish_current(false);
        assert_eq!(batch.next_task().unwrap().callback, 4);
        batch.finish_current(false);
        assert!(batch.next_task().is_none());
    }
    #[test]
    fn early_batch_drop_retains_current_and_unprocessed_before_pending() {
        let ticker = CoroutineTicker::default();
        ticker.register(WaitOwner::AutoFish, 10);
        ticker.register(WaitOwner::AutoRespawn, 11);
        {
            let mut batch = ticker.begin_tick().unwrap();
            assert_eq!(batch.next_task().unwrap().callback, 10);
            ticker.register(WaitOwner::HandDerp, 12);
        }
        let mut batch = ticker.begin_tick().unwrap();
        for callback in [10, 11, 12] {
            assert_eq!(batch.next_task().unwrap().callback, callback);
            batch.finish_current(false);
        }
        assert!(batch.next_task().is_none());
    }
}
