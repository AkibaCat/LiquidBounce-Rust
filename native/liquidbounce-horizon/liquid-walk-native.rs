// SPDX-License-Identifier: GPL-3.0-or-later
//! The original bottom-row liquid lookup and direct non-liquid shape scan.
use crate::{collisions::SceneCollisions, motion::PlayerMotion, world::World};
use horizon_core::{
    geometry::{Aabb, Vec3},
    position::BlockPos,
};
pub use horizon_world::liquid_walk::{clear, packet_context, set_mode, shift};
use horizon_world::{
    block_collision::BlockCollisionRegistry,
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
};
fn liquid(
    binding: &BlockCollisionRegistry<'_>,
    state: horizon_world::registry::BlockStateId,
) -> bool {
    binding
        .registry()
        .state(state)
        .is_some_and(|s| matches!(s.name.as_str(), "minecraft:water" | "minecraft:lava"))
}
pub fn row(world: &World, binding: &BlockCollisionRegistry<'_>, bounds: Aabb, down: f64) -> bool {
    let min = bounds.min_position();
    let max = bounds.max_position();
    let y = (min.y - down).floor() as i32;
    for x in min.x.floor() as i32..=max.x.floor() as i32 {
        for z in min.z.floor() as i32..=max.z.floor() as i32 {
            if liquid(binding, world.block(x, y, z)) {
                return true;
            }
        }
    }
    false
}
fn other(world: &World, player: &PlayerMotion, binding: &BlockCollisionRegistry<'_>) -> bool {
    let b = player.bounds();
    let min = b.min_position();
    let max = b.max_position();
    let bounds = Aabb::new(Vec3::new(min.x, min.y - 0.5, min.z), max);
    let min = bounds.min_position();
    let context = CollisionContext::for_entity(
        CollisionEntity {
            identity: EntityIdentity(0),
            kind: EntityContextKind::Player,
            y: player.position.y,
            descending: player.shift_key_down,
            main_hand: Some(CollisionItem::EMPTY),
        },
        false,
    );
    let mut scene = SceneCollisions {
        world,
        binding,
        subject: Some(player.powder_snow_subject()),
    };
    for x in min.x.floor() as i32..=max.x.floor() as i32 {
        for y in min.y.floor() as i32..=max.y.floor() as i32 {
            for z in min.z.floor() as i32..=max.z.floor() as i32 {
                let state = world.block(x, y, z);
                if liquid(binding, state) {
                    continue;
                }
                let shape = binding
                    .collision_shape_with_context(
                        &mut scene,
                        state,
                        BlockPos::new(x, y, z),
                        &context,
                    )
                    .expect("LiquidWalk direct collision shape");
                if shape.boxes().iter().any(|b| {
                    b.move_by(Vec3::new(x as f64, y as f64, z as f64))
                        .intersects(bounds)
                }) {
                    return true;
                }
            }
        }
    }
    false
}
pub fn context(
    world: &World,
    player: &PlayerMotion,
    binding: &BlockCollisionRegistry<'_>,
) -> [bool; 4] {
    let inside = row(world, binding, player.bounds(), 0.);
    let standing = row(world, binding, player.bounds(), 0.01);
    let p = player.position;
    let bubble = binding
        .registry()
        .state(world.block(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32))
        .is_some_and(|s| s.name == "minecraft:bubble_column");
    let value = [
        inside,
        standing,
        other(world, player, binding),
        player.fluids.was_touching_water || bubble,
    ];
    horizon_world::liquid_walk::context(value, player.fluids.was_touching_water);
    value
}
pub fn shape_player(world: &World, player: &PlayerMotion, binding: &BlockCollisionRegistry<'_>) {
    horizon_world::liquid_walk::player(
        player.fall_distance,
        player.fire_ticks > 0,
        row(world, binding, player.bounds(), 0.),
    );
}
pub fn cancel_jump(
    world: &World,
    player: &PlayerMotion,
    binding: &BlockCollisionRegistry<'_>,
) -> bool {
    horizon_world::liquid_walk::cancels_jump(row(world, binding, player.bounds(), 0.01))
}
