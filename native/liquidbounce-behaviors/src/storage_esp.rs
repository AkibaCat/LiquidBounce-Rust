// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleStorageESP, LiquidBounce ae0eb396. Actual block-entity/model providers live in Horizon.
use crate::breadcrumbs::{Effect, StyleSettings};
use std::collections::{BTreeSet, VecDeque};
#[path = "storage_esp_surface.rs"]
mod surface;
pub type Point = [f64; 3];
pub type Bounds = [f64; 6];
#[derive(Default)]
pub struct Mesh {
    pub quads: Vec<[Point; 4]>,
    pub lines: Vec<[Point; 2]>,
}
pub const TYPES: [&str; 13] = [
    "Chest",
    "TrappedChest",
    "Barrel",
    "EnderChest",
    "Furnace",
    "BrewingStand",
    "Dispenser",
    "Dropper",
    "Hopper",
    "ShulkerBox",
    "Pot",
    "Bookshelf",
    "Shelf",
];
pub const FILLS: [f32; 9] = [0.12, 0., 0.08, 0.1, 0.08, 0.18, 0., 0.08, 0.1];
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    pub enabled: bool,
    pub color: i32,
    pub tracers: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StorageEsp {
    pub glow: bool,
    pub outline: bool,
    pub style: usize,
    pub styles: [StyleSettings; 9],
    pub categories: [Category; 13],
    pub requires_chest_stealer: bool,
    pub merge_adjacent: bool,
    pub fade: [f32; 4],
}
impl Default for StorageEsp {
    fn default() -> Self {
        Self {
            glow: true,
            outline: true,
            style: 1,
            styles: std::array::from_fn(|i| {
                let mut s = StyleSettings::new(i);
                s.values.insert("Fill".into(), FILLS[i]);
                s
            }),
            categories: [
                0x0064ff, 0xdc3c28, 0xf6821f, 0xff00ff, 0x4f4f4f, 0x8b4513, 0xc0c0c0, 0x826e5a,
                0x808080, 0x9d6e9d, 0xd18600, 0x8b5a2b, 0xa0522d,
            ]
            .map(|v| Category {
                enabled: true,
                color: (0xff000000u32 | v) as i32,
                tracers: false,
            }),
            requires_chest_stealer: false,
            merge_adjacent: false,
            fade: [0., 0., 512., 512.],
        }
    }
}
pub fn block_category(name: &str) -> Option<usize> {
    let n = name.strip_prefix("minecraft:").unwrap_or(name);
    Some(match n {
        "chest" => 0,
        "trapped_chest" => 1,
        "barrel" => 2,
        "ender_chest" => 3,
        "furnace" | "blast_furnace" | "smoker" => 4,
        "brewing_stand" => 5,
        "dispenser" | "crafter" => 6,
        "dropper" => 7,
        "hopper" => 8,
        "decorated_pot" => 10,
        "chiseled_bookshelf" => 11,
        n if n == "shulker_box" || n.ends_with("_shulker_box") => 9,
        n if n.ends_with("_shelf") => 12,
        _ => return None,
    })
}
pub fn entity_category(name: &str, chested: bool) -> Option<usize> {
    let n = name.strip_prefix("minecraft:").unwrap_or(name);
    match n {
        "hopper_minecart" => Some(8),
        "chest_minecart" | "chest_boat" | "chest_raft" => Some(0),
        "donkey" | "mule" | "llama" | "trader_llama" if chested => Some(0),
        n if n.ends_with("_chest_boat") || n.ends_with("_chest_raft") => Some(0),
        _ => None,
    }
}
pub fn rgba(c: i32) -> [f32; 4] {
    let c = c as u32;
    [
        ((c >> 16) & 255) as f32 / 255.,
        ((c >> 8) & 255) as f32 / 255.,
        (c & 255) as f32 / 255.,
        (c >> 24) as f32 / 255.,
    ]
}
pub fn distance(a: Point, b: Point) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    if a == b {
        return if x < a { 0. } else { 1. };
    }
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
impl StorageEsp {
    pub fn active(&self, chest_stealer: bool) -> bool {
        !self.requires_chest_stealer || chest_stealer
    }
    pub fn show_tracers(&self) -> bool {
        self.categories.iter().any(|c| c.tracers)
    }
    pub fn category(&self, i: usize) -> Option<&Category> {
        self.categories.get(i).filter(|c| c.enabled)
    }
    pub fn within(&self, p: Point, camera: Point) -> bool {
        distance(p, camera) < f64::from(self.fade[3]).powi(2)
    }
    pub fn fade_at(&self, p: Point, camera: Point) -> f32 {
        let d = distance(p, camera).sqrt() as f32;
        smooth(self.fade[0], self.fade[1], d) * (1. - smooth(self.fade[2], self.fade[3], d))
    }
    pub fn set_fade(&mut self, i: usize, v: f32) {
        let v = v.clamp(0., 512.);
        self.fade[i] = match i {
            0 => v.min(self.fade[1]),
            1 => v.clamp(self.fade[0], self.fade[2]),
            2 => v.clamp(self.fade[1], self.fade[3]),
            _ => v.max(self.fade[2]),
        };
    }
    pub fn effect(&self) -> Option<Effect> {
        if !self.glow {
            return None;
        }
        if self.style == 0 {
            return Some(Effect {
                style: 9,
                params: [0.; 8],
                accent: [1.; 4],
                range: 1.,
                opacity: 1.,
            });
        }
        Some(self.styles[self.style - 1].effect(self.style - 1))
    }
}
pub struct Block {
    pub position: [i32; 3],
    pub category: usize,
    pub boxes: Vec<Bounds>,
}
/// Source's six-neighbour/category components; disconnected shapes remain independent.
pub fn shapes(blocks: &[Block], merge: bool) -> Vec<(usize, Mesh)> {
    let mut used = BTreeSet::new();
    let mut output = vec![];
    for start in 0..blocks.len() {
        if !used.insert(start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut boxes = vec![];
        while let Some(i) = queue.pop_front() {
            boxes.extend_from_slice(&blocks[i].boxes);
            if merge {
                for j in 0..blocks.len() {
                    if !used.contains(&j)
                        && blocks[j].category == blocks[start].category
                        && (0..3)
                            .map(|a| {
                                (blocks[i].position[a] as i64 - blocks[j].position[a] as i64).abs()
                            })
                            .sum::<i64>()
                            == 1
                    {
                        used.insert(j);
                        queue.push_back(j);
                    }
                }
            }
        }
        if !boxes.is_empty() {
            output.push((
                blocks[start].category,
                surface::Surface::new(&boxes).mesh(None),
            ));
        }
    }
    output
}
pub fn box_mesh(b: Bounds) -> Mesh {
    surface::box_mesh(b, None)
}
