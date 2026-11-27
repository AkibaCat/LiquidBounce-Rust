// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSpeed: independent mode scopes, GameTick, jump and SELF movement.
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::LazyLock};
pub static SCHEMA: LazyLock<Vec<Value>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("speed_schema.json")).expect("owned Speed schema")
});
pub const MODES: [&str; 22] = [
    "LegitHop",
    "Custom",
    "YPort",
    "PiercingAttack",
    "Verus-b3882",
    "HypixelBHop",
    "HypixelLowHop",
    "Spartan-4.0.4.3",
    "Spartan-4.0.4.3-FastFall",
    "CubeCraft-2024-06-30",
    "Vulcan-2.8.6",
    "Vulcan-2.8.8",
    "Vulcan-2.8.6-Ground",
    "Grim-2.3.73",
    "NCP-2024-08-31",
    "Intave-14",
    "Intave-14-Fast",
    "HylexLowHop",
    "HylexGround",
    "BlocksMC",
    "Matrix-7",
    "Rewound",
];
pub const SCOPES: [&str; 3] = ["Mode", "OnlyInCombat.Mode", "OnlyOnPotionEffect.Mode"];
pub fn join(p: &str, n: &str) -> String {
    if p.is_empty() {
        n.into()
    } else {
        format!("{p}.{n}")
    }
}
pub fn defaults() -> BTreeMap<String, Value> {
    fn add(a: &[Value], p: &str, out: &mut BTreeMap<String, Value>) {
        for n in a {
            let p = join(p, n["name"].as_str().unwrap());
            if let Some(d) = n.get("default") {
                out.insert(p.clone(), d.clone());
            }
            for k in ["children", "modes"] {
                if let Some(c) = n[k].as_array() {
                    add(c, &p, out)
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    add(&SCHEMA, "", &mut out);
    out
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub values: BTreeMap<String, Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self { values: defaults() }
    }
}
impl Settings {
    pub fn flag(&self, p: &str) -> bool {
        self.values.get(p).and_then(Value::as_bool).unwrap_or(false)
    }
    pub fn number(&self, p: &str) -> f64 {
        self.values.get(p).and_then(Value::as_f64).unwrap_or(0.)
    }
    pub fn tag(&self, p: &str) -> &str {
        self.values.get(p).and_then(Value::as_str).unwrap_or("")
    }
    pub fn wire(&self) -> Value {
        json!(self.values)
    }
    pub fn from_wire(v: &Value) -> Self {
        let mut s = Self::default();
        if let Some(o) = v.as_object() {
            for (k, v) in o {
                if s.values.contains_key(k) {
                    s.values.insert(k.clone(), v.clone());
                }
            }
        }
        s
    }
    pub fn potion_matches(&self, c: &Context) -> bool {
        let p = "OnlyOnPotionEffect.PotionEffect";
        let mode = self.tag(p);
        let path = join(p, mode);
        let value = match mode {
            "Speed" => c.speed_level as f64,
            "Slowness" => c.slowness_level as f64,
            "Both" => {
                (1. + 0.2 * c.speed_level as f64) * (1. - 0.15 * c.slowness_level as f64).max(0.)
            }
            _ => return false,
        };
        let range = &self.values[&join(
            &path,
            if mode == "Both" {
                "PotionEffectsBoostRange"
            } else {
                "LevelRange"
            },
        )];
        value >= range[0].as_f64().unwrap() && value <= range[1].as_f64().unwrap()
    }
    pub fn running(&self, enabled: bool, c: &Context) -> [bool; 3] {
        let mut passes = c.in_game;
        for n in self.values["Not"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            passes &= !match n {
                "WhileUsingItem" => c.using_item,
                "DuringScaffold" => c.scaffold || c.fly,
                "WhileSneaking" => c.sneaking,
                "IsFallFlying" => c.fall_flying,
                "IsInLiquid" => c.in_liquid,
                _ => false,
            };
        }
        let combat = self.flag("OnlyInCombat.Enabled") && c.combat;
        let potion = self.flag("OnlyOnPotionEffect.Enabled") && self.potion_matches(c);
        [
            passes && (enabled || c.scaffold_auto_speed) && !combat && !potion,
            passes && enabled && combat,
            passes && enabled && potion,
        ]
    }
    pub fn yaw_offset(&self, c: &Context) -> Option<f32> {
        if !self.flag("YawOffset.Enabled") {
            return None;
        }
        let forward = c.input & 1 != 0;
        let back = c.input & 2 != 0;
        let left = c.input & 4 != 0;
        let right = c.input & 8 != 0;
        let directional = if forward && left {
            45.
        } else if forward && right {
            -45.
        } else if back && left {
            135.
        } else if back && right {
            -135.
        } else if back {
            180.
        } else if left {
            90.
        } else if right {
            -90.
        } else {
            0.
        };
        Some(match self.tag("YawOffset.YawOffsetMode") {
            "Ground" => {
                if c.ground {
                    directional
                } else {
                    0.
                }
            }
            "Constant" => directional,
            _ => {
                if !c.ground && forward && !left && !right {
                    -45.
                } else {
                    0.
                }
            }
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub in_game: bool,
    pub ground: bool,
    pub velocity: [f64; 3],
    pub yaw: f32,
    pub input: u8,
    pub physical_jump: bool,
    pub sprinting: bool,
    pub sneaking: bool,
    pub using_item: bool,
    pub fall_flying: bool,
    pub in_liquid: bool,
    pub fly: bool,
    pub scaffold: bool,
    pub scaffold_auto_speed: bool,
    pub towering: bool,
    pub combat: bool,
    pub speed_level: i32,
    pub slowness_level: i32,
    pub jump_level: i32,
    pub hurt: i32,
    pub fall_distance: f64,
    pub fly_distance: f32,
    pub horizontal_collision: bool,
    pub bottom_collision: bool,
    pub ground_exempt: bool,
    pub ground_within: bool,
    pub jump_blocked: bool,
    pub leather_boots: bool,
    pub collisions: [i32; 3],
    pub now: i64,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            in_game: true,
            ground: false,
            velocity: [0.; 3],
            yaw: 0.,
            input: 0,
            physical_jump: false,
            sprinting: false,
            sneaking: false,
            using_item: false,
            fall_flying: false,
            in_liquid: false,
            fly: false,
            scaffold: false,
            scaffold_auto_speed: false,
            towering: false,
            combat: false,
            speed_level: 0,
            slowness_level: 0,
            jump_level: 0,
            hurt: 0,
            fall_distance: 0.,
            fly_distance: 0.,
            horizontal_collision: false,
            bottom_collision: false,
            ground_exempt: false,
            ground_within: false,
            jump_blocked: false,
            leather_boots: false,
            collisions: [0; 3],
            now: 0,
        }
    }
}
impl Context {
    pub fn moving(self) -> bool {
        (self.input & 1 != 0) != (self.input & 2 != 0)
            || (self.input & 4 != 0) != (self.input & 8 != 0)
    }
    pub fn speed(self) -> f64 {
        self.velocity[0].hypot(self.velocity[2])
    }
    pub fn amplifier(self) -> i32 {
        (self.speed_level - 1).max(0)
    }
}
pub fn movement_yaw(yaw: f32, input: u8) -> f32 {
    let forward = input & 1 != 0;
    let backward = input & 2 != 0;
    let left = input & 4 != 0;
    let right = input & 8 != 0;
    let mut yaw = yaw;
    let multiplier = if backward && !forward {
        yaw += 180.;
        -0.5
    } else if forward && !backward {
        0.5
    } else {
        1.
    };
    if left && !right {
        yaw -= 90. * multiplier
    } else if right && !left {
        yaw += 90. * multiplier
    }
    yaw
}
pub fn strafe(v: [f64; 3], yaw: f32, input: u8, speed: f64, strength: f64) -> [f64; 3] {
    let angle = f64::from(movement_yaw(yaw, input)).to_radians();
    let [x, y, z] = v;
    [
        -angle.sin() * speed * strength + x * (1. - strength),
        y,
        angle.cos() * speed * strength + z * (1. - strength),
    ]
}
fn scale(v: &mut [f64; 3], m: f64) {
    v[0] *= m;
    v[2] *= m;
}
fn set_strafe(c: &mut Context, speed: f64, strength: f64) {
    c.velocity = strafe(c.velocity, c.yaw, c.input, speed, strength)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimerRequest {
    pub speed: f32,
    pub priority: i32,
    pub ticks: i32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovePacket {
    pub y_offset: f64,
    pub ground: bool,
    pub full: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Effects {
    pub timers: Vec<TimerRequest>,
    pub packets: Vec<MovePacket>,
    pub enable_ping_spoof: bool,
    pub disable_speed: bool,
    pub piercing: Vec<(usize, i32, i32, bool, bool, bool, i32)>,
    pub hypixel_should_strafe: bool,
    pub jump_repetitions: u32,
}
impl Effects {
    fn timer(&mut self, speed: f64, priority: i32, ticks: i32) {
        self.timers.push(TimerRequest {
            speed: speed as f32,
            priority,
            ticks,
        });
    }
}
#[derive(Clone, Debug)]
pub struct Scope {
    pub mode: Option<usize>,
    pub air_ticks: i32,
    pub ground_ticks: i32,
    pub tick_air_ticks: i32,
    pub tick_ground_ticks: i32,
    pub since_jump: Option<i32>,
    pub flag_delay: i32,
    pub was_flagged: bool,
    pub velocity_timeout: i32,
    pub pending_knock: Option<bool>,
    pub pending_jump_boost: Option<i32>,
    pub state: i32,
    pub has_been_hurt: bool,
    pub adjusted: bool,
    pub boost_wait: i32,
    pub enabled_time: i64,
    pub last_damage: i64,
    pub external_damage_adjust: i32,
    pub recover: i32,
    pub phase_ticks: i32,
    pub phase: i32,
    pub collided: bool,
    pub stair_ticks: i32,
    pub hop_load: f64,
    pub slow_hop: bool,
    pub slow_after_setback: bool,
    pub last_setback: Option<i64>,
    pub timer_wait: i32,
}
impl Default for Scope {
    fn default() -> Self {
        Self {
            mode: None,
            air_ticks: 0,
            ground_ticks: 0,
            tick_air_ticks: 0,
            tick_ground_ticks: 0,
            since_jump: None,
            flag_delay: 0,
            was_flagged: false,
            velocity_timeout: 0,
            pending_knock: None,
            pending_jump_boost: None,
            state: 0,
            has_been_hurt: false,
            adjusted: false,
            boost_wait: 0,
            enabled_time: 0,
            last_damage: 0,
            external_damage_adjust: 0,
            recover: 0,
            phase_ticks: 2,
            phase: 2,
            collided: false,
            stair_ticks: 0,
            hop_load: 0.,
            slow_hop: false,
            slow_after_setback: false,
            last_setback: None,
            timer_wait: 0,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Speed {
    pub scopes: [Scope; 3],
    pub main_running: bool,
    pub effects: Effects,
}
impl Speed {
    pub fn active(&self) -> bool {
        self.scopes.iter().any(|s| s.mode.is_some())
    }
    pub fn synchronize(&mut self, settings: &Settings, enabled: bool, c: &mut Context) {
        let running = settings.running(enabled, c);
        self.main_running = running[0];
        for (i, s) in self.scopes.iter_mut().enumerate() {
            let next = running[i]
                .then(|| MODES.iter().position(|m| *m == settings.tag(SCOPES[i])))
                .flatten();
            if next != s.mode {
                if matches!(s.mode, Some(8 | 9 | 19)) {
                    c.velocity[0] = 0.;
                    c.velocity[2] = 0.;
                }
                let old = s.clone();
                *s = Scope {
                    mode: next,
                    collided: c.horizontal_collision,
                    enabled_time: c.now,
                    last_setback: old.last_setback,
                    ..Default::default()
                };
                if next == Some(9) {
                    self.effects.enable_ping_spoof = true;
                }
            }
        }
        if !self.active() {
            self.effects.hypixel_should_strafe = false;
        }
    }
    pub fn input(&self, settings: &Settings, c: Context, jump: &mut bool, sprint: &mut bool) {
        for s in &self.scopes {
            let Some(mode) = s.mode else { continue };
            match mode {
                7 | 8 | 13 | 18 => {}
                9 => {
                    if c.moving() && s.has_been_hurt {
                        *jump = true;
                    }
                }
                19 => {
                    if c.moving() && s.state != 0 {
                        *jump = true;
                        *sprint = true;
                    }
                }
                21 => {
                    if c.moving() && !c.towering {
                        *jump = false;
                    }
                }
                _ => {
                    if c.ground && c.moving() && !(settings.flag("AvoidEdgeBump") && c.jump_blocked)
                    {
                        *jump = true;
                    }
                }
            }
        }
    }
    pub fn tick(&mut self, settings: &Settings, c: &mut Context) {
        self.effects.hypixel_should_strafe = false;
        for (i, s) in self.scopes.iter_mut().enumerate() {
            let Some(mode) = s.mode else { continue };
            if c.ground {
                s.tick_air_ticks = 0;
                s.tick_ground_ticks += 1;
            } else {
                s.tick_ground_ticks = 0;
                s.tick_air_ticks += 1;
            }
            let prefix = format!("{}.{}", SCOPES[i], MODES[mode]);
            let flag = |n: &str| settings.flag(&join(&prefix, n));
            let num = |n: &str| settings.number(&join(&prefix, n));
            if let Some(t) = s.since_jump.as_mut() {
                *t += 1;
            }
            if let Some(t) = s.pending_jump_boost.as_mut() {
                if *t == 0 {
                    scale(
                        &mut c.velocity,
                        1. + num("HorizontalModification.HorizontalJumpOff"),
                    );
                    s.pending_jump_boost = None;
                } else {
                    *t -= 1;
                }
            }
            if let Some(fall) = s.pending_knock.take() {
                let minimum = if mode == 5 {
                    0.2857671997172534 * c.amplifier() as f64
                } else {
                    0.2857671997172534
                };
                set_strafe(
                    c,
                    if fall {
                        c.speed().max(minimum)
                    } else {
                        c.speed()
                    },
                    1.,
                );
            }
            match mode {
                0 | 7 | 8 | 10 | 12 | 21 => {}
                1 => {
                    if c.moving() {
                        if flag("HorizontalModification.Enabled") {
                            scale(
                                &mut c.velocity,
                                1. + num("HorizontalModification.HorizontalAcceleration"),
                            );
                        }
                        if flag("VerticalModification.Enabled") {
                            c.velocity[1] -= num(if c.velocity[1] <= 0. {
                                "VerticalModification.PullDownDuringFall"
                            } else {
                                "VerticalModification.Pulldown"
                            });
                        }
                        if num("TimerSpeed") != 1. {
                            self.effects.timer(num("TimerSpeed"), 20, 1);
                        }
                    }
                    if flag("Strafe.Enabled") {
                        if s.velocity_timeout > 0 {
                            s.velocity_timeout -= 1;
                        } else if c.moving() {
                            let speed = if flag("Strafe.CustomSpeed") {
                                num("Strafe.Speed")
                            } else {
                                c.speed()
                            };
                            set_strafe(c, speed, num("Strafe.Strength"));
                        }
                    }
                }
                2 => {
                    if !c.ground && c.moving() {
                        c.velocity[1] = -1.;
                    }
                }
                3 => {
                    let r = &settings.values[&join(&prefix, "HoldTime")];
                    self.effects.piercing.push((
                        i,
                        r[0].as_i64().unwrap() as i32,
                        r[1].as_i64().unwrap() as i32,
                        flag("OnGround"),
                        flag("IgnoreHunger"),
                        flag("WaitForCooldown"),
                        num("MinimumDurability") as i32,
                    ));
                }
                4 => {
                    if s.timer_wait <= 0 {
                        self.effects.timer(2., 20, 1);
                        s.timer_wait = 101;
                    } else {
                        s.timer_wait -= 1;
                    }
                }
                5 => {
                    if c.ground {
                        set_strafe(c, c.speed(), 1.);
                    } else {
                        let m = if flag("HorizontalAcceleration") {
                            0.0004 + 0.0007 * c.amplifier() as f64
                        } else {
                            0.
                        };
                        scale(&mut c.velocity, 1. + m);
                        if flag("VerticalAcceleration")
                            && c.velocity[1] < 0.
                            && c.fall_distance < 1.
                        {
                            c.velocity[1] *= 1.0004;
                        }
                    }
                }
                6 => {
                    if c.ground {
                        set_strafe(c, c.speed(), 1.);
                        self.effects.hypixel_should_strafe = true;
                    } else {
                        match s.air_ticks {
                            1 => {
                                set_strafe(c, c.speed(), 1.);
                                self.effects.hypixel_should_strafe = true;
                                c.velocity[1] += 0.0568;
                            }
                            3 => {
                                scale(&mut c.velocity, 0.95);
                                c.velocity[1] -= 0.13;
                            }
                            4 => c.velocity[1] -= 0.2,
                            7 => {
                                if flag("Glide") && c.ground_exempt {
                                    c.velocity[1] = 0.;
                                }
                            }
                            _ => {}
                        }
                        if c.ground_exempt {
                            set_strafe(c, c.speed(), 1.);
                        }
                        if c.hurt == 9 {
                            set_strafe(c, c.speed().max(0.281), 1.);
                        }
                        if c.amplifier() == 2 && [1, 2, 5, 6, 8].contains(&s.air_ticks) {
                            scale(&mut c.velocity, 1.2);
                        }
                    }
                }
                9 => {
                    if c.moving() {
                        if s.boost_wait > 0 {
                            s.boost_wait -= 1;
                        } else {
                            if s.external_damage_adjust > 0 {
                                s.boost_wait = s.external_damage_adjust;
                                s.external_damage_adjust = 0;
                            } else {
                                s.last_damage = c.now;
                                s.enabled_time = c.now;
                                s.has_been_hurt = false;
                                for (y, ground) in
                                    [(0., false), (3.25, false), (0., false), (0., true)]
                                {
                                    self.effects.packets.push(MovePacket {
                                        y_offset: y,
                                        ground,
                                        full: false,
                                    });
                                }
                                s.boost_wait = num("ReboostTicks") as i32 - 1;
                            }
                        }
                    }
                }
                11 => {
                    if c.amplifier() != 0 && !c.ground && c.fall_distance.abs() > 0. {
                        scale(&mut c.velocity, 1.055);
                    }
                }
                13 => {
                    if c.moving() {
                        let yaw = f64::from(c.yaw).to_radians();
                        let boost = num("BoostSpeed") * c.collisions[i] as f64;
                        c.velocity[0] -= yaw.sin() * boost;
                        c.velocity[2] += yaw.cos() * boost;
                    }
                }
                14 => {
                    if flag("PullDown.Enabled") && !c.ground {
                        if s.tick_air_ticks == num("PullDown.OnTick") as i32 {
                            set_strafe(c, c.speed(), 1.);
                            c.velocity[1] -= 0.1523351824467155 * num("PullDown.MotionMultiplier");
                        }
                        if flag("PullDown.OnHurt") && c.hurt >= 5 && c.velocity[1] >= 0. {
                            c.velocity[1] -= 0.1;
                        }
                    }
                    if flag("Boost.Enabled") && c.moving() {
                        scale(
                            &mut c.velocity,
                            1. + 0.00718 * num("Boost.InitialBoostMultiplier"),
                        );
                    }
                    if c.moving() {
                        let speed = c.amplifier() as f64;
                        if c.ground {
                            set_strafe(c, c.speed().max(0.281 + 0.199999999 * speed), 1.);
                        } else if flag("AirStrafe") {
                            set_strafe(c, c.speed().max(0.2 + 0.199999999 * speed), 0.7);
                        }
                    }
                    if flag("Timer") {
                        self.effects.timer(1.08, 20, 1);
                    }
                    if c.hurt >= 1 && flag("DamageBoost") {
                        set_strafe(c, c.speed().max(0.5), 1.);
                    }
                }
                15 => {
                    if flag("Strafe.Enabled") && c.sprinting && (c.ground || s.air_ticks == 11) {
                        set_strafe(c, c.speed(), num("Strafe.Strength"));
                    }
                    if flag("AirBoost.Enabled") && c.velocity[1] > 0.003 && c.sprinting {
                        scale(&mut c.velocity, 1.00075);
                    }
                }
                16 => {
                    match s.air_ticks {
                        1 => scale(&mut c.velocity, 1.04),
                        2..=4 => scale(&mut c.velocity, 1.02),
                        _ => {}
                    }
                    if flag("Timer") {
                        self.effects.timer(1.002, -20, 1);
                    }
                }
                17 => {
                    if c.ground {
                        if c.moving() && c.speed() < 0.32 {
                            scale(&mut c.velocity, 1.1);
                        }
                    } else {
                        if s.tick_air_ticks == 9 && c.speed() < 0.29 {
                            scale(&mut c.velocity, 1.007);
                        }
                        if s.tick_air_ticks == 1 && c.speed() < 0.2 {
                            scale(&mut c.velocity, 1.01);
                        }
                        if c.velocity[1] > 0. && s.tick_air_ticks <= 2 && c.speed() < 0.2 {
                            scale(&mut c.velocity, 1.02);
                        }
                    }
                }
                18 => {
                    if c.ground && s.tick_ground_ticks > 5 && c.hurt < 1 && c.amplifier() < 1 {
                        scale(
                            &mut c.velocity,
                            if c.input & 12 == 0 { 1.2174 } else { 1.214 },
                        );
                    }
                }
                19 => {
                    if c.towering {
                        s.state = 0;
                        continue;
                    }
                    if c.ground {
                        s.state = if c.scaffold { 2 } else { 1 };
                    }
                    let mut speed = 0.06 + if c.ground { 0.12 } else { 0.21 } + c.velocity[1] / 20.;
                    if c.amplifier() == 1 {
                        speed += 0.1;
                    }
                    if s.flag_delay > 0 {
                        s.flag_delay -= 1;
                        speed -= 0.007 * s.flag_delay as f64;
                    }
                    match s.state {
                        1 => {
                            if s.air_ticks == 4 {
                                c.velocity[1] = -0.09800000190734863;
                            }
                        }
                        2 => match s.air_ticks {
                            1 => c.velocity[1] += 0.0568,
                            3 => c.velocity[1] -= 0.13,
                            4 => c.velocity[1] -= 0.2,
                            _ => {}
                        },
                        _ => {}
                    }
                    if !c.ground && s.state != 0 {
                        let yaw = if flag("RoundStrafeYaw") {
                            (movement_yaw(c.yaw, c.input) / 45.).round() * 45.
                        } else {
                            movement_yaw(c.yaw, c.input)
                        };
                        c.velocity = strafe(c.velocity, yaw, 1, speed, 1.);
                    }
                }
                20 => {
                    if c.moving() {
                        if c.ground {
                            c.velocity[1] = 0.419652;
                            set_strafe(c, c.speed(), 1.);
                        } else if c.speed() * c.speed() < 0.04 {
                            set_strafe(c, c.speed(), 1.);
                        }
                    }
                }
                _ => unreachable!(),
            }
            if mode == 12 && c.moving() && c.bottom_collision && !c.physical_jump {
                set_strafe(
                    c,
                    if c.amplifier() > 0 {
                        0.59
                    } else if c.input & 12 != 0 {
                        0.41
                    } else {
                        0.42
                    },
                    1.,
                );
                c.velocity[1] = 0.005;
            }
            if let Some(t) = s.since_jump {
                match mode {
                    10 => match t {
                        1 => set_strafe(
                            c,
                            if c.input & 12 != 0 {
                                0.3345
                            } else {
                                0.3355 * (1. + c.amplifier() as f64 * 0.3819)
                            },
                            1.,
                        ),
                        2 => {
                            if c.sprinting {
                                set_strafe(
                                    c,
                                    if c.input & 12 != 0 {
                                        0.3235
                                    } else {
                                        0.3284 * (1. + c.amplifier() as f64 * 0.355)
                                    },
                                    1.,
                                );
                            }
                        }
                        4 => c.velocity[1] = -0.376,
                        6 => {
                            if c.fly_distance > 0.298 {
                                set_strafe(c, 0.298, 1.);
                            }
                            s.since_jump = None;
                        }
                        _ => {}
                    },
                    11 => {
                        let has = c.amplifier() != 0;
                        match t {
                            1 => set_strafe(c, if has { 0.605 } else { 0.31 }, 1.),
                            2 => {
                                set_strafe(c, if has { 0.57 } else { 0.29 }, 1.);
                                c.velocity[1] = if has { -0.5 } else { -0.37 };
                            }
                            3 => set_strafe(c, if has { 0.595 } else { 0.27 }, 1.),
                            4 => {
                                set_strafe(c, if has { 0.595 } else { 0.28 }, 1.);
                                s.since_jump = None;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn jump_before(&mut self, settings: &Settings, c: &mut Context, power: &mut f32) -> bool {
        let mut allowed = true;
        for (i, s) in self.scopes.iter_mut().enumerate() {
            let Some(mode) = s.mode else { continue };
            let p = format!("{}.{}", SCOPES[i], MODES[mode]);
            match mode {
                1 => {
                    if settings.flag(&join(&p, "VerticalModification.Enabled")) {
                        let h =
                            settings.number(&join(&p, "VerticalModification.JumpHeight")) as f32;
                        if h != 0.42 {
                            *power = h;
                        }
                    }
                }
                5 => set_strafe(
                    c,
                    c.speed().max(if s.was_flagged {
                        0.
                    } else {
                        0.281 + 0.008003278196411223 * c.amplifier() as f64
                    }),
                    1.,
                ),
                6 => {
                    set_strafe(c, c.speed().max(0.247 + 0.15 * c.amplifier() as f64), 1.);
                    self.effects.hypixel_should_strafe = true;
                }
                12 => {
                    if !c.physical_jump {
                        allowed = false;
                    }
                }
                14 => {
                    if settings.flag(&join(&p, "LowHop")) {
                        *power = 0.4;
                    }
                }
                17 => *power = 0.33,
                _ => {}
            }
        }
        allowed
    }
    pub fn jump_after(&mut self, settings: &Settings, c: &mut Context) {
        for (i, s) in self.scopes.iter_mut().enumerate() {
            let Some(mode) = s.mode else { continue };
            let p = format!("{}.{}", SCOPES[i], MODES[mode]);
            s.since_jump = Some(0);
            match mode {
                1 => {
                    if settings.flag(&join(&p, "HorizontalModification.Enabled")) {
                        let ticks = settings
                            .number(&join(&p, "HorizontalModification.TicksToBoostOff"))
                            as i32;
                        if ticks == 0 {
                            scale(
                                &mut c.velocity,
                                1. + settings
                                    .number(&join(&p, "HorizontalModification.HorizontalJumpOff")),
                            );
                        } else {
                            s.pending_jump_boost = Some(ticks - 1);
                        }
                    }
                }
                2 => set_strafe(c, settings.number(&join(&p, "Speed")), 1.),
                4 => scale(&mut c.velocity, 1.1),
                11 => set_strafe(c, if c.amplifier() != 0 { 0.771 } else { 0.5 }, 1.),
                _ => {}
            }
        }
    }
    pub fn moved(&mut self, ground: bool) {
        for s in &mut self.scopes {
            if ground {
                s.air_ticks = 0;
                s.ground_ticks += 1;
            } else {
                s.air_ticks += 1;
                s.ground_ticks = 0;
            }
        }
    }
    pub fn step(&mut self, height: f64) {
        for s in &mut self.scopes {
            if s.mode == Some(21) {
                if height > 0.7 {
                    s.hop_load = 0.;
                }
                if (height - 0.5).abs() < 1e-4 {
                    s.stair_ticks = 3;
                }
            }
        }
    }
    pub fn setback(&mut self, now: i64) {
        for s in &mut self.scopes {
            match s.mode {
                Some(5) => s.was_flagged = true,
                Some(19) => s.flag_delay = 20,
                Some(21) => {
                    s.phase = 0;
                    s.recover = 4;
                    s.last_setback = Some(now);
                }
                _ => {}
            }
        }
    }
    pub fn knockback(&mut self, settings: &Settings, x: f64, z: f64, fall_damage: bool) {
        for (i, s) in self.scopes.iter_mut().enumerate() {
            let p = format!("{}.Custom.Strafe", SCOPES[i]);
            match s.mode {
                Some(1) => {
                    s.velocity_timeout = settings.number(&join(&p, "VelocityTimeout")) as i32;
                    if settings.flag(&join(&p, "Enabled"))
                        && settings.flag(&join(&p, "StrafeKnock"))
                    {
                        s.pending_knock = Some(x == 0. && z == 0. && fall_damage);
                    }
                }
                Some(5) => s.pending_knock = Some(x == 0. && z == 0. && fall_damage),
                _ => {}
            }
        }
    }
    pub fn move_self(
        &mut self,
        settings: &Settings,
        c: &mut Context,
        mut movement: [f64; 3],
    ) -> [f64; 3] {
        for (i, s) in self.scopes.iter_mut().enumerate() {
            let Some(mode) = s.mode else { continue };
            let p = format!("{}.{}", SCOPES[i], MODES[mode]);
            match mode {
                4 => {
                    if c.moving() {
                        movement =
                            strafe(movement, c.yaw, c.input, movement[0].hypot(movement[2]), 1.);
                    }
                }
                7 | 8 => {
                    if c.input & 1 != 0 {
                        if c.ground {
                            let fast = mode == 8;
                            let multiplier = if fast {
                                if c.leather_boots { 1.2 } else { 1.05 }
                            } else if c.leather_boots {
                                1.8
                            } else {
                                1.3
                            };
                            movement[0] = c.velocity[0] * multiplier;
                            movement[2] = c.velocity[2] * multiplier;
                            let count = if fast {
                                if c.leather_boots { 7 } else { 3 }
                            } else {
                                4
                            };
                            self.effects.jump_repetitions += count;
                            let y = if fast {
                                0.42
                            } else {
                                0.42 + 0.1 * c.jump_level as f64
                            };
                            c.velocity[1] = c.velocity[1].max(y);
                            movement[1] = y;
                        } else if mode == 8 && s.air_ticks == 1 {
                            self.effects.timer(0.5, 0, 0);
                            self.effects.packets.push(MovePacket {
                                y_offset: 0.,
                                ground: true,
                                full: true,
                            });
                            movement[1] = -0.0784;
                        }
                    }
                }
                9 => {
                    if c.fly {
                        self.effects.disable_speed = true;
                        continue;
                    }
                    if c.hurt > 0 && !s.has_been_hurt {
                        s.has_been_hurt = true;
                        s.adjusted = true;
                    } else if c.hurt == 10 {
                        s.external_damage_adjust =
                            ((c.now - s.last_damage) as f64 / 50.).ceil() as i32;
                    }
                    if (s.has_been_hurt || s.adjusted) && c.moving() {
                        movement = strafe(
                            movement,
                            c.yaw,
                            c.input,
                            settings.number(&join(&p, "Speed")),
                            1.,
                        );
                    }
                }
                21 => {
                    if c.towering {
                        continue;
                    }
                    s.collided |= c.horizontal_collision;
                    if s.collided {
                        s.phase = 0;
                        s.recover = 1;
                    } else if settings.flag(&join(&p, "Timer")) {
                        self.effects.timer(1.2, 20, 1);
                    }
                    s.stair_ticks = (s.stair_ticks - 1).max(0);
                    s.hop_load = (s.hop_load - if s.hop_load > 1. { 0.12 } else { 0.11 }).max(0.);
                    if !c.moving() {
                        continue;
                    }
                    let mut y = movement[1];
                    if !c.in_liquid && c.ground_within {
                        s.collided = c.horizontal_collision;
                        if s.phase != 0 || s.collided {
                            s.phase = 1;
                            if s.stair_ticks == 0 {
                                y = 0.407 + 0.1 * c.jump_level as f64;
                                c.velocity[1] = y;
                            }
                            s.hop_load += 1.;
                            s.slow_hop = s.hop_load > 1. && !s.slow_hop;
                            s.hop_load = s.hop_load.min(1.12);
                        }
                    }
                    let recent = s.last_setback.is_some_and(|t| c.now - t < 500);
                    if s.phase == 1 {
                        s.slow_after_setback = recent;
                    }
                    let base = 0.2873 * (1. + 0.2 * c.speed_level as f64)
                        / (1. + 0.2 * c.slowness_level as f64);
                    let level = c.speed_level as f64;
                    let floor = base + 0.028 * level;
                    let hop = if s.slow_after_setback {
                        if s.phase == 1 { 0. } else { 0.2 }
                    } else if recent || s.collided {
                        floor
                    } else {
                        let first = 0.4145 + level / 12.5;
                        (if s.phase == 1 {
                            0.64 + (level + 0.028 * level) * 0.134
                        } else if s.phase_ticks == 1 {
                            first
                        } else {
                            first - s.phase_ticks as f64 / 250.
                        })
                        .max(floor)
                    };
                    let mut speed = if c.in_liquid {
                        0.12
                    } else if s.phase == 0 {
                        base
                    } else if s.stair_ticks > 0 {
                        (hop + 0.0331) * 0.91 * (0.7 - level * 0.1)
                    } else {
                        (hop + 0.0331) * 0.91
                    };
                    if s.slow_hop && !c.in_liquid {
                        speed *= 0.95;
                    }
                    movement = strafe([0., y, 0.], c.yaw, c.input, speed, 1.);
                    match s.phase {
                        0 => {
                            s.recover -= 1;
                            if s.recover == 0 {
                                s.phase = 1;
                            }
                        }
                        1 => {
                            s.phase = 2;
                            s.phase_ticks = 1;
                        }
                        _ => s.phase_ticks += 1,
                    }
                }
                _ => {}
            }
        }
        movement
    }
    pub fn movement_packet(&self, c: Context, y: &mut f64, ground: &mut bool) {
        for s in &self.scopes {
            match s.mode {
                Some(11) => {
                    if c.velocity[1] < 0. {
                        *ground = true;
                    }
                }
                Some(12) => {
                    if c.bottom_collision && !c.physical_jump {
                        *y += 0.005;
                    }
                }
                _ => {}
            }
        }
    }
    pub fn take_effects(&mut self) -> Effects {
        let flag = self.effects.hypixel_should_strafe;
        let out = std::mem::take(&mut self.effects);
        self.effects.hypixel_should_strafe = flag;
        out
    }
}
