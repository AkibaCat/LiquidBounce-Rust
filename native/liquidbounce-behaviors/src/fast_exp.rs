// SPDX-License-Identifier: GPL-3.0-or-later
// LiquidBounce ae0eb396 ModuleFastExp; each tickUntil is a real FIFO suspension.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub rotate: bool,
    pub no_waste: bool,
    pub min_durability: i32,
    pub max_percent: i32,
    pub fast: bool,
    pub ticks_per_item: [f32; 2],
    pub items_per_tick: [f32; 2],
    pub combat_pause: i32,
    pub slot_reset: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            rotate: true,
            no_waste: true,
            min_durability: 64,
            max_percent: 85,
            fast: false,
            ticks_per_item: [2., 3.],
            items_per_tick: [3., 5.],
            combat_pause: 0,
            slot_reset: [0, 0],
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct RepairItem {
    pub damage: i32,
    pub max_damage: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub valid: bool,
    pub handled: bool,
    pub slot: Option<i32>,
    pub count: i32,
    pub orbs: bool,
    pub items: Vec<RepairItem>,
    pub pitch: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Idle,
    BeforeOrbs(i32),
    Aim,
    Appear(i32),
    Absorb,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Effects {
    pub pause: bool,
    pub rotation: bool,
    pub throws: i32,
    pub pitch: f32,
    pub register: Option<u64>,
}
pub struct FastExp {
    pub settings: Settings,
    pub bottles_required: i32,
    pub bottles_used: i32,
    pub items_to_throw: f32,
    pub repairing: bool,
    pub stage: Stage,
    pub callback: u64,
    pub repair_run: bool,
}
impl Default for FastExp {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            bottles_required: 0,
            bottles_used: 0,
            items_to_throw: 0.,
            repairing: false,
            stage: Stage::Idle,
            callback: 0,
            repair_run: false,
        }
    }
}
impl FastExp {
    pub fn cancel(&mut self) {
        self.callback = self.callback.wrapping_add(1);
        self.stage = Stage::Idle;
        self.bottles_required = 0;
        self.bottles_used = 0;
        self.repairing = false;
    }
    pub fn disabled(&mut self) {
        self.cancel();
        self.items_to_throw = 0.;
    }
    fn wait(&mut self, stage: Stage, e: &mut Effects) {
        self.callback = self.callback.wrapping_add(1);
        self.stage = stage;
        e.register = Some(self.callback);
    }
    pub fn required(&mut self, c: &Context) -> i32 {
        if !self.settings.no_waste {
            return i32::MAX;
        }
        if self.bottles_used == 0
            && self.repairing
            && !c.items.iter().any(|i| {
                100. * (i.max_damage.wrapping_sub(i.damage)) as f32 / i.max_damage as f32
                    <= self.settings.max_percent as f32
            })
        {
            self.repairing = false;
            return 0;
        }
        if self.bottles_used == 0
            && !self.repairing
            && !c
                .items
                .iter()
                .any(|i| i.max_damage.wrapping_sub(i.damage) <= self.settings.min_durability)
        {
            return 0;
        }
        let damage = c.items.iter().fold(0i32, |a, i| a.wrapping_add(i.damage));
        (damage / 2 / 7).min(c.count)
    }
    pub fn start(&mut self, c: &Context, random: &mut impl FnMut([f32; 2]) -> f32) -> Effects {
        let mut e = Effects::default();
        if self.stage != Stage::Idle || c.handled {
            return e;
        }
        if !c.valid || c.slot.is_none() {
            self.cancel();
            return e;
        }
        self.repair_run = self.settings.no_waste;
        if self.settings.no_waste {
            let n = self.required(c);
            if n < 1 {
                self.bottles_used = 0;
                self.bottles_required = 0;
                return e;
            }
            self.wait(Stage::BeforeOrbs(n), &mut e);
        } else {
            self.action(c, random, &mut e);
        }
        e
    }
    fn action(&mut self, c: &Context, random: &mut impl FnMut([f32; 2]) -> f32, e: &mut Effects) {
        e.pause = true;
        if self.settings.rotate {
            self.wait(Stage::Aim, e);
        } else {
            self.throw(c, random, e);
        }
    }
    fn throw(&mut self, c: &Context, random: &mut impl FnMut([f32; 2]) -> f32, e: &mut Effects) {
        self.items_to_throw += if self.settings.fast {
            random(self.settings.items_per_tick)
        } else {
            1. / random(self.settings.ticks_per_item)
        };
        let times = self.items_to_throw as i32;
        self.items_to_throw -= times as f32;
        // Kotlin repeat rejects negative raw imported values before any item use.
        if times < 0 {
            self.cancel();
            return;
        }
        e.throws = times;
        e.pitch = if self.settings.rotate { c.pitch } else { 90. };
        if self.settings.no_waste {
            self.bottles_used = self.bottles_used.wrapping_add(times);
        }
        self.stage = Stage::Idle;
        if self.repair_run && self.bottles_used > 0 && self.bottles_used >= self.bottles_required {
            self.wait(Stage::Appear(0), e);
        }
    }
    pub fn resume(
        &mut self,
        id: u64,
        c: &Context,
        random: &mut impl FnMut([f32; 2]) -> f32,
    ) -> (bool, Effects) {
        let mut e = Effects::default();
        if id != self.callback || self.stage == Stage::Idle {
            return (false, e);
        }
        if !c.valid || c.slot.is_none() {
            self.cancel();
            return (false, e);
        }
        match self.stage {
            Stage::BeforeOrbs(n) => {
                if !self.repairing && c.orbs {
                    return (true, e);
                }
                self.repairing = true;
                self.bottles_required = self.bottles_required.max(n);
                self.action(c, random, &mut e);
            }
            Stage::Aim => {
                e.rotation = true;
                if c.pitch <= 85. {
                    return (true, e);
                }
                self.throw(c, random, &mut e);
            }
            Stage::Appear(t) => {
                let t = t + 1;
                if t < 10 && !c.orbs {
                    self.stage = Stage::Appear(t);
                    return (true, e);
                }
                self.wait(Stage::Absorb, &mut e);
            }
            Stage::Absorb => {
                if c.orbs {
                    return (true, e);
                }
                self.bottles_used = 0;
                self.bottles_required = 0;
                self.stage = Stage::Idle;
            }
            Stage::Idle => {}
        }
        (false, e)
    }
}
