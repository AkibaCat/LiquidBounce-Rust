// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleNoPose at ae0eb396. A pose override, not a swimming-state override.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SneakHeight {
    V1_8,
    V1_9,
    #[default]
    V1_15,
}
impl SneakHeight {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::V1_8 => "1.8",
            Self::V1_9 => "1.9",
            Self::V1_15 => "1.15",
        }
    }
    /// Zero is reserved for the module being inactive.
    pub const fn policy_code(self) -> u8 {
        match self {
            Self::V1_8 => 1,
            Self::V1_9 => 2,
            Self::V1_15 => 3,
        }
    }
    pub const fn dimensions(self) -> [f32; 3] {
        match self {
            Self::V1_8 => [0.6, 1.8, 1.54],
            Self::V1_9 => [0.6, 1.65, 1.54],
            Self::V1_15 => [0.6, 1.5, 1.27],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoPose {
    pub no_swim: bool,
    pub sneak_height: SneakHeight,
}
impl NoPose {
    pub const fn policy(self, running: bool) -> (bool, u8) {
        if running {
            (self.no_swim, self.sneak_height.policy_code())
        } else {
            (false, 0)
        }
    }
    pub const fn cancel_pose(self, running: bool, swimming_pose: bool) -> bool {
        running && self.no_swim && swimming_pose
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_original_three_tags() {
        let state = NoPose::default();
        assert_eq!(state.policy(true), (false, 3));
        for (choice, tag, dimensions) in [
            (SneakHeight::V1_8, "1.8", [0.6, 1.8, 1.54]),
            (SneakHeight::V1_9, "1.9", [0.6, 1.65, 1.54]),
            (SneakHeight::V1_15, "1.15", [0.6, 1.5, 1.27]),
        ] {
            assert_eq!(choice.tag(), tag);
            assert_eq!(choice.dimensions(), dimensions);
            let state = NoPose {
                no_swim: true,
                sneak_height: choice,
            };
            assert_eq!(state.policy(false), (false, 0));
            assert!(state.cancel_pose(true, true));
            assert!(!state.cancel_pose(true, false));
            assert!(!state.cancel_pose(false, true));
        }
    }
}
