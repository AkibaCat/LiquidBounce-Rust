// SPDX-License-Identifier: GPL-3.0-or-later
//! LiquidBounce ae0eb396 ModuleAutoWalk. This state machine is not registered
//! until its Minecraft Agent navigator and native body are complete.
use super::{
    geometry::{BlockPos, Vec3},
    input::Input,
    java_case::equals_ignore_case,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Forward,
    Direction,
    Goto,
    Follow,
}
impl Mode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Direction => "Direction",
            Self::Goto => "Goto",
            Self::Follow => "Follow",
        }
    }
}
#[derive(Clone, Debug)]
pub struct FollowPlayer {
    pub is_local_player: bool,
    pub profile_name: String,
    pub position: Vec3,
}
pub trait AutoWalkHost {
    fn position(&self) -> Vec3;
    fn yaw(&self) -> f32;
    fn root_vehicle_position(&self) -> Vec3;
    /// Vec3.directionFromRotation(0f, yaw), with the game's float trig table.
    fn direction_from_rotation(&self, yaw: f32) -> Vec3;
    fn is_standable(&mut self, pos: BlockPos) -> bool;
    fn has_chunk_at(&self, pos: BlockPos) -> bool;
    fn players(&self) -> Vec<FollowPlayer>;
    fn navigate(&mut self, input: &mut Input, goal: Vec3, radius: f64);
    fn halt(&mut self);
    fn reset(&mut self);
    fn arrived(&mut self, destination: BlockPos);
}

#[derive(Clone, Debug)]
pub struct AutoWalk {
    mode: Mode,
    enabled: bool,
    pub destination: BlockPos,
    pub target: String,
    pub follow_distance: f32,
    direction_goal: Option<Vec3>,
    goal_yaw: f32,
    goto_goal_of: Option<BlockPos>,
    goto_goal: Vec3,
    last_seen: Option<Vec3>,
}
impl Default for AutoWalk {
    fn default() -> Self {
        Self {
            mode: Mode::Forward,
            enabled: false,
            destination: BlockPos::default(),
            target: String::new(),
            follow_distance: 3.0,
            direction_goal: None,
            goal_yaw: 0.0,
            goto_goal_of: None,
            goto_goal: Vec3::ZERO,
            last_seen: None,
        }
    }
}
impl AutoWalk {
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    /// `running` includes the caller's real EventListener and in-game gates.
    pub fn select_mode(&mut self, mode: Mode, running: bool, host: &mut impl AutoWalkHost) {
        if self.mode == mode {
            return;
        }
        if running {
            self.disable_active(host);
        }
        self.mode = mode;
    }
    /// ToggleableValueGroup skips child lifecycle while outside the game.
    pub fn set_enabled(&mut self, enabled: bool, in_game: bool, host: &mut impl AutoWalkHost) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        if !enabled && in_game {
            self.disable_active(host);
        }
    }
    pub fn disable_active(&mut self, host: &mut impl AutoWalkHost) {
        match self.mode {
            Mode::Forward => {}
            Mode::Direction => {
                host.reset();
                self.direction_goal = None;
            }
            Mode::Goto => host.reset(),
            Mode::Follow => {
                host.reset();
                self.last_seen = None;
            }
        }
    }
    pub fn go_to(
        &mut self,
        position: Vec3,
        running: bool,
        in_game: bool,
        host: &mut impl AutoWalkHost,
    ) {
        self.destination = BlockPos::containing(position);
        self.select_mode(Mode::Goto, running, host);
        self.set_enabled(true, in_game, host);
    }
    pub fn follow(
        &mut self,
        name: String,
        running: bool,
        in_game: bool,
        host: &mut impl AutoWalkHost,
    ) {
        self.target = name;
        self.select_mode(Mode::Follow, running, host);
        self.set_enabled(true, in_game, host);
    }
    /// Called only when the actual event listener is running, at CRITICAL_MODIFICATION.
    pub fn movement_input(&mut self, input: &mut Input, host: &mut impl AutoWalkHost) {
        let (goal, radius) = match self.mode {
            Mode::Forward => {
                input.forward = true;
                return;
            }
            Mode::Direction => (Some(self.direction_goal(host)), 2.0),
            Mode::Goto => (self.goto_goal(host), 1.0),
            Mode::Follow => {
                if let Some(player) = host.players().into_iter().find(|player| {
                    !player.is_local_player
                        && equals_ignore_case(&player.profile_name, &self.target)
                }) {
                    self.last_seen = Some(player.position);
                }
                (self.last_seen, f64::from(self.follow_distance))
            }
        };
        if let Some(goal) = goal {
            host.navigate(input, goal, radius);
        } else {
            host.halt();
        }
    }
    fn direction_goal(&mut self, host: &impl AutoWalkHost) -> Vec3 {
        let position = host.position();
        let yaw = host.yaw();
        if let Some(goal) = self.direction_goal {
            let dx = position.x - goal.x;
            let dz = position.z - goal.z;
            if dx * dx + dz * dz > 512.0 * 512.0 && wrap_degrees(yaw - self.goal_yaw).abs() <= 20.0
            {
                return goal;
            }
        }
        self.goal_yaw = yaw;
        let goal = position.fma(1024.0, host.direction_from_rotation(yaw));
        self.direction_goal = Some(goal);
        goal
    }
    fn goto_goal(&mut self, host: &mut impl AutoWalkHost) -> Option<Vec3> {
        if self.goto_goal_of != Some(self.destination) {
            let pos = self.destination;
            let feet = if !host.is_standable(pos) && host.is_standable(pos.above()) {
                pos.above()
            } else {
                pos
            };
            self.goto_goal = Vec3::bottom_center(feet);
            self.goto_goal_of = host.has_chunk_at(pos).then_some(pos);
        }
        if host.root_vehicle_position().distance_sqr(self.goto_goal) > 1.0 {
            return Some(self.goto_goal);
        }
        host.arrived(self.destination);
        self.set_enabled(false, true, host);
        None
    }
}
fn wrap_degrees(angle: f32) -> f32 {
    let mut wrapped = angle % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}
