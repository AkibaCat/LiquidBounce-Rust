// SPDX-License-Identifier: GPL-3.0-or-later
// Original ClipContext.Block.COLLIDER, Fluid.NONE, CollisionContext.empty().
use super::Game;
use horizon_client::collisions::SceneCollisions;
use horizon_core::{
    geometry::{Vec3, VoxelShape},
    position::BlockPos,
};
use horizon_world::{
    collision_context::CollisionContext,
    registry::BlockStateId,
    world_ray::{RayBlockMode, RayFluidMode, RayWorld, clip_world},
};
use std::io;
struct Ground<'a, 'r> {
    scene: SceneCollisions<'a, 'r>,
}
impl RayWorld for Ground<'_, '_> {
    type Error = io::Error;
    type Block = BlockStateId;
    type Fluid = ();
    fn block_state(&mut self, p: BlockPos) -> io::Result<BlockStateId> {
        Ok(self.scene.world.block(p.x, p.y, p.z))
    }
    fn fluid_state(&mut self, _: BlockPos) -> io::Result<()> {
        Ok(())
    }
    fn block_shape(
        &mut self,
        state: BlockStateId,
        p: BlockPos,
        _: RayBlockMode,
    ) -> io::Result<VoxelShape> {
        let binding = self.scene.binding;
        binding
            .collision_shape_with_context(
                &mut self.scene,
                state,
                p,
                &CollisionContext::empty(false),
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    fn interaction_shape(&mut self, _: BlockStateId, _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
    fn fluid_is_source(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_is_empty(&mut self, _: ()) -> io::Result<bool> {
        Ok(true)
    }
    fn fluid_is_water(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_shape(&mut self, _: (), _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
}
impl Game<'_> {
    pub(super) fn client_potion_fx_ground(&self, p: [f64; 3]) -> Option<[f64; 3]> {
        let start = Vec3::new(p[0], p[1], p[2]);
        let hit = clip_world(
            &mut Ground {
                scene: SceneCollisions {
                    world: &self.world,
                    binding: self.collision,
                    subject: None,
                },
            },
            start,
            start.add_vector(Vec3::new(0., -1., 0.)),
            RayBlockMode::Collider,
            RayFluidMode::None,
            16,
        )
        .ok()?;
        (!hit.miss).then_some([hit.hit.location.x, hit.hit.location.y, hit.hit.location.z])
    }
}
