// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent WalkAbilities.kt: immutable search snapshot and withMotion copy.
use super::{WalkState, costs::Costs, water::Water};
use crate::navigation::simulation::motion_traits::MotionTraits;
use std::{collections::HashSet, sync::Arc};

pub type BreakTicks<B> = Arc<dyn Fn(&B) -> f64 + Send + Sync>;

#[derive(Clone)]
pub struct WalkAbilities<B: WalkState> {
    pub can_break: bool,
    pub can_place: bool,
    pub placeable_blocks: i32,
    pub can_parkour: bool,
    pub risky_jumps: bool,
    pub can_sprint: bool,
    pub safe_fall: i32,
    pub water_fall: i32,
    pub break_ticks: BreakTicks<B>,
    pub avoid: Arc<HashSet<i64>>,
    pub air: i32,
    pub motion: MotionTraits,
    pub jump_climbs: bool,
    pub bridge_ticks: f64,
    pub unbreakable: Arc<HashSet<i64>>,
    pub health: f32,
    pub reserved: Arc<HashSet<i64>>,
}
impl<B: WalkState> WalkAbilities<B> {
    /// Required original constructor arguments, with the original defaults for
    /// optional fields. All collections and attributes belong to one capture.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        can_break: bool,
        can_place: bool,
        placeable_blocks: i32,
        can_parkour: bool,
        can_sprint: bool,
        safe_fall: i32,
        water_fall: i32,
        break_ticks: BreakTicks<B>,
        avoid: Arc<HashSet<i64>>,
    ) -> Self {
        Self {
            can_break,
            can_place,
            placeable_blocks,
            can_parkour,
            risky_jumps: false,
            can_sprint,
            safe_fall,
            water_fall,
            break_ticks,
            avoid,
            air: Water::MAX_AIR,
            motion: MotionTraits::default(),
            jump_climbs: true,
            bridge_ticks: Costs::PLACE,
            unbreakable: Arc::new(HashSet::new()),
            health: 0.0,
            reserved: Arc::new(HashSet::new()),
        }
    }
    pub fn with_motion(&self, motion: MotionTraits) -> Self {
        Self {
            motion,
            ..self.clone()
        }
    }
}
