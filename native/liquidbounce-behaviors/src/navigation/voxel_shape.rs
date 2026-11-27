// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent 0abf69ac VoxelShape and Shapes. Raw box ordering, including
//! inverted/degenerate boxes, is retained separately from normalized AABBs.
use super::{
    aabb::{Aabb, Axis},
    geometry::{java_max, java_min, Vec3},
};
use std::sync::{Arc, OnceLock};
#[derive(Debug)]
struct Data {
    boxes: Vec<[f64; 6]>,
    aabbs: OnceLock<Vec<Aabb>>,
}
#[derive(Clone, Debug)]
pub struct VoxelShape(Arc<Data>);
impl PartialEq for VoxelShape {
    fn eq(&self, other: &Self) -> bool {
        self.same_instance(other)
            || self.0.boxes.len() == other.0.boxes.len()
                && self
                    .0
                    .boxes
                    .iter()
                    .zip(&other.0.boxes)
                    .all(|(a, b)| a.iter().zip(b).all(|(&a, &b)| bits(a) == bits(b)))
    }
}
impl Eq for VoxelShape {}
fn bits(v: f64) -> u64 {
    if v.is_nan() {
        f64::NAN.to_bits()
    } else {
        v.to_bits()
    }
}
impl VoxelShape {
    pub fn new(boxes: Vec<[f64; 6]>) -> Self {
        Self(Arc::new(Data {
            boxes,
            aabbs: OnceLock::new(),
        }))
    }
    pub fn same_instance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn is_empty(&self) -> bool {
        self.0.boxes.is_empty()
    }
    pub fn box_count(&self) -> usize {
        self.0.boxes.len()
    }
    pub fn aabbs(&self) -> &[Aabb] {
        self.0.aabbs.get_or_init(|| {
            self.0
                .boxes
                .iter()
                .map(|b| Aabb::new(Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5])))
                .collect()
        })
    }
    pub fn to_array(&self) -> Vec<f64> {
        self.0.boxes.iter().flatten().copied().collect()
    }
    pub fn min(&self, axis: Axis) -> f64 {
        self.0
            .boxes
            .iter()
            .fold(f64::INFINITY, |min, b| java_min(min, b[axis.index()]))
    }
    pub fn max(&self, axis: Axis) -> f64 {
        self.0.boxes.iter().fold(f64::NEG_INFINITY, |max, b| {
            java_max(max, b[axis.index() + 3])
        })
    }
    pub fn bounds(&self) -> Option<Aabb> {
        (!self.is_empty()).then(|| {
            Aabb::new(
                Vec3::new(self.min(Axis::X), self.min(Axis::Y), self.min(Axis::Z)),
                Vec3::new(self.max(Axis::X), self.max(Axis::Y), self.max(Axis::Z)),
            )
        })
    }
    pub fn moved(&self, offset: Vec3) -> Self {
        if self.is_empty() {
            return self.clone();
        }
        Self::new(
            self.0
                .boxes
                .iter()
                .map(|b| {
                    [
                        b[0] + offset.x,
                        b[1] + offset.y,
                        b[2] + offset.z,
                        b[3] + offset.x,
                        b[4] + offset.y,
                        b[5] + offset.z,
                    ]
                })
                .collect(),
        )
    }
    pub fn collide(&self, axis: Axis, body: Aabb, distance: f64) -> f64 {
        if self.is_empty() {
            return distance;
        }
        if distance.abs() < 1.0e-7 {
            return 0.0;
        }
        let a = axis.index();
        let b = (a + 1) % 3;
        let c = (a + 2) % 3;
        let mut moved = distance;
        for coords in &self.0.boxes {
            if coords[3 + b] - 1.0e-7 <= body.min(Axis::ALL[b])
                || coords[b] + 1.0e-7 >= body.max(Axis::ALL[b])
                || coords[3 + c] - 1.0e-7 <= body.min(Axis::ALL[c])
                || coords[c] + 1.0e-7 >= body.max(Axis::ALL[c])
            {
                continue;
            }
            if moved > 0.0 {
                let gap = coords[a] - body.max(axis);
                if gap > -1.0e-7 {
                    moved = java_min(moved, gap);
                }
            } else {
                let gap = coords[3 + a] - body.min(axis);
                if gap < 1.0e-7 {
                    moved = java_max(moved, gap);
                }
            }
        }
        moved
    }
    pub fn block() -> Self {
        static BLOCK: OnceLock<VoxelShape> = OnceLock::new();
        BLOCK
            .get_or_init(|| Self::new(vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]))
            .clone()
    }
    pub fn empty() -> Self {
        static EMPTY: OnceLock<VoxelShape> = OnceLock::new();
        EMPTY.get_or_init(|| Self::new(Vec::new())).clone()
    }
    pub fn box_shape(coords: [f64; 6]) -> Self {
        if coords == [0.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
            Self::block()
        } else {
            Self::new(vec![coords])
        }
    }
    pub fn collide_all<'a>(
        axis: Axis,
        body: Aabb,
        shapes: impl IntoIterator<Item = &'a Self>,
        distance: f64,
    ) -> f64 {
        let mut moved = distance;
        for shape in shapes {
            if moved.abs() < 1.0e-7 {
                return 0.0;
            }
            moved = shape.collide(axis, body, moved);
        }
        moved
    }
}
