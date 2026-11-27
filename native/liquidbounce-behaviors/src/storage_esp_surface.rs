// SPDX-License-Identifier: GPL-3.0-or-later
//! StorageESP voxel union surface, derived from ShapeExtensions.kt.
use super::{Bounds, Mesh, Point};
const EPSILON: f64 = 1.0e-7;
pub(super) struct Surface {
    axes: [Vec<f64>; 3],
    sizes: [usize; 3],
    cells: Vec<bool>,
}
fn face(axis: usize, bounds: Bounds) -> [Point; 4] {
    let u = if axis == 0 { 2 } else { 0 };
    let v = if axis == 1 { 2 } else { 1 };
    let mut points = [[bounds[0], bounds[1], bounds[2]]; 4];
    points[1][u] = bounds[u + 3];
    points[2][u] = bounds[u + 3];
    points[2][v] = bounds[v + 3];
    points[3][v] = bounds[v + 3];
    points
}
pub(super) fn box_mesh(bounds: Bounds, side: Option<u8>) -> Mesh {
    let mut mesh = Mesh::default();
    for side in side.map(|s| vec![s]).unwrap_or_else(|| (0..6).collect()) {
        let axis = [1, 1, 2, 2, 0, 0][side as usize];
        let mut b = bounds;
        let at = b[axis + if side % 2 == 1 { 3 } else { 0 }];
        b[axis] = at;
        b[axis + 3] = at;
        let q = face(axis, b);
        mesh.quads.push(q);
    }
    // A single-side box has four selected edges even when its extents are flat.
    if side.is_some() {
        let q = mesh.quads[0];
        mesh.lines.extend((0..4).map(|i| [q[i], q[(i + 1) % 4]]));
    } else {
        for axis in 0..3 {
            let a = (axis + 1) % 3;
            let b = (axis + 2) % 3;
            for bits in 0..4 {
                let mut start = [bounds[0], bounds[1], bounds[2]];
                start[a] = bounds[a + if bits & 1 != 0 { 3 } else { 0 }];
                start[b] = bounds[b + if bits & 2 != 0 { 3 } else { 0 }];
                let mut end = start;
                end[axis] = bounds[axis + 3];
                mesh.lines.push([start, end]);
            }
        }
    }
    mesh
}
impl Surface {
    pub(super) fn new(boxes: &[Bounds]) -> Self {
        let axes = std::array::from_fn(|a| {
            let mut v = boxes
                .iter()
                .flat_map(|b| [b[a], b[a + 3]])
                .collect::<Vec<_>>();
            v.sort_by(f64::total_cmp);
            v.dedup();
            v
        });
        let sizes = axes.each_ref().map(|a| a.len().saturating_sub(1));
        let mut result = Self {
            axes,
            sizes,
            cells: vec![false; sizes.iter().product()],
        };
        for x in 0..sizes[0] {
            for y in 0..sizes[1] {
                for z in 0..sizes[2] {
                    let p = [x, y, z];
                    let mid = std::array::from_fn::<_, 3, _>(|i| {
                        (result.axes[i][p[i]] + result.axes[i][p[i] + 1]) * 0.5
                    });
                    let occupied = boxes
                        .iter()
                        .any(|b| (0..3).all(|i| mid[i] >= b[i] && mid[i] < b[i + 3]));
                    let index = result.index([x, y, z]);
                    result.cells[index] = occupied;
                }
            }
        }
        result
    }
    fn index(&self, p: [usize; 3]) -> usize {
        (p[0] * self.sizes[1] + p[1]) * self.sizes[2] + p[2]
    }
    fn occupied(&self, p: [isize; 3]) -> bool {
        if (0..3).any(|a| p[a] < 0 || p[a] >= self.sizes[a] as isize) {
            return false;
        }
        self.cells[self.index(p.map(|v| v as usize))]
    }
    pub(super) fn single_box(&self) -> Option<Bounds> {
        (!self.cells.is_empty() && self.cells.iter().all(|v| *v)).then(|| {
            std::array::from_fn(|i| {
                if i < 3 {
                    self.axes[i][0]
                } else {
                    *self.axes[i - 3].last().unwrap()
                }
            })
        })
    }
    fn point(&self, axis: usize, plane: usize, u: usize, v: usize) -> Point {
        match axis {
            1 => [self.axes[0][u], self.axes[1][plane], self.axes[2][v]],
            2 => [self.axes[0][u], self.axes[1][v], self.axes[2][plane]],
            _ => [self.axes[0][plane], self.axes[1][v], self.axes[2][u]],
        }
    }
    fn mask(&self, side: u8, plane: usize) -> Mask {
        let axis = [1, 1, 2, 2, 0, 0][side as usize];
        let u = if axis == 0 { 2 } else { 0 };
        let v = if axis == 1 { 2 } else { 1 };
        let width = self.sizes[u];
        let height = self.sizes[v];
        let mut cells = vec![false; width * height];
        for y in 0..height {
            for x in 0..width {
                let mut p = [0isize; 3];
                p[u] = x as isize;
                p[v] = y as isize;
                p[axis] = plane as isize - if side % 2 == 1 { 1 } else { 0 };
                let mut q = p;
                q[axis] += if side % 2 == 1 { 1 } else { -1 };
                cells[y * width + x] = self.occupied(p) && !self.occupied(q);
            }
        }
        Mask {
            width,
            height,
            cells,
        }
    }
    pub(super) fn mesh(&self, selected: Option<(u8, Point)>) -> Mesh {
        let mut mesh = Mesh::default();
        for side in selected
            .map(|(s, _)| vec![s])
            .unwrap_or_else(|| (0..6).collect())
        {
            let axis = [1, 1, 2, 2, 0, 0][side as usize];
            let start = usize::from(side % 2 == 1);
            for plane in start..self.sizes[axis] + start {
                let mut mask = self.mask(side, plane);
                if let Some((_, hit)) = selected {
                    let seed = (0..mask.height)
                        .flat_map(|v| (0..mask.width).map(move |u| (u, v)))
                        .find(|&(u, v)| {
                            if !mask.get(u, v) {
                                return false;
                            }
                            let min = self.point(axis, plane, u, v);
                            let max = self.point(axis, plane, u + 1, v + 1);
                            (hit[axis] - min[axis]).abs() <= EPSILON
                                && (0..3).all(|a| {
                                    hit[a] >= min[a] - EPSILON && hit[a] <= max[a] + EPSILON
                                })
                        });
                    let Some(seed) = seed else {
                        continue;
                    };
                    mask = mask.component(seed);
                    for [u0, v0, u1, v1] in mask.perimeter() {
                        mesh.lines.push([
                            self.point(axis, plane, u0, v0),
                            self.point(axis, plane, u1, v1),
                        ]);
                    }
                }
                for [u0, v0, u1, v1] in mask.rectangles() {
                    let min = self.point(axis, plane, u0, v0);
                    let max = self.point(axis, plane, u1, v1);
                    let b = [min[0], min[1], min[2], max[0], max[1], max[2]];
                    mesh.quads.push(face(axis, b));
                }
                if selected.is_some() {
                    return mesh;
                }
            }
        }
        if selected.is_none() {
            mesh.lines = self.edges();
        }
        mesh
    }
    fn edges(&self) -> Vec<[Point; 2]> {
        let mut out = Vec::new();
        for a in 0..3 {
            let u = (a + 1) % 3;
            let v = (a + 2) % 3;
            for j in 0..=self.sizes[u] {
                for k in 0..=self.sizes[v] {
                    let mut start = None;
                    for i in 0..=self.sizes[a] {
                        let edge = if i == self.sizes[a] {
                            false
                        } else {
                            let mut occ = [false; 4];
                            for bits in 0..4 {
                                let mut p = [0isize; 3];
                                p[a] = i as isize;
                                p[u] = j as isize - if bits & 1 == 0 { 1 } else { 0 };
                                p[v] = k as isize - if bits & 2 == 0 { 1 } else { 0 };
                                occ[bits] = self.occupied(p);
                            }
                            let n = occ.iter().filter(|x| **x).count();
                            n == 1 || n == 3 || (n == 2 && occ[0] == occ[3])
                        };
                        if edge && start.is_none() {
                            start = Some(i);
                        }
                        if !edge {
                            if let Some(s) = start.take() {
                                let mut p = [0.; 3];
                                p[a] = self.axes[a][s];
                                p[u] = self.axes[u][j];
                                p[v] = self.axes[v][k];
                                let mut q = p;
                                q[a] = self.axes[a][i];
                                out.push([p, q]);
                            }
                        }
                    }
                }
            }
        }
        out
    }
}
struct Mask {
    width: usize,
    height: usize,
    cells: Vec<bool>,
}
impl Mask {
    fn get(&self, u: usize, v: usize) -> bool {
        self.cells[v * self.width + u]
    }
    fn component(&self, (u, v): (usize, usize)) -> Self {
        let mut cells = vec![false; self.cells.len()];
        let mut stack = vec![v * self.width + u];
        while let Some(i) = stack.pop() {
            if cells[i] || !self.cells[i] {
                continue;
            }
            cells[i] = true;
            let (u, v) = (i % self.width, i / self.width);
            if u > 0 {
                stack.push(i - 1);
            }
            if u + 1 < self.width {
                stack.push(i + 1);
            }
            if v > 0 {
                stack.push(i - self.width);
            }
            if v + 1 < self.height {
                stack.push(i + self.width);
            }
        }
        Self {
            width: self.width,
            height: self.height,
            cells,
        }
    }
    fn rectangles(&self) -> Vec<[usize; 4]> {
        let mut visited = vec![false; self.cells.len()];
        let mut output = Vec::new();
        for v in 0..self.height {
            let mut u = 0;
            while u < self.width {
                if !self.get(u, v) || visited[v * self.width + u] {
                    u += 1;
                    continue;
                }
                let mut end_u = u + 1;
                while end_u < self.width && self.get(end_u, v) && !visited[v * self.width + end_u] {
                    end_u += 1;
                }
                let mut end_v = v + 1;
                while end_v < self.height
                    && (u..end_u).all(|x| self.get(x, end_v) && !visited[end_v * self.width + x])
                {
                    end_v += 1;
                }
                for y in v..end_v {
                    for x in u..end_u {
                        visited[y * self.width + x] = true;
                    }
                }
                output.push([u, v, end_u, end_v]);
                u = end_u;
            }
        }
        output
    }
    fn perimeter(&self) -> Vec<[usize; 4]> {
        let mut out = Vec::new();
        for row in 0..self.height {
            for upper in [false, true] {
                let mut start = None;
                for u in 0..=self.width {
                    let boundary = u < self.width
                        && self.get(u, row)
                        && if upper {
                            row + 1 == self.height || !self.get(u, row + 1)
                        } else {
                            row == 0 || !self.get(u, row - 1)
                        };
                    if boundary {
                        start.get_or_insert(u);
                    } else if let Some(begin) = start.take() {
                        let v = row + usize::from(upper);
                        out.push([begin, v, u, v]);
                    }
                }
            }
        }
        for column in 0..self.width {
            for right in [false, true] {
                let mut start = None;
                for v in 0..=self.height {
                    let boundary = v < self.height
                        && self.get(column, v)
                        && if right {
                            column + 1 == self.width || !self.get(column + 1, v)
                        } else {
                            column == 0 || !self.get(column - 1, v)
                        };
                    if boundary {
                        start.get_or_insert(v);
                    } else if let Some(begin) = start.take() {
                        let u = column + usize::from(right);
                        out.push([u, begin, u, v]);
                    }
                }
            }
        }
        out
    }
}
