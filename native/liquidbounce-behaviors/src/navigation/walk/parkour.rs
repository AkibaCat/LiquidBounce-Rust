// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Parkour.kt. Candidate callbacks preserve interleaved terrain queries.
use super::{abilities::WalkAbilities, costs::Costs};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    simulation::block_grid::SimulationState,
    terrain::{Terrain, TerrainLevel},
    walk_jump::{jump::Jump, jump_targets::JumpTargets, jumps::Jumps},
};
pub struct Parkour<B: SimulationState> {
    max_jump_drop: i32,
    jump_penalty: f64,
    jumps: Jumps<B>,
    jump_targets: JumpTargets,
}
impl<B: SimulationState> Parkour<B> {
    pub fn new(abilities: WalkAbilities<B>) -> Self {
        Self {
            max_jump_drop: abilities.safe_fall.wrapping_add(if abilities.risky_jumps {
                3
            } else {
                1
            }),
            jump_penalty: if abilities.risky_jumps {
                0.0
            } else {
                Costs::JUMP_PENALTY
            },
            jump_targets: JumpTargets::new(abilities.motion),
            jumps: Jumps::new(abilities),
        }
    }
    pub fn for_each<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        from_height: f64,
        is_jump_worth: &mut impl FnMut(BlockPos) -> bool,
        sink: &mut impl FnMut(&mut Terrain<B, L>, BlockPos, f64, Jump),
    ) {
        let feet = Vec3::new(
            f64::from(from.x) + 0.5,
            from_height,
            f64::from(from.z) + 0.5,
        );
        let (mut planned, mut drops) = (0, 0);
        let jumps = &mut self.jumps;
        let jump_penalty = self.jump_penalty;
        self.jump_targets.for_each(
            t,
            from,
            from_height,
            self.max_jump_drop,
            &mut |t, tx, ty, tz, top| {
                let drop = top < from_height - 0.25;
                let target = BlockPos::new(tx, ty, tz);
                if drop && drops >= 24 || !is_jump_worth(target) {
                    return planned < 24;
                }
                if let Some(plan) = jumps.plan(t, from, from_height, target, top, false) {
                    sink(t, target, plan.ticks(feet) + jump_penalty, plan);
                    if drop {
                        drops += 1;
                    } else {
                        planned += 1;
                    }
                }
                planned < 24
            },
        );
    }
}
