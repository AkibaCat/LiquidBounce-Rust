// SPDX-License-Identifier: GPL-3.0-or-later
use crate::placement_renderer::{Box6, PlacementRenderer, Pos};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq)]
pub struct VoidEsp {
    pub y_threshold: i32,
    pub range_side: i32,
    pub range_facing: i32,
    pub render: PlacementRenderer,
    pub previous: BTreeSet<Pos>,
}
impl Default for VoidEsp {
    fn default() -> Self {
        Self {
            y_threshold: 16,
            range_side: 3,
            range_facing: 8,
            render: PlacementRenderer::new(true, 0x5aff0000),
            previous: Default::default(),
        }
    }
}
impl VoidEsp {
    pub fn update(&mut self, positions: Vec<(Pos, Box6)>, time: i64) {
        let next = positions.iter().map(|p| p.0).collect::<BTreeSet<_>>();
        for p in self.previous.difference(&next) {
            self.render.remove(*p, time);
        }
        for (p, b) in positions {
            self.render.add(p, b, time, true);
        }
        self.previous = next;
        self.render.update_all();
    }
    pub fn disabled(&mut self, time: i64) {
        self.previous.clear();
        self.render.clear_silently(time);
    }
    pub fn world_change(&mut self) {
        self.render.clear();
    }
}
