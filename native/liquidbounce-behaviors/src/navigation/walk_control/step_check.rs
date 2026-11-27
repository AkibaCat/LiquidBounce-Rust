// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent StepCheck.kt; check raw block facts without search caching.
use super::{
    REACH_HORIZONTAL, SHORTCUT_STEPS,
    body::{WalkPlayer, is_stable},
    horizontal,
    path_points::{StepControl, continues_straight, has_passed},
};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    terrain::TerrainLevel,
    walk::{
        WalkState,
        route::{Route, Step},
    },
};
pub trait ActionConfirmation {
    fn is_unconfirmed(&self, pos: BlockPos) -> bool;
}
pub fn is_cleared<B: WalkState, L: TerrainLevel<B>>(level: &L, pos: BlockPos) -> bool {
    level
        .block_state(pos)
        .collision_shape(level, pos)
        .is_empty()
}
pub fn is_placed<B: WalkState, L: TerrainLevel<B>>(level: &L, pos: BlockPos) -> bool {
    !level.block_state(pos).can_be_replaced()
}
pub fn is_open<B: WalkState, L: TerrainLevel<B>>(level: &L, pos: BlockPos) -> bool {
    level.block_state(pos).open().unwrap_or(true)
}
pub fn is_reached<B: WalkState, L: TerrainLevel<B>>(
    player: &impl WalkPlayer,
    level: &L,
    actions: &impl ActionConfirmation,
    step: &Step,
    next: Option<&Step>,
) -> bool {
    let built = step.places.iter().all(|&p| is_placed(level, p))
        && !step.places.iter().any(|&p| actions.is_unconfirmed(p));
    if !step.breaks.iter().all(|&p| is_cleared(level, p))
        || !built
        || !step.opens.iter().all(|&p| is_open(level, p))
    {
        return false;
    }
    let position = player.position();
    let block = BlockPos::containing(position);
    if step.jump.is_some() {
        let support = player.supporting_block();
        return player.on_climbable() && block == step.to
            || player.on_ground()
                && (block == step.to
                    || support == Some(step.to)
                    || support == Some(step.to.offset(0, -1, 0)));
    }
    let close = horizontal(position, step.feet) < REACH_HORIZONTAL;
    if close && (position.y - step.feet.y).abs() < 0.6 {
        return true;
    }
    if (step.is_plain() || step.is_bridge()) && has_passed(step, position) {
        return true;
    }
    is_stable(player)
        && next.is_some()
        && block == step.to
        && continues_straight(step, next.unwrap())
}
pub fn skip_ahead<B: WalkState>(player: &impl WalkPlayer, path: &Route<B>, index: usize) -> usize {
    if !player.on_ground() {
        return index;
    }
    let block = BlockPos::containing(player.position());
    for i in index..path.steps().len().min(index + SHORTCUT_STEPS) {
        let step = &path.steps()[i];
        if !step.is_plain() {
            return index;
        }
        if step.to == block {
            return i + 1;
        }
    }
    index
}
pub fn wants_jump_at(step: &Step, position: Vec3, stable: bool, in_water: bool) -> bool {
    let keeps_level = step.to.y >= step.from.y;
    match step.movement {
        crate::navigation::walk::route::Move::Swim => keeps_level,
        crate::navigation::walk::route::Move::ClimbUp => true,
        crate::navigation::walk::route::Move::ClimbDown
        | crate::navigation::walk::route::Move::Fall
        | crate::navigation::walk::route::Move::Downward => false,
        _ => {
            let rises = step.feet.y - position.y > 0.6 + 0.01;
            let in_reach = horizontal(position, step.feet) < 1.5;
            rises && in_reach && stable || in_water && keeps_level
        }
    }
}
