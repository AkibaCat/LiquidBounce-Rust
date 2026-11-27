// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent Terrain.invalidate and dependent stand/trapdoor cache invalidation.
use super::{BlockKind, BlockPos, Terrain, TerrainLevel, TerrainState, VoxelShape};
impl<B: TerrainState, L: TerrainLevel<B>> Terrain<B, L> {
    pub fn invalidate(&mut self, pos: BlockPos, state: &B) -> bool {
        let key = pos.packed();
        if let Some(old) = self.states.get(&key)
            && self.is_alike(old, state, pos)
        {
            if state.open() == Some(true) {
                self.closed.remove(&key);
            }
            return false;
        }
        let mut cached = self.states.remove(&key).is_some();
        cached = self.drop_at(pos) || cached;
        if self.can_open_doors {
            for dy in [-2, -1, 1] {
                let neighbor = pos.offset(0, dy, 0);
                if self
                    .states
                    .get(&neighbor.packed())
                    .is_some_and(|state| state.kind(BlockKind::Trapdoor))
                {
                    cached = self.drop_at(neighbor) || cached;
                }
            }
        }
        cached
    }
    fn drop_at(&mut self, pos: BlockPos) -> bool {
        let key = pos.packed();
        let mut cached = self.shapes.remove(&key).is_some();
        self.closed.remove(&key);
        for dy in -2..=2 {
            let stand_key = pos.offset(0, dy, 0).packed();
            if self.stands.remove(&stand_key).is_some() {
                cached = true;
            }
            if self.crouch_stands.remove(&stand_key).is_some() {
                cached = true;
            }
        }
        cached
    }
    fn is_alike(&self, old: &B, new: &B, pos: BlockPos) -> bool {
        if old.same_instance(new) {
            return true;
        }
        if !old.same_block(new) || !old.same_fluid(new) || old.is_hazard() != new.is_hazard() {
            return false;
        }
        let old_shape = self
            .shapes
            .get(&pos.packed())
            .cloned()
            .unwrap_or_else(|| old.collision_shape(&self.level, pos));
        let new_shape = if new.is_hazard() {
            VoxelShape::block()
        } else {
            new.collision_shape(&self.level, pos)
        };
        old_shape == new_shape
    }
}
