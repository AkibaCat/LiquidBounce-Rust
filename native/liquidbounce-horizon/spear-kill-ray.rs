// SPDX-License-Identifier: GPL-3.0-or-later
//! SpearKill COLLIDER/no-fluid ray with the actual entity collision context.
use super::Game;
use horizon_client::collisions::SceneCollisions;
use horizon_core::{
    geometry::{Vec3, VoxelShape},
    position::BlockPos,
};
use horizon_world::{
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
    registry::BlockStateId,
    world_ray::{RayBlockMode, RayFluidMode, RayWorld},
};
struct Ray<'a, 'r, 'c> {
    scene: SceneCollisions<'a, 'r>,
    context: CollisionContext<'c>,
}
impl RayWorld for Ray<'_, '_, '_> {
    type Error = std::io::Error;
    type Block = BlockStateId;
    type Fluid = ();
    fn block_state(&mut self, p: BlockPos) -> Result<BlockStateId, Self::Error> {
        Ok(self.scene.world.block(p.x, p.y, p.z))
    }
    fn fluid_state(&mut self, _: BlockPos) -> Result<(), Self::Error> {
        Ok(())
    }
    fn block_shape(
        &mut self,
        state: BlockStateId,
        p: BlockPos,
        _: RayBlockMode,
    ) -> Result<VoxelShape, Self::Error> {
        let binding = self.scene.binding;
        binding
            .collision_shape_with_context(&mut self.scene, state, p, &self.context)
            .map_err(|e| std::io::Error::other(format!("{e:?}")))
    }
    fn interaction_shape(
        &mut self,
        _: BlockStateId,
        _: BlockPos,
    ) -> Result<VoxelShape, Self::Error> {
        Ok(VoxelShape::empty())
    }
    fn fluid_is_source(&mut self, _: ()) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn fluid_is_empty(&mut self, _: ()) -> Result<bool, Self::Error> {
        Ok(true)
    }
    fn fluid_is_water(&mut self, _: ()) -> Result<bool, Self::Error> {
        Ok(false)
    }
    fn fluid_shape(&mut self, _: (), _: BlockPos) -> Result<VoxelShape, Self::Error> {
        Ok(VoxelShape::empty())
    }
}
pub(super) fn clip(game: &Game<'_>, start: Vec3, end: Vec3) -> Option<(bool, Vec3)> {
    let p = game.player.as_ref()?;
    let local = game.remote.as_ref().map_or(0, |r| r.entity_id) as u32;
    let held = game
        .inventory
        .get(game.client_hotbar_slot())
        .and_then(Option::as_ref);
    let mut ray = Ray {
        scene: SceneCollisions {
            world: &game.world,
            binding: game.collision,
            subject: Some(p.powder_snow_subject()),
        },
        context: CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(u64::from(local)),
                kind: EntityContextKind::Player,
                y: p.position.y,
                descending: p.shift_key_down,
                main_hand: Some(held.map_or(CollisionItem::EMPTY, |s| CollisionItem {
                    item: &s.item,
                    count: s.count as i32,
                })),
            },
            false,
        ),
    };
    // CollisionContext borrows held stack; it stays inside this synchronous ray.
    super::client_ghost_hand::clip(
        game,
        &mut ray,
        start,
        end,
        RayBlockMode::Collider,
        RayFluidMode::None,
        4096,
    )
    .ok()
    .map(|hit| (hit.miss, hit.hit.location))
}
