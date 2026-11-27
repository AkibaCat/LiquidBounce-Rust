// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleNoPush at ae0eb396.
// Copyright (c) 2015-2026 CCBlueX.

//! PushBy's source selection allows those pushes; Sinking has its own handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushBy {
    Entities,
    Blocks,
    FishingRod,
    Liquids,
    Sinking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoPush {
    pub entities: bool,
    pub blocks: bool,
    pub fishing_rod: bool,
    pub liquids: bool,
    pub sinking: bool,
}
impl Default for NoPush {
    fn default() -> Self {
        Self {
            entities: true,
            blocks: false,
            fishing_rod: false,
            liquids: true,
            sinking: false,
        }
    }
}
impl NoPush {
    pub fn can_push(self, running: bool, by: PushBy) -> bool {
        !running
            || match by {
                PushBy::Entities => self.entities,
                PushBy::Blocks => self.blocks,
                PushBy::FishingRod => self.fishing_rod,
                PushBy::Liquids => self.liquids,
                PushBy::Sinking => self.sinking,
            }
    }

    /// GameTick uses the actual option keys, before input events transform them.
    pub fn sinking_velocity(
        self,
        running: bool,
        original_y: f64,
        in_liquid: bool,
        physical_jump: bool,
        physical_shift: bool,
    ) -> f64 {
        if running
            && self.sinking
            && !physical_jump
            && !physical_shift
            && in_liquid
            && original_y < 0.0
        {
            0.0
        } else {
            original_y
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_push_sources_follow_original_allow_list() {
        let policy = NoPush::default();
        assert!(policy.can_push(true, PushBy::Entities));
        assert!(policy.can_push(true, PushBy::Liquids));
        assert!(!policy.can_push(true, PushBy::Blocks));
        assert!(!policy.can_push(true, PushBy::FishingRod));
        for source in [
            PushBy::Entities,
            PushBy::Blocks,
            PushBy::FishingRod,
            PushBy::Liquids,
            PushBy::Sinking,
        ] {
            assert!(policy.can_push(false, source));
        }
    }
    #[test]
    fn sinking_handler_preserves_input_controls_nonliquid_and_float_edges() {
        let policy = NoPush {
            sinking: true,
            ..NoPush::default()
        };
        assert_eq!(
            policy.sinking_velocity(true, -0.04, true, false, false),
            0.0
        );
        assert_eq!(
            policy.sinking_velocity(true, -0.04, true, true, false),
            -0.04
        );
        assert_eq!(
            policy.sinking_velocity(true, -0.04, true, false, true),
            -0.04
        );
        assert_eq!(
            policy.sinking_velocity(true, -0.04, false, false, false),
            -0.04
        );
        assert_eq!(
            policy.sinking_velocity(false, -0.04, true, false, false),
            -0.04
        );
        assert_eq!(
            policy
                .sinking_velocity(true, -0.0, true, false, false)
                .to_bits(),
            (-0.0f64).to_bits()
        );
        assert!(
            policy
                .sinking_velocity(true, f64::NAN, true, false, false)
                .is_nan()
        );
        assert_eq!(
            NoPush::default().sinking_velocity(true, -0.04, true, false, false),
            -0.04
        );
    }
}
