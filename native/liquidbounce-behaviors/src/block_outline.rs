// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleBlockOutline and its ShapeSurfaceMesh consumer, LiquidBounce ae0eb396.
use crate::Easing;
#[path = "navigation/java_case.rs"]
mod java_case;
#[path = "block_outline_surface.rs"]
mod surface;
pub type Point = [f64; 3];
pub type Bounds = [f64; 6];
pub fn easing_tag(tag: &str) -> Option<Easing> {
    Easing::ALL
        .into_iter()
        .find(|v| java_case::equals_ignore_case(v.tag(), tag))
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub quads: Vec<[Point; 4]>,
    pub lines: Vec<[Point; 2]>,
}
pub struct Target<'a> {
    pub boxes: &'a [Bounds],
    pub edges: &'a [[Point; 2]],
    /// DOWN, UP, NORTH, SOUTH, WEST, EAST, matching Direction.entries.
    pub side: u8,
    pub hit: Point,
}
#[derive(Clone, Debug, PartialEq)]
pub struct BlockOutline {
    pub side_only: bool,
    pub color: i32,
    pub outline: i32,
    pub slide: bool,
    pub time: i32,
    pub easing: Easing,
    current: Option<Bounds>,
    previous: Option<Bounds>,
    last_change: i64,
}
impl Default for BlockOutline {
    fn default() -> Self {
        Self {
            side_only: true,
            color: 0x464677ff,
            outline: 0x964677ffu32 as i32,
            slide: true,
            time: 150,
            easing: Easing::Linear,
            current: None,
            previous: None,
            last_change: 0,
        }
    }
}
impl BlockOutline {
    fn reset(&mut self) {
        self.current = None;
        self.previous = None;
    }
    pub fn world_change(&mut self, now: i64) {
        self.reset();
        self.last_change = now;
    }
    // Source has no onDisabled override: it retains its animation positions.
    pub fn render(&mut self, target: Option<Target<'_>>, now: i64) -> Mesh {
        let Some(target) = target.filter(|t| !t.boxes.is_empty()) else {
            self.reset();
            return Mesh::default();
        };
        let shape = surface::Surface::new(target.boxes);
        if let Some(mut bounds) = shape.single_box() {
            if self.side_only {
                let axis = [1, 1, 2, 2, 0, 0][target.side as usize];
                let at = bounds[axis + if target.side % 2 == 1 { 3 } else { 0 }];
                bounds[axis] = at;
                bounds[axis + 3] = at;
            }
            if self.current != Some(bounds) {
                self.previous = self.current;
                self.current = Some(bounds);
                self.last_change = now;
            }
            let rendered = if self.slide {
                self.previous
                    .map(|previous| {
                        let f = f64::from(self.easing.get_factor(
                            self.last_change,
                            now,
                            self.time as f32,
                        ));
                        std::array::from_fn(|i| previous[i] + f * (bounds[i] - previous[i]))
                    })
                    .unwrap_or(bounds)
            } else {
                bounds
            };
            surface::box_mesh(rendered, self.side_only.then_some(target.side))
        } else {
            self.reset();
            let mut mesh = shape.mesh(self.side_only.then_some((target.side, target.hit)));
            if !self.side_only {
                mesh.lines.extend_from_slice(target.edges);
            }
            mesh
        }
    }
}
