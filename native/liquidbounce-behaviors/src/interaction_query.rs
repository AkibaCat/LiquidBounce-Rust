// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce's MixinLocalPlayer and MixinPlayer,
// Copyright (c) 2015-2026 CCBlueX.

//! Query changes for LiquidPlace, PortalMenu and AntiReducedDebugInfo.
//! These functions do not alter a server rule, the local debug option,
//! portal effects, or an item's independently chosen fluid ray mode.

/// `LocalPlayer.pick`'s own-player camera ray uses Fluid.ANY. Keep this
/// separate from bucket/source-only, boat and camera-collision raycasts.
pub const fn pick_any_fluid(original: bool, running: bool) -> bool {
    running || original
}

/// `Screen.isAllowedInPortal` during handlePortalTransitionEffect only.
pub const fn portal_allowed(original: bool, running: bool) -> bool {
    running || original
}

/// `Player.isReducedDebugInfo`. The separate Options.reducedDebugInfo
/// expression must be ORed with this result afterwards by its caller.
pub const fn player_reduced_debug(original: bool, running: bool) -> bool {
    !running && original
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_debug_keeps_the_local_option_independent_of_the_server_query() {
        let server_rule = true;
        let local_option = true;
        let player_query = player_reduced_debug(server_rule, true);
        assert!(!player_query);
        assert!(local_option || player_query);
        assert!(server_rule);
        assert!(player_reduced_debug(server_rule, false));
        assert!(!player_reduced_debug(false, false));
    }

    #[test]
    fn allowed_queries_keep_existing_permissions_after_disable() {
        // Chat/death screens already allow portals, and a preselected ANY
        // fluid query already allows fluids. Disabling either module must
        // preserve those original permissions.
        assert!(portal_allowed(true, false));
        assert!(pick_any_fluid(true, false));
        assert!(portal_allowed(false, true));
        assert!(pick_any_fluid(false, true));
        assert!(!portal_allowed(false, false));
        assert!(!pick_any_fluid(false, false));
    }
}
