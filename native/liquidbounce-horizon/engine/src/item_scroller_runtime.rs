// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use liquidbounce_behaviors::random::with_thread_random;

impl LiquidBounceHooks {
    pub(crate) fn item_scroller_query(&self, shift: bool, left_pressed: bool, allowed: &mut bool) {
        let state = self.state();
        let modules = state.behaviors.borrow();
        *allowed |= modules.item_scroller.can_perform_scroll(
            modules.query_running(&state.registry, "ItemScroller"), shift, left_pressed,
            |first, last| with_thread_random(|random| random.next_i32_inclusive(first, last)),
            || match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(elapsed) => elapsed.as_millis() as i64,
                Err(error) => (error.duration().as_millis() as i64).wrapping_neg(),
            },
        );
    }
    pub(crate) fn item_scroller_reset(&self, now: i64) {
        self.state().behaviors.borrow_mut().item_scroller.reset_chronometer(now);
    }
}
