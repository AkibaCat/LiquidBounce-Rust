// SPDX-License-Identifier: GPL-3.0-or-later
//! Pearl owner COLLIDER/OUTLINE rays with real equipment and powder-snow context.
use super::Game;
use horizon_client::collisions::SceneCollisions;
use horizon_core::{
    geometry::{Aabb, ShapeOp, Vec3, VoxelShape},
    position::BlockPos,
};
use horizon_world::{
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
    registry::BlockStateId,
    world_ray::{RayBlockMode, RayFluidMode, RayWorld},
};
struct Ray<'a, 'r, 'c> {
    game: &'a Game<'r>,
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
        mode: RayBlockMode,
    ) -> Result<VoxelShape, Self::Error> {
        if mode == RayBlockMode::Outline {
            if matches!(
                self.game.res.facts().get(state).name.as_ref(),
                "minecraft:water" | "minecraft:lava" | "minecraft:bubble_column"
            ) {
                return Ok(VoxelShape::empty());
            }
            let mut shape = VoxelShape::empty();
            for bounds in self.game.outline_boxes([p.x, p.y, p.z]) {
                let bounds: [f64; 6] =
                    std::array::from_fn(|i| bounds[i] - f64::from([p.x, p.y, p.z][i % 3]));
                let part = VoxelShape::from_box(Aabb::new(
                    Vec3::new(bounds[0], bounds[1], bounds[2]),
                    Vec3::new(bounds[3], bounds[4], bounds[5]),
                ))
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
                shape = shape
                    .join(&part, ShapeOp::OR)
                    .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            }
            return Ok(shape);
        }
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
pub(super) fn clip(
    game: &Game<'_>,
    start: Vec3,
    end: Vec3,
    owner: i32,
    outline: bool,
) -> Option<(bool, Vec3)> {
    let p = game.player.as_ref()?;
    let local = game.remote.as_ref().map_or(0, |r| r.entity_id);
    let entity = game.native_entity(owner as u32);
    let living = entity.and_then(|e| game.native_living(e));
    let held = living
        .and_then(|e| game.native_living_hand_item(e, horizon_client::first_person::Hand::Main));
    let boots = living.and_then(|e| {
        game.native_living_equipment(e, horizon_client::mod_api::EquipmentSlot::Feet)
    });
    let mut y = entity
        .and_then(|e| game.native_entity_position(e))
        .map_or(p.position.y, |a| a[1]);
    let mut descending = false;
    let mut subject = None;
    let kind = if owner == local {
        y = p.position.y;
        descending = p.shift_key_down;
        subject = Some(p.powder_snow_subject());
        EntityContextKind::Player
    } else if let Some(g) = game
        .lan
        .iter()
        .flat_map(|l| l.hub.guests.iter())
        .find(|g| g.placed && g.entity_id == owner)
    {
        descending = g.sneaking;
        if let Some(body) = g.motion.as_ref() {
            subject = Some(body.powder_snow_subject());
        }
        EntityContextKind::Player
    } else if let Some(m) = game
        .mobs
        .iter()
        .find(|m| m.id as i32 == owner && !m.removed)
    {
        y = m.position.y;
        descending = m.extra.crouching;
        subject = Some(horizon_client::collisions::PowderSnowSubject {
            fall_distance: m.fall_distance,
            leather_boots: boots
                .as_ref()
                .is_some_and(|s| s.count > 0 && s.item.as_ref() == "minecraft:leather_boots"),
            walkable_type: game
                .res
                .mob_data()
                .entity_has("minecraft:powder_snow_walkable_mobs", &m.kind.id),
            lava_walker: m.kind.id.as_str() == "minecraft:strider",
            falling_block: false,
        });
        if horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player" {
            EntityContextKind::Player
        } else {
            EntityContextKind::Living
        }
    } else if living.is_some() {
        EntityContextKind::Living
    } else {
        EntityContextKind::Other
    };
    let mut ray = Ray {
        game,
        scene: SceneCollisions {
            world: &game.world,
            binding: game.collision,
            subject,
        },
        context: CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(u64::from(owner as u32)),
                kind,
                y,
                descending,
                main_hand: living.map(|_| {
                    held.as_ref()
                        .map_or(CollisionItem::EMPTY, |s| CollisionItem {
                            item: &s.item,
                            count: s.count as i32,
                        })
                }),
            },
            false,
        ),
    };
    super::client_ghost_hand::clip(
        game,
        &mut ray,
        start,
        end,
        if outline {
            RayBlockMode::Outline
        } else {
            RayBlockMode::Collider
        },
        RayFluidMode::None,
        4096,
    )
    .ok()
    .map(|h| (h.miss, h.hit.location))
}
