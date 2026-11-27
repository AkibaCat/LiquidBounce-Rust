// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce's MixinLivingEntity and MixinEntity,
// Copyright (c) 2015-2026 CCBlueX.

//! The particular movement queries changed by the original mixins. Callers
//! must keep the stored effects and sneak input intact, and restrict these
//! client decisions to client simulation rather than server or bot prediction.

/// `LivingEntity.aiStep` HEAD. AirJump's allowJump and an active Scaffold
/// tower are independent inputs; the tower also clears the delay when
/// NoJumpDelay is disabled or AirJump owns jumping.
pub const fn clear_jump_delay(
    running: bool,
    air_jump_allow: bool,
    scaffold_tower_active: bool,
) -> bool {
    (running && !air_jump_allow) || scaffold_tower_active
}

/// Apply the HEAD decision before vanilla decrements or reads noJumpDelay.
pub const fn jump_delay(
    original: i32,
    running: bool,
    air_jump_allow: bool,
    scaffold_tower_active: bool,
) -> i32 {
    if clear_jump_delay(running, air_jump_allow, scaffold_tower_active) {
        0
    } else {
        original
    }
}

/// Only `Entity.isSuppressingBounce`'s isShiftKeyDown expression changes.
pub const fn suppress_bounce(original: bool, running: bool) -> bool {
    running || original
}

/// Only `LivingEntity.travelInAir`'s LEVITATION lookup changes. Other
/// levitation queries (gliding, fall resets, HUD and particles) retain it.
pub const fn levitation_amplifier(original: Option<i32>, running: bool) -> Option<i32> {
    if running { None } else { original }
}

/// Only `LivingEntity.getEffectiveGravity`'s SLOW_FALLING query changes.
pub const fn slow_falling(original: bool, running: bool) -> bool {
    !running && original
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump_delay_keeps_air_jump_ownership_and_tower_precedence() {
        // A previous ground jump leaves 10. At aiStep HEAD NoJumpDelay
        // clears it before vanilla's decrement; disabling restores the
        // still-stored delay rather than forcing another jump at tick end.
        assert_eq!(jump_delay(10, true, false, false), 0);
        assert_eq!(jump_delay(10, true, true, false), 10);
        assert_eq!(jump_delay(10, true, true, true), 0);
        assert_eq!(jump_delay(10, false, false, true), 0);
        // An untouched field is returned exactly, even outside its usual
        // nonnegative domain. The source resets unconditionally when active.
        assert_eq!(jump_delay(-1, false, false, false), -1);
        assert_eq!(jump_delay(-1, true, false, false), 0);
        assert_eq!(jump_delay(i32::MAX, false, true, false), i32::MAX);
    }

    #[test]
    fn effect_queries_restore_the_original_without_removing_status() {
        // Amplifier 0 is an existing level-I effect, not an absent effect.
        // Reading it through the enabled module must not erase the stored
        // value that gliding/fall logic and the next disabled query receive.
        for original in [None, Some(0), Some(4), Some(i32::MAX)] {
            assert_eq!(levitation_amplifier(original, true), None);
            assert_eq!(levitation_amplifier(original, false), original);
        }
        let stored_slow_falling = true;
        assert!(!slow_falling(stored_slow_falling, true));
        assert!(stored_slow_falling);
        assert!(slow_falling(stored_slow_falling, false));
        // AntiBounce's query is separate from sneaking: callers retain the
        // same input for crouching, edge backoff and sprint conditions.
        let shift_down = false;
        assert!(suppress_bounce(shift_down, true));
        assert!(!shift_down);
        assert!(!suppress_bounce(shift_down, false));
        assert!(suppress_bounce(true, false));
    }
}
