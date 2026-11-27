// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent navigation/walk/Landings.kt.
use super::{
    assisted_jump::{Landing, LandingRef},
    jump_targets::JumpTargets,
    jumps::Jumps,
    vector_yaw, wrap_degrees,
};
use crate::navigation::{
    geometry::{BlockPos, Vec3, java_max},
    simulation::{block_grid::SimulationState, player_motion::PlayerMotion},
    terrain::{Terrain, TerrainLevel},
};
use std::sync::Arc;
pub(crate) struct Landings {
    pub best: Option<LandingRef>,
    best_score: f64,
    best_intent: f64,
    missed: Vec<Missed>,
}
pub(crate) struct Missed {
    pub pos: BlockPos,
    pub top: f64,
    pub intent: f64,
}
impl Landings {
    fn new() -> Self {
        Self {
            best: None,
            best_score: f64::MAX,
            best_intent: f64::MAX,
            missed: Vec::new(),
        }
    }
    pub fn run_up(
        &mut self,
        mut run_up: impl FnMut(&Missed) -> Option<LandingRef>,
    ) -> Option<LandingRef> {
        self.missed.sort_by(|a, b| java_compare(a.intent, b.intent));
        for candidate in self.missed.iter().take(3) {
            if candidate.intent > self.best_intent - 0.5 {
                break;
            }
            if let Some(landing) = run_up(candidate) {
                return Some(landing);
            }
        }
        None
    }
    #[allow(clippy::too_many_arguments)]
    pub fn find<B: SimulationState, L: TerrainLevel<B>>(
        t: &mut Terrain<B, L>,
        targets: &mut JumpTargets,
        jumps: &mut Jumps<B>,
        state: &PlayerMotion,
        y_rot: f32,
        heading: f32,
        max_angle: f32,
        feet: BlockPos,
        height: f64,
    ) -> Self {
        let position = state.position();
        let mut found = Self::new();
        targets.for_each_options(
            t,
            feet,
            height,
            4,
            12,
            state.delta_y,
            true,
            &mut |t, x, y, z, top| {
                let angle = angle(
                    position,
                    Vec3::new(f64::from(x) + 0.5, top, f64::from(z) + 0.5),
                    heading,
                );
                if angle > max_angle {
                    return true;
                }
                let restitution = t.restitution(x, top, z);
                let bounces = restitution > 0.0;
                let drop = java_max(0.0, height - top);
                // Original angle division is float, then widens for the drop.
                let intent = f64::from(angle / max_angle) + if bounces { 0.0 } else { drop * 0.5 };
                let pos = BlockPos::new(x, y, z);
                if let Some(jump) = jumps.from_here(t, state, y_rot, pos, top, None) {
                    if intent - jump.margin < found.best_score {
                        let takes_off = bounces && drop * restitution * restitution >= 1.0;
                        let score = intent - jump.margin;
                        found.best = Some(Arc::new(Landing {
                            pos,
                            feet: top,
                            jump,
                            step: None,
                            bounces,
                            takes_off,
                            settles: false,
                        }));
                        found.best_score = score;
                        found.best_intent = intent;
                    }
                } else {
                    found.missed.push(Missed { pos, top, intent });
                }
                true
            },
        );
        found
    }
}
pub(crate) fn angle(from: Vec3, to: Vec3, heading: f32) -> f32 {
    wrap_degrees(vector_yaw(to.x - from.x, to.z - from.z) - heading).abs()
}
fn java_compare(a: f64, b: f64) -> std::cmp::Ordering {
    if a.is_nan() {
        if b.is_nan() {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    } else if b.is_nan() {
        std::cmp::Ordering::Less
    } else {
        a.total_cmp(&b)
    }
}
