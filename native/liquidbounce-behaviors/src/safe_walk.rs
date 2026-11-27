// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSafeWalk ae0eb396. Geometry/simulation queries are synchronous inputs
//! from the game adapter; this unit retains all three modes and OnEdge state.
use crate::random::{ThreadRandom, with_thread_random};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    None,
    #[default]
    Safe,
    OnEdge,
}
impl Mode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Safe => "Safe",
            Self::OnEdge => "OnEdge",
        }
    }
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "None" => Some(Self::None),
            "Safe" => Some(Self::Safe),
            "OnEdge" => Some(Self::OnEdge),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeMode {
    #[default]
    Stop,
    Invert,
    Center,
}
impl EdgeMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::Invert => "Invert",
            Self::Center => "Center",
        }
    }
    pub fn parse(name: &str) -> Option<Self> {
        [Self::Stop, Self::Invert, Self::Center]
            .into_iter()
            .find(|m| crate::navigation::java_case::equals_ignore_case(m.name(), name))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub distance: [f32; 2],
    pub keep: [i32; 2],
    pub edge_mode: EdgeMode,
    pub sneak: [i32; 2],
    pub jump: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Safe,
            distance: [0.1, 0.15],
            keep: [1, 2],
            edge_mode: EdgeMode::Stop,
            sneak: [0, 0],
            jump: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Direction {
    pub forwards: bool,
    pub backwards: bool,
    pub left: bool,
    pub right: bool,
}
impl Direction {
    pub const NONE: Self = Self {
        forwards: false,
        backwards: false,
        left: false,
        right: false,
    };
    pub const fn invert(self) -> Self {
        Self {
            forwards: self.backwards,
            backwards: self.forwards,
            left: self.right,
            right: self.left,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    pub direction: Direction,
    pub jump: bool,
    pub sneak: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub on_ground: bool,
    pub horizontal_speed: f64,
    pub position: [f64; 3],
    pub yaw: f32,
    pub block_bottom_center: [f64; 3],
}

/// These operations are the original isCloseToEdge, PlayerSimulationCache(1),
/// and wouldBeCloseToFallOff calls in their original lazy evaluation order.
pub trait Queries {
    fn close_to_edge(&mut self, direction: Direction, distance: f64) -> bool;
    fn next_simulated_position(&mut self) -> [f64; 3];
    fn close_to_fall_off(&mut self, position: [f64; 3]) -> bool;
    fn debug_input(&mut self, _direction: Direction) {}
    fn debug_center(&mut self, _position: [f64; 3], _current: f64, _next: f64) {}
    fn debug_overwrite_ticks(&mut self, _ticks: i32) {}
}
pub trait Random {
    fn distance(&mut self, range: [f32; 2]) -> Result<f32, &'static str>;
    fn ticks(&mut self, range: [i32; 2]) -> Result<i32, &'static str>;
}
impl Random for ThreadRandom {
    fn distance(&mut self, [start, end]: [f32; 2]) -> Result<f32, &'static str> {
        if start >= end {
            return Ok(start);
        }
        let bound = end.next_up();
        if !start.is_finite() || !bound.is_finite() {
            return Err("random float range must be finite and nonempty");
        }
        Ok(self.next_f32_between(start, bound))
    }
    fn ticks(&mut self, [start, end]: [i32; 2]) -> Result<i32, &'static str> {
        if start > end {
            return Err("random range must be nonempty");
        }
        Ok(self.next_i32_inclusive(start, end))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SafeWalk {
    pub settings: Settings,
    pub center: Option<[f64; 3]>,
    pub overwrite_ticks: i32,
    /// OnEdge.disable intentionally does not reset this original field.
    pub sneak_ticks: i32,
    pub edge_distance: f32,
}
impl Default for SafeWalk {
    fn default() -> Self {
        with_thread_random(|r| Self::new(r).expect("default SafeWalk range"))
    }
}
impl SafeWalk {
    pub fn new(random: &mut impl Random) -> Result<Self, &'static str> {
        let settings = Settings::default();
        let edge_distance = random.distance(settings.distance)?;
        Ok(Self {
            settings,
            center: None,
            overwrite_ticks: 0,
            sneak_ticks: 0,
            edge_distance,
        })
    }
    pub fn safe_walk(&self, running: bool, original: bool) -> bool {
        if running && self.settings.mode == Mode::Safe {
            true
        } else {
            original
        }
    }
    pub fn refresh_distance(&mut self, random: &mut impl Random) -> Result<(), &'static str> {
        self.edge_distance = random.distance(self.settings.distance)?;
        Ok(())
    }
    pub fn set_distance(
        &mut self,
        range: [f32; 2],
        random: &mut impl Random,
    ) -> Result<(), &'static str> {
        let old = self.settings.distance;
        let equal = (old[0] > old[1] && range[0] > range[1]) || old == range;
        if !equal {
            self.settings.distance = range;
            self.refresh_distance(random)?;
        }
        Ok(())
    }
    /// Called only when the OnEdge mode is actually disabled by its lifecycle.
    pub fn disable_on_edge(&mut self, random: &mut impl Random) -> Result<(), &'static str> {
        self.center = None;
        self.overwrite_ticks = 0;
        self.refresh_distance(random)
    }
    pub fn set_mode(
        &mut self,
        mode: Mode,
        running: bool,
        random: &mut impl Random,
    ) -> Result<(), &'static str> {
        if self.settings.mode == mode {
            return Ok(());
        }
        if running && self.settings.mode == Mode::OnEdge {
            self.disable_on_edge(random)?;
        }
        self.settings.mode = mode;
        Ok(())
    }
    /// OBJECTION_AGAINST_EVERYTHING (-100), after shared simulation MODEL_STATE.
    pub fn movement(
        &mut self,
        running: bool,
        snapshot: Snapshot,
        input: &mut Input,
        queries: &mut impl Queries,
        random: &mut impl Random,
    ) -> Result<(), &'static str> {
        if !running || self.settings.mode != Mode::OnEdge {
            return Ok(());
        }
        if snapshot.on_ground && !input.sneak {
            let distance = java_min(snapshot.horizontal_speed, f64::from(self.edge_distance));
            if queries.close_to_edge(input.direction, distance) {
                queries.debug_input(input.direction);
                if let Some(center) = self.center {
                    let next = queries.next_simulated_position();
                    let current_distance = horizontal_distance_sqr(center, snapshot.position);
                    let next_distance = horizontal_distance_sqr(center, next);
                    queries.debug_center(center, current_distance, next_distance);
                    if next_distance <= current_distance {
                        return Ok(());
                    }
                }
                if self.overwrite_ticks == 0 {
                    self.refresh_distance(random)?;
                    self.overwrite_ticks = random.ticks(self.settings.keep)?;
                }
                if self.sneak_ticks == 0 {
                    self.sneak_ticks = random.ticks(self.settings.sneak)?;
                }
            }
        }
        if self.overwrite_ticks > 0 {
            queries.debug_overwrite_ticks(self.overwrite_ticks);
            self.overwrite_ticks -= 1;
            if self.settings.edge_mode == EdgeMode::Invert {
                input.direction = input.direction.invert();
                input.jump = false;
            } else if self.settings.edge_mode == EdgeMode::Center
                || snapshot.horizontal_speed > 0.05
            {
                let center = self.center.unwrap_or(snapshot.block_bottom_center);
                input.direction = towards(center, snapshot.position, snapshot.yaw);
            } else if self.settings.edge_mode == EdgeMode::Stop {
                input.direction = Direction::NONE;
                input.jump = false;
            }
            if self.settings.jump {
                input.jump = true;
            }
        }
        if self.sneak_ticks > 0 {
            self.sneak_ticks -= 1;
            input.sneak = true;
        }
        if !queries.close_to_fall_off(snapshot.block_bottom_center) {
            self.center = Some(snapshot.block_bottom_center);
        }
        Ok(())
    }
}
fn horizontal_distance_sqr(a: [f64; 3], b: [f64; 3]) -> f64 {
    let x = a[0] - b[0];
    let z = a[2] - b[2];
    x * x + z * z
}
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a == 0.0 && b == 0.0 {
        f64::from_bits(a.to_bits() | b.to_bits())
    } else {
        a.min(b)
    }
}
pub fn towards(center: [f64; 3], position: [f64; 3], yaw: f32) -> Direction {
    // Vec3.yaw casts atan2 to float before multiplication by Mth.RAD_TO_DEG.
    let optimal =
        (-(center[0] - position[0])).atan2(center[2] - position[2]) as f32 * 57.295776_f32;
    let d = crate::rotations::wrap_degrees(optimal - crate::rotations::wrap_degrees(yaw));
    Direction {
        forwards: d > -70.0 && d < 70.0,
        backwards: d < -110.0 || d > 110.0,
        right: d > 20.0 && d < 160.0,
        left: d > -160.0 && d < -20.0,
    }
}
