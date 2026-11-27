// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTrajectories, descriptors, shot resolution and simulation at ae0eb396.
//! Collision queries belong to the host; physics uses the same source in both crates.
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub type V3 = [f64; 3];
pub const TYPES: [&str; 11] = [
    "Arrow",
    "Potion",
    "EnderPearl",
    "FishingBobber",
    "Trident",
    "Snowball",
    "Egg",
    "ExpBottle",
    "FireworkRocket",
    "Fireball",
    "WindCharge",
];
pub const SHOW: [&str; 4] = [
    "AlwaysShowBow",
    "OtherPlayers",
    "ActiveTrajectoryArrow",
    "ActiveTrajectoryOther",
];
#[derive(Clone, Debug)]
pub struct Spec {
    pub path: &'static str,
    pub kind: &'static str,
    pub default: Value,
    pub range: Option<[f32; 2]>,
    pub choices: Vec<&'static str>,
    pub suffix: &'static str,
}
pub fn specs() -> Vec<Spec> {
    fn n(
        path: &'static str,
        kind: &'static str,
        default: Value,
        range: Option<[f32; 2]>,
        choices: Vec<&'static str>,
        suffix: &'static str,
    ) -> Spec {
        Spec {
            path,
            kind,
            default,
            range,
            choices,
            suffix,
        }
    }
    vec![
        n(
            "MaxSimulatedTicks",
            "INT",
            json!(240),
            Some([1., 1000.]),
            vec![],
            "ticks",
        ),
        n(
            "MaxRenderDistance",
            "INT",
            json!(96),
            Some([16., 512.]),
            vec![],
            "m",
        ),
        n(
            "CullBehindPlayer",
            "BOOLEAN",
            json!(false),
            None,
            vec![],
            "",
        ),
        n("ShowMultiShot", "BOOLEAN", json!(true), None, vec![], ""),
        n("LineWidth", "FLOAT", json!(1.), Some([1., 16.]), vec![], ""),
        n(
            "ActiveLineWidth",
            "FLOAT",
            json!(2.),
            Some([1., 16.]),
            vec![],
            "",
        ),
        n(
            "TrajectoryTypes",
            "MULTI_CHOOSE",
            json!(TYPES),
            None,
            TYPES.to_vec(),
            "",
        ),
        n(
            "Show",
            "MULTI_CHOOSE",
            json!(["OtherPlayers", "ActiveTrajectoryArrow"]),
            None,
            SHOW.to_vec(),
            "",
        ),
        n(
            "ShowDetailedInfo.Enabled",
            "BOOLEAN",
            json!(false),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.ShowAt",
            "CHOOSE",
            json!("Entity"),
            None,
            vec!["Owner", "Entity", "Landing"],
            "",
        ),
        n(
            "ShowDetailedInfo.Item",
            "BOOLEAN",
            json!(true),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.OwnerName",
            "BOOLEAN",
            json!(true),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.Distance",
            "BOOLEAN",
            json!(true),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.TimeUnit",
            "CHOOSE",
            json!("Ticks"),
            None,
            vec!["Ticks", "Seconds"],
            "",
        ),
        n(
            "ShowDetailedInfo.Color",
            "COLOR",
            json!(-1),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.Scale",
            "CURVE",
            json!([{"x":0.,"y":1.},{"x":200.,"y":1.}]),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.RenderOffset",
            "VECTOR3_D",
            json!({"x":0.,"y":0.,"z":0.}),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.RenderOffscreen.Enabled",
            "BOOLEAN",
            json!(true),
            None,
            vec![],
            "",
        ),
        n(
            "ShowDetailedInfo.RenderOffscreen.EdgeInset",
            "FLOAT",
            json!(0.),
            Some([0., 0.9]),
            vec![],
            "",
        ),
    ]
}
#[derive(Clone, Debug, PartialEq)]
pub struct Trajectories {
    pub values: BTreeMap<String, Value>,
    pub simulation_results: Vec<Value>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TrajectoriesSetting(pub String, pub Value);
impl Default for Trajectories {
    fn default() -> Self {
        Self {
            values: specs()
                .into_iter()
                .map(|s| (s.path.into(), s.default))
                .collect(),
            simulation_results: vec![],
        }
    }
}
impl TrajectoriesSetting {
    pub fn apply(self, s: &mut Trajectories) {
        if s.values.contains_key(&self.0) {
            s.values.insert(self.0, self.1);
        }
    }
}
impl Trajectories {
    pub fn value(&self, p: &str) -> Value {
        self.values.get(p).cloned().unwrap_or_default()
    }
    pub fn boolean(&self, p: &str) -> bool {
        self.values.get(p).and_then(Value::as_bool).unwrap_or(false)
    }
    pub fn number(&self, p: &str) -> f64 {
        self.values.get(p).and_then(Value::as_f64).unwrap_or(0.)
    }
    pub fn text(&self, p: &str) -> &str {
        self.values.get(p).and_then(Value::as_str).unwrap_or("")
    }
    pub fn includes(&self, p: &str, name: &str) -> bool {
        self.values
            .get(p)
            .and_then(Value::as_array)
            .is_some_and(|v| v.iter().any(|s| s == name))
    }
    pub fn reset(&mut self) {
        self.simulation_results.clear();
    }
    pub fn options(&self) -> Value {
        json!(self.values)
    }
    pub fn culled(&self, pos: V3, camera: V3, forward: V3) -> bool {
        let d = sub(pos, camera);
        let n = dot(d, d);
        n > self.number("MaxRenderDistance").powi(2)
            || (self.boolean("CullBehindPlayer") && dot(d, forward) < 0. && n > 9.)
    }
}
pub fn add(a: V3, b: V3) -> V3 {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: V3, b: V3) -> V3 {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: V3, n: f64) -> V3 {
    a.map(|x| x * n)
}
pub fn dot(a: V3, b: V3) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub fn distance(a: V3, b: V3) -> f64 {
    dot(sub(a, b), sub(a, b)).sqrt()
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Info {
    pub gravity: f64,
    pub radius: f64,
    pub initial_velocity: f64,
    pub drag: f64,
    pub water_drag: f64,
    pub roll: f32,
    pub copies_player_velocity: bool,
}
impl Info {
    pub fn for_type(kind: &str, real: bool) -> Self {
        let generic = Self {
            gravity: 0.03,
            radius: 0.25,
            initial_velocity: 1.5,
            drag: 0.99,
            water_drag: 0.8,
            roll: 0.,
            copies_player_velocity: true,
        };
        let persistent = Self {
            gravity: 0.05,
            radius: 0.5,
            water_drag: 0.6,
            ..generic
        };
        match kind {
            "Arrow" => Self {
                radius: if real { 0.3 } else { 0.5 },
                initial_velocity: 3.,
                ..persistent
            },
            "Potion" => Self {
                gravity: 0.05,
                initial_velocity: 0.5,
                roll: -20.,
                ..generic
            },
            "ExpBottle" => Self {
                gravity: 0.07,
                initial_velocity: 0.7,
                roll: -20.,
                ..generic
            },
            "FishingBobber" => Self {
                gravity: 0.04,
                drag: 0.92,
                water_drag: 0.92,
                ..generic
            },
            "Trident" => Self {
                initial_velocity: 2.5,
                water_drag: 0.99,
                ..persistent
            },
            "FireworkRocket" => Self {
                gravity: 0.,
                initial_velocity: 1.6,
                drag: 1.,
                water_drag: 1.,
                copies_player_velocity: false,
                ..generic
            },
            "Fireball" => Self {
                gravity: 0.,
                radius: 1.,
                water_drag: 0.6,
                ..generic
            },
            "WindCharge" => Self {
                gravity: 0.,
                radius: 1.,
                drag: 1.,
                water_drag: 1.,
                ..generic
            },
            _ => generic,
        }
    }
    pub fn step(self, velocity: V3, fluid: bool) -> V3 {
        let mut v = mul(velocity, if fluid { self.water_drag } else { self.drag });
        v[1] -= self.gravity;
        v
    }
}
pub fn initial_correction(kind: &str) -> bool {
    !matches!(kind, "Arrow" | "Trident" | "FireworkRocket")
}
#[derive(Clone, Debug)]
pub struct Shot {
    pub kind: &'static str,
    pub info: Info,
    pub yaw_offset: f32,
    pub icon: Value,
    pub color_source: Value,
}
/// Stack snapshots use component values, including charged-projectile item identities.
pub fn held_shots(
    stack: &Value,
    use_ticks: Option<u32>,
    always_bow: bool,
    multishot: bool,
) -> Vec<Shot> {
    let id = stack["id"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches("minecraft:");
    let kind = match id {
        "bow" | "crossbow" => "Arrow",
        "fishing_rod" => "FishingBobber",
        "splash_potion" | "lingering_potion" => "Potion",
        "trident" => "Trident",
        "snowball" => "Snowball",
        "ender_pearl" => "EnderPearl",
        "egg" => "Egg",
        "experience_bottle" => "ExpBottle",
        "fire_charge" => "Fireball",
        "wind_charge" => "WindCharge",
        _ => return vec![],
    };
    let mut info = Info::for_type(kind, false);
    let mut kind = kind;
    if id == "bow" {
        let Some(ticks) = use_ticks.or(if always_bow { Some(40) } else { None }) else {
            return vec![];
        };
        let f = ticks as f32 / 20.;
        let power = ((f * f + f * 2.) / 3.).min(1.);
        if power < 0.1 {
            return vec![];
        }
        info.initial_velocity = f64::from(power * 3.);
    }
    let mut count = 1;
    let level = stack["multishot"].as_u64().unwrap_or(0) as usize;
    let charged = stack["charged"].as_array();
    if id == "crossbow" {
        if charged.is_some_and(|a| {
            a.iter()
                .any(|s| s["id"] == "minecraft:firework_rocket" || s == "minecraft:firework_rocket")
        }) {
            kind = "FireworkRocket";
            info = Info::for_type(kind, false);
        } else {
            info.initial_velocity = 3.15;
            info.copies_player_velocity = false;
        }
        if multishot {
            count = charged.filter(|a| !a.is_empty()).map_or(
                if level > 0 {
                    level.saturating_mul(2).saturating_add(1)
                } else {
                    1
                },
                Vec::len,
            );
        }
    }
    let max = 10. * level as f32;
    let step = if count > 1 {
        2. * max / (count - 1) as f32
    } else {
        0.
    };
    let offset = ((count - 1) % 2) as f32 * step / 2.;
    (0..count.min(16))
        .map(|i| Shot {
            kind,
            info,
            yaw_offset: if i % 2 == 0 {
                offset + ((i + 1) / 2) as f32 * step
            } else {
                offset - ((i + 1) / 2) as f32 * step
            },
            icon: stack.clone(),
            color_source: if id == "crossbow" {
                charged
                    .and_then(|a| a.first())
                    .cloned()
                    .unwrap_or(Value::Null)
            } else if id == "bow" {
                stack.get("ammo").cloned().unwrap_or(Value::Null)
            } else {
                stack.clone()
            },
        })
        .collect()
}
pub fn hypothetical(
    pos: V3,
    eye_height: f64,
    velocity: V3,
    on_ground: bool,
    rotation: [f32; 2],
    interpolated: V3,
    info: Info,
    frozen: bool,
) -> (V3, V3, V3) {
    let yaw = rotation[0].to_radians();
    let pitch = rotation[1].to_radians();
    let dir = [
        -f64::from(yaw.sin()) * f64::from(pitch.cos()),
        f64::from(-(rotation[1] + info.roll).to_radians().sin()),
        f64::from(yaw.cos()) * f64::from(pitch.cos()),
    ];
    let mut v = mul(dir, info.initial_velocity / dot(dir, dir).sqrt());
    if info.copies_player_velocity && !frozen {
        v[0] += velocity[0];
        v[2] += velocity[2];
        if !on_ground {
            v[1] += velocity[1];
        }
    }
    (
        [pos[0], pos[1] + eye_height - 0.10000000149011612, pos[2]],
        v,
        add(
            sub(interpolated, pos),
            [
                -f64::from(yaw.cos()) * 0.16,
                0.,
                -f64::from(yaw.sin()) * 0.16,
            ],
        ),
    )
}
#[derive(Clone, Debug)]
pub enum Hit {
    Block {
        position: V3,
        block: [i32; 3],
        normal: [i32; 3],
    },
    Entity {
        position: V3,
        id: u32,
        bounds: [V3; 2],
    },
}
impl Hit {
    pub fn position(&self) -> V3 {
        match self {
            Self::Block { position, .. } | Self::Entity { position, .. } => *position,
        }
    }
}
pub trait World {
    fn min_y(&self) -> f64;
    fn nonempty_fluid(&self, position: V3) -> bool;
    fn block_hit(&self, from: V3, to: V3, owner: u32) -> Option<Hit>;
    fn entity_hit(&self, from: V3, to: V3, owner: u32, radius: f64) -> Option<Hit>;
}
#[derive(Clone, Debug)]
pub struct Simulation {
    pub positions: Vec<V3>,
    pub hit: Option<Hit>,
}
pub fn simulate(
    world: &impl World,
    mut pos: V3,
    mut velocity: V3,
    info: Info,
    kind: &str,
    owner: u32,
    max_ticks: usize,
) -> Simulation {
    let mut ticks = 0;
    let mut positions = vec![];
    if initial_correction(kind) {
        velocity = info.step(velocity, world.nonempty_fluid(pos));
        ticks = 1;
    }
    while ticks < max_ticks {
        if pos[1] < world.min_y() {
            break;
        }
        let next = add(pos, velocity);
        // The original resolver intentionally evaluates the block ray before entities.
        if let Some(hit) = world
            .block_hit(pos, next, owner)
            .or_else(|| world.entity_hit(pos, next, owner, info.radius))
        {
            positions.push(hit.position());
            return Simulation {
                positions,
                hit: Some(hit),
            };
        }
        pos = next;
        velocity = info.step(velocity, world.nonempty_fluid(pos));
        positions.push(pos);
        ticks += 1;
    }
    if positions.is_empty() {
        positions.push(pos);
    }
    Simulation {
        positions,
        hit: None,
    }
}
pub fn clip_aabb(from: V3, to: V3, bounds: [V3; 2], inflate: f64) -> Option<V3> {
    let delta = sub(to, from);
    let mut low = 0f64;
    let mut high = 1f64;
    for i in 0..3 {
        let a = bounds[0][i] - inflate;
        let b = bounds[1][i] + inflate;
        if delta[i].abs() < 1e-12 {
            if from[i] < a || from[i] > b {
                return None;
            }
        } else {
            let x = (a - from[i]) / delta[i];
            let y = (b - from[i]) / delta[i];
            low = low.max(x.min(y));
            high = high.min(x.max(y));
            if low > high {
                return None;
            }
        }
    }
    Some(add(from, mul(delta, low)))
}
pub fn color(kind: &str, component: Option<u32>) -> u32 {
    let (a, r, g, b) = match kind {
        "Arrow" => (220, 255, 255, 255),
        "Potion" => (220, 56, 93, 198),
        "EnderPearl" => (200, 128, 0, 128),
        "FishingBobber" => (200, 0, 255, 255),
        "Trident" => (220, 180, 210, 255),
        "Snowball" => (220, 192, 192, 192),
        "Egg" => (220, 240, 234, 214),
        "ExpBottle" => (220, 120, 230, 120),
        "FireworkRocket" | "Fireball" => (220, 255, 165, 0),
        "WindCharge" => (220, 180, 235, 255),
        _ => (220, 255, 255, 255),
    };
    ((a as u32) << 24)
        | component
            .filter(|_| matches!(kind, "Arrow" | "Potion"))
            .map_or((r << 16) | (g << 8) | b, |c| c & 0xffffff)
}
pub fn snapshot(result: &Simulation, owner: u32, kind: &str, real: bool) -> Value {
    json!({"owner":owner,"kind":kind,"real":real,"positions":result.positions,"hit":result.hit.as_ref().map(|h|match h{Hit::Block{position,block,normal}=>json!({"kind":"block","position":position,"block":block,"normal":normal}),Hit::Entity{position,id,bounds}=>json!({"kind":"entity","position":position,"id":id,"bounds":bounds})})})
}
#[derive(Clone, Debug, PartialEq)]
pub struct Detail {
    pub position: V3,
    pub crosshair: bool,
    pub lines: Vec<String>,
    pub visible: [bool; 2],
    pub edge_inset: f32,
}
pub fn detail(
    settings: &Trajectories,
    result: &Simulation,
    owner: u32,
    owner_position: V3,
    local: u32,
    player_position: V3,
    real: bool,
    display_name: Option<&str>,
) -> Option<Detail> {
    if !settings.boolean("ShowDetailedInfo.Enabled") {
        return None;
    }
    let &landing = result.positions.last()?;
    let at = settings.text("ShowDetailedInfo.ShowAt");
    let crosshair = at == "Owner" && owner == local && real;
    let mut position = match at {
        "Owner" if owner == local && !real => landing,
        "Owner" => owner_position,
        "Landing" => landing,
        _ => result.positions[0],
    };
    let offset = settings.value("ShowDetailedInfo.RenderOffset");
    for (i, k) in ["x", "y", "z"].iter().enumerate() {
        position[i] += offset[k].as_f64().unwrap_or(0.);
    }
    let ticks = result.positions.len();
    let mut lines = vec![if settings.text("ShowDetailedInfo.TimeUnit") == "Seconds" {
        let s = format!("{:.1}", ticks as f64 * 0.05);
        format!("{}s", s.strip_suffix(".0").unwrap_or(&s))
    } else {
        ticks.to_string()
    }];
    if settings.boolean("ShowDetailedInfo.Distance") {
        lines.push(format!("Dist: {:.1}m", distance(player_position, landing)));
    }
    if settings.boolean("ShowDetailedInfo.OwnerName") && owner != local {
        if let Some(name) = display_name {
            lines.push(format!(
                "Owner: {}",
                name.chars()
                    .filter(|c| !c.is_control() && *c != '§')
                    .take(256)
                    .collect::<String>()
            ));
        }
    }
    Some(Detail {
        position,
        crosshair,
        lines,
        visible: [
            true,
            settings.boolean("ShowDetailedInfo.RenderOffscreen.Enabled"),
        ],
        edge_inset: settings.number("ShowDetailedInfo.RenderOffscreen.EdgeInset") as f32,
    })
}
