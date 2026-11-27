// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent Stride.kt, including reads across sprint-speed changes.
use super::body::{WalkPlayer, capture_motion};
use crate::navigation::{
    geometry::Vec3,
    simulation::motion_traits::MotionTraits,
    terrain::TerrainLevel,
    walk::{WalkState, abilities::WalkAbilities},
};
pub struct Stride {
    factor: f64,
    moved: Vec3,
    last_position: Option<Vec3>,
    carried: Vec3,
    was_on_ground: bool,
    was_crouching: bool,
    last_speed: f64,
    last_tick: i32,
}
impl Default for Stride {
    fn default() -> Self {
        Self {
            factor: 1.0,
            moved: Vec3::ZERO,
            last_position: None,
            carried: Vec3::ZERO,
            was_on_ground: false,
            was_crouching: false,
            last_speed: 0.0,
            last_tick: 0,
        }
    }
}
impl Stride {
    pub fn factor(&self) -> f64 {
        self.factor
    }
    pub fn moved(&self) -> Vec3 {
        self.moved
    }
    pub fn tick(&mut self, player: &impl WalkPlayer) {
        let position = player.position();
        let last = self.last_position;
        let speed = player.movement_speed();
        self.moved = if let Some(last) =
            last.filter(|_| player.tick_count() == self.last_tick.wrapping_add(1))
        {
            super::subtract(position, last)
        } else {
            Vec3::ZERO
        };
        if let Some(last) = last.filter(|_| {
            player.tick_count() == self.last_tick.wrapping_add(1)
                && self.was_on_ground
                && !self.was_crouching
                && speed == self.last_speed
                && walks_plainly(player)
        }) {
            let dx = position.x - last.x - self.carried.x;
            let dz = position.z - last.z - self.carried.z;
            let pushed = (dx * dx + dz * dz).sqrt();
            let vanilla = speed * 0.98;
            if vanilla > 0.0 && pushed > vanilla * 0.25 {
                self.factor =
                    (self.factor + (pushed / vanilla - self.factor) * 0.5).clamp(0.25, 4.0);
            }
        }
        self.last_position = Some(position);
        self.carried = player.delta_movement();
        self.was_on_ground = player.on_ground();
        self.was_crouching = player.is_crouching();
        self.last_speed = speed;
        self.last_tick = player.tick_count();
    }
    pub fn abilities<B: WalkState>(
        &self,
        searched: &WalkAbilities<B>,
        player: &impl WalkPlayer,
    ) -> WalkAbilities<B> {
        let now = capture_motion(player);
        let walking_speed = (f64::from(now.walking_speed) * self.factor) as f32;
        let motion = searched.motion;
        if (walking_speed / motion.walking_speed - 1.0_f32).abs() < 0.05
            && now.jump_power == motion.jump_power
        {
            return searched.clone();
        }
        searched.with_motion(MotionTraits::new(
            walking_speed,
            now.jump_strength,
            now.jump_boost_power,
            now.mechanics,
        ))
    }
}
fn walks_plainly(player: &impl WalkPlayer) -> bool {
    if !player.on_ground()
        || player.horizontal_collision()
        || player.is_in_water()
        || player.on_climbable()
    {
        return false;
    }
    if player.is_crouching() || player.is_using_item() {
        return false;
    }
    player
        .level()
        .block_state(player.block_below_movement())
        .friction()
        == 0.6_f32
}
