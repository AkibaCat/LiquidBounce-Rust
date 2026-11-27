// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAimbot at LiquidBounce ae0eb396. Its interpolation endpoints are not
//! RotationManager requests; each render reads Timer and applies a local look.
use crate::{
    auto_bow::Priority,
    combat_rotations::Interpolation,
    no_rotate_set::{RotationsSettings, SmoothMode as Basic},
    rotations::Rotation,
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Smooth {
    #[default]
    Interpolation,
    Sigmoid,
    Linear,
}
impl Smooth {
    pub const ALL: [Self; 3] = [Self::Interpolation, Self::Sigmoid, Self::Linear];
    pub fn tag(self) -> &'static str {
        match self {
            Self::Interpolation => "Interpolation",
            Self::Sigmoid => "Sigmoid",
            Self::Linear => "Linear",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub range: f32,
    pub wall_range: f32,
    pub priority: Vec<Priority>,
    pub fov: f32,
    pub hurt_time: i32,
    pub lazy: bool,
    pub crystals: bool,
    pub crystals_only: bool,
    pub crystals_using: bool,
    pub requires: [bool; 6],
    pub axis: [bool; 2],
    pub ignore: [bool; 2],
    pub smooth: Smooth,
    pub interpolation: Interpolation,
    pub basic: RotationsSettings,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            range: 4.2,
            wall_range: 0.,
            priority: vec![Priority::Type, Priority::Direction],
            fov: 180.,
            hurt_time: 10,
            lazy: false,
            crystals: false,
            crystals_only: false,
            crystals_using: false,
            requires: [false; 6],
            axis: [true; 2],
            ignore: [false; 2],
            smooth: Smooth::Interpolation,
            interpolation: Default::default(),
            basic: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Aimbot {
    pub settings: Settings,
    pub target: Option<i32>,
    pub target_rotation: Option<Rotation>,
    pub player_rotation: Option<Rotation>,
}
impl Aimbot {
    pub fn requirements_met(&self, screen: bool, requirements: [bool; 6]) -> bool {
        !screen
            && self
                .settings
                .requires
                .iter()
                .zip(requirements)
                .all(|(&selected, met)| !selected || met)
    }
    pub fn clear(&mut self) {
        self.target = None;
        self.target_rotation = None;
    }
    pub fn prepare(&mut self, player: Rotation, met: bool) {
        self.player_rotation = Some(player);
        if !met {
            self.clear();
        }
    }
    pub fn aim(
        &mut self,
        rotation: Option<Rotation>,
        previous_target: Option<Rotation>,
        random: &mut impl FnMut() -> f32,
    ) -> Result<(), &'static str> {
        let Some(target) = rotation else {
            self.target_rotation = None;
            return Ok(());
        };
        let from = self.player_rotation.unwrap_or(Rotation::ZERO);
        self.target_rotation = Some(match self.settings.smooth {
            Smooth::Interpolation => {
                self.settings
                    .interpolation
                    .process(from, target, previous_target, random)?
            }
            Smooth::Sigmoid => {
                self.settings
                    .basic
                    .process(Basic::Sigmoid, from, target, None, from, random)?
            }
            Smooth::Linear => {
                self.settings
                    .basic
                    .process(Basic::Linear, from, target, None, from, random)?
            }
        });
        Ok(())
    }
    pub fn mouse(&mut self, x: f64, y: f64) {
        self.player_rotation = self.player_rotation.map(|r| r.mouse_turn(x, y));
        self.target_rotation = self.target_rotation.map(|r| r.mouse_turn(x, y));
    }
    pub fn render(
        &self,
        partial: f32,
        timer: f32,
        screen: bool,
        container: bool,
    ) -> Option<Rotation> {
        if !self.settings.ignore[0] && screen || !self.settings.ignore[1] && container {
            return None;
        }
        let player = self.player_rotation?;
        let target = self.target_rotation?;
        let r = player.interpolate_to(target, timer * partial);
        Some(Rotation::new(
            if self.settings.axis[0] {
                r.yaw
            } else {
                player.yaw
            },
            if self.settings.axis[1] {
                r.pitch
            } else {
                player.pitch
            },
        ))
    }
}
