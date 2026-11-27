// SPDX-License-Identifier: GPL-3.0-or-later
//! Original AvoidHazardInputPlanner ordering and BlockShapeEvent selection.
pub const TAGS: [&str; 10] = [
    "Cacti",
    "BerryBush",
    "Fire",
    "Cobwebs",
    "Ladders",
    "PressurePlates",
    "MagmaBlocks",
    "Lava",
    "WitherRose",
    "PowderSnow",
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Shape,
    Input,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Shape => "Shape",
            Self::Input => "Input",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Shape" => Some(Self::Shape),
            "Input" | "Movement" => Some(Self::Input),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AvoidHazards {
    pub mode: Mode,
    pub avoid: u32,
}
impl Default for AvoidHazards {
    fn default() -> Self {
        Self {
            mode: Mode::Shape,
            avoid: 1023,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub hazards: u32,
    pub ladder: bool,
    pub open_trapdoor: bool,
    pub facing: u8,
    pub magma: bool,
}
pub fn ladder(current: Flags, below: Flags) -> bool {
    current.ladder
        || (current.open_trapdoor
            && below.ladder
            && current.facing != 0
            && current.facing == below.facing)
}
/// Returns the height of the replacement shape; None preserves its old shape.
pub fn shape(mask: u32, current: Flags, below: Flags) -> Option<f64> {
    let mut bits = current.hazards;
    if ladder(current, below) {
        bits |= 1 << 4;
    }
    if below.magma {
        bits |= 1 << 6;
    }
    let first = (bits & mask).trailing_zeros();
    if first == 32 {
        None
    } else {
        Some(if first == 5 || first == 6 { 0.25 } else { 1. })
    }
}
pub type Direction = [bool; 4];
pub fn moving(k: Direction) -> bool {
    k[0] != k[1] || k[2] != k[3]
}
pub fn choose(original: Direction, mut safe: impl FnMut(Direction) -> bool) -> Direction {
    if !moving(original) {
        return [false; 4];
    }
    if safe(original) {
        return original;
    }
    let x = i32::from(original[2]) - i32::from(original[3]);
    let z = i32::from(original[0]) - i32::from(original[1]);
    if x == 0 && z == 0 {
        return [false; 4];
    }
    let angle = (x as f64).atan2(z as f64).to_degrees();
    let mut candidates = [
        ([true, false, false, false], 0.),
        ([true, false, false, true], 45.),
        ([false, false, false, true], 90.),
        ([false, true, false, true], 135.),
        ([false, true, false, false], 180.),
        ([false, true, true, false], -135.),
        ([false, false, true, false], -90.),
        ([true, false, true, false], -45.),
    ];
    let difference = |a: f64| {
        let mut v = (angle - a) % 360.;
        if v >= 180. {
            v -= 360.;
        }
        if v < -180. {
            v += 360.;
        }
        v.abs()
    };
    candidates.sort_by(|a, b| difference(a.1).total_cmp(&difference(b.1)));
    candidates
        .into_iter()
        .find_map(|(k, _)| (k != original && safe(k)).then_some(k))
        .unwrap_or([false; 4])
}
