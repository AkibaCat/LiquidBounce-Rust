// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleXRay, pinned LiquidBounce ae0eb396. Meshing reads an immutable build snapshot.
use std::collections::BTreeSet;
include!("xray_defaults.rs");
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub full_bright: bool,
    pub exposed_only: bool,
    pub background_opacity: u8,
    pub blocks: BTreeSet<String>,
}
impl Default for Settings {
    fn default() -> Self {
        let mut s = Self {
            full_bright: true,
            exposed_only: false,
            background_opacity: 0,
            blocks: DEFAULT_BLOCKS
                .iter()
                .map(|n| format!("minecraft:{n}"))
                .collect(),
        };
        s.blocks
            .extend(EXTRA_BLOCKS.iter().map(|n| format!("minecraft:{n}")));
        s
    }
}
impl Settings {
    /// Source CommandXRay.reset intentionally does not re-add the initial families.
    pub fn reset_blocks(&mut self) {
        self.blocks = DEFAULT_BLOCKS
            .iter()
            .map(|n| format!("minecraft:{n}"))
            .collect();
    }
    pub fn render(&self, selected: bool, neighbor_conductors: [bool; 6]) -> bool {
        selected && (!self.exposed_only || neighbor_conductors.into_iter().any(|v| !v))
    }
    pub fn background(&self, selected: bool, air: bool) -> bool {
        self.background_opacity > 0 && !selected && !air
    }
    pub fn face(
        &self,
        original: bool,
        selected: bool,
        background: bool,
        skip_rendering: bool,
    ) -> bool {
        if background {
            original
        } else if !selected {
            false
        } else if self.exposed_only {
            !skip_rendering
        } else {
            true
        }
    }
    pub fn draw_side(
        &self,
        original: bool,
        render: bool,
        adjacent_full_face: bool,
        same_block: bool,
        adjacent_solid: bool,
        adjacent_render: bool,
    ) -> bool {
        original
            || (render
                && (!adjacent_full_face || !same_block || !adjacent_solid || !adjacent_render))
    }
}
