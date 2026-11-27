// SPDX-License-Identifier: GPL-3.0-or-later
// Shared BlockExtensions.doPlacement. Result callbacks observe real dispatch/swing phases.
use super::*;

impl Game<'_> {
    /// Caller verifies the hit and selects the silent slot before this call.
    /// Return true only for the source's onPlacementSuccess/onItemUseSuccess callback.
    /// Interaction swing has no Punch packet; HideForServer equals DoNotHide.
    pub(super) fn client_do_placement(
        &mut self,
        hit: Hit,
        rotation: [f32; 2],
        hand: Hand,
        swing_mode: &str,
        mut on_success: impl FnMut(&mut Self, bool) -> bool,
    ) -> bool {
        let previous_hit = self.forced_hit.replace(hit);
        let previous_guest = self.guest_use;
        // 0: no predicted swing (including CONSUME), 1: predicted SUCCESS,
        // -1: SUCCESS_SERVER. The action bool alone cannot distinguish these.
        let previous_capture = self.client_placement_result.replace(0);
        let before = self.with_hand(hand, |g| g.held().cloned());
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.native_with_item_rotation(rotation[0], rotation[1], |g| {
                g.remote_send_carried();
                g.use_failed = false;
                let ended = if g.remote.is_some() {
                    g.remote_use_item_on_at(hand, Some(hit))
                } else {
                    g.guest_use = Some(GuestUse::OnBlock(hand));
                    g.with_hand(hand, |g| g.use_hand(Some(&hit)))
                        .unwrap_or_else(|_| {
                            g.use_failed = true;
                            true
                        })
                };
                if ended {
                    let after = g.with_hand(hand, |g| g.held().map(|s| s.count));
                    let stack_used = before
                        .as_ref()
                        .is_some_and(|s| s.count > 0 && (after != Some(s.count) || g.creative));
                    let callback =
                        !g.use_failed && g.client_placement_result.is_some_and(|v| v >= 0);
                    if callback {
                        let swing = on_success(g, false);
                        if swing && !matches!(swing_mode, "HideForBoth" | "HideForClient") {
                            let animation = g.auto_rod_animation(hand);
                            g.swing_hand(hand, animation);
                        }
                        if stack_used {
                            g.heights.item_used(hand);
                        }
                    }
                    return callback;
                }
                // PASS alone permits Item.use; FAIL/CONSUME have ended above.
                if before
                    .as_ref()
                    .is_none_or(|s| s.count <= 0 || &*s.item == "minecraft:air")
                {
                    return false;
                }
                g.client_placement_result = Some(0);
                let used = if g.remote.is_some() {
                    g.remote_use_item_rotated(hand, rotation[0], rotation[1])
                } else {
                    g.guest_use = previous_guest;
                    g.with_hand(hand, |g| g.client_use_item_only(false))
                        .unwrap_or(false)
                        && !g.use_failed
                };
                let predicted = used && g.client_placement_result == Some(1);
                if predicted {
                    let swing = on_success(g, true);
                    if swing && !matches!(swing_mode, "HideForBoth" | "HideForClient") {
                        let animation = g.auto_rod_animation(hand);
                        g.swing_hand(hand, animation);
                    }
                }
                if used && matches!(g.client_placement_result, Some(1 | -1)) {
                    g.heights.item_used(hand);
                }
                predicted
            })
        }));
        self.forced_hit = previous_hit;
        self.guest_use = previous_guest;
        self.client_placement_result = previous_capture;
        let accepted = match outcome {
            Ok(v) => v,
            Err(e) => std::panic::resume_unwind(e),
        };
        accepted
    }
}
