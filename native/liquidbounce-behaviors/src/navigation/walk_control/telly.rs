// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Telly.kt, including simulated landing and predicted-eye aim.
use super::{
    body::{SimulatedInput, WalkBody, WalkPlayer, WalkSimulation},
    bridge::Bridge,
    bridge_host::BridgeHost,
    directional_input,
    step_progress::Movement,
    subtract,
    turning::{LookPurpose, Turning},
    view_rotation::ViewRotation,
};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    input::{Input, MovementKeys},
    simulation::player_motion::MotionPlayer,
    walk::route::Step,
    walk_jump::wrap_degrees,
};

#[derive(Default)]
pub struct Telly {
    pub view: Option<ViewRotation>,
    pub acting: Option<BlockPos>,
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
fn center(pos: BlockPos) -> Vec3 {
    Vec3::new(
        f64::from(pos.x) + 0.5,
        f64::from(pos.y) + 0.5,
        f64::from(pos.z) + 0.5,
    )
}
impl Telly {
    pub fn tick(
        &mut self,
        host: &mut impl BridgeHost,
        keys: &mut MovementKeys,
        frame_y_rot: f32,
        steps: &[Step],
        bridge: &Bridge,
        x_rot: f32,
    ) -> Movement {
        let first = &steps[bridge.start];
        let way = Vec3::new(
            f64::from(first.to.x.wrapping_sub(first.from.x)),
            f64::from(first.to.y.wrapping_sub(first.from.y)),
            f64::from(first.to.z.wrapping_sub(first.from.z)),
        );
        let end = steps[bridge.end - 1].feet;
        let forward = ViewRotation::new(bridge.y_rot, x_rot);
        let back = ViewRotation::new(wrap_degrees(bridge.y_rot + 180.0), x_rot);
        keys.directional_input = directional_input(way, frame_y_rot);
        keys.jump = false;
        keys.sneak = false;
        self.acting = None;
        let pending = (bridge.start..bridge.end).find_map(|i| {
            steps[i]
                .places
                .iter()
                .find(|&&pos| !host.is_placed(pos))
                .copied()
        });
        let Some(pending) = pending else {
            self.turn(host, forward);
            return Movement {
                target: Some(end),
                sprint: true,
                ..Movement::default()
            };
        };
        let edge = dot(center(pending), way) - 0.5;
        let along = dot(host.player().position(), way);
        let is_passing = along > edge + 0.5;
        if host.player().on_ground() {
            if is_passing {
                keys.directional_input = Input::EMPTY;
                self.place(host, keys, pending, way, edge, back);
                return Movement {
                    target: Some(end),
                    ..Movement::default()
                };
            }
            let y_rot = self.turn(host, forward);
            keys.jump = along >= edge - 0.3 && wrap_degrees(y_rot - bridge.y_rot).abs() < 45.0;
            return Movement {
                target: Some(end),
                sprint: true,
                ..Movement::default()
            };
        }
        if !is_passing && Self::lands(host, keys.to_input(false), f64::from(first.from.y)) {
            self.turn(host, forward);
            return Movement {
                target: Some(end),
                sprint: true,
                ..Movement::default()
            };
        }
        self.place(host, keys, pending, way, edge, back);
        Movement {
            target: Some(end),
            ..Movement::default()
        }
    }
    fn place(
        &mut self,
        host: &mut impl BridgeHost,
        keys: &MovementKeys,
        pending: BlockPos,
        way: Vec3,
        edge: f64,
        back: ViewRotation,
    ) {
        let eye = {
            let mut simulated = host.simulate(
                SimulatedInput::new(keys.to_input(false)),
                host.input_y_rot(),
            );
            simulated.tick();
            simulated.player().eye_position()
        };
        let face = subtract(
            center(pending),
            Vec3::new(way.x * 0.5, way.y * 0.5, way.z * 0.5),
        );
        let aim = if dot(eye, way) > edge {
            ViewRotation::from_vector(subtract(face, eye))
        } else {
            back
        };
        self.turn(host, aim);
        host.place(pending, true, aim);
        self.acting = Some(pending);
    }
    fn turn(&mut self, host: &mut impl BridgeHost, goal: ViewRotation) -> f32 {
        self.view = Some(goal);
        let turned = Turning::eased(Turning::FLICK_TURN, Turning::FLICK_TURN * 0.7, 0.6, 10.0)
            .turn(host.sent_rotation(), goal, LookPurpose::Movement);
        host.look(turned, LookPurpose::Steer);
        turned.y_rot
    }
    fn lands(host: &impl WalkBody, input: Input, y: f64) -> bool {
        let mut simulated = host.simulate(SimulatedInput::new(input), host.input_y_rot());
        simulated.tick();
        for _ in 0..30 {
            if simulated.player().on_ground() {
                return simulated.player().y() >= y - 0.01;
            }
            if simulated.player().y() < y - 1.0 {
                return false;
            }
            simulated.tick();
        }
        false
    }
}
