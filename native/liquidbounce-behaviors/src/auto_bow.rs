// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoBow's three upstream features; world and item side effects stay in Host.
use std::cmp::Ordering;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    Bow,
    Crossbow,
    Trident,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Type,
    Health,
    Distance,
    Direction,
    HurtTime,
    Age,
}
pub const PRIORITIES: [Priority; 6] = [
    Priority::Type,
    Priority::Health,
    Priority::Distance,
    Priority::Direction,
    Priority::HurtTime,
    Priority::Age,
];
impl Priority {
    pub fn name(self) -> &'static str {
        match self {
            Self::Type => "Type",
            Self::Health => "Health",
            Self::Distance => "Distance",
            Self::Direction => "Direction",
            Self::HurtTime => "HurtTime",
            Self::Age => "Age",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub auto_shoot: bool,
    pub charged: i32,
    pub charged_random: [f32; 2],
    pub delay: f32,
    pub aim_threshold: f32,
    pub requires_hit: bool,
    pub precharged: bool,
    pub aimbot: bool,
    pub through_walls: bool,
    pub fov: f32,
    pub hurt_time: i32,
    pub priority: Vec<Priority>,
    pub fast_charge: bool,
    pub speed: i32,
    pub ground_only: bool,
    pub stationary_only: bool,
    pub no_regeneration: bool,
    pub packet: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_shoot: true,
            charged: 15,
            charged_random: [0., 0.],
            delay: 0.,
            aim_threshold: 1.5,
            requires_hit: false,
            precharged: false,
            aimbot: true,
            through_walls: true,
            fov: 180.,
            hurt_time: 10,
            priority: vec![Priority::Type, Priority::Distance],
            fast_charge: false,
            speed: 20,
            ground_only: true,
            stationary_only: false,
            no_regeneration: false,
            packet: 3,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: i32,
    pub type_weight: i32,
    pub health: f32,
    pub distance: f64,
    pub cosine: f64,
    pub hurt: i32,
    pub age: i32,
}
// java.lang.Float/Double.compare canonicalize every NaN and distinguish signed zero.
fn compare_float(a: f32, b: f32) -> Ordering {
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    let bits = |v: f32| {
        if v.is_nan() {
            f32::NAN.to_bits() as i32
        } else {
            v.to_bits() as i32
        }
    };
    bits(a).cmp(&bits(b))
}
fn compare_double(a: f64, b: f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    }
    if a > b {
        return Ordering::Greater;
    }
    let bits = |v: f64| {
        if v.is_nan() {
            f64::NAN.to_bits() as i64
        } else {
            v.to_bits() as i64
        }
    };
    bits(a).cmp(&bits(b))
}
pub fn targets(settings: &Settings, mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    let min_cos = (f64::from(settings.fov) * f64::from(std::f32::consts::PI / 180.)).cos();
    candidates
        .retain(|c| c.hurt <= settings.hurt_time && (settings.fov >= 180. || min_cos <= c.cosine));
    candidates.sort_by(|a, b| {
        for p in &settings.priority {
            let o = match p {
                Priority::Type => a.type_weight.cmp(&b.type_weight),
                Priority::Health => compare_float(a.health, b.health),
                Priority::Distance => compare_double(a.distance, b.distance),
                Priority::Direction => compare_double(b.cosine, a.cosine),
                Priority::HurtTime => a.hurt.cmp(&b.hurt),
                Priority::Age => b.age.cmp(&a.age),
            };
            if o != Ordering::Equal {
                return o;
            }
        }
        Ordering::Equal
    });
    candidates
}
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub weapon: Weapon,
    pub hand: u8,
    pub charged: bool,
    pub ticks: i32,
    pub pull_time: i32,
    pub now: i64,
    pub hit: bool,
    pub active_aim: Option<f32>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShootEffect {
    #[default]
    None,
    Use(u8),
}
#[derive(Clone, Debug, Default)]
pub struct AutoBow {
    pub settings: Settings,
    pub force_uncharged: bool,
    pub charge_random: Option<i32>,
    pub last_shot: i64,
    pub target: Option<i32>,
    pub closest_squared_enemy_distance: f64,
}
impl AutoBow {
    // The source keeps the chronometer, Gaussian generator and charge random on disable.
    pub fn disabled(&mut self) {
        self.target = None;
        self.force_uncharged = false;
    }
    pub fn stop_using(&mut self, bow: bool, now: i64) {
        if bow {
            self.last_shot = now;
        }
    }
    fn random_charge(&mut self, gaussian: &mut impl FnMut() -> f64) {
        let [from, to] = self.settings.charged_random;
        let half = (to - from) / 2.;
        let mid = from + half;
        self.charge_random = Some((f64::from(mid) + gaussian() * f64::from(half)) as i32);
        if let Some(v) = self.charge_random.as_mut() {
            *v = (*v).clamp(from as i32, to as i32);
        }
    }
    pub fn shoot(&mut self, shot: Option<Shot>, gaussian: &mut impl FnMut() -> f64) -> ShootEffect {
        self.force_uncharged = false;
        if !self.settings.auto_shoot {
            return ShootEffect::None;
        }
        let Some(s) = shot else {
            return ShootEffect::None;
        };
        match s.weapon {
            Weapon::Crossbow if !s.charged && s.ticks < s.pull_time => return ShootEffect::None,
            Weapon::Bow => {
                if self.charge_random.is_none() {
                    self.random_charge(gaussian);
                }
                if s.ticks
                    < self
                        .settings
                        .charged
                        .wrapping_add(self.charge_random.unwrap())
                {
                    return ShootEffect::None;
                }
            }
            Weapon::Trident if s.ticks <= 10 => return ShootEffect::None,
            _ => {}
        }
        let delay = (self.settings.delay * 1000.) as i64;
        if self.last_shot.wrapping_add(delay) >= s.now {
            return ShootEffect::None;
        }
        if self.settings.requires_hit {
            if !s.hit {
                return ShootEffect::None;
            }
        } else if self.settings.aimbot
            && (self.target.is_none()
                || s.active_aim.is_none_or(|d| d > self.settings.aim_threshold))
        {
            return ShootEffect::None;
        }
        if s.weapon == Weapon::Crossbow && s.charged {
            self.last_shot = s.now;
            return ShootEffect::Use(s.hand);
        }
        self.force_uncharged = true;
        if s.weapon == Weapon::Bow {
            self.random_charge(gaussian);
        }
        ShootEffect::None
    }
    pub fn select(&mut self, candidates: Vec<Candidate>) -> Vec<Candidate> {
        self.target = None;
        let sorted = targets(&self.settings, candidates);
        if !sorted.is_empty() {
            self.closest_squared_enemy_distance = sorted
                .iter()
                .map(|c| c.distance)
                .fold(f64::INFINITY, f64::min);
        }
        sorted
    }
    pub fn fast_charge(
        &self,
        using_bow: bool,
        ground: bool,
        moving: bool,
        regenerating: bool,
    ) -> bool {
        self.settings.fast_charge
            && using_bow
            && (!self.settings.ground_only || ground)
            && (!self.settings.stationary_only || !moving)
            && (!self.settings.no_regeneration || !regenerating)
    }
}
