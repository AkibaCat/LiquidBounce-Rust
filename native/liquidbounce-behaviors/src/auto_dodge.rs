// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoDodge, DodgePlanner and Minecraft-Agent SimulatedArrow, pinned sources.
#[path = "navigation/walk_jump/java_math.rs"]
mod java_math;
pub type V = [f64; 3];
pub const SAFE: f64 = 1.5 * 0.3 + 1.5 * 0.5;
pub const PADDED: f64 = 0.3 * 5.;
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub rotation: bool,
    pub jump: bool,
    pub timer: bool,
    pub speed: f32,
    pub ignore: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            rotation: false,
            jump: true,
            timer: false,
            speed: 2.,
            ignore: 7,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Arrow {
    pub id: u32,
    pub position: V,
    pub velocity: V,
    pub stopped: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub tick: i32,
    pub id: u32,
    pub position: V,
    pub previous: V,
    pub velocity: V,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub keys: [bool; 4],
    pub jump: bool,
    pub yaw: Option<f32>,
    pub timer: bool,
}
pub fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: V, s: f64) -> V {
    a.map(|v| v * s)
}
pub fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn length(a: V) -> f64 {
    dot(a, a).sqrt()
}
pub fn similarity(a: V, b: V) -> f64 {
    dot(a, b) / (length(a) * length(b))
}
pub fn yaw(v: V) -> f32 {
    (java_math::fdlibm_atan2(-v[0], v[2]) as f32) * f32::from_bits(0x4265_2ee0)
}
pub fn wrap(v: f32) -> f32 {
    let v = v % 360.;
    if v >= 180. {
        v - 360.
    } else if v < -180. {
        v + 360.
    } else {
        v
    }
}
pub fn directional(degrees: f32) -> [bool; 4] {
    [
        degrees > -70. && degrees < 70.,
        degrees < -110. || degrees > 110.,
        degrees > -160. && degrees < -20.,
        degrees > 20. && degrees < 160.,
    ]
}
pub fn heading(mut yaw: f32, k: [bool; 4]) -> f32 {
    let factor = if k[1] && !k[0] {
        yaw += 180.;
        -0.5
    } else if k[0] && !k[1] {
        0.5
    } else {
        1.
    };
    if k[2] && !k[3] {
        yaw -= 90. * factor;
    }
    if k[3] && !k[2] {
        yaw += 90. * factor;
    }
    yaw
}
/// Minecraft AABB.clip: entry faces only, epsilon 1e-7, no inside-origin hit.
pub fn clip_box(from: V, to: V, min: V, max: V) -> Option<V> {
    let d = sub(to, from);
    let mut scale = 1.;
    let mut hit = false;
    for a in 0..3 {
        if d[a].abs() <= 1e-7 {
            continue;
        }
        let b = (a + 1) % 3;
        let c = (a + 2) % 3;
        let t = ((if d[a] > 0. { min[a] } else { max[a] }) - from[a]) / d[a];
        let pb = from[b] + t * d[b];
        let pc = from[c] + t * d[c];
        if 0. < t
            && t < scale
            && min[b] - 1e-7 < pb
            && pb < max[b] + 1e-7
            && min[c] - 1e-7 < pc
            && pc < max[c] + 1e-7
        {
            scale = t;
            hit = true;
        }
    }
    hit.then(|| add(from, mul(d, scale)))
}
pub fn inflicted(
    positions: impl IntoIterator<Item = V>,
    arrows: &[Arrow],
    expansion: f64,
    mut block: impl FnMut(V, V) -> Option<V>,
) -> Option<Hit> {
    let mut arrows = arrows.to_vec();
    for (tick, player) in positions.into_iter().enumerate() {
        for arrow in &mut arrows {
            if arrow.stopped {
                continue;
            }
            let old = arrow.position;
            let next = add(old, arrow.velocity);
            arrow.velocity = [
                arrow.velocity[0] * 0.99,
                arrow.velocity[1] * 0.99 - 0.05000000074505806,
                arrow.velocity[2] * 0.99,
            ];
            arrow.position = if let Some(at) = block(old, next) {
                arrow.stopped = true;
                at
            } else {
                next
            };
            let min = add(player, [-0.3 - expansion, -expansion, -0.3 - expansion]);
            let max = add(player, [0.3 + expansion, 1.8 + expansion, 0.3 + expansion]);
            if let Some(at) = clip_box(old, arrow.position, min, max) {
                return Some(Hit {
                    tick: tick as i32,
                    id: arrow.id,
                    position: at,
                    previous: old,
                    velocity: arrow.velocity,
                });
            }
        }
    }
    None
}
/// Packet index increments even for samples rejected by the 0.9 distance rule.
pub fn avoiding(
    positions: &[V],
    player: V,
    mut hit: impl FnMut(V) -> Option<Hit>,
) -> Option<(i32, bool)> {
    let mut last = None;
    let mut best = None;
    let mut best_time = 0;
    for (i, &position) in positions.iter().enumerate() {
        if last.is_some_and(|last| dot(sub(last, position), sub(last, position)) < 0.9 * 0.9) {
            continue;
        }
        last = Some(position);
        match hit(position) {
            None => return Some((i as i32, false)),
            Some(h) if h.tick > best_time => {
                best_time = h.tick;
                best = Some((i, position));
            }
            _ => {}
        }
    }
    best.filter(|(_, p)| dot(sub(*p, player), sub(*p, player)) > 0.9 * 0.9)
        .map(|(i, _)| (i as i32, true))
}
pub fn plan(
    player: V,
    velocity: V,
    player_yaw: f32,
    frame_yaw: f32,
    allow_rotation: bool,
    hit: Hit,
    mut walk: impl FnMut(V, V) -> f64,
    mut direction: impl FnMut(f32) -> V,
) -> Option<Plan> {
    let v = [hit.velocity[0], 0., hit.velocity[2]];
    let vv = dot(v, v);
    if vv.abs() < 1e-5 {
        return None;
    }
    let base = [hit.previous[0], 0., hit.previous[2]];
    let p = [player[0], 0., player[2]];
    let nearest = |base: V, point: V| add(base, mul(v, dot(sub(point, base), v) / vv));
    let centre = nearest(base, p);
    let distance = length(sub(centre, p));
    if distance > PADDED {
        return None;
    }
    let future = add(p, [velocity[0] * 2., 0., velocity[2] * 2.]);
    let ortho = mul([-v[2], 0., v[0]], PADDED / length(v));
    let candidates = [
        nearest(sub(base, ortho), future),
        nearest(add(base, ortho), future),
    ];
    let chosen = if walk(centre, candidates[0]) < SAFE {
        candidates[1]
    } else if walk(centre, candidates[1]) < SAFE {
        candidates[0]
    } else if length(sub(candidates[0], future)) < length(sub(candidates[1], future)) - 0.05 {
        candidates[0]
    } else {
        candidates[1]
    };
    let relative = sub(chosen, p);
    let keys = directional(wrap(yaw(relative) - wrap(frame_yaw)));
    let mut plan = Plan {
        keys,
        jump: false,
        yaw: None,
        timer: false,
    };
    if distance > SAFE {
        return Some(plan);
    }
    let effective = similarity(direction(heading(player_yaw, keys)), relative);
    let travel = length(relative) - (PADDED - SAFE);
    let time = hit.tick + 1;
    if time as f64 > travel / (effective * 0.11) {
        return Some(plan);
    }
    plan.timer = (travel / (if allow_rotation { 0.155 } else { 0.11 })) / (time + 1) as f64 > 1.6;
    if allow_rotation {
        plan.keys = [true, false, false, false];
        plan.jump = (time as f64) < travel / 0.13
            && length(velocity) * similarity(velocity, relative) > 0.11;
        let radians = java_math::fdlibm_atan2(relative[2], relative[0]);
        plan.yaw = Some(wrap(
            (radians * f64::from(f32::from_bits(0x4265_2ee0))) as f32 - 90.,
        ));
    }
    Some(plan)
}
