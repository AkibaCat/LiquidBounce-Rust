// SPDX-License-Identifier: GPL-3.0-or-later
//! Original Bridging cadence, eligibility and per-bridge automatic selection.
use super::bridge::Bridge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bridging {
    Auto,
    Ninja,
    Godbridge,
    Moonwalk,
    Breezily,
    OneStack,
    SwiftSneak,
    Telly,
}
impl Bridging {
    pub const ALL: [Self; 8] = [
        Self::Auto,
        Self::Ninja,
        Self::Godbridge,
        Self::Moonwalk,
        Self::Breezily,
        Self::OneStack,
        Self::SwiftSneak,
        Self::Telly,
    ];
    pub fn x_rot(self) -> f32 {
        match self {
            Self::Godbridge | Self::Telly => 75.0,
            Self::Moonwalk | Self::Breezily => 79.0,
            _ => 80.0,
        }
    }
    pub fn y_rot_offset(self) -> f32 {
        if matches!(self, Self::Godbridge | Self::SwiftSneak) {
            45.0
        } else {
            0.0
        }
    }
    pub fn clicks_past_edge(self) -> bool {
        matches!(
            self,
            Self::Godbridge | Self::Moonwalk | Self::Breezily | Self::Telly
        )
    }
    pub fn sneaks_throughout(self) -> bool {
        self == Self::SwiftSneak
    }
    pub fn hops_on_reset(self) -> bool {
        self == Self::Godbridge
    }
    pub fn resets(self, block: i32) -> bool {
        match self {
            Self::Godbridge => block > 0 && matches!(block % 15, 0 | 7),
            Self::Moonwalk | Self::Breezily => block > 0 && block % 12 == 0,
            _ => false,
        }
    }
    pub fn tap(self, block: i32, along: f64, turn: i32) -> i32 {
        if !(0.35..=0.6).contains(&along) {
            return 0;
        }
        match self {
            Self::Moonwalk => {
                if turn == 0 {
                    1
                } else {
                    turn
                }
            }
            Self::Breezily => {
                if block % 2 == 0 {
                    -1
                } else {
                    1
                }
            }
            _ => 0,
        }
    }
    pub fn can_cross(self, bridge: &Bridge) -> bool {
        match self {
            Self::Godbridge | Self::Moonwalk | Self::Breezily => {
                bridge.is_straight && !bridge.rises
            }
            Self::Telly => bridge.is_straight && !bridge.rises && bridge.length() >= 10,
            Self::OneStack => bridge.rises,
            _ => true,
        }
    }
    pub fn pick(self, bridge: &Bridge, risky: bool, sneaking_speed: f64, can_sprint: bool) -> Self {
        if self != Self::Auto {
            return if self.can_cross(bridge) {
                self
            } else {
                Self::Ninja
            };
        }
        if bridge.rises {
            Self::OneStack
        } else if sneaking_speed >= 0.7 {
            Self::SwiftSneak
        } else if !risky || !bridge.is_straight || bridge.length() <= 3 {
            Self::Ninja
        } else if can_sprint && Self::Telly.can_cross(bridge) {
            Self::Telly
        } else if bridge.length() >= 15 {
            Self::Godbridge
        } else if bridge.turn_after != 0 {
            Self::Moonwalk
        } else {
            Self::Breezily
        }
    }
}
