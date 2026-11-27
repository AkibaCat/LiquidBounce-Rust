// SPDX-License-Identifier: GPL-3.0-or-later
use crate::placement_renderer::{Box6, PlacementRenderer, Pos};
#[derive(Clone, Debug, PartialEq)]
pub struct ProphuntEsp {
    pub tracking: u32,
    pub render: PlacementRenderer,
}
impl Default for ProphuntEsp {
    fn default() -> Self {
        Self {
            tracking: 7,
            render: PlacementRenderer::new(false, 0x5affb348),
        }
    }
}
impl ProphuntEsp {
    pub fn add(
        &mut self,
        source: u32,
        positions: impl IntoIterator<Item = (Pos, Box6)>,
        time: i64,
    ) {
        if self.tracking & source != 0 {
            for (p, b) in positions {
                self.render.add(p, b, time, false);
            }
        }
    }
}
