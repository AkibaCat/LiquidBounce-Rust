// SPDX-License-Identifier: GPL-3.0-or-later
//! Global native policy, captured once per section build. No per-vertex locking.
use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock, RwLock},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub name: Arc<str>,
    pub air: bool,
    pub conductor: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub full_bright: bool,
    pub exposed: bool,
    pub opacity: u8,
    pub selected: Vec<bool>,
    pub states: Arc<[State]>,
}
static POLICY: OnceLock<RwLock<Option<Arc<Policy>>>> = OnceLock::new();
pub fn current() -> Option<Arc<Policy>> {
    POLICY.get_or_init(Default::default).read().unwrap().clone()
}
pub fn configure(input: &str, states: impl FnOnce() -> Vec<State>) -> bool {
    let v = serde_json::from_str::<serde_json::Value>(input).unwrap_or_default();
    let next = if v["active"] == true {
        let names: BTreeSet<&str> = v["blocks"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|n| n.as_str())
            .collect();
        let states: Arc<[State]> = states().into();
        Some(Arc::new(Policy {
            full_bright: v["fullBright"] == true,
            exposed: v["exposed"] == true,
            opacity: v["opacity"].as_u64().unwrap_or(0).min(255) as u8,
            selected: states
                .iter()
                .map(|s| names.contains(s.name.as_ref()))
                .collect(),
            states,
        }))
    } else {
        None
    };
    let mut policy = POLICY.get_or_init(Default::default).write().unwrap();
    if *policy == next {
        return false;
    }
    *policy = next;
    true
}
impl Policy {
    pub fn selected(&self, id: u32) -> bool {
        self.selected.get(id as usize).copied().unwrap_or(false)
    }
    pub fn render(&self, id: u32, neighbors: impl IntoIterator<Item = u32>) -> bool {
        self.selected(id)
            && (!self.exposed
                || neighbors
                    .into_iter()
                    .any(|n| self.states.get(n as usize).is_some_and(|s| !s.conductor)))
    }
    pub fn background(&self, id: u32) -> bool {
        self.opacity > 0
            && !self.selected(id)
            && self.states.get(id as usize).is_some_and(|s| !s.air)
    }
    pub fn same(&self, a: u32, b: u32) -> bool {
        self.states
            .get(a as usize)
            .zip(self.states.get(b as usize))
            .is_some_and(|(a, b)| a.name == b.name)
    }
    /// The high layer bit in a translucent quad is otherwise unused. Its color alpha
    /// is coverage, carried in terrain wire d.w; ordinary HDR/fluid metadata is untouched.
    pub fn background_vertices(&self, mesh: &mut crate::mesher::SectionMesh, start: [usize; 2]) {
        let mut moved = mesh.solid.split_off(start[0]);
        mesh.solid_tints.truncate(start[0] / 4);
        moved.extend(mesh.translucent.drain(start[1]..));
        for v in &mut moved {
            v.layer |= crate::mesher::OPAQUE_LAYER;
            v.color = (v.color & 0xffffff) | u32::from(self.opacity) << 24;
        }
        mesh.translucent.extend(moved);
    }
}
