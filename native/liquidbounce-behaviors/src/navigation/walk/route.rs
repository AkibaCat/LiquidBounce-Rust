// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete Agent walk Route.kt; immutable route ownership preserves lazy points.
use super::{WalkState, abilities::WalkAbilities};
use crate::navigation::{
    geometry::{BlockPos, Vec3},
    walk_jump::jump::Jump,
};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Traverse,
    Diagonal,
    Ascend,
    Fall,
    Parkour,
    Pillar,
    Downward,
    Swim,
    ClimbUp,
    ClimbDown,
}
#[derive(Clone, Debug)]
pub struct Step {
    pub from: BlockPos,
    pub to: BlockPos,
    pub movement: Move,
    pub feet: Vec3,
    pub breaks: Vec<BlockPos>,
    pub places: Vec<BlockPos>,
    pub opens: Vec<BlockPos>,
    pub cost: f64,
    pub shift: bool,
    pub edge: bool,
    pub is_under_water: bool,
    pub jump: Option<Jump>,
}
impl Step {
    pub fn is_bridge(&self) -> bool {
        self.movement == Move::Traverse
            && self.to.y == self.from.y
            && self.breaks.is_empty()
            && self.opens.is_empty()
            && self.jump.is_none()
            && self.places.len() == 1
            && self.places[0] == self.to.offset(0, -1, 0)
    }
}
#[derive(Clone)]
pub struct Route<B: WalkState> {
    start: Vec3,
    steps: Vec<Step>,
    can_reach: bool,
    abilities: WalkAbilities<B>,
    points: OnceLock<Vec<Vec3>>,
}
impl<B: WalkState> Route<B> {
    pub fn new(
        start: Vec3,
        steps: Vec<Step>,
        can_reach: bool,
        abilities: WalkAbilities<B>,
    ) -> Self {
        Self {
            start,
            steps,
            can_reach,
            abilities,
            points: OnceLock::new(),
        }
    }
    pub fn start(&self) -> Vec3 {
        self.start
    }
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
    pub fn can_reach(&self) -> bool {
        self.can_reach
    }
    pub fn abilities(&self) -> &WalkAbilities<B> {
        &self.abilities
    }
    pub fn end(&self) -> Vec3 {
        self.steps.last().map_or(self.start, |s| s.feet)
    }
    pub fn points(&self) -> &[Vec3] {
        self.points.get_or_init(|| {
            std::iter::once(self.start)
                .chain(self.steps.iter().map(|s| s.feet))
                .collect()
        })
    }
    pub fn cost(&self) -> f64 {
        self.steps
            .iter()
            .map(|s| s.cost)
            .fold(0.0, |cost, c| cost + c)
    }
}
