    /// Original BlockGetter.clip override applies to every clip context.
    pub(super) fn native_client_ghost_hand(
        &self,
        running: &mut bool,
        targeted_blocks: &mut Vec<String>,
    ) {
        mod_api::dispatch_client_ghost_hand(self, mod_api::mods(), running, targeted_blocks);
    }
    pub(super) fn client_ghost_hand_targets(&self) -> Option<std::collections::BTreeSet<String>> {
        let mut running = false;
        let mut targets = Vec::new();
        self.native_client_ghost_hand(&mut running, &mut targets);
        running.then(|| targets.into_iter().collect())
    }
    /// The legacy Game outline DDA shares the target-first traversal too.
    pub(super) fn client_ghost_hand_outline(
        &self,
        origin: [f64; 3],
        direction: [f64; 3],
        reach: f64,
    ) -> Option<Hit> {
        use horizon_core::geometry::{Vec3, traverse_blocks};
        let targets = self.client_ghost_hand_targets()?;
        let start = Vec3::new(origin[0], origin[1], origin[2]);
        let end = start.add_vector(Vec3::new(direction[0], direction[1], direction[2]).scale(reach));
        let delta = end.subtract(start);
        let visits = (delta.x.abs() + delta.y.abs() + delta.z.abs()).ceil() as usize;
        traverse_blocks(start, end, visits.saturating_add(8), |p| {
            let state = self.world.block(p.x, p.y, p.z);
            if !targets.contains(&*self.res.facts().get(state).name) { return None; }
            self.clip_declared_outline(state, [p.x, p.y, p.z], origin, direction, reach)
        }).ok().flatten()
    }
