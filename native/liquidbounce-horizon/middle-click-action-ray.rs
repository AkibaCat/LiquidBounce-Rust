// SPDX-License-Identifier: GPL-3.0-or-later
//! FriendClicker hasLineOfSight uses COLLIDER rather than outline shapes.
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
struct Ray<'a, 'r> {
    scene: SceneCollisions<'a, 'r>,
    context: CollisionContext<'static>,
}
impl RayWorld for Ray<'_, '_> {
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
pub(super) fn visible(game: &Game<'_>, start: Vec3, end: Vec3, camera: u32) -> bool {
    let Some(p) = game.player.as_ref() else {
        return false;
    };
    let local = game.remote.as_ref().map_or(0, |r| r.entity_id) as u32;
    let (y, descending, kind) = if camera == local {
        (p.position.y, p.shift_key_down, EntityContextKind::Player)
    } else if let Some(m) = game.camera_mob() {
        (
            m.position.y,
            false,
            if horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player" {
                EntityContextKind::Player
            } else {
                EntityContextKind::None
            },
        )
    } else {
        (
            game.camera_entity().map_or(p.position.y, |c| c.position[1]),
            false,
            EntityContextKind::None,
        )
    };
    let mut ray = Ray {
        scene: SceneCollisions {
            world: &game.world,
            binding: game.collision,
            subject: (camera == local).then(|| p.powder_snow_subject()),
        },
        context: CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(u64::from(camera)),
                kind,
                y,
                descending,
                main_hand: Some(CollisionItem::EMPTY),
            },
            false,
        ),
    };
    super::client_ghost_hand::clip(
        game,
        &mut ray,
        start,
        end,
        RayBlockMode::Collider,
        RayFluidMode::None,
        4096,
    )
    .is_ok_and(|hit| hit.miss)
}
