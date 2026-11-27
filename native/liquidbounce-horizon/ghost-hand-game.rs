// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_core::geometry::Vec3;
use horizon_world::{registry::BlockStateId,
    world_ray::{RayBlockMode, RayError, RayFluidMode, RayResult, RayWorld}};

mod ray {
__RAY__
}

#[allow(clippy::too_many_arguments)] // The original clip context is unchanged.
pub(super) fn clip<W: RayWorld<Block = BlockStateId>>(
    game: &Game<'_>,
    world: &mut W,
    start: Vec3,
    end: Vec3,
    blocks: RayBlockMode,
    fluids: RayFluidMode,
    max_visits: usize,
) -> Result<RayResult, RayError<W::Error>> {
    let targets = game.client_ghost_hand_targets();
    let accepts = |state| targets.as_ref().is_some_and(|targets|
        targets.contains(&*game.res.facts().get(state).name));
    ray::clip(world, start, end, blocks, fluids, max_visits,
        targets.as_ref().map(|_| &accepts as &dyn Fn(BlockStateId) -> bool))
}
