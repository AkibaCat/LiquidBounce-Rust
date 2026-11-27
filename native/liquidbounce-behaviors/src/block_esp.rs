// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBlockESP settings, map/static/rainbow colours and voxel-shape output.
use crate::{
    breadcrumbs::{STYLES, StyleSettings},
    storage_esp::{self, Bounds, Point, StorageEsp},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    MapColor,
    Static,
    Rainbow,
}
impl ColorMode {
    pub fn tag(self) -> &'static str {
        match self {
            Self::MapColor => "MapColor",
            Self::Static => "Static",
            Self::Rainbow => "Rainbow",
        }
    }
    pub fn parse(v: &str) -> Option<Self> {
        Some(match v {
            "MapColor" => Self::MapColor,
            "Static" => Self::Static,
            "Rainbow" => Self::Rainbow,
            _ => return None,
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub glow: bool,
    pub outline: bool,
    pub style: usize,
    pub styles: [StyleSettings; 9],
    pub targets: BTreeSet<String>,
    pub color_mode: ColorMode,
    pub color: u32,
    pub fade: [f32; 4],
    pub merge_adjacent: bool,
}
impl Default for Settings {
    fn default() -> Self {
        let mut targets = [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "light_gray",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ]
        .map(|v| format!("minecraft:{v}_bed"))
        .into_iter()
        .collect::<BTreeSet<_>>();
        targets.insert("minecraft:dragon_egg".into());
        let style = StorageEsp::default();
        Self {
            glow: false,
            outline: true,
            style: style.style,
            styles: style.styles,
            targets,
            color_mode: ColorMode::MapColor,
            color: 0x32ffb348,
            fade: [0., 0., 512., 512.],
            merge_adjacent: false,
        }
    }
}
impl Settings {
    pub fn set_fade(&mut self, index: usize, value: f32) {
        let mut helper = StorageEsp::default();
        helper.fade = self.fade;
        helper.set_fade(index, value);
        self.fade = helper.fade;
    }
    pub fn color(&self, map_color: u32, nanos: f64) -> u32 {
        match self.color_mode {
            ColorMode::MapColor => 0x64000000 | (map_color & 0xffffff),
            ColorMode::Static => self.color,
            ColorMode::Rainbow => {
                let rgb = crate::breadcrumbs::rainbow(nanos);
                0x32000000
                    | ((rgb[0] * 255.) as u32) << 16
                    | ((rgb[1] * 255.) as u32) << 8
                    | (rgb[2] * 255.) as u32
            }
        }
    }
    pub fn style_name(&self) -> &'static str {
        if self.style == 0 {
            "Vanilla"
        } else {
            STYLES[self.style - 1]
        }
    }
    pub fn frame(&self, blocks: &[Block], camera: Point, nanos: f64) -> Value {
        let mut helper = StorageEsp::default();
        helper.glow = self.glow;
        helper.outline = self.outline;
        helper.style = self.style;
        helper.styles = self.styles.clone();
        helper.fade = self.fade;
        let mut categories = BTreeMap::<(String, Option<u32>), usize>::new();
        let mut colors = Vec::new();
        let mut shapes = Vec::new();
        for block in blocks {
            if block.name == "minecraft:air" || !self.targets.contains(&block.name) {
                continue;
            }
            let color = self.color(block.map_color, nanos);
            let next = categories.len();
            let key = (
                block.name.clone(),
                (self.color_mode == ColorMode::MapColor).then_some(color),
            );
            let i = *categories.entry(key).or_insert_with(|| {
                colors.push(color);
                next
            });
            shapes.push(storage_esp::Block {
                position: block.position,
                category: i,
                boxes: block.boxes.clone(),
            });
        }
        let (mut quads, mut lines) = (Vec::new(), Vec::new());
        for (i, mesh) in storage_esp::shapes(&shapes, self.merge_adjacent) {
            let color = storage_esp::rgba(colors[i] as i32);
            let alpha = if self.glow { 1. } else { color[3] };
            let outline_alpha = if self.color_mode == ColorMode::MapColor {
                color[3]
            } else {
                150. / 255.
            };
            let vertex = |p: Point, a: f32| {
                let mut c = color;
                c[3] = a * helper.fade_at(p, camera);
                json!({"position":p,"color":c})
            };
            for q in mesh.quads {
                quads.push(json!(q.map(|p| vertex(p, alpha))));
            }
            if !self.glow && self.outline {
                for l in mesh.lines {
                    lines.push(json!(l.map(|p| vertex(p, outline_alpha))));
                }
            }
        }
        let effect=helper.effect().map(|e|json!({"style":e.style,"params":e.params,"accent":e.accent,"range":e.range,"opacity":e.opacity}));
        json!({"quads":quads,"lines":lines,"effect":effect,"targets":[]})
    }
}
#[derive(Clone, Debug)]
pub struct Block {
    pub position: [i32; 3],
    pub name: String,
    pub map_color: u32,
    pub boxes: Vec<Bounds>,
}
impl Block {
    pub fn from_json(v: &Value) -> Option<Self> {
        let position = serde_json::from_value(v["position"].clone()).ok()?;
        let name = v["name"].as_str()?.to_owned();
        let map_color = v["mapColor"].as_u64().unwrap_or(0) as u32;
        let boxes = serde_json::from_value::<Vec<Bounds>>(v["boxes"].clone())
            .ok()?
            .into_iter()
            .filter(|b| b.iter().all(|v| v.is_finite()) && (0..3).all(|i| b[i] < b[i + 3]))
            .collect();
        Some(Self {
            position,
            name,
            map_color,
            boxes,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn cube(x: i32, name: &str, map: u32) -> Block {
        Block {
            position: [x, 0, 0],
            name: name.into(),
            map_color: map,
            boxes: vec![[x as f64, 0., 0., x as f64 + 1., 1., 1.]],
        }
    }
    #[test]
    fn default_target_filter_live_shapes_merge_and_colours() {
        let mut s = Settings::default();
        assert_eq!(s.targets.len(), 17);
        assert!(!s.glow);
        let blocks = [
            cube(0, "minecraft:red_bed", 0xff0000),
            cube(1, "minecraft:red_bed", 0xff0000),
            cube(2, "minecraft:stone", 0x808080),
        ];
        let f = s.frame(&blocks, [0.; 3], 0.);
        assert_eq!(f["quads"].as_array().unwrap().len(), 12);
        assert!(f["effect"].is_null());
        assert_eq!(f["quads"][0][0]["color"][0], 1.);
        s.merge_adjacent = true;
        let merged = s.frame(&blocks, [0.; 3], 0.);
        assert!(merged["quads"].as_array().unwrap().len() < 12);
        assert!(merged["lines"].as_array().unwrap().len() < f["lines"].as_array().unwrap().len());
        s.color_mode = ColorMode::Static;
        s.color = 0x32abcdef;
        assert_eq!(s.color(0, 0.), 0x32abcdef);
        s.color_mode = ColorMode::Rainbow;
        assert_eq!(s.color(0, 0.), 0x32ff0000);
        assert_eq!(s.color(0, 5_000_000_000.), 0x3200ffff);
    }
    #[test]
    fn glow_opacity_style_and_distance_fade_change_output() {
        let mut s = Settings::default();
        s.glow = true;
        s.style = 0;
        let blocks = [cube(1, "minecraft:dragon_egg", 0x010203)];
        let f = s.frame(&blocks, [0.; 3], 0.);
        assert_eq!(f["effect"]["style"], 9);
        assert!(f["lines"].as_array().unwrap().is_empty());
        assert_eq!(f["quads"][0][0]["color"][3], 1.);
        s.fade = [0., 0., 0., 1.];
        let f = s.frame(&blocks, [0.; 3], 0.);
        assert_eq!(f["quads"][0][0]["color"][3], 0.);
        s.set_fade(0, 400.);
        assert_eq!(s.fade[0], 0.);
        s.set_fade(3, 0.5);
        assert_eq!(s.fade[3], 0.5);
    }
}
