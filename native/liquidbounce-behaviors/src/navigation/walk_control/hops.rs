// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Hops.kt.
use super::{
    body::{ControlState, WalkPlayer},
    horizontal,
    stride::Stride,
};
use crate::navigation::{
    geometry::{BlockPos, Vec3, java_max},
    terrain::{Terrain, TerrainLevel},
    walk_jump::{vector_yaw, wrap_degrees},
};
#[derive(Default)]
pub struct Hops {
    aim: Option<Vec3>,
}
impl Hops {
    pub fn aim(&self) -> Option<Vec3> {
        self.aim
    }
    pub fn clear(&mut self) {
        self.aim = None;
    }
    #[allow(clippy::too_many_arguments)]
    pub fn starts<B: ControlState, L: TerrainLevel<B>>(
        &mut self,
        player: &impl WalkPlayer,
        terrain: &mut Terrain<B, L>,
        movement_y_rot: impl FnMut() -> f32,
        stride: &Stride,
        position: Vec3,
        aim: Vec3,
    ) -> bool {
        if !can_hop(player, terrain, movement_y_rot, stride, position, aim) {
            return false;
        }
        self.aim = Some(aim);
        true
    }
}
fn can_hop<B: ControlState, L: TerrainLevel<B>>(
    player: &impl WalkPlayer,
    terrain: &mut Terrain<B, L>,
    mut movement_y_rot: impl FnMut() -> f32,
    stride: &Stride,
    position: Vec3,
    aim: Vec3,
) -> bool {
    let distance = horizontal(position, aim);
    let able = player.on_ground() && player.is_sprinting() && player.food_level() > 10;
    if !able || distance < 4.0 {
        return false;
    }
    if wrap_degrees(vector_yaw(aim.x - position.x, aim.z - position.z) - movement_y_rot()).abs()
        > 15.0
    {
        return false;
    }
    let dx = (aim.x - position.x) / distance;
    let dz = (aim.z - position.z) / distance;
    let length = 3.5 * java_max(1.0, stride.factor());
    let mut travelled = 0.0;
    while travelled <= length {
        let x = position.x + dx * travelled;
        let z = position.z + dz * travelled;
        let floor = terrain.state(BlockPos::containing(Vec3::new(x, position.y - 0.5, z)));
        if !terrain.is_clear(terrain.body_box(Vec3::new(x, position.y + 1.25, z)))
            || floor.is_farmland()
        {
            return false;
        }
        travelled += 0.5;
    }
    true
}
