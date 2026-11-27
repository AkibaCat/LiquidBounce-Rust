// SPDX-License-Identifier: GPL-3.0-or-later
// Prepared for the coordinator's next centralized Rust batch; not run alone.
#[path = "../liquidbounce-behaviors/src/scaffold_block_selection.rs"]
mod selection;
use selection::*;
use std::cmp::Ordering;

fn stack(block: &str, count: i32) -> BlockStack {
    BlockStack {
        block: block.into(), count, player_can_stand_up: true, falling: false,
        base_entity: false, full_cube: true, redstone_conductor: true,
        friction: 0.6, speed_factor: 1.0, jump_factor: 1.0, hardness: 1.5,
    }
}
fn main() {
    let policy = Policy::default();
    let stone = stack("minecraft:stone", 32);
    assert!(stone.valid(&policy));
    assert!(!stack("minecraft:tnt", 64).valid(&policy));
    let mut falling = stack("minecraft:sand", 64);
    falling.falling = true;
    assert!(!falling.valid(&policy));
    let mut slab = stack("minecraft:stone_slab", 64);
    slab.player_can_stand_up = false;
    assert!(!slab.valid(&policy));
    let mut ice = stack("minecraft:ice", 64);
    ice.friction = 0.98;
    assert!(ice.unfavorable(&policy));
    assert_eq!(compare(&stone, &ice, &policy), Ordering::Greater);
    let mut custom = policy.clone();
    custom.disallowed.clear();
    assert!(stack("minecraft:tnt", 64).valid(&custom));
    let fewer = stack("minecraft:stone", 2);
    assert_eq!(compare(&fewer, &stone, &policy), Ordering::Greater);
    let mut odd_jump = stone.clone();
    odd_jump.jump_factor = 1.1;
    assert_eq!(compare(&odd_jump, &stone, &policy), Ordering::Greater);
    let mut hotbar: [Option<BlockStack>; 9] = std::array::from_fn(|_| None);
    hotbar[1] = Some(stone.clone());
    hotbar[3] = Some(stone.clone());
    assert_eq!(best_slot(&hotbar, 1, &policy), Some(1));
    hotbar[5] = Some(fewer.clone());
    assert_eq!(best_slot(&hotbar, 1, &policy), Some(5));
    assert_eq!(best_slot(&hotbar, 2, &policy), Some(1));
    assert_eq!(best_slot(&hotbar, 64, &policy), Some(5));
    hotbar[0] = Some(falling);
    assert_eq!(block_count(&hotbar, Some(&fewer), 1, true, &policy), 68);
    assert_eq!(block_count(&hotbar, Some(&fewer), 1, false, &policy), 34);
    assert_eq!(block_count(&hotbar, Some(&fewer), 0, false, &policy), 2);
    assert_eq!(AutoBlock::default(), AutoBlock { enabled:true, always:false, slot_reset_delay:5, do_not_use_below_count:1 });
}
