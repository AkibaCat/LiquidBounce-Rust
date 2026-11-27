// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned ModuleAutoRespawn and its parallel, undispatched ScreenEvent runs.

#[derive(Debug, Default)]
pub struct AutoRespawn {
    delay: i32,
    waiting: Vec<(u64, i32)>,
    callback_counter: u64,
    wait_registrations: Vec<u64>,
}

impl AutoRespawn {
    pub const fn delay(&self) -> i32 {
        self.delay
    }

    /// RangedValue's 0..20 is UI metadata. ConfigSystem deserializes raw Ints
    /// without clamping; preserve negative and above-UI-range imported values.
    pub fn set_delay(&mut self, ticks: i32) {
        self.delay = ticks;
    }

    /// Call for the proposed ScreenEvent while this listener is running.
    /// True means respawn then setScreen(null) immediately. This does not cancel
    /// the enclosing screen-opening operation, exactly as the Kotlin handler.
    pub fn screen_event(&mut self, running: bool, death_screen: bool) -> bool {
        if !running {
            return false;
        }
        if !death_screen {
            return false;
        }
        if self.delay <= 0 {
            return true;
        }
        // sequenceHandler launches a parallel run for each event. Changing the
        // setting later does not change a run's already captured waitTicks.
        self.callback_counter = self
            .callback_counter
            .checked_add(1)
            .expect("AutoRespawn callback id exhausted");
        self.waiting.push((self.callback_counter, self.delay));
        self.wait_registrations.push(self.callback_counter);
        false
    }

    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }

    /// Only the callback selected by the shared CoroutineTicker advances.
    /// The running guard is checked on completion, matching waitTicks.
    pub fn advance_callback(&mut self, callback: u64, running: bool) -> (bool, bool) {
        let Some(index) = self.waiting.iter().position(|(id, _)| *id == callback) else {
            return (false, false);
        };
        self.waiting[index].1 -= 1;
        if self.waiting[index].1 != 0 {
            return (true, false);
        }
        self.waiting.remove(index);
        (false, running)
    }

    /// Advance existing waits at the beginning of each actual game tick.
    /// For every returned continuation, the caller must freshly check whether
    /// the current screen is DeathScreen, then respawn and setScreen(null).
    /// Checking separately matters when parallel runs expire together: the
    /// first continuation closes the screen before the second one resumes.
    pub fn game_tick(&mut self, running: bool) -> usize {
        let before = self.waiting.len();
        self.waiting.retain_mut(|(_, remaining)| {
            *remaining -= 1;
            *remaining != 0
        });
        // With no requires{}, activeGuards is null. The continuation interceptor
        // checks running when the wait completes, rather than on every elapsed
        // tick. A temporary out-of-game interval can end before this deadline.
        if running {
            before - self.waiting.len()
        } else {
            0
        }
    }

    /// Call only for the actual CancelSequences lifecycle callback. Both
    /// ClientModule and ToggleableValueGroup skip onToggled outside inGame;
    /// neither a temporary running=false nor an out-of-game value edit cancels
    /// the scope immediately.
    pub fn on_disabled(&mut self) {
        self.waiting.clear();
        self.wait_registrations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fifo_wait_keeps_captured_delay_and_canceled_identity_cannot_advance_replacement() {
        let mut module = AutoRespawn::default();
        module.set_delay(2);
        assert!(!module.screen_event(true, true));
        let first = module.take_wait_registrations()[0];
        assert_eq!(module.advance_callback(first, false), (true, false));
        module.set_delay(20);
        assert_eq!(module.advance_callback(first, true), (false, true));
        module.set_delay(2);
        module.screen_event(true, true);
        let canceled = module.take_wait_registrations()[0];
        module.on_disabled();
        module.screen_event(true, true);
        let replacement = module.take_wait_registrations()[0];
        assert_eq!(module.advance_callback(canceled, false), (false, false));
        assert_eq!(module.advance_callback(replacement, true), (true, false));
        assert_eq!(module.advance_callback(replacement, false), (false, false));
    }
}
