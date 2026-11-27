// SPDX-License-Identifier: GPL-3.0-or-later
//! StrongholdFinder from the fixed LiquidBounce source: deterministic hypotheses,
//! log-space posterior, consumed eye throws and ordered entity sampling.
use std::collections::{BTreeMap, VecDeque};
pub type Position = [f64; 3];
pub type Chunk = [i32; 2];
pub type Hypothesis = [Chunk; 128];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub sigma: f32,
    pub hypothesis_count: i32,
    pub require_same: bool,
    pub sample_delay: i32,
    pub min_speed: f32,
    pub max_age: i32,
    pub max_spawn_distance: f32,
    pub top_candidates: i32,
    pub render_rays: bool,
    pub render_best: bool,
    pub render_top: bool,
    pub announce: bool,
    pub reset_on_world_change: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sigma: 0.03,
            hypothesis_count: 20_000,
            require_same: true,
            sample_delay: 2,
            min_speed: 0.02,
            max_age: 20,
            max_spawn_distance: 8.,
            top_candidates: 3,
            render_rays: true,
            render_best: true,
            render_top: true,
            announce: true,
            reset_on_world_change: true,
        }
    }
}
/// java.util.SplittableRandom(long), including bounded nextInt rejection draws.
struct Splittable(u64);
impl Splittable {
    fn seed(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        self.0
    }
    fn double(&mut self) -> f64 {
        let mut z = self.seed();
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((z ^ (z >> 31)) >> 11) as f64 * (1. / 9_007_199_254_740_992.)
    }
    fn int(&mut self) -> i32 {
        let mut z = self.seed();
        z = (z ^ (z >> 33)).wrapping_mul(0x62a9d9ed799705f5);
        ((z ^ (z >> 28)).wrapping_mul(0xcb24d0a5c88c35b3) >> 32) as i32
    }
    fn snap(&mut self) -> i32 {
        let mut u = (self.int() as u32 >> 1) as i32;
        loop {
            let r = u % 15;
            if u.wrapping_add(14).wrapping_sub(r) >= 0 {
                return r - 7;
            }
            u = (self.int() as u32 >> 1) as i32;
        }
    }
}
pub fn generate_single(seed: u64) -> Hypothesis {
    // Mth.TWO_PI is a float. In angle += TWO_PI/currentSpread the division
    // also takes place in float, before Kotlin promotes the result to double.
    let two_pi = std::f32::consts::PI * 2.;
    let mut random = Splittable(seed);
    let mut angle = random.double() * f64::from(two_pi);
    let (mut ring, mut spread, mut in_ring) = (0, 3, 0);
    std::array::from_fn(|index| {
        let radius = f64::from(128 + 192 * ring) + (random.double() - 0.5) * 80.;
        let x = (angle.cos() * radius + 0.5).floor() as i32 + random.snap();
        let z = (angle.sin() * radius + 0.5).floor() as i32 + random.snap();
        angle += f64::from(two_pi / spread as f32);
        in_ring += 1;
        if in_ring == spread {
            ring += 1;
            in_ring = 0;
            spread += 2 * spread / (ring + 1);
            spread = spread.min(128 - index as i32);
            angle += random.double() * f64::from(two_pi);
        }
        [x, z]
    })
}
pub fn generate(count: usize) -> Vec<Hypothesis> {
    (0..count)
        .map(|i| generate_single(0x51f15eed + i as u64))
        .collect()
}
pub fn yaw(dx: f64, dz: f64) -> f32 {
    (-dx).atan2(dz) as f32 * (180. / std::f32::consts::PI)
}
fn wrap(mut degrees: f32) -> f32 {
    degrees %= 360.;
    if degrees >= 180. {
        degrees -= 360.;
    }
    if degrees < -180. {
        degrees += 360.;
    }
    degrees
}
fn center(chunk: Chunk) -> [f64; 2] {
    chunk.map(|v| f64::from(v) * 16. + 8.)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    pub position: Position,
    pub angle: f32,
    pub tick: i32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub chunk: Chunk,
    pub probability: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Posterior {
    pub candidates: Vec<Candidate>,
    pub confidence: f64,
    pub sample_count: usize,
}
pub fn estimate(
    samples: &[Measurement],
    hypotheses: &[Hypothesis],
    sigma: f64,
    same: bool,
    top: usize,
) -> Option<Posterior> {
    if samples.is_empty() || hypotheses.is_empty() || sigma <= 0. || top == 0 {
        return None;
    }
    let mut scores = Vec::with_capacity(hypotheses.len());
    for h in hypotheses {
        let mut first = None;
        let mut counts = [0usize; 128];
        let mut weight = 0.;
        let mut valid = true;
        for m in samples {
            let mut nearest = None;
            let mut distance = f64::INFINITY;
            for (i, &chunk) in h.iter().enumerate() {
                let [x, z] = center(chunk);
                let d = (x - m.position[0]).powi(2) + (z - m.position[2]).powi(2);
                if d < distance {
                    distance = d;
                    nearest = Some(i);
                }
            }
            let Some(i) = nearest else {
                valid = false;
                break;
            };
            if same && first.is_some_and(|j| j != i) {
                valid = false;
                break;
            }
            first.get_or_insert(i);
            counts[i] += 1;
            let [x, z] = center(h[i]);
            let delta = f64::from(wrap(m.angle - yaw(x - m.position[0], z - m.position[2])));
            weight -= delta * delta / (2. * sigma * sigma);
        }
        if !valid {
            continue;
        }
        let i = if same {
            first?
        } else {
            // Kotlin maxByOrNull retains the first index on ties.
            let mut best = 0;
            for j in 1..128 {
                if counts[j] > counts[best] {
                    best = j;
                }
            }
            best
        };
        scores.push((h[i], weight));
    }
    let max = scores.iter().map(|(_, w)| *w).reduce(f64::max)?;
    let mut chunks = BTreeMap::<Chunk, f64>::new();
    let mut sum = 0.;
    for (chunk, w) in scores {
        let linear = (w - max).exp();
        *chunks.entry(chunk).or_default() += linear;
        sum += linear;
    }
    if sum <= 0. {
        return None;
    }
    let mut candidates = chunks
        .into_iter()
        .map(|(chunk, w)| Candidate {
            chunk,
            probability: w / sum,
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| b.probability.total_cmp(&a.probability));
    candidates.truncate(top);
    Some(Posterior {
        confidence: candidates.first()?.probability,
        candidates,
        sample_count: samples.len(),
    })
}
#[derive(Clone, Copy, Debug)]
struct Pending {
    position: Position,
    tick: i32,
    dimension: i32,
}
#[derive(Clone, Copy, Debug)]
struct Tracked {
    id: i32,
    position: Position,
    spawn_tick: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Eye {
    pub id: i32,
    pub position: Position,
    pub velocity: Position,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Notice {
    Sample(usize),
    Best(Candidate),
}
#[derive(Default)]
pub struct StrongholdFinder {
    pub settings: Settings,
    pending: VecDeque<Pending>,
    tracked: Vec<Tracked>,
    pub measurements: Vec<Measurement>,
    pub posterior: Option<Posterior>,
    pub portals: BTreeMap<[i32; 3], bool>, // true = portal, false = frame
    hypotheses: Vec<Hypothesis>,
    cached_count: i32,
    announced: Option<Chunk>,
}
impl StrongholdFinder {
    pub fn clear(&mut self) {
        self.pending.clear();
        self.tracked.clear();
        self.measurements.clear();
        self.posterior = None;
        self.announced = None;
        self.portals.clear();
    }
    pub fn world_change(&mut self) {
        if self.settings.reset_on_world_change {
            self.clear();
        }
    }
    fn trim(&mut self, tick: i32) {
        while self
            .pending
            .front()
            .is_some_and(|p| tick.wrapping_sub(p.tick) > self.settings.max_age)
        {
            self.pending.pop_front();
        }
    }
    pub fn used(&mut self, position: Position, tick: i32, dimension: i32) {
        self.trim(tick);
        self.pending.push_back(Pending {
            position,
            tick,
            dimension,
        });
    }
    pub fn spawn(&mut self, id: i32, at: Position, tick: i32, dimension: i32) {
        self.trim(tick);
        let range = f64::from(self.settings.max_spawn_distance * self.settings.max_spawn_distance);
        let distance =
            |p: &Pending| (p.position[0] - at[0]).powi(2) + (p.position[2] - at[2]).powi(2);
        let nearest = self
            .pending
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.dimension == dimension
                    && (0..=self.settings.max_age).contains(&tick.wrapping_sub(p.tick))
                    && distance(p) <= range
            })
            .min_by(|(_, a), (_, b)| {
                distance(a)
                    .total_cmp(&distance(b))
                    .then_with(|| tick.wrapping_sub(a.tick).cmp(&tick.wrapping_sub(b.tick)))
            })
            .map(|(i, _)| i);
        if let Some(i) = nearest {
            let p = self.pending.remove(i).unwrap();
            let track = Tracked {
                id,
                position: p.position,
                spawn_tick: tick,
            };
            if let Some(old) = self.tracked.iter_mut().find(|t| t.id == id) {
                *old = track;
            } else {
                self.tracked.push(track);
            }
        }
    }
    pub fn tick(&mut self, tick: i32, eyes: &[Eye]) -> Vec<Notice> {
        self.trim(tick);
        let mut notices = Vec::new();
        let mut i = 0;
        while i < self.tracked.len() {
            let t = self.tracked[i];
            if tick.wrapping_sub(t.spawn_tick) < self.settings.sample_delay {
                i += 1;
                continue;
            }
            let Some(eye) = eyes.iter().find(|e| e.id == t.id) else {
                self.tracked.remove(i);
                continue;
            };
            if (eye.velocity[0].hypot(eye.velocity[2]) as f32) < self.settings.min_speed {
                i += 1;
                continue;
            }
            self.measurements.push(Measurement {
                position: t.position,
                angle: yaw(
                    eye.position[0] - t.position[0],
                    eye.position[2] - t.position[2],
                ),
                tick,
            });
            self.tracked.remove(i);
            notices.push(Notice::Sample(self.measurements.len()));
            if let Some(best) = self.recompute(true) {
                notices.push(Notice::Best(best));
            }
        }
        notices
    }
    pub fn settings_changed(&mut self) {
        if !self.measurements.is_empty() {
            self.recompute(false);
        }
    }
    fn recompute(&mut self, announce: bool) -> Option<Candidate> {
        if self.hypotheses.is_empty() || self.cached_count != self.settings.hypothesis_count {
            self.hypotheses = generate(self.settings.hypothesis_count.max(0) as usize);
            self.cached_count = self.settings.hypothesis_count;
        }
        self.posterior = estimate(
            &self.measurements,
            &self.hypotheses,
            f64::from(self.settings.sigma),
            self.settings.require_same,
            self.settings.top_candidates.max(0) as usize,
        );
        let best = *self.posterior.as_ref()?.candidates.first()?;
        if announce && self.settings.announce && self.announced != Some(best.chunk) {
            self.announced = Some(best.chunk);
            Some(best)
        } else {
            None
        }
    }
    pub fn block(&mut self, pos: [i32; 3], name: &str) {
        match name {
            "minecraft:end_portal" => {
                self.portals.insert(pos, true);
            }
            "minecraft:end_portal_frame" => {
                self.portals.insert(pos, false);
            }
            _ => {
                self.portals.remove(&pos);
            }
        }
    }
    pub fn forget(&mut self, chunk: Chunk) {
        self.portals.retain(|p, _| [p[0] >> 4, p[2] >> 4] != chunk);
    }
}
