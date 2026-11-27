// SPDX-License-Identifier: GPL-3.0-or-later
//! Pinned HoleTracker, HoleManager and MovableRegionScanner semantics.
use std::collections::{BTreeMap, HashMap};
pub type Pos = [i32; 3];
pub const INDESTRUCTIBLE: i8 = -2;
pub const BLAST_RESISTANT: i8 = -1;
pub const AIR: i8 = 0;
pub const BREAKABLE: i8 = 1;
// Direction.BY_2D_DATA, not the enum's declaration order.
const HORIZONTAL: [Pos; 4] = [[0, 0, 1], [-1, 0, 0], [0, 0, -1], [1, 0, 0]];
fn add(a: Pos, b: Pos) -> Pos {
    std::array::from_fn(|i| a[i] + b[i])
}
fn opposite(p: Pos) -> Pos {
    p.map(|v| -v)
}
fn order(p: Pos) -> (i32, i32, i32) {
    (p[1], p[2], p[0])
}
pub fn classify(air: bool, resistance: f32) -> i8 {
    if air {
        AIR
    } else if resistance >= 3_600_000. {
        INDESTRUCTIBLE
    } else if resistance >= 600. {
        BLAST_RESISTANT
    } else {
        BREAKABLE
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub min: Pos,
    pub max: Pos,
}
impl Default for Region {
    fn default() -> Self {
        Self {
            min: [0; 3],
            max: [0; 3],
        }
    }
}
impl Region {
    pub fn around(p: Pos, h: i32, v: i32) -> Self {
        Self {
            min: [p[0] - h, p[1] - v, p[2] - h],
            max: [p[0] + h, p[1] + v, p[2] + h],
        }
    }
    pub fn chunk(x: i32, z: i32, min_y: i32, max_y: i32) -> Self {
        Self {
            min: [x * 16, min_y, z * 16],
            max: [x * 16 + 15, max_y, z * 16 + 15],
        }
    }
    pub fn intersects(self, r: Self) -> bool {
        (0..3).all(|i| self.min[i] <= r.max[i] && self.max[i] >= r.min[i])
    }
    pub fn contains(self, p: Pos) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }
    pub fn difference(self, old: Self) -> Vec<Self> {
        if old == self || old.contains(self.min) && old.contains(self.max) {
            return vec![];
        }
        if !old.intersects(self) {
            return vec![self];
        }
        let mut out = vec![];
        for axis in 0..3 {
            if self.max[axis] > old.max[axis] {
                let mut r = self;
                r.min[axis] = old.max[axis] + 1;
                out.push(r);
            }
            if self.min[axis] < old.min[axis] {
                let mut r = self;
                r.max[axis] = old.min[axis] - 1;
                out.push(r);
            }
        }
        out
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hole {
    pub position: Pos,
    pub size: [i32; 2],
    pub kind: u8,
}
impl Hole {
    pub fn region(self) -> Region {
        Region {
            min: self.position,
            max: [
                self.position[0] + self.size[0] - 1,
                self.position[1],
                self.position[2] + self.size[1] - 1,
            ],
        }
    }
    pub fn cells(self) -> impl Iterator<Item = Pos> {
        (0..self.size[1]).flat_map(move |z| {
            (0..self.size[0])
                .map(move |x| [self.position[0] + x, self.position[1], self.position[2] + z])
        })
    }
    pub fn invalidated_by_filling(self, p: Pos) -> bool {
        let mut r = self.region();
        r.max[1] += 2;
        r.contains(p)
    }
}
#[derive(Default, Debug)]
pub struct HoleTracker {
    pub holes: BTreeMap<(i32, i32, i32), Hole>,
}
struct Buffer<'a, F> {
    states: HashMap<Pos, i8>,
    read: &'a mut F,
}
impl<F: FnMut(Pos) -> i8> Buffer<'_, F> {
    fn state(&mut self, p: Pos) -> i8 {
        *self.states.entry(p).or_insert_with(|| (self.read)(p))
    }
    fn resistant(&mut self, p: Pos) -> bool {
        matches!(self.state(p), BLAST_RESISTANT | INDESTRUCTIBLE)
    }
    fn column(&mut self, p: Pos) -> bool {
        self.resistant(add(p, [0, -1, 0])) && (0..3).all(|y| self.state(add(p, [0, y, 0])) == AIR)
    }
    fn walls(&mut self, p: Pos, dirs: impl IntoIterator<Item = Pos>) -> bool {
        dirs.into_iter().all(|d| self.resistant(add(p, d)))
    }
    fn check(&mut self, p: Pos, dirs: [Pos; 2]) -> bool {
        self.column(p) && self.walls(p, dirs)
    }
}
impl HoleTracker {
    fn contains(&self, p: Pos) -> bool {
        [0, -1].into_iter().any(|z| {
            [0, -1].into_iter().any(|x| {
                self.holes
                    .get(&order(add(p, [x, 0, z])))
                    .is_some_and(|h| h.region().contains(p))
            })
        })
    }
    pub fn clear(&mut self) {
        self.holes.clear();
    }
    pub fn invalidate(&mut self, r: Region) {
        self.holes.retain(|_, h| !h.region().intersects(r));
    }
    pub fn scan(&mut self, r: Region, max_y: i32, read: &mut impl FnMut(Pos) -> i8) {
        let mut b = Buffer {
            states: HashMap::new(),
            read,
        };
        // BlockPos.betweenClosed: X first, then Y, then Z.
        for z in r.min[2]..=r.max[2] {
            for y in r.min[1]..=r.max[1].min(max_y - 3) {
                for x in r.min[0]..=r.max[0] {
                    let p = [x, y, z];
                    if self.contains(p) || !b.column(p) {
                        continue;
                    }
                    let resistant = HORIZONTAL.map(|d| b.resistant(add(p, d)));
                    let mut hole = None;
                    match resistant.into_iter().filter(|v| *v).count() {
                        4 => {
                            let bedrock = HORIZONTAL
                                .into_iter()
                                .chain([[0, -1, 0]])
                                .all(|d| b.state(add(p, d)) == INDESTRUCTIBLE);
                            hole = Some(Hole {
                                position: p,
                                size: [1, 1],
                                kind: if bedrock { 0 } else { 1 },
                            });
                        }
                        3 => {
                            let d = HORIZONTAL[resistant.iter().position(|v| !*v).unwrap()];
                            let other = add(p, d);
                            if b.column(other)
                                && b.walls(
                                    other,
                                    HORIZONTAL.into_iter().filter(|v| *v != opposite(d)),
                                )
                            {
                                hole = Some(Hole {
                                    position: if order(p) < order(other) { p } else { other },
                                    size: if d[0] == 0 { [1, 2] } else { [2, 1] },
                                    kind: 2,
                                });
                            }
                        }
                        2 => {
                            let dirs = HORIZONTAL
                                .into_iter()
                                .zip(resistant)
                                .filter_map(|(d, v)| (!v).then_some(d))
                                .collect::<Vec<_>>();
                            let (a, c) = (dirs[0], dirs[1]);
                            let diagonal = add(add(p, a), c);
                            if b.check(add(p, a), [a, opposite(c)])
                                && b.check(add(p, c), [c, opposite(a)])
                                && b.check(diagonal, [a, c])
                            {
                                hole = Some(Hole {
                                    position: [x.min(diagonal[0]), y, z.min(diagonal[2])],
                                    size: [2, 2],
                                    kind: 3,
                                });
                            }
                        }
                        _ => {}
                    }
                    if let Some(h) = hole {
                        self.holes.entry(order(h.position)).or_insert(h);
                    }
                }
            }
        }
    }
    pub fn record_block(
        &mut self,
        p: Pos,
        state: i8,
        max_y: i32,
        read: &mut impl FnMut(Pos) -> i8,
    ) {
        if state == AIR {
            self.holes.retain(|_, h| {
                !h.cells()
                    .any(|c| (0..3).map(|i| (c[i] - p[i]).abs()).sum::<i32>() == 1)
            });
        } else {
            self.holes.retain(|_, h| !h.invalidated_by_filling(p));
        }
        let r = Region {
            min: [p[0] - 2, p[1] - 3, p[2] - 2],
            max: [p[0] + 2, p[1] + 3, p[2] + 2],
        };
        self.invalidate(r);
        self.scan(r, max_y, read);
    }
    pub fn chunk_update(
        &mut self,
        r: Region,
        manager_region: Region,
        max_y: i32,
        read: &mut impl FnMut(Pos) -> i8,
    ) {
        if r.intersects(manager_region) {
            self.invalidate(r);
            self.scan(r, max_y, read);
        }
    }
}
#[derive(Default, Debug)]
pub struct HoleManager {
    pub tracker: HoleTracker,
    pub region: Region,
    player: Pos,
    // Reference subscriber identity; max ranges are evaluated only at region changes.
    subscribers: BTreeMap<u64, (i32, i32)>,
}
impl HoleManager {
    pub fn active(&self) -> bool {
        !self.subscribers.is_empty()
    }
    pub fn subscribe(
        &mut self,
        id: u64,
        h: i32,
        v: i32,
        p: Pos,
        max_y: i32,
        read: &mut impl FnMut(Pos) -> i8,
    ) {
        let first = !self.active();
        self.subscribers.insert(id, (h, v));
        if first {
            self.update(p, max_y, read);
        }
    }
    pub fn settings(&mut self, id: u64, h: i32, v: i32) {
        if let Some(value) = self.subscribers.get_mut(&id) {
            *value = (h, v);
        }
    }
    pub fn unsubscribe(&mut self, id: u64) {
        self.subscribers.remove(&id);
        if !self.active() {
            self.region = Region::default();
            self.tracker.clear();
        }
    }
    pub fn player_post_tick(&mut self, p: Pos, max_y: i32, read: &mut impl FnMut(Pos) -> i8) {
        if self.active() && (0..3).map(|i| (p[i] - self.player[i]).abs()).sum::<i32>() >= 4 {
            self.update(p, max_y, read);
        }
    }
    fn update(&mut self, p: Pos, max_y: i32, read: &mut impl FnMut(Pos) -> i8) {
        self.player = p;
        let h = self.subscribers.values().map(|v| v.0).max().unwrap_or(0);
        let v = self.subscribers.values().map(|v| v.1).max().unwrap_or(0);
        let next = Region::around(p, h, v);
        let differences = next.difference(self.region);
        self.region = next;
        if differences.is_empty() {
            return;
        }
        self.tracker
            .holes
            .retain(|_, hole| hole.region().intersects(next));
        for r in differences {
            self.tracker.scan(r, max_y, read);
        }
    }
}
