// SPDX-License-Identifier: GPL-3.0-or-later
impl<B: SimulationState> MoveCore<B> {
    fn pass_or_break<L: TerrainLevel<B>>(&mut self, t: &mut Terrain<B, L>, p: BlockPos) -> bool {
        self.changes
            .pass_or_break(t, &self.nodes, &self.abilities, p)
    }
    fn traverse<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        tx: i32,
        tz: i32,
        standing: bool,
    ) {
        let p = BlockPos::new(tx, self.from.y, tz);
        self.changes.clear();
        let wet = self.is_wet(t, tx, tz);
        let top = t.stand_height(p);
        if !top.is_nan() {
            let jump = self.jump_cost(top - self.from_height);
            let fits = jump == 0.0 || !jump.is_nan() && self.has_jump_room(t, tx, tz);
            if fits && self.can_cross(t, tx, tz, java_max(self.from_height, top)) {
                let cost = self.walk_cost(t, tx, tz, top, wet) + jump;
                self.emit(t, sink, p, Move::Traverse, cost, None);
            }
        } else if self.nodes.climbs(t, p) {
            let cost = self.walk_cost(t, tx, tz, f64::from(p.y), wet);
            self.emit(t, sink, p, Move::Traverse, cost, None);
        } else if self.nodes.crouches(t, p) {
            if t.crouch_height(p) - self.from_height <= Terrain::<B, L>::STEP_HEIGHT {
                self.emit(t, sink, p, Move::Traverse, Costs::SNEAK, None);
            }
        } else {
            self.traverse_changing(t, sink, tx, tz, standing, wet);
        }
    }
    fn traverse_changing<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        tx: i32,
        tz: i32,
        standing: bool,
        wet: bool,
    ) {
        let p = BlockPos::new(tx, self.from.y, tz);
        if !self.pass_or_break(t, p.above()) || !self.pass_or_break(t, p) {
            return;
        }
        let on_floor = standing && self.from_height <= f64::from(p.y) + 1.0e-6;
        let mut floor = self.nodes.floor_top(t, p);
        if floor.is_nan()
            && self.nodes.is_placeable(t, p.offset(0, -1, 0))
            && self
                .changes
                .can_place_against(t, &self.nodes, p.offset(0, -1, 0), on_floor)
        {
            self.changes.places.push(p.offset(0, -1, 0));
            floor = f64::from(p.y);
        }
        let jump = self.jump_cost(floor - self.from_height);
        if self.changes.breaks.is_empty() && self.changes.places.is_empty()
            || jump.is_nan()
            || jump > 0.0 && !self.has_jump_room(t, tx, tz)
        {
            return;
        }
        if self.changes.breaks.is_empty()
            && !self.can_cross(t, tx, tz, java_max(self.from_height, floor))
        {
            return;
        }
        let place_cost = if self.changes.breaks.is_empty() {
            self.abilities.bridge_ticks
        } else {
            Costs::PLACE
        };
        let cost = self.walk_cost(t, tx, tz, floor, wet)
            + jump
            + self.changes.break_cost
            + self.changes.places.len() as f64 * place_cost;
        self.emit(t, sink, p, Move::Traverse, cost, None);
    }
    fn diagonal<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        tx: i32,
        tz: i32,
    ) {
        let from = self.from;
        let p = BlockPos::new(tx, from.y, tz);
        self.changes.clear();
        let top = t.stand_height(p);
        if top.is_nan() || top - self.from_height > Terrain::<B, L>::STEP_HEIGHT {
            return;
        }
        if !t.is_open(BlockPos::new(tx, from.y, from.z))
            || !t.is_open(BlockPos::new(from.x, from.y, tz))
            || !self.can_cross(t, tx, tz, java_max(self.from_height, top))
        {
            return;
        }
        let wet = self.is_wet(t, tx, tz);
        let cost = f64::from(std::f32::consts::SQRT_2) * self.walk_cost(t, tx, tz, top, wet);
        self.emit(t, sink, p, Move::Diagonal, cost, None);
    }
    fn ascend<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        tx: i32,
        tz: i32,
    ) {
        let from = self.from;
        let p = BlockPos::new(tx, from.y.wrapping_add(1), tz);
        self.changes.clear();
        if !self.pass_or_break(t, from.offset(0, 2, 0)) {
            return;
        }
        let top = t.stand_height(p);
        if !top.is_nan() {
            let jump = self.jump_cost(top - self.from_height);
            if !jump.is_nan() && self.can_cross(t, tx, tz, top) {
                let cost = self.walk_cost(t, tx, tz, top, false) + jump + self.changes.break_cost;
                self.emit(t, sink, p, Move::Ascend, cost, None);
            }
            return;
        }
        let head_breaks = self.changes.breaks.len();
        if !self.pass_or_break(t, p.above()) || !self.pass_or_break(t, p) {
            return;
        }
        let mut stacked = false;
        let mut floor = self.nodes.floor_top(t, p);
        if floor.is_nan() && self.nodes.is_placeable(t, p.offset(0, -1, 0)) {
            let support = self
                .nodes
                .is_solid(t, BlockPos::new(tx, from.y.wrapping_sub(1), tz))
                || {
                    stacked = self.stacks(
                        t,
                        BlockPos::new(tx, from.y.wrapping_sub(1), tz),
                        head_breaks,
                    );
                    stacked
                };
            if support {
                self.changes.places.push(p.offset(0, -1, 0));
                floor = f64::from(p.y);
            }
        }
        let jump = self.jump_cost(floor - self.from_height);
        if jump.is_nan()
            || self.changes.breaks.len() == head_breaks && self.changes.places.is_empty()
        {
            return;
        }
        let place_cost = if stacked
            && !self
                .nodes
                .is_solid(t, BlockPos::new(tx, from.y.wrapping_sub(2), tz))
        {
            self.abilities.bridge_ticks
        } else {
            Costs::PLACE
        };
        let cost = self.walk_cost(t, tx, tz, floor, false)
            + jump
            + self.changes.break_cost
            + self.changes.places.len() as f64 * place_cost;
        self.emit(t, sink, p, Move::Ascend, cost, None);
    }
    fn stacks<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        p: BlockPos,
        head_breaks: usize,
    ) -> bool {
        if head_breaks > 0
            || !self.changes.breaks.is_empty()
            || self.nodes.swims(t, self.from)
            || self.nodes.climbs(t, self.from)
            || self.from_height > f64::from(p.y.wrapping_add(1)) + 1.0e-6
            || !self.nodes.is_placeable(t, p)
        {
            return false;
        }
        self.changes.places.push(p);
        true
    }
}
