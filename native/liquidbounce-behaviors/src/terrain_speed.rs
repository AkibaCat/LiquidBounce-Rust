// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete TerrainSpeed groups and handlers, LiquidBounce ae0eb396.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClimbMode {
    #[default]
    Motion,
    Clip,
}
impl ClimbMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Motion => "Motion",
            Self::Clip => "Clip",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FastClimb {
    pub enabled: bool,
    pub mode: ClimbMode,
    pub motion: f32,
}
impl Default for FastClimb {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ClimbMode::Motion,
            motion: 0.2872,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IceSpeed {
    pub enabled: bool,
    pub slipperiness: f32,
    pub motion_enabled: bool,
    pub horizontal_motion: f32,
}
impl Default for IceSpeed {
    fn default() -> Self {
        Self {
            enabled: true,
            slipperiness: 0.6,
            motion_enabled: false,
            horizontal_motion: 0.5,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterSpeed {
    pub enabled: bool,
    pub auto_swim: bool,
    pub base_horizontal: f32,
    pub base_vertical: f32,
    pub sprint_enabled: bool,
    pub sprint_horizontal: f32,
    pub sprint_vertical: f32,
}
impl Default for WaterSpeed {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_swim: true,
            base_horizontal: 0.44,
            base_vertical: 0.44,
            sprint_enabled: true,
            sprint_horizontal: 1.0,
            sprint_vertical: 1.0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerrainSpeed {
    pub fast_climb: FastClimb,
    pub ice_speed: IceSpeed,
    pub water_speed: WaterSpeed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
}
pub trait ClimbWorld {
    type Error;
    fn horizontal_collision(&self) -> bool;
    fn on_climbable(&mut self) -> Result<bool, Self::Error>;
    fn forward_key(&self) -> bool;
    fn position(&self) -> [f64; 3];
    fn block_position(&self) -> [i32; 3];
    /// Class identity LadderBlock or VineBlock, rather than climbable tags.
    fn ladder_or_vine(&mut self, position: [i32; 3]) -> Result<bool, Self::Error>;
    fn direction(&self) -> Direction;
    fn abs_snap_to(&mut self, position: [f64; 3]) -> Result<(), Self::Error>;
}
impl FastClimb {
    /// Original PlayerMoveEvent handler has no MoverType or on-ground guard.
    pub fn move_event<W: ClimbWorld>(
        self,
        running: bool,
        world: &mut W,
        movement: &mut [f64; 3],
    ) -> Result<(), W::Error> {
        if !running || !self.enabled {
            return Ok(());
        }
        match self.mode {
            ClimbMode::Motion => {
                if world.horizontal_collision() && world.on_climbable()? {
                    movement[1] = f64::from(self.motion);
                }
            }
            ClimbMode::Clip => {
                if world.on_climbable()? && world.forward_key() {
                    let start = world.position();
                    let mut block = world.block_position();
                    for y in 1..=8 {
                        block[1] = block[1].wrapping_add(1);
                        if world.ladder_or_vine(block)? {
                            world.abs_snap_to([start[0], start[1] + f64::from(y), start[2]])?;
                        } else {
                            let (x, z) = match world.direction() {
                                Direction::North => (0., -1.),
                                Direction::South => (0., 1.),
                                Direction::West => (-1., 0.),
                                Direction::East => (1., 0.),
                                Direction::Up | Direction::Down => break,
                            };
                            world.abs_snap_to([
                                start[0] + x,
                                start[1] + f64::from(y) + 1.,
                                start[2] + z,
                            ])?;
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
impl IceSpeed {
    /// The event is global: its subject need not be the local player.
    pub fn block_friction(
        self,
        running: bool,
        block: &str,
        friction: f32,
        local_velocity: &mut [f64; 3],
    ) -> f32 {
        if running
            && self.enabled
            && matches!(
                block,
                "minecraft:ice"
                    | "minecraft:blue_ice"
                    | "minecraft:frosted_ice"
                    | "minecraft:packed_ice"
            )
        {
            if self.motion_enabled {
                let motion = f64::from(self.horizontal_motion);
                local_velocity[0] *= motion;
                local_velocity[1] *= 1.0;
                local_velocity[2] *= motion;
            }
            self.slipperiness
        } else {
            friction
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct WaterContext {
    pub in_water: bool,
    pub physical_sprint: bool,
    pub physical_jump: bool,
    pub physical_sneak: bool,
    pub moving: bool,
    /// Untransformed directional flags: forward=1, backward=2, left=4, right=8.
    pub input: u8,
    pub yaw: f32,
}
impl WaterSpeed {
    pub fn movement_input(
        self,
        running: bool,
        in_water: bool,
        physical_sneak: bool,
        jump: &mut bool,
    ) {
        if running && self.enabled && self.auto_swim && in_water && !physical_sneak {
            *jump = true;
        }
    }
    /// PlayerTickEvent uses physical keys, independently of event.jump/shift.
    pub fn player_tick(
        self,
        running: bool,
        context: WaterContext,
        velocity: &mut [f64; 3],
        java_sin_cos: impl FnOnce(f64) -> [f64; 2],
    ) {
        if !running || !self.enabled || !context.in_water {
            return;
        }
        let sprint = context.physical_sprint && self.sprint_enabled;
        let horizontal = f64::from(if sprint {
            self.sprint_horizontal
        } else {
            self.base_horizontal
        });
        let vertical = f64::from(if sprint {
            self.sprint_vertical
        } else {
            self.base_vertical
        });
        if context.moving {
            if (context.input & 1 != 0) == (context.input & 2 != 0)
                && (context.input & 4 != 0) == (context.input & 8 != 0)
            {
                velocity[0] = 0.;
                velocity[2] = 0.;
            } else {
                let angle = f64::from(crate::strafe::movement_direction(
                    context.yaw,
                    context.input,
                ))
                .to_radians();
                let [sin, cos] = java_sin_cos(angle);
                // withStrafe's default strength still multiplies the previous
                // components by zero, preserving its IEEE exceptional behavior.
                velocity[0] = velocity[0] * 0.0 - sin * (horizontal * 1.0);
                velocity[2] = velocity[2] * 0.0 + cos * (horizontal * 1.0);
            }
        }
        if context.physical_jump {
            velocity[1] = vertical;
        } else if context.physical_sneak {
            velocity[1] = -vertical;
        }
    }
}
