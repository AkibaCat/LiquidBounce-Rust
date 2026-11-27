// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Moves.kt. The sink executes in place so Step construction and search
//! filters observe the same interleaved Terrain cache queries as the source.
use super::{
    abilities::WalkAbilities,
    changes::Changes,
    costs::Costs,
    nodes::{DX, DZ, Nodes},
    parkour::Parkour,
    route::{Move, Step},
};
use crate::navigation::{
    geometry::{BlockPos, Vec3, java_max},
    simulation::block_grid::SimulationState,
    terrain::{Terrain, TerrainLevel},
    walk_jump::jump::Jump,
};
pub struct MoveOffer<'a> {
    pub from: BlockPos,
    pub from_height: f64,
    pub to: BlockPos,
    pub movement: Move,
    pub cost: f64,
    pub breaks: &'a [BlockPos],
    pub places: &'a [BlockPos],
    pub opens: &'a [BlockPos],
    pub jump: Option<&'a Jump>,
}
pub type MoveSink<'a, B, L> = dyn for<'s> FnMut(&mut Terrain<B, L>, &Nodes<B>, MoveOffer<'s>) + 'a;
pub struct Moves<B: SimulationState> {
    core: MoveCore<B>,
    parkour: Parkour<B>,
}
struct MoveCore<B: SimulationState> {
    nodes: Nodes<B>,
    abilities: WalkAbilities<B>,
    walk: f64,
    changes: Changes,
    opens: Vec<BlockPos>,
    from: BlockPos,
    from_height: f64,
    from_stands: bool,
    from_floor_factor: f64,
    can_jump: bool,
}
impl<B: SimulationState> Moves<B> {
    pub fn new(width: f64, abilities: WalkAbilities<B>) -> Self {
        Self {
            parkour: Parkour::new(abilities.clone()),
            core: MoveCore {
                nodes: Nodes::new(width, abilities.clone()),
                walk: if abilities.can_sprint {
                    Costs::SPRINT
                } else {
                    Costs::WALK
                },
                abilities,
                changes: Changes::default(),
                opens: Vec::new(),
                from: BlockPos::default(),
                from_height: 0.0,
                from_stands: true,
                from_floor_factor: 1.0,
                can_jump: true,
            },
        }
    }
    pub fn nodes(&self) -> &Nodes<B> {
        &self.core.nodes
    }
    pub fn step<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        from: BlockPos,
        to: BlockPos,
    ) -> Option<Step> {
        let mut best: Option<Step> = None;
        self.for_each(
            t,
            from,
            &mut |t, nodes, offer| {
                if offer.to == to
                    && best
                        .as_ref()
                        .is_none_or(|current| offer.cost < current.cost)
                {
                    let feet = nodes.feet(t, to);
                    let shift = nodes.crouches(t, from) || nodes.crouches(t, to);
                    let edge = is_at_edge(t, nodes, from, offer.from_height, to, offer.movement);
                    let is_under_water = nodes.is_underwater(t, to);
                    best = Some(Step {
                        from,
                        to,
                        movement: offer.movement,
                        feet,
                        breaks: offer.breaks.to_vec(),
                        places: offer.places.to_vec(),
                        opens: offer.opens.to_vec(),
                        cost: offer.cost,
                        shift,
                        edge,
                        is_under_water,
                        jump: offer.jump.cloned(),
                    });
                }
            },
            &mut |target| target == to,
        );
        best
    }
    pub fn for_each<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        node: BlockPos,
        sink: &mut MoveSink<'_, B, L>,
        is_jump_worth: &mut impl FnMut(BlockPos) -> bool,
    ) {
        let c = &mut self.core;
        let water = c.nodes.swims(t, node);
        let climb = c.nodes.climbs(t, node);
        c.from = node;
        c.changes.from = node;
        c.from_height = c.nodes.height(t, node);
        c.from_stands = c.nodes.stands(t, node);
        let standing = c.from_stands || !water && !climb;
        if c.nodes.crouches(t, node) {
            c.crouch_moves(t, sink);
            return;
        }
        let floor = c.nodes.floor_block(t, node.x, node.z, c.from_height);
        c.from_floor_factor = if water || climb {
            1.0
        } else {
            Costs::floor_factor(&floor)
        };
        c.can_jump = water || climb || floor.jump_factor() >= 1.0;
        for i in 0..4 {
            let tx = node.x.wrapping_add(DX[i]);
            let tz = node.z.wrapping_add(DZ[i]);
            if standing || climb {
                c.traverse(t, sink, tx, tz, standing);
                c.ascend(t, sink, tx, tz);
            }
            if standing {
                c.fall(t, sink, tx, tz);
                c.diagonal(
                    t,
                    sink,
                    node.x.wrapping_add([1, 1, -1, -1][i]),
                    node.z.wrapping_add([1, -1, 1, -1][i]),
                );
            }
            c.swim_into(t, sink, BlockPos::new(tx, node.y, tz));
            if water {
                c.swim_out(t, sink, tx, tz);
            }
        }
        if c.abilities.can_parkour && c.from_stands && !t.is_water(node) {
            c.changes.clear();
            self.parkour.for_each(
                t,
                node,
                c.from_height,
                is_jump_worth,
                &mut |t, to, cost, jump| c.emit(t, sink, to, Move::Parkour, cost, Some(jump)),
            );
        }
        c.swim_into(t, sink, node.above());
        c.swim_into(t, sink, node.offset(0, -1, 0));
        c.climb(t, sink);
        if standing {
            c.pillar(t, sink);
            c.downward(t, sink);
        }
    }
}
include!("moves_surface.rs");
include!("moves_actions.rs");
include!("moves_emit.rs");

fn is_at_edge<B: SimulationState, L: TerrainLevel<B>>(
    t: &mut Terrain<B, L>,
    nodes: &Nodes<B>,
    from: BlockPos,
    from_height: f64,
    to: BlockPos,
    movement: Move,
) -> bool {
    if !matches!(movement, Move::Traverse | Move::Diagonal)
        || nodes.swims(t, from)
        || nodes.swims(t, to)
    {
        return false;
    }
    let to_height = nodes.height(t, to);
    if (to_height - from_height).abs() > Terrain::<B, L>::STEP_HEIGHT {
        return false;
    }
    nodes.is_near_drop(t, from, from_height) || nodes.is_near_drop(t, to, to_height)
}
