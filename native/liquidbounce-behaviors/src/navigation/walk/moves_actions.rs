// SPDX-License-Identifier: GPL-3.0-or-later
impl<B: SimulationState> MoveCore<B> {
    fn fall<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        tx: i32,
        tz: i32,
    ) {
        let p = BlockPos::new(tx, self.from.y, tz);
        if !self.nodes.floor_top(t, p).is_nan() || t.is_water(p) {
            return;
        }
        self.changes.clear();
        if !self.pass_or_break(t, p.above()) || !self.pass_or_break(t, p) {
            return;
        }
        if self.changes.breaks.is_empty() && !self.can_cross(t, tx, tz, self.from_height) {
            return;
        }
        let Some(landing) = self
            .nodes
            .landing(t, tx, p.y.wrapping_sub(1), tz, self.from_height)
        else {
            return;
        };
        let blocks = landing.drop.ceil() as i32;
        let floor = self
            .nodes
            .floor_block(t, tx, tz, self.from_height - landing.drop);
        let bounce = Costs::bounce(&floor, blocks);
        let cost = self.walk * self.from_floor_factor
            + Costs::fall(blocks)
            + bounce
            + f64::from(landing.damage) * Costs::HURT
            + self.changes.break_cost;
        self.emit(
            t,
            sink,
            BlockPos::new(tx, landing.y, tz),
            Move::Fall,
            cost,
            None,
        );
    }
    fn pillar<L: TerrainLevel<B>>(&mut self, t: &mut Terrain<B, L>, sink: &mut MoveSink<'_, B, L>) {
        let p = self.from;
        if !self.can_jump || !t.is_passable(p) || !self.nodes.is_placeable(t, p) {
            return;
        }
        self.changes.clear();
        if !self.pass_or_break(t, p.offset(0, 2, 0)) {
            return;
        }
        self.changes.places.push(p);
        self.emit(
            t,
            sink,
            p.above(),
            Move::Pillar,
            Costs::jump() + Costs::PLACE + self.changes.break_cost,
            None,
        );
    }
    fn downward<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
    ) {
        let p = self.from.offset(0, -1, 0);
        if t.is_passable(p) {
            return;
        }
        self.changes.clear();
        let ticks = self
            .changes
            .break_ticks(t, &self.nodes, &self.abilities, p, true);
        if ticks.is_nan() || self.nodes.floor_top(t, p).is_nan() {
            return;
        }
        self.changes.breaks.push(p);
        self.emit(
            t,
            sink,
            p,
            Move::Downward,
            ticks + Costs::BREAK_OVERHEAD + Costs::fall(1),
            None,
        );
    }
    fn swim_into<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        p: BlockPos,
    ) {
        if !self.nodes.swims(t, p)
            || self.nodes.water.is_low(t, p) && !self.nodes.swims(t, self.from)
        {
            return;
        }
        self.changes.clear();
        let sprints =
            self.abilities.can_sprint && p.y == self.from.y && self.nodes.water.is_submerged(t, p);
        self.emit(
            t,
            sink,
            p,
            Move::Swim,
            if sprints {
                Costs::SPRINT_SWIM
            } else {
                Costs::SWIM
            },
            None,
        );
    }
    fn swim_out<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        tx: i32,
        tz: i32,
    ) {
        let p = BlockPos::new(tx, self.from.y, tz);
        self.changes.clear();
        if self.nodes.stands(t, p) {
            self.emit(t, sink, p, Move::Swim, Costs::SWIM, None);
        } else if self.nodes.stands(t, p.above()) && t.is_passable(self.from.offset(0, 2, 0)) {
            self.emit(
                t,
                sink,
                p.above(),
                Move::Swim,
                Costs::SWIM + Costs::jump(),
                None,
            );
        }
    }
    fn climb<L: TerrainLevel<B>>(&mut self, t: &mut Terrain<B, L>, sink: &mut MoveSink<'_, B, L>) {
        let p = self.from;
        if !t.is_climbable(p) {
            return;
        }
        self.changes.clear();
        let can_climb_up = self.abilities.jump_climbs || t.climb_side(p).is_some();
        if can_climb_up && (self.nodes.climbs(t, p.above()) || self.nodes.stands(t, p.above())) {
            self.emit(t, sink, p.above(), Move::ClimbUp, Costs::CLIMB_UP, None);
        }
        if self.nodes.climbs(t, p.offset(0, -1, 0)) {
            self.emit(
                t,
                sink,
                p.offset(0, -1, 0),
                Move::ClimbDown,
                Costs::CLIMB_DOWN,
                None,
            );
        } else if !self.from_stands && t.is_passable(p.offset(0, -1, 0)) {
            let Some(landing) =
                self.nodes
                    .landing(t, p.x, p.y.wrapping_sub(1), p.z, self.from_height)
            else {
                return;
            };
            let blocks = landing.drop.ceil() as i32;
            let floor = self
                .nodes
                .floor_block(t, p.x, p.z, self.from_height - landing.drop);
            let bounce = Costs::bounce(&floor, blocks);
            self.emit(
                t,
                sink,
                BlockPos::new(p.x, landing.y, p.z),
                Move::Fall,
                Costs::fall(blocks) + bounce + f64::from(landing.damage) * Costs::HURT,
                None,
            );
        }
    }
    fn crouch_moves<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
    ) {
        self.changes.clear();
        for (dx, dz) in DX.into_iter().zip(DZ) {
            let p = self.from.offset(dx, 0, dz);
            let mut top = t.stand_height(p);
            if top.is_nan() {
                top = t.crouch_height(p);
            }
            if !top.is_nan() && top - self.from_height <= Terrain::<B, L>::STEP_HEIGHT {
                self.emit(t, sink, p, Move::Traverse, Costs::SNEAK, None);
            }
        }
    }
}
