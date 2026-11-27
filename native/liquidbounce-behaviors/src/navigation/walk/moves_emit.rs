// SPDX-License-Identifier: GPL-3.0-or-later
impl<B: SimulationState> MoveCore<B> {
    fn emit<L: TerrainLevel<B>>(
        &mut self,
        t: &mut Terrain<B, L>,
        sink: &mut MoveSink<'_, B, L>,
        to: BlockPos,
        movement: Move,
        cost: f64,
        jump: Option<Jump>,
    ) {
        if t.is_bowl(to) {
            return;
        }
        self.opens.clear();
        if t.can_open_doors() {
            let body = t.body_box(Vec3::new(
                f64::from(self.from.x) + 0.5,
                self.from_height,
                f64::from(self.from.z) + 0.5,
            ));
            let target_y = self.nodes.height(t, to);
            let target = t.body_box(Vec3::new(
                f64::from(to.x) + 0.5,
                target_y,
                f64::from(to.z) + 0.5,
            ));
            t.closed_in(body.minmax(target), body, &mut self.opens);
        }
        let cost = cost + self.opens.len() as f64 * Costs::OPEN;
        sink(
            t,
            &self.nodes,
            MoveOffer {
                from: self.from,
                from_height: self.from_height,
                to,
                movement,
                cost,
                breaks: &self.changes.breaks,
                places: &self.changes.places,
                opens: &self.opens,
                jump: jump.as_ref(),
            },
        );
    }
    fn walk_cost<L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        x: i32,
        z: i32,
        top: f64,
        wet: bool,
    ) -> f64 {
        if wet {
            Costs::SWIM
        } else {
            self.walk
                * (self.from_floor_factor
                    + Costs::floor_factor(&self.nodes.floor_block(t, x, z, top)))
                / 2.0
        }
    }
    fn is_wet<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, tx: i32, tz: i32) -> bool {
        t.is_water(self.from) || t.is_water(BlockPos::new(tx, self.from.y, tz))
    }
    fn jump_cost(&self, rise: f64) -> f64 {
        if rise <= 0.6 {
            0.0
        } else if self.can_jump && rise <= 1.25 {
            Costs::jump()
        } else {
            f64::NAN
        }
    }
    fn has_jump_room<L: TerrainLevel<B>>(&self, t: &mut Terrain<B, L>, tx: i32, tz: i32) -> bool {
        t.is_passable(self.from.offset(0, 2, 0))
            && t.is_passable(BlockPos::new(tx, self.from.y.wrapping_add(2), tz))
    }
    fn can_cross<L: TerrainLevel<B>>(
        &self,
        t: &mut Terrain<B, L>,
        tx: i32,
        tz: i32,
        height: f64,
    ) -> bool {
        if !self.from_stands {
            return true;
        }
        let x = f64::from(self.from.x.wrapping_add(tx)) / 2.0 + 0.5;
        let z = f64::from(self.from.z.wrapping_add(tz)) / 2.0 + 0.5;
        t.is_clear(t.body_box(Vec3::new(x, height, z)))
            || t.is_clear(t.body_box(Vec3::new(x, height + Terrain::<B, L>::STEP_HEIGHT, z)))
    }
}
