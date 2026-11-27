// SPDX-License-Identifier: GPL-3.0-or-later
//! NoWeb collision decisions and PlaceWater's ordered placement/pickup state.
use std::collections::VecDeque;
pub type Pos = [i32; 3];
pub type Point = [f64; 3];
pub const FACES: [Pos; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];
pub const MODES: [&str; 5] = [
    "Air",
    "Grim-2.3.65",
    "Intave-14.8.4",
    "PlaceWater",
    "Strafe",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Air,
    Grim,
    Intave,
    PlaceWater,
    Strafe,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(v: &str) -> Option<Self> {
        Some(match v {
            "Air" => Self::Air,
            "Grim-2.3.65" | "Grim2365" => Self::Grim,
            "Intave-14.8.4" | "Intave14" => Self::Intave,
            "PlaceWater" => Self::PlaceWater,
            "Strafe" => Self::Strafe,
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub break_on_world: bool,
    pub strength: f32,
    pub only_on_ground: bool,
    pub motion_y: bool,
    pub motion_y_strength: f32,
    pub pickup: bool,
    pub pickup_span: [f32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Air,
            break_on_world: true,
            strength: 0.23,
            only_on_ground: false,
            motion_y: false,
            motion_y_strength: 0.6,
            pickup: true,
            pickup_span: [0.8, 3.],
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Collision {
    pub cancel: bool,
    pub break_world: bool,
    pub destroy_packets: bool,
    pub jump: bool,
    pub speed: Option<f64>,
    pub blend: f64,
    pub y: Option<f64>,
}
#[derive(Clone, Debug, Default)]
pub struct NoWeb {
    pub settings: Settings,
    pub webs: VecDeque<Pos>,
    pub pickups: VecDeque<(Pos, u64)>,
    pub last_success: Option<(Pos, u64)>,
}
impl NoWeb {
    pub fn collision(
        &mut self,
        running: bool,
        pos: Pos,
        moving: bool,
        ground: bool,
        tick: i32,
    ) -> Collision {
        let mut e = Collision {
            blend: 1.,
            ..Default::default()
        };
        if !running {
            return e;
        }
        match self.settings.mode {
            Mode::Air => e.cancel = true,
            Mode::Grim => {
                e.cancel = true;
                e.break_world = self.settings.break_on_world;
                e.destroy_packets = true;
            }
            Mode::Intave => {
                if moving && ground {
                    if tick % 3 == 0 {
                        e.blend = 0.734;
                    } else {
                        e.jump = true;
                        e.blend = 0.346;
                    }
                    e.speed = Some(-1.);
                }
            }
            Mode::Strafe => {
                if moving {
                    if ground || !self.settings.only_on_ground {
                        e.speed = Some(self.settings.strength as f64);
                    }
                    if self.settings.motion_y {
                        e.y = Some(self.settings.motion_y_strength as f64);
                    }
                }
            }
            Mode::PlaceWater => {
                if !self.webs.contains(&pos) {
                    self.webs.push_back(pos);
                }
                while self.webs.len() > 8 {
                    self.webs.pop_front();
                }
            }
        }
        e
    }
    pub fn reset(&mut self) {
        self.webs.clear();
        self.pickups.clear();
    }
    pub fn record(&mut self, pos: Pos, now: u64) {
        self.pickups.push_back((pos, now));
        while self.pickups.len() > 16 {
            self.pickups.pop_front();
        }
    }
    pub fn prune(&mut self, now: u64, max: u64, w: &impl World) {
        self.pickups
            .retain(|(p, t)| now.saturating_sub(*t) <= max && w.block(*p).water_source);
    }
    pub fn plan(&mut self, w: &impl World, p: Player, now: u64) -> Option<Action> {
        if p.evaporates {
            return None;
        }
        self.webs.retain(|pos| w.block(*pos).web);
        if let Some(slot) = p.water_slot {
            for web in self.webs.iter().copied() {
                if self
                    .last_success
                    .is_some_and(|(q, t)| q == web && now.saturating_sub(t) <= 250)
                {
                    continue;
                }
                if let Some(action) = place(w, p, web, slot) {
                    return Some(action);
                }
            }
        }
        if !self.settings.pickup {
            return None;
        }
        self.prune(now, (self.settings.pickup_span[1] * 1000.) as u64, w);
        let delay = (self.settings.pickup_span[0] * 1000.) as u64;
        let pos = self
            .pickups
            .iter()
            .find(|(pos, t)| {
                now.saturating_sub(*t) > delay && box_distance(*pos, p.eyes) <= p.reach * p.reach
            })?
            .0;
        Some(Action {
            slot: p.bucket_slot?,
            hit: Hit {
                pos,
                face: FACES[1],
                point: center(pos),
            },
            kind: ActionKind::Pickup { pos },
        })
    }
    pub fn success(&mut self, action: Action, hit: Hit, now: u64, w: &impl World) {
        match action.kind {
            ActionKind::Top { web, target, .. } => {
                self.mark(web, now);
                self.record(target, now);
            }
            ActionKind::Directional { web, side } => {
                self.mark(web, now);
                let mut candidates = vec![add(hit.pos, hit.face), web, add(web, side)];
                for d in FACES {
                    candidates.push(add(web, d));
                }
                let mut unique = Vec::new();
                for p in candidates {
                    if !unique.contains(&p) {
                        unique.push(p);
                        self.record(p, now);
                    }
                }
            }
            ActionKind::Pickup { .. } => self.prune(now, 0, w),
        }
    }
    fn mark(&mut self, web: Pos, now: u64) {
        self.webs.retain(|p| *p != web);
        self.last_success = Some((web, now));
    }
}
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub web: bool,
    pub air: bool,
    pub lava_source: bool,
    pub water_source: bool,
    pub replaceable: bool,
    pub center_support: bool,
    pub outline: Vec<[f64; 6]>,
}
pub trait World {
    fn block(&self, p: Pos) -> Block;
}
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub eyes: Point,
    pub look: Point,
    pub reach: f64,
    pub evaporates: bool,
    pub water_slot: Option<i32>,
    pub bucket_slot: Option<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub pos: Pos,
    pub face: Pos,
    pub point: Point,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionKind {
    Top { web: Pos, target: Pos, min_y: f64 },
    Directional { web: Pos, side: Pos },
    Pickup { pos: Pos },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub slot: i32,
    pub hit: Hit,
    pub kind: ActionKind,
}
impl Action {
    pub fn rotation(self, eyes: Point) -> [f32; 2] {
        let d = std::array::from_fn::<_, 3, _>(|i| self.hit.point[i] - eyes[i]);
        [
            (d[2].atan2(d[0]).to_degrees() - 90.) as f32,
            (-d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees()) as f32,
        ]
    }
    pub fn resolve(self, ray: Option<Hit>, fluid: Option<Hit>) -> Option<Hit> {
        match self.kind {
            ActionKind::Top { min_y, .. } => ray.filter(|r| {
                r.pos == self.hit.pos && r.face == self.hit.face && r.point[1] >= min_y - 1e-7
            }),
            ActionKind::Directional { web, side } => Some(
                ray.filter(|r| {
                    (r.pos == web && r.face == side)
                        || (r.pos == add(web, side) && r.face == side.map(|v| -v))
                })
                .unwrap_or(self.hit),
            ),
            ActionKind::Pickup { pos } => Some(if let Some(r) = fluid.filter(|r| r.pos == pos) {
                r
            } else if let Some(r) = ray.filter(|r| r.pos == pos) {
                r
            } else if let Some(r) = ray.filter(|r| add(r.pos, r.face) == pos) {
                Hit {
                    pos,
                    face: r.face.map(|v| -v),
                    point: center(pos),
                }
            } else {
                self.hit
            }),
        }
    }
}
pub fn add(a: Pos, b: Pos) -> Pos {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn center(p: Pos) -> Point {
    p.map(|v| v as f64 + 0.5)
}
fn box_distance(p: Pos, eye: Point) -> f64 {
    (0..3)
        .map(|i| {
            let d = (p[i] as f64 - eye[i])
                .max(0.)
                .max(eye[i] - (p[i] as f64 + 1.));
            d * d
        })
        .sum()
}
fn dot(a: Point, b: Pos) -> f64 {
    (0..3).map(|i| a[i] * b[i] as f64).sum()
}
fn place(w: &impl World, p: Player, web: Pos, slot: i32) -> Option<Action> {
    let inside = (0..3).all(|i| p.eyes[i] >= web[i] as f64 && p.eyes[i] < (web[i] + 1) as f64);
    if !inside && p.eyes[1] > (web[1] + 1) as f64 {
        return top(w, p, web, slot);
    }
    let inside_dirs = [FACES[1], FACES[4], FACES[5], FACES[2], FACES[3]];
    let horizontal = [FACES[3], FACES[4], FACES[2], FACES[5]];
    let mut best = None;
    let mut score = f64::NEG_INFINITY;
    for side in if inside {
        &inside_dirs[..]
    } else {
        &horizontal[..]
    } {
        let b = w.block(add(web, *side));
        if !(b.air || b.lava_source) {
            continue;
        }
        let d = dot(p.look, *side);
        if d > score {
            best = Some(*side);
            score = d;
        }
    }
    let side = best?;
    let point = std::array::from_fn(|i| web[i] as f64 + 0.5 + side[i] as f64 * 0.5);
    Some(Action {
        slot,
        hit: Hit {
            pos: web,
            face: side,
            point,
        },
        kind: ActionKind::Directional { web, side },
    })
}
fn top(w: &impl World, p: Player, web: Pos, slot: i32) -> Option<Action> {
    let target = add(web, FACES[1]);
    if w.block(target).center_support {
        return None;
    }
    let mut best = None;
    let mut best_facing = f64::NEG_INFINITY;
    for face in FACES {
        let neighbor = add(target, face.map(|v| -v));
        for clicked in [neighbor, target] {
            let b = w.block(clicked);
            let mut selected = None;
            let mut metric = (f64::NEG_INFINITY, f64::NEG_INFINITY);
            for bounds in b.outline {
                let axis = face.iter().position(|v| *v != 0).unwrap();
                let mut point =
                    std::array::from_fn::<_, 3, _>(|i| (bounds[i] + bounds[i + 3]) * 0.5);
                point[axis] = if face[axis] > 0 {
                    bounds[axis + 3]
                } else {
                    bounds[axis]
                };
                if (p.eyes[axis] - clicked[axis] as f64 - point[axis]) * (face[axis] as f64) < 0. {
                    continue;
                }
                let m = ((point[axis] - 0.5).powi(2), point[1]);
                if m > metric {
                    metric = m;
                    selected = Some((point, bounds[1]));
                }
            }
            let Some((local, min_y)) = selected else {
                continue;
            };
            let point = std::array::from_fn(|i| local[i] + clicked[i] as f64);
            let placed = if b.replaceable {
                clicked
            } else {
                add(clicked, face)
            };
            // BlockPlaceContext: a non-BlockItem can place when the target is replaceable.
            if placed != target || !w.block(target).replaceable {
                continue;
            }
            let delta = std::array::from_fn::<_, 3, _>(|i| p.eyes[i] - point[i]);
            let distance = delta.iter().map(|x| x * x).sum::<f64>().sqrt();
            let facing = dot(delta, face) / distance;
            if facing < 0. || facing <= best_facing {
                continue;
            }
            best_facing = facing;
            best = Some(Action {
                slot,
                hit: Hit {
                    pos: clicked,
                    face,
                    point,
                },
                kind: ActionKind::Top {
                    web,
                    target,
                    min_y: min_y + clicked[1] as f64,
                },
            });
        }
    }
    best
}
