// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleBlockBounce.kt and
// AABB.isBlockAtPosition, Copyright (c) 2015-2026 CCBlueX.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockBounce {
    pub motion: f32,
}
impl Default for BlockBounce {
    fn default() -> Self {
        Self { motion: 0.42 }
    }
}
impl BlockBounce {
    pub const MOTION_RANGE: (f32, f32) = (0.2, 2.0);
    pub fn jump_power(self, running: bool, on_bouncy_block: bool, original: f32) -> f32 {
        if running && on_bouncy_block {
            original + self.motion
        } else {
            original
        }
    }
}

/// Complete vanilla 26.3 SlimeBlock/BedBlock/HoneyBlock identity set. State
/// properties do not affect the original Kotlin `is` class checks.
pub fn is_bouncy_block(name: &str) -> bool {
    matches!(
        name,
        "minecraft:slime_block"
            | "minecraft:honey_block"
            | "minecraft:white_bed"
            | "minecraft:orange_bed"
            | "minecraft:magenta_bed"
            | "minecraft:light_blue_bed"
            | "minecraft:yellow_bed"
            | "minecraft:lime_bed"
            | "minecraft:pink_bed"
            | "minecraft:gray_bed"
            | "minecraft:light_gray_bed"
            | "minecraft:cyan_bed"
            | "minecraft:purple_bed"
            | "minecraft:blue_bed"
            | "minecraft:brown_bed"
            | "minecraft:green_bed"
            | "minecraft:red_bed"
            | "minecraft:black_bed"
    )
}

/// The detection box changes only minY by -0.01. The source extension scans
/// exactly that floorY plane and inclusive floorX/floorZ bounds; it does not
/// query block collision shapes, support-block identity or all vertical cells.
pub fn standing_on_bouncy_block(
    min: [f64; 3],
    max: [f64; 3],
    mut is_bouncy: impl FnMut([i32; 3]) -> bool,
) -> bool {
    let y = (min[1] - 0.01).floor() as i32;
    for x in min[0].floor() as i32..=max[0].floor() as i32 {
        for z in min[2].floor() as i32..=max[2].floor() as i32 {
            if is_bouncy([x, y, z]) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jump_event_adds_float_motion_after_all_vanilla_power_contributions() {
        let module = BlockBounce::default();
        assert_eq!(module.jump_power(true, true, 0.42), 0.42f32 + 0.42f32);
        assert_eq!(module.jump_power(false, true, 0.42), 0.42);
        assert_eq!(module.jump_power(true, false, 0.42), 0.42);
        assert_eq!(
            BlockBounce { motion: -3.0 }.jump_power(true, true, 0.42),
            0.42f32 - 3.0f32
        );
    }
    #[test]
    fn detection_retains_bottom_layer_inclusive_edges_and_x_then_z_early_exit() {
        let mut visited = Vec::new();
        let found = standing_on_bouncy_block([-0.1, 64., -0.1], [1., 65.8, 1.], |position| {
            visited.push(position);
            position == [0, 63, 0]
        });
        assert!(found);
        assert_eq!(
            visited,
            vec![
                [-1, 63, -1],
                [-1, 63, 0],
                [-1, 63, 1],
                [0, 63, -1],
                [0, 63, 0]
            ]
        );
        let mut visited = Vec::new();
        assert!(!standing_on_bouncy_block(
            [1., 64.01, 2.],
            [2., 65., 3.],
            |position| {
                visited.push(position);
                false
            }
        ));
        assert_eq!(
            visited,
            vec![[1, 64, 2], [1, 64, 3], [2, 64, 2], [2, 64, 3]]
        );
    }
}
