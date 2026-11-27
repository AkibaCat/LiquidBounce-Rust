// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleAirJump at ae0eb396.
// Copyright (c) 2015-2026 CCBlueX.

//! Source state transitions; allowJump is also NoJumpDelay's ownership input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AirJumpMode {
    #[default]
    JumpFreely,
    DoubleJump,
    GhostBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AirJump {
    pub mode: AirJumpMode,
    double_jump: bool,
}

impl Default for AirJump {
    fn default() -> Self {
        Self {
            mode: AirJumpMode::JumpFreely,
            double_jump: true,
        }
    }
}

impl AirJump {
    /// The original field has no enabled/disabled/world-change reset.
    pub fn allow_jump(self, running: bool) -> bool {
        running
            && (self.mode == AirJumpMode::JumpFreely
                || self.mode == AirJumpMode::DoubleJump && self.double_jump)
    }

    pub fn game_tick(&mut self, running: bool, on_ground: bool) {
        if running && on_ground {
            self.double_jump = true;
        }
    }

    /// Every real local PlayerJumpEvent consumes it in air, including one
    /// triggered by another module or a zero-power jumpFromGround call.
    pub fn player_jump(&mut self, running: bool, on_ground: bool) {
        if running && self.double_jump && !on_ground {
            self.double_jump = false;
        }
    }

    /// BlockShapeEvent occurs inside the block-collision iterator; direct
    /// outline, placement and interaction shape queries remain independent.
    pub fn ghost_block(
        self,
        running: bool,
        physical_jump: bool,
        block_y: i32,
        player_block_y: i32,
    ) -> bool {
        running && self.mode == AirJumpMode::GhostBlock && physical_jump && block_y < player_block_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn double_jump_is_consumed_by_air_event_and_rearmed_only_by_running_ground_tick() {
        let mut jump = AirJump {
            mode: AirJumpMode::DoubleJump,
            ..AirJump::default()
        };
        assert!(jump.allow_jump(true));
        jump.player_jump(true, true);
        assert!(jump.allow_jump(true));
        jump.player_jump(true, false);
        assert!(!jump.allow_jump(true));
        jump.game_tick(false, true);
        assert!(!jump.allow_jump(true));
        jump.game_tick(true, false);
        assert!(!jump.allow_jump(true));
        jump.game_tick(true, true);
        assert!(jump.allow_jump(true));
    }
    #[test]
    fn other_modes_still_consume_state_and_ghost_requires_physical_key() {
        let mut jump = AirJump::default();
        jump.player_jump(true, false);
        assert!(jump.allow_jump(true));
        jump.mode = AirJumpMode::DoubleJump;
        assert!(!jump.allow_jump(true));
        jump.mode = AirJumpMode::GhostBlock;
        assert!(!jump.allow_jump(true));
        assert!(jump.ghost_block(true, true, -3, -2));
        assert!(!jump.ghost_block(true, true, -2, -2));
        assert!(!jump.ghost_block(true, false, -3, -2));
        assert!(!jump.ghost_block(false, true, -3, -2));
        jump.game_tick(true, true);
        jump.mode = AirJumpMode::DoubleJump;
        assert!(jump.allow_jump(true));
    }
}
