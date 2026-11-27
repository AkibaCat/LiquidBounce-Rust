// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ModuleSprint, MixinClientInput, MixinLocalPlayer
// and EntityExtensions at ae0eb396. Copyright (c) 2015-2026 CCBlueX.

//! Sprint's input, movement-query, network and jump phases. This policy never
//! writes hunger/effects/collision state and does not register a module.
use crate::rotations::{DefaultRotationTarget, DirectionalInput, MovementCorrection, Rotation};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SprintMode {
    #[default]
    Legit,
    Omnidirectional,
    Omnirotational,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ignore {
    pub blindness: bool,
    pub hunger: bool,
    pub collision: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StopOn {
    pub ground: bool,
    pub air: bool,
    pub sneaking: bool,
    pub using_item: bool,
}
impl Default for StopOn {
    fn default() -> Self {
        Self {
            ground: true,
            air: true,
            sneaking: true,
            using_item: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sprint {
    pub mode: SprintMode,
    pub ignore: Ignore,
    pub stop_on: StopOn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Input,
    MovementTick,
    Network,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Context {
    pub on_ground: bool,
    /// The actual `isUsingItem && !USE_EFFECTS.canSprint` result.
    pub slow_due_to_item: bool,
    pub shift_key_down: bool,
    pub player_yaw: f32,
    /// The input's actual moveVector, before LocalPlayer.modifyInput slows it.
    pub sideways: f32,
    pub forward: f32,
    pub current_rotation: Option<Rotation>,
    pub movement_correction: Option<MovementCorrection>,
}

/// `getMovementDirectionOfInput`: four flags retain opposite-key cancellation
/// without reducing the event input to two axes before this phase.
pub fn movement_direction(facing_yaw: f32, input: DirectionalInput) -> f32 {
    let mut yaw = facing_yaw;
    let forward_multiplier = if input.backwards && !input.forwards {
        yaw += 180.0;
        -0.5
    } else if input.forwards && !input.backwards {
        0.5
    } else {
        1.0
    };
    if input.left && !input.right {
        yaw -= 90.0 * forward_multiplier;
    }
    if input.right && !input.left {
        yaw += 90.0 * forward_multiplier;
    }
    yaw
}

impl Sprint {
    pub fn omnidirectional(self, running: bool, scaffold_allows: bool) -> bool {
        running && self.mode == SprintMode::Omnidirectional || scaffold_allows
    }

    pub fn ignore_blindness(self, running: bool) -> bool {
        running && self.ignore.blindness
    }
    pub fn ignore_hunger(self, running: bool) -> bool {
        running && self.ignore.hunger
    }
    pub fn ignore_collision(self, running: bool) -> bool {
        running && self.ignore.collision
    }

    /// Return a query override for ModuleSprint's two MOVEMENT_TICK handlers.
    /// CRITICAL_MODIFICATION runs first; the normal prevention handler can
    /// then clear it. The supplied trig is vanilla Mth of widened float radians.
    pub fn movement_override(
        self,
        running: bool,
        input: DirectionalInput,
        context: Context,
        scaffold_allows: bool,
        mut vanilla_sin_cos: impl FnMut(f64) -> (f32, f32),
    ) -> Option<bool> {
        if !running {
            return None;
        }
        let prevented = if (self.stop_on.using_item && context.slow_due_to_item)
            || (self.stop_on.sneaking && context.shift_key_down)
        {
            true
        } else if let Some(rotation) = context.current_rotation {
            let radians =
                f64::from((context.player_yaw - rotation.yaw) * f32::from_bits(0x3c8e_fa35));
            let (sin, cos) = vanilla_sin_cos(radians);
            let forward = context.forward * cos + context.sideways * sin;
            (if context.on_ground {
                self.stop_on.ground
            } else {
                self.stop_on.air
            }) && !self.omnidirectional(running, scaffold_allows)
                && context.movement_correction == Some(MovementCorrection::Off)
                && (forward <= 1.0e-5 || forward.is_nan())
        } else {
            false
        };
        if prevented {
            Some(false)
        } else if input.is_moving() {
            Some(true)
        } else {
            None
        }
    }

    pub fn event(
        self,
        running: bool,
        source: Source,
        input: DirectionalInput,
        original: bool,
        movement_override: Option<bool>,
    ) -> bool {
        if !running {
            return original;
        }
        match source {
            Source::Input if input.is_moving() => true,
            Source::MovementTick => movement_override.unwrap_or(original),
            Source::Input | Source::Network => original,
        }
    }

    pub fn has_forward_impulse(
        self,
        running: bool,
        scaffold_allows: bool,
        original: bool,
        axes: [f32; 2],
    ) -> bool {
        if self.omnidirectional(running, scaffold_allows) {
            axes[0].abs() > 1.0e-5f32 || axes[1].abs() > 1.0e-5f32
        } else {
            original
        }
    }

    pub fn can_start_forward_impulse(
        self,
        running: bool,
        scaffold_allows: bool,
        original: bool,
        axes: [f32; 2],
        under_water: bool,
    ) -> bool {
        if !self.omnidirectional(running, scaffold_allows) {
            return original;
        }
        if under_water {
            axes[0].abs() > 1.0e-5f32 || axes[1].abs() > 1.0e-5f32
        } else {
            f64::from(axes[0].abs()) >= 0.8 || f64::from(axes[1].abs()) >= 0.8
        }
    }

    pub fn jump_yaw(
        self,
        running: bool,
        original: f32,
        player_yaw: f32,
        input: DirectionalInput,
    ) -> f32 {
        if running && self.mode == SprintMode::Omnidirectional {
            movement_direction(player_yaw, input)
        } else {
            original
        }
    }

    /// Submit this after RotationManager's FIRST_PRIORITY GameTick update.
    /// RotationsValueGroup is deliberately absent from Sprint's setting tree.
    pub fn rotation_target(
        self,
        running: bool,
        player: Rotation,
        input: DirectionalInput,
    ) -> Option<DefaultRotationTarget> {
        (running && self.mode == SprintMode::Omnirotational).then(|| {
            DefaultRotationTarget::sprint(Rotation::new(
                movement_direction(player.yaw, input),
                player.pitch,
            ))
        })
    }
}
