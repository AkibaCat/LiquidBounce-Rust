// SPDX-License-Identifier: GPL-3.0-or-later
//! Retain the traced BlockHitResult.inside through the shared placement helper.
use super::{Game, Hand, Hit};
impl Game<'_> {
    pub(super) fn client_scaffold_do_placement(
        &mut self,
        hit: Hit,
        inside: bool,
        rotation: [f32; 2],
        hand: Hand,
        swing: &str,
        on_success: impl FnMut(&mut Self, bool) -> bool,
    ) -> bool {
        let previous = self.client_scaffold_click_inside.replace(inside);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.client_do_placement(hit, rotation, hand, swing, on_success)
        }));
        self.client_scaffold_click_inside = previous;
        match outcome {
            Ok(result) => result,
            Err(error) => std::panic::resume_unwind(error),
        }
    }
}
