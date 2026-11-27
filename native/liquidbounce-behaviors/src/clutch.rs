// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleClutch and Minecraft-Agent FallingPlayer's event/state decisions.
//! World queries and body replay are supplied by the native game thread.

pub const MAX_FALL_TICKS: usize = 60;
pub type Position = [f64; 3];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub min_fall_distance: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            min_fall_distance: 5.,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Facts {
    pub passenger: bool,
    pub fall_flying: bool,
    pub flying: bool,
    pub scaffold_enabled: bool,
    pub block_count: i32,
    pub on_ground: bool,
    pub steered: bool,
    pub horizontal_velocity_squared: f64,
    pub in_reach: bool,
}
impl Facts {
    pub fn disallowed(self) -> bool {
        self.passenger
            || self.fall_flying
            || self.flying
            || self.scaffold_enabled
            || self.block_count <= 0
    }
    pub fn grounded_without_fall(self) -> bool {
        self.on_ground && (self.steered || self.horizontal_velocity_squared == 0.)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Clutch {
    pub settings: Settings,
    path: Option<Vec<Position>>,
    guarding: bool,
}
impl Clutch {
    pub fn fall(&self, running: bool) -> Option<&[Position]> {
        self.path.as_deref().filter(|_| running)
    }
    pub fn guarding(&self, running: bool) -> bool {
        self.guarding && running
    }
    /// Whether source release() calls Scaffold.reset().
    pub fn disabled(&mut self, scaffold_enabled: bool) -> bool {
        self.guarding = false;
        self.path.take().is_some() && !scaffold_enabled
    }
    /// Source WorldChange clears state without release().
    pub fn world_change(&mut self) {
        self.guarding = false;
        self.path = None;
    }
    /// FIRST_PRIORITY RotationUpdate. Native closures perform real queries.
    pub fn rotation_update(
        &mut self,
        running: bool,
        facts: Facts,
        find_collision: impl FnOnce(usize) -> bool,
        predict_fall: impl FnOnce(f32, usize) -> Option<Vec<Position>>,
    ) -> bool {
        if !running {
            return false;
        }
        let was_falling = self.path.is_some();
        self.guarding = !facts.disallowed()
            && (facts.on_ground || facts.in_reach || find_collision(MAX_FALL_TICKS));
        self.path = if self.guarding && !facts.grounded_without_fall() {
            predict_fall(self.settings.min_fall_distance, MAX_FALL_TICKS)
        } else {
            None
        };
        was_falling && self.path.is_none() && !facts.scaffold_enabled
    }
}

/// FallingPlayer.fallPath observes each post-movement snapshot, tests strictly
/// greater fallDistance before grounded safety, and preserves unsteered momentum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FallingSnapshot {
    pub position: Position,
    pub fall_distance: f64,
    pub on_ground: bool,
    pub horizontal_velocity_squared: f64,
}
pub fn fall_path(
    distance: f32,
    steered: bool,
    snapshots: impl IntoIterator<Item = FallingSnapshot>,
) -> Option<Vec<Position>> {
    let mut positions = Vec::new();
    for snapshot in snapshots.into_iter().take(MAX_FALL_TICKS) {
        positions.push(snapshot.position);
        if snapshot.fall_distance > f64::from(distance) {
            return Some(positions);
        }
        if snapshot.on_ground && (steered || snapshot.horizontal_velocity_squared == 0.) {
            return None;
        }
    }
    None
}
