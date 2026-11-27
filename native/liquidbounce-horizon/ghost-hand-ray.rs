// SPDX-License-Identifier: GPL-3.0-or-later
// TweakedMethods.tweakedRaycast: a complete selected-block pass precedes fallback.
use horizon_core::geometry::{Vec3, traverse_blocks};
use horizon_world::world_ray::{
    RayBlockMode, RayError, RayFluidMode, RayResult, RayWorld, clip_with_interaction, clip_world,
};

#[allow(clippy::too_many_arguments)] // Mirrors the actual clip context plus the module predicate.
pub fn clip<W: RayWorld>(
    world: &mut W,
    start: Vec3,
    end: Vec3,
    blocks: RayBlockMode,
    fluids: RayFluidMode,
    max_visits: usize,
    targeted: Option<&dyn Fn(W::Block) -> bool>,
) -> Result<RayResult, RayError<W::Error>> {
    if let Some(targeted) = targeted {
        let selected = traverse_blocks(start, end, max_visits, |p| {
            let mut query = || -> Result<_, W::Error> {
                let state = world.block_state(p)?;
                if !targeted(state) {
                    return Ok(None);
                }
                let shape = world.block_shape(state, p, blocks)?;
                clip_with_interaction(world, start, end, p, &shape, state)
            };
            match query() {
                Ok(Some(hit)) => Some(Ok(hit)),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            }
        })
        .map_err(RayError::Traversal)?;
        if let Some(hit) = selected {
            return Ok(RayResult {
                hit: hit.map_err(RayError::World)?,
                miss: false,
            });
        }
    }
    // No selected shape hit: preserve ordinary blocks, fluids and miss factory.
    clip_world(world, start, end, blocks, fluids, max_visits)
}
