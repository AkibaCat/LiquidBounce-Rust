// SPDX-License-Identifier: GPL-3.0-or-later
// PlacementRenderer, PlacementRenderHandler and BlockCuller, LiquidBounce ae0eb396.
use crate::Easing;
use std::collections::BTreeMap;
pub type Pos = [i32; 3];
pub type Box6 = [f64; 6];
pub const FULL: Box6 = [0., 0., 0., 1., 1., 1.];
#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry {
    start: i64,
    faces: u32,
    edges: u32,
    bounds: Box6,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw {
    pub position: Pos,
    pub bounds: Box6,
    pub color: u32,
    pub outline: u32,
    pub faces: u32,
    pub edges: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlacementRenderer {
    pub enabled: bool,
    pub clump: bool,
    pub start_size: f32,
    pub start_curve: Easing,
    pub end_size: f32,
    pub end_curve: Easing,
    pub fade_in_curve: Easing,
    pub fade_out_curve: Easing,
    pub in_time: i32,
    pub out_time: i32,
    pub color: i32,
    pub outline: i32,
    pub keep: bool,
    incoming: BTreeMap<Pos, Entry>,
    current: BTreeMap<Pos, Entry>,
    outgoing: BTreeMap<Pos, Entry>,
    out_finished: bool,
}
fn offset(p: Pos, d: Pos) -> Pos {
    std::array::from_fn(|i| p[i].wrapping_add(d[i]))
}
fn fade(color: i32, factor: f32) -> u32 {
    let color = color as u32;
    if factor >= 1. {
        color
    } else {
        (color & 0x00ffffff) | ((((color >> 24) as f32 * factor) as i32 as u32 & 255) << 24)
    }
}
fn bounds(expand: f32, b: Box6) -> Box6 {
    if expand == 1. {
        return b;
    }
    if expand == 0. {
        return [0.; 6];
    }
    let factor = if expand < 1. {
        -0.5 * f64::from(expand)
    } else {
        f64::from(expand - 1.) * 0.5
    };
    std::array::from_fn(|i| {
        let a = i % 3;
        let amount = (b[a + 3] - b[a]) * factor;
        if i < 3 { b[i] - amount } else { b[i] + amount }
    })
}
impl PlacementRenderer {
    pub fn new(keep: bool, color: i32) -> Self {
        Self {
            enabled: true,
            clump: true,
            start_size: 1.,
            start_curve: Easing::Linear,
            end_size: 0.8,
            end_curve: Easing::Linear,
            fade_in_curve: Easing::Linear,
            fade_out_curve: Easing::Linear,
            in_time: 500,
            out_time: 500,
            color,
            outline: (color as u32 | 0xff000000) as i32,
            keep,
            incoming: Default::default(),
            current: Default::default(),
            outgoing: Default::default(),
            out_finished: true,
        }
    }
    pub fn contains(&self, p: Pos) -> bool {
        self.incoming.contains_key(&p)
            || self.current.contains_key(&p)
            || self.outgoing.contains_key(&p)
    }
    pub fn clear(&mut self) {
        self.incoming.clear();
        self.current.clear();
        self.outgoing.clear();
    }
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
        if !value {
            self.clear();
        }
    }
    pub fn running(&self, module_running: bool) -> bool {
        (module_running && self.enabled) || !self.out_finished
    }
    pub fn tick(&mut self) {
        if !self.out_finished && self.outgoing.is_empty() {
            self.out_finished = true;
        }
    }
    pub fn clear_silently(&mut self, time: i64) {
        if !self.enabled {
            self.clear();
        }
        for (p, mut e) in std::mem::take(&mut self.incoming)
            .into_iter()
            .chain(std::mem::take(&mut self.current))
        {
            e.start = time;
            self.outgoing.insert(p, e);
        }
        self.out_finished = false;
    }
    pub fn add(&mut self, p: Pos, b: Box6, time: i64, update: bool) {
        if !self.enabled || b == [0.; 6] {
            return;
        }
        if !self.current.contains_key(&p) && !self.incoming.contains_key(&p) {
            self.incoming.insert(
                p,
                Entry {
                    start: time,
                    faces: u32::MAX,
                    edges: u32::MAX,
                    bounds: b,
                },
            );
            if update {
                self.update_neighbors(p);
            }
        }
        self.outgoing.remove(&p);
    }
    pub fn remove(&mut self, p: Pos, time: i64) {
        if !self.enabled {
            return;
        }
        if let Some(mut e) = self.current.remove(&p).or_else(|| self.incoming.remove(&p)) {
            e.start = time;
            self.outgoing.insert(p, e);
        }
    }
    pub fn update_box(&mut self, p: Pos, b: Box6) {
        if !self.enabled {
            return;
        }
        for map in [&mut self.incoming, &mut self.current] {
            if let Some(e) = map.get_mut(&p) {
                e.bounds = b;
            }
        }
    }
    pub fn cull(&self, p: Pos) -> (u32, u32) {
        let directions = [
            [0, -1, 0],
            [0, 1, 0],
            [0, 0, -1],
            [1, 0, 0],
            [0, 0, 1],
            [-1, 0, 0],
        ];
        let present = directions.map(|d| self.contains(offset(p, d)));
        let mut faces = 0;
        let mut edges = 0;
        for (i, yes) in present.into_iter().enumerate() {
            if !yes {
                faces |= 15 << (i * 4);
            }
        }
        for (i, [a, b]) in [
            [2, 0],
            [3, 0],
            [4, 0],
            [5, 0],
            [2, 5],
            [2, 3],
            [4, 3],
            [4, 5],
            [2, 1],
            [3, 1],
            [4, 1],
            [5, 1],
        ]
        .into_iter()
        .enumerate()
        {
            let diagonal = self.contains(offset(offset(p, directions[a]), directions[b]));
            if (!present[a] && !present[b]) || (present[a] && present[b] && !diagonal) {
                edges |= 3 << (i * 2);
            }
        }
        (faces, edges)
    }
    fn update_neighbors(&mut self, p: Pos) {
        if !self.clump {
            return;
        }
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let p = offset(p, [x, y, z]);
                    let (faces, edges) = self.cull(p);
                    if let Some(e) = self
                        .incoming
                        .get_mut(&p)
                        .or_else(|| self.current.get_mut(&p))
                    {
                        e.faces = faces;
                        e.edges = edges;
                    }
                }
            }
        }
    }
    pub fn update_all(&mut self) {
        if !self.clump {
            return;
        }
        let keys = self
            .incoming
            .keys()
            .chain(self.current.keys())
            .copied()
            .collect::<Vec<_>>();
        for p in keys {
            let (faces, edges) = self.cull(p);
            if let Some(e) = self
                .incoming
                .get_mut(&p)
                .or_else(|| self.current.get_mut(&p))
            {
                e.faces = faces;
                e.edges = edges;
            }
        }
    }
    fn draw(&self, p: Pos, e: Entry, b: Box6, alpha: f32) -> Draw {
        Draw {
            position: p,
            bounds: b,
            color: fade(self.color, alpha),
            outline: fade(self.outline, alpha),
            faces: e.faces,
            edges: e.edges,
        }
    }
    pub fn render(&mut self, time: i64) -> Vec<Draw> {
        let mut draws = Vec::new();
        for (p, e) in self.incoming.clone() {
            let t = self
                .start_curve
                .get_factor(e.start, time, self.in_time as f32);
            let expand = self.start_size + t * (1. - self.start_size);
            draws.push(
                self.draw(
                    p,
                    e,
                    bounds(if expand < 1. { 1. - expand } else { expand }, e.bounds),
                    self.fade_in_curve
                        .get_factor(e.start, time, self.in_time as f32),
                ),
            );
            if time.wrapping_sub(e.start) >= i64::from(self.in_time) {
                self.incoming.remove(&p);
                if self.keep {
                    self.current.insert(p, e);
                } else {
                    self.outgoing.insert(p, Entry { start: time, ..e });
                }
            }
        }
        for (&p, &e) in &self.current {
            draws.push(self.draw(p, e, e.bounds, 1.));
        }
        let mut removed = Vec::new();
        for (p, e) in self.outgoing.clone() {
            let t = self
                .end_curve
                .get_factor(e.start, time, self.out_time as f32);
            let expand = 1. - (1. + t * (self.end_size - 1.));
            draws.push(
                self.draw(
                    p,
                    e,
                    bounds(expand, e.bounds),
                    1. - self
                        .fade_out_curve
                        .get_factor(e.start, time, self.out_time as f32),
                ),
            );
            if time.wrapping_sub(e.start) >= i64::from(self.out_time) {
                self.outgoing.remove(&p);
                removed.push(p);
            }
        }
        for p in removed {
            self.update_neighbors(p);
        }
        draws
    }
}
