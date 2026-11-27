// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleFlagCheck, LiquidBounce ae0eb396. Packet angles remain un-resolved,
//! including relative corrections; the original performs only one wrap step.
use crate::Easing;
#[derive(Clone, Debug)]
pub struct Settings {
    pub chat: bool,
    pub notification: bool,
    pub invalid: bool,
    pub reset: bool,
    pub after: i32,
    pub render: bool,
    pub not_first: bool,
    pub alive: i32,
    pub fade: Easing,
    pub out_time: i32,
    pub color: i32,
    pub outline: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            chat: true,
            notification: false,
            invalid: false,
            reset: true,
            after: 30,
            render: true,
            not_first: true,
            alive: 1000,
            fade: Easing::QuadOut,
            out_time: 500,
            color: 0x64b20000,
            outline: 0xffb20000u32 as i32,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Player {
    pub ticks: i32,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub alive: bool,
    pub food: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub reason: Reason,
    pub extra: Option<String>,
    pub count: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Invalid,
    ForceRotate,
    Lagback,
}
impl Reason {
    pub fn key(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::ForceRotate => "forceRotate",
            Self::Lagback => "lagback",
        }
    }
}
#[derive(Clone, Debug)]
pub struct State {
    pub settings: Settings,
    pub count: i32,
    pub last_rotation: [f32; 2],
    pub position: [f64; 3],
    pub rotation: [f32; 2],
    pub created: i64,
    pub finished: bool,
    running: bool,
    reset_running: bool,
    render_running: bool,
    next_reset: i64,
}
impl Default for State {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            count: 0,
            last_rotation: [0.; 2],
            position: [0.; 3],
            rotation: [0.; 2],
            created: 0,
            finished: true,
            running: false,
            reset_running: false,
            render_running: false,
            next_reset: 0,
        }
    }
}
pub fn angle_delta(new: f32, old: f32) -> f32 {
    let mut d = new - old;
    if d > 180. {
        d -= 360.;
    }
    if d < -180. {
        d += 360.;
    }
    d.abs()
}
// Kotlin roundToLong uses Math.round, including NaN=0 and saturation.
fn round_long(v: f32) -> i64 {
    (f64::from(v) + 0.5).floor() as i64
}
impl State {
    pub fn sync(&mut self, running: bool, now: i64) {
        let reset = running && self.settings.reset;
        if reset && !self.reset_running {
            self.count = 0;
            self.next_reset = now.saturating_add(i64::from(self.settings.after) * 1000);
        }
        let render = running && self.settings.render;
        if render && !self.render_running {
            self.finished = true;
        }
        self.running = running;
        self.reset_running = reset;
        self.render_running = render;
    }
    /// The background ticker resets immediately, then delays using the value
    /// at each iteration. Settings edits do not restart the in-flight delay.
    pub fn clock(&mut self, now: i64) {
        if self.reset_running && now >= self.next_reset {
            self.count = 0;
            self.next_reset = now.saturating_add((i64::from(self.settings.after) * 1000).max(1));
        }
    }
    pub fn position(
        &mut self,
        p: Player,
        position: [f64; 3],
        rotation: [f32; 2],
        now: i64,
    ) -> Option<Alert> {
        if !self.running || p.ticks <= 25 {
            return None;
        }
        let dy = angle_delta(rotation[0], self.last_rotation[0]);
        let dp = angle_delta(rotation[1], self.last_rotation[1]);
        self.count = self.count.wrapping_add(1);
        self.created = now;
        self.finished = false;
        self.position = position;
        self.rotation = rotation;
        self.last_rotation = [p.yaw, p.pitch];
        Some(if dy >= 90. || dp >= 90. {
            Alert {
                reason: Reason::ForceRotate,
                extra: Some(format!("({}° | {}°)", round_long(dy), round_long(dp))),
                count: self.count,
            }
        } else {
            Alert {
                reason: Reason::Lagback,
                extra: None,
                count: self.count,
            }
        })
    }
    pub fn disconnect(&mut self, p: Player) {
        if self.running && p.ticks > 25 {
            self.count = 0;
        }
    }
    pub fn tick(&mut self, p: Player) -> Option<Alert> {
        if !self.running || !self.settings.invalid {
            return None;
        }
        let health = p.health <= 0. && p.alive;
        let hunger = p.food <= 0;
        if !health && !hunger {
            return None;
        }
        self.count = self.count.wrapping_add(1);
        let extra = match (health, hunger) {
            (true, true) => "Health, Hunger",
            (true, false) => "Health",
            _ => "Hunger",
        };
        Some(Alert {
            reason: Reason::Invalid,
            extra: Some(extra.into()),
            count: self.count,
        })
    }
    pub fn render_colors(&mut self, now: i64, first: bool) -> Option<[i32; 2]> {
        if !self.running
            || !self.settings.render
            || self.finished
            || (self.settings.not_first && first)
        {
            return None;
        }
        let factor = if now.wrapping_sub(self.created) < i64::from(self.settings.alive) {
            1.
        } else {
            1. - self.settings.fade.get_factor(
                self.created.wrapping_add(i64::from(self.settings.alive)),
                now,
                self.settings.out_time as f32,
            )
        };
        if factor == 0. {
            self.finished = true;
            return None;
        }
        Some([self.settings.color, self.settings.outline].map(|c| {
            let alpha = ((c as u32 >> 24) as f32 * factor) as u32;
            ((c as u32 & 0x00ffffff) | (alpha.min(255) << 24)) as i32
        }))
    }
}
