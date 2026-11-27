// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleEasyPearl state transitions, pinned LiquidBounce ae0eb396.
use crate::{auto_bow_math::V, rotations::Rotation};
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub aim_off_threshold: f32,
    pub reachable_check: bool,
    pub show_distance: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            aim_off_threshold: 2.,
            reachable_check: true,
            show_distance: true,
        }
    }
}
#[derive(Default, Debug)]
pub struct EasyPearl {
    pub settings: Settings,
    pub target_position: Option<V>,
    pub current_target_rotation: Option<Rotation>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Interaction {
    pub cancel: bool,
    pub warning: bool,
}
impl EasyPearl {
    pub fn rotation_done(&self, solved: Option<Rotation>, direction_angle: f32) -> bool {
        solved.is_none() || direction_angle <= self.settings.aim_off_threshold
    }
    pub fn interact(
        &mut self,
        holding: bool,
        use_down: bool,
        hit: V,
        miss: bool,
        solved: Option<Rotation>,
        angle: f32,
    ) -> Interaction {
        if !holding || !use_down {
            return Interaction::default();
        }
        if self.settings.reachable_check && solved.is_none() && !miss {
            self.target_position = None;
            return Interaction {
                cancel: true,
                warning: true,
            };
        }
        self.target_position = Some(hit);
        if self.rotation_done(solved, angle) {
            self.target_position = None;
            Interaction::default()
        } else {
            Interaction {
                cancel: true,
                warning: false,
            }
        }
    }
    /// The source's Elvis return does not assign null when the target/solve is absent.
    pub fn tick(
        &mut self,
        solved: Option<Rotation>,
        done: bool,
        slot: Option<i32>,
    ) -> Option<(i32, Rotation)> {
        self.target_position?;
        let rotation = solved?;
        self.current_target_rotation = Some(rotation);
        if !done {
            return None;
        }
        Some((slot?, rotation))
    }
    /// Called after real item use, preserving reentrant PlayerInteractItemEvent order.
    pub fn thrown(&mut self) {
        self.target_position = None;
        self.current_target_rotation = None;
    }
    pub fn disabled(&mut self) {
        self.current_target_rotation = None;
    }
}
pub fn target_color(reachable: bool, alpha: u8) -> u32 {
    (u32::from(alpha) << 24) | if reachable { 0x20c206 } else { 0xd70909 }
}
