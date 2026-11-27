// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete navigation/glide/RoofBounce.kt.
use super::super::{
    geometry::{BlockPos, Vec3, java_min},
    input::Input,
    terrain::{Terrain, TerrainLevel, TerrainState},
    walk_jump::{vector_yaw, wrap_degrees},
};
use super::abilities::GlideAbilities;
pub trait RoofPlayer {
    fn position(&self) -> Vec3;
    fn on_ground(&self) -> bool;
    fn is_fall_flying(&self) -> bool;
}
pub trait RoofBody {
    type Player: RoofPlayer;
    type State: TerrainState;
    type Level: TerrainLevel<Self::State>;
    fn player(&self) -> &Self::Player;
    fn recasts_elytra(&self) -> bool;
    fn live_level(&self) -> Self::Level;
    fn press(&mut self, input: Input, exact: bool);
    fn look_steer(&mut self, y_rot: f32, x_rot: f32);
}
pub trait RoofReport {
    fn blocks_empty(&mut self);
    fn following(&mut self, points: &[Vec3], progress: &RoofBounce);
}
struct Bounce {
    goal: Vec3,
    floor: f64,
    line_end: Vec3,
    ticks: i32,
}
#[derive(Default)]
pub struct RoofBounce {
    bounce: Option<Bounce>,
}
impl RoofBounce {
    pub fn is_active(&self) -> bool {
        self.bounce.is_some()
    }
    pub fn target(&self) -> Option<Vec3> {
        self.bounce.as_ref().map(|b| b.line_end)
    }
    pub fn navigate(
        &mut self,
        body: &mut impl RoofBody,
        report: &mut impl RoofReport,
        goal: Vec3,
        abilities: Option<GlideAbilities>,
        frame_y_rot: f32,
    ) -> bool {
        let player = body.player();
        let position = player.position();
        let active = self.bounce.take();
        self.bounce = if abilities.is_none() || !body.recasts_elytra() {
            None
        } else if let Some(mut active) = active {
            if horizontal_distance(position, goal) < 12.0 || position.y < active.floor - 0.5 {
                None
            } else {
                let feet = Vec3::new(position.x, active.floor, position.z);
                bounce_line(&mut Terrain::player(body.live_level()), feet, goal, 12.0).map(|end| {
                    active.line_end = end;
                    active
                })
            }
        } else if !body.player().on_ground() || horizontal_distance(position, goal) < 32.0 {
            None
        } else {
            bounce_line(
                &mut Terrain::player(body.live_level()),
                position,
                goal,
                24.0,
            )
            .map(|end| Bounce {
                goal,
                floor: position.y,
                line_end: end,
                ticks: 0,
            })
        };
        let Some(bounce) = &mut self.bounce else {
            return false;
        };
        bounce.ticks = bounce.ticks.wrapping_add(1);
        let direction = directional_input(sub(goal, position), frame_y_rot);
        let jump = player.is_fall_flying() || player.on_ground() || bounce.ticks % 2 == 0;
        body.press(direction.with_keys(jump, false, false), true);
        let points = [position, bounce.line_end];
        let y_rot = vector_yaw(bounce.goal.x - position.x, bounce.goal.z - position.z);
        report.blocks_empty();
        report.following(&points, self);
        body.look_steer(y_rot, 75.0);
        true
    }
    pub fn halt(&mut self) {
        self.bounce = None;
    }
}
pub fn bounce_line<B: TerrainState, L: TerrainLevel<B>>(
    terrain: &mut Terrain<B, L>,
    feet: Vec3,
    goal: Vec3,
    lookahead: f64,
) -> Option<Vec3> {
    let dx = goal.x - feet.x;
    let dz = goal.z - feet.z;
    let length = (dx * dx + dz * dz).sqrt();
    if length < 0.5 {
        return None;
    }
    let ux = dx / length;
    let uz = dz / length;
    let reach = java_min(lookahead, length);
    let y = (feet.y + 1.0e-3).floor() as i32;
    let at = |along| Vec3::new(feet.x + ux * along, feet.y, feet.z + uz * along);
    let mut along = 0.0;
    while along <= reach {
        for side in [-0.4, 0.0, 0.4] {
            let x = (feet.x + ux * along - uz * side).floor() as i32;
            let z = (feet.z + uz * along + ux * side).floor() as i32;
            let pos = BlockPos::new(x, y, z);
            if !terrain.is_known(pos) {
                return if along >= 8.0 { Some(at(along)) } else { None };
            }
            let stand = terrain.stand_height(pos);
            if stand.is_nan()
                || (stand - feet.y).abs() > 1.0e-3
                || !terrain.is_passable(pos.offset(0, 2, 0))
            {
                return None;
            }
        }
        along += 0.5;
    }
    Some(at(reach))
}
pub(crate) fn horizontal_distance(a: Vec3, b: Vec3) -> f64 {
    ((a.x - b.x) * (a.x - b.x) + (a.z - b.z) * (a.z - b.z)).sqrt()
}
pub(crate) fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
pub(crate) fn directional_input(direction: Vec3, frame_y_rot: f32) -> Input {
    if direction.x * direction.x + direction.z * direction.z < 1.0e-6 {
        return Input::EMPTY;
    }
    let angle = wrap_degrees(vector_yaw(direction.x, direction.z) - wrap_degrees(frame_y_rot));
    Input {
        forward: angle.abs() < 70.0,
        backward: angle.abs() > 110.0,
        left: angle > -160.0 && angle < -20.0,
        right: angle > 20.0 && angle < 160.0,
        ..Input::EMPTY
    }
}
