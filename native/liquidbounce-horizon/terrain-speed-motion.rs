// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    motion::MotionWorld,
    terrain_speed::{ClimbWorld, Direction},
    terrain_speed_bridge as bridge,
};
use horizon_core::{geometry::Vec3, position::BlockPos};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl ClimbWorld for MotionWorld<'_, '_, '_> {
    type Error = Box<dyn std::error::Error>;
    fn horizontal_collision(&self) -> bool {
        self.player.collision.horizontal
    }
    fn on_climbable(&mut self) -> Result<bool> {
        self.climbable()
    }
    fn forward_key(&self) -> bool {
        bridge::climb().is_some_and(|(_, forward)| forward)
    }
    fn position(&self) -> [f64; 3] {
        let p = self.player.position;
        [p.x, p.y, p.z]
    }
    fn block_position(&self) -> [i32; 3] {
        let p = BlockPos::containing(self.player.position);
        [p.x, p.y, p.z]
    }
    fn ladder_or_vine(&mut self, p: [i32; 3]) -> Result<bool> {
        let state = self.scene.block(p[0], p[1], p[2]);
        Ok(self.resources.terrain_climb_blocks.contains(&state))
    }
    fn direction(&self) -> Direction {
        // Entity.getDirection -> Direction.fromYRot: floor(yaw/90 + .5) & 3.
        // MC26.3 Mth.floor(double) now delegates to Math.floor then d2i.
        let index=(f64::from(self.player.yaw)/90.0+0.5).floor() as i32;
        match index & 3 {
            0 => Direction::South,
            1 => Direction::West,
            2 => Direction::North,
            _ => Direction::East,
        }
    }
    fn abs_snap_to(&mut self, p: [f64; 3]) -> Result<()> {
        // Actual MC26.3 Entity.absSnapTo: clamp X/Z; write xo/yo/zo; setPos.
        // Mth.clamp uses comparisons, preserving NaN and signed zero.
        fn clamp(v: f64) -> f64 {
            v.clamp(-30_000_000.0, 30_000_000.0)
        }
        let position = Vec3::new(clamp(p[0]), p[1], clamp(p[2]));
        bridge::record_snap(position);
        self.player.position = position;
        Ok(())
    }
}
pub fn move_event(world: &mut MotionWorld<'_, '_, '_>, movement: Vec3) -> Result<Vec3> {
    if !world.player.native_client_queries_allowed() {
        return Ok(movement);
    }
    let Some((climb, _)) = bridge::climb() else {
        return Ok(movement);
    };
    let mut raw = [movement.x, movement.y, movement.z];
    climb.move_event(true, world, &mut raw)?;
    Ok(Vec3::new(raw[0], raw[1], raw[2]))
}
