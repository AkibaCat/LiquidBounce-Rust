// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent common walking control, commit 0abf69ac. Native binding and
//! module registration are separate from these original control source units.
pub mod block_actions;
pub mod ability_capture;
pub mod break_times;
pub mod block_sight;
pub mod block_targets;
pub mod body;
pub mod bridge;
pub mod bridge_host;
pub mod bridging;
pub mod climbing;
pub mod crossing;
pub mod floor_prediction;
pub mod follower;
pub mod hops;
pub mod leaping;
pub mod overshoot;
pub mod path_points;
pub mod planning;
pub mod prediction;
pub mod setbacks;
pub mod search_factory;
pub mod shortcut;
pub mod step_check;
pub mod step_progress;
pub mod stride;
pub mod stuck_spots;
pub mod swimming;
pub mod telly;
pub mod turning;
pub mod view_rotation;

use super::{
    geometry::Vec3,
    input::Input,
    walk_jump::{vector_yaw, wrap_degrees},
};
pub const PROGRESS: f64 = 0.1;
pub const REACH_HORIZONTAL: f64 = 0.35;
pub const SHORTCUT_STEPS: usize = 8;

pub(crate) fn horizontal(a: Vec3, b: Vec3) -> f64 {
    let x = a.x - b.x;
    let z = a.z - b.z;
    (x * x + z * z).sqrt()
}
pub(crate) fn subtract(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
pub fn directional_input(direction: Vec3, frame_y_rot: f32) -> Input {
    if direction.x * direction.x + direction.z * direction.z < 1.0e-6 {
        return Input::EMPTY;
    }
    let angle = wrap_degrees(vector_yaw(direction.x, direction.z) - wrap_degrees(frame_y_rot));
    Input {
        forward: angle.abs() < 70.0,
        backward: angle.abs() > 110.0,
        left: angle < -20.0 && angle > -160.0,
        right: angle > 20.0 && angle < 160.0,
        ..Input::EMPTY
    }
}
pub fn center_input(position: Vec3, feet: Vec3, frame_y_rot: f32) -> Input {
    if horizontal(position, feet) > 0.1 {
        directional_input(subtract(feet, position), frame_y_rot)
    } else {
        Input::EMPTY
    }
}
pub fn directional_input_for_y_rot(
    forward: i32,
    strafe: i32,
    frame_y_rot: f32,
    silent_y_rot: Option<f32>,
) -> Input {
    let axis = |f: i32, s: i32| Input {
        forward: f > 0,
        backward: f < 0,
        left: s > 0,
        right: s < 0,
        ..Input::EMPTY
    };
    let Some(silent) = silent_y_rot else {
        return axis(forward, strafe);
    };
    let turn = (frame_y_rot - silent) * 0.017453292_f32;
    let cosine = super::simulation::sine_table::cos(turn);
    let sine = super::simulation::sine_table::sin(turn);
    let round = |value: f32| (f64::from(value) + 0.5).floor() as i32;
    for pressed_forward in -1..=1 {
        for pressed_strafe in -1..=1 {
            let turned_strafe =
                round(pressed_strafe as f32 * cosine - pressed_forward as f32 * sine);
            let turned_forward =
                round(pressed_forward as f32 * cosine + pressed_strafe as f32 * sine);
            if turned_strafe == strafe && turned_forward == forward {
                return axis(pressed_forward, pressed_strafe);
            }
        }
    }
    axis(forward, strafe)
}
pub(crate) fn distance_segment(point: Vec3, from: Vec3, to: Vec3) -> f64 {
    let along = subtract(to, from);
    let length = along.x * along.x + along.y * along.y + along.z * along.z;
    if length.abs() < f64::from(1.0e-5_f32) {
        return point.distance(from);
    }
    let offset = subtract(point, from);
    let t = (along.x * offset.x + along.y * offset.y + along.z * offset.z) / length;
    assert!(t.is_finite(), "cannot project onto walk segment");
    point.distance(from.fma(t.clamp(0.0, 1.0), along))
}
