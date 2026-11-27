// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleFastUse ae0eb396. Native operations run outside the behavior borrow.
#[path = "navigation/java_case.rs"]
mod java_case;
pub const MODES: [&str; 3] = ["Immediate", "ItemUseTime", "Crossbow"];
pub const CONDITIONS: [&str; 3] = ["NotInTheAir", "NotDuringMove", "NotDuringRegeneration"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Immediate,
    ItemUseTime,
    Crossbow,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(s: &str) -> Option<Self> {
        [Self::Immediate, Self::ItemUseTime, Self::Crossbow]
            .into_iter()
            .find(|v| java_case::equals_ignore_case(v.tag(), s))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub conditions: u32,
    pub stop_input: bool,
    pub packet_type: u8,
    pub delay: i32,
    pub timer: f32,
    pub immediate_speed: i32,
    pub consume_time: i32,
    pub timed_speed: i32,
    pub tick_cooldown: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Immediate,
            conditions: 1,
            stop_input: false,
            packet_type: 3,
            delay: 0,
            timer: 1.0,
            immediate_speed: 20,
            consume_time: 15,
            timed_speed: 20,
            tick_cooldown: 1,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub ground: bool,
    pub moving: bool,
    pub regeneration: bool,
    pub using: bool,
    pub consumable: bool,
    pub ticks_using: i32,
    pub crossbow: bool,
    pub player_ticks: i32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Step {
    pub timer: Option<(f32, i32)>,
    pub wait: Option<u64>,
    pub send: i32,
    pub release: bool,
    pub crossbow: bool,
}
#[derive(Clone, Copy, Debug)]
struct Pending {
    id: u64,
    elapsed: i32,
    ticks: i32,
}
#[derive(Clone, Debug, Default)]
pub struct FastUse {
    pub settings: Settings,
    pending: Option<Pending>,
    next_id: u64,
}
impl FastUse {
    pub fn set_mode(&mut self, mode: Mode) {
        if self.settings.mode != mode {
            self.cancel();
            self.settings.mode = mode;
        }
    }
    pub fn cancel(&mut self) {
        self.pending = None;
    }
    pub fn accelerate(&self, c: Context) -> bool {
        c.using
            && c.consumable
            && !(self.settings.conditions & 1 != 0 && !c.ground
                || self.settings.conditions & 2 != 0 && c.moving
                || self.settings.conditions & 4 != 0 && c.regeneration)
    }
    pub fn game_tick(&mut self, running: bool, c: Context) -> Step {
        if !running || self.pending.is_some() {
            return Step::default();
        }
        let s = self.settings;
        match s.mode {
            Mode::Immediate if self.accelerate(c) => {
                let mut step = Step {
                    timer: Some((s.timer, s.delay.wrapping_add(1))),
                    ..Step::default()
                };
                if s.delay == 0 {
                    step.send = s.immediate_speed.max(0);
                    step.release = true;
                } else {
                    self.next_id = self
                        .next_id
                        .checked_add(1)
                        .expect("FastUse callback exhausted");
                    self.pending = Some(Pending {
                        id: self.next_id,
                        elapsed: 0,
                        ticks: s.delay,
                    });
                    step.wait = Some(self.next_id);
                }
                step
            }
            Mode::ItemUseTime if self.accelerate(c) && c.ticks_using >= s.consume_time => Step {
                send: s.timed_speed.max(0),
                release: true,
                ..Step::default()
            },
            Mode::Crossbow
                if c.using
                    && c.crossbow
                    && s.tick_cooldown != 0
                    && c.player_ticks.wrapping_rem(s.tick_cooldown) == 0 =>
            {
                Step {
                    crossbow: true,
                    ..Step::default()
                }
            }
            _ => Step::default(),
        }
    }
    pub fn callback(&mut self, id: u64, running: bool, c: Context) -> (bool, Step) {
        let Some(mut p) = self.pending.filter(|p| p.id == id) else {
            return (false, Step::default());
        };
        if !running || self.settings.mode != Mode::Immediate || !self.accelerate(c) {
            self.cancel();
            return (false, Step::default());
        }
        p.elapsed = p.elapsed.wrapping_add(1);
        if p.elapsed < p.ticks {
            self.pending = Some(p);
            (true, Step::default())
        } else {
            self.pending = None;
            (
                false,
                Step {
                    send: self.settings.immediate_speed.max(0),
                    release: true,
                    ..Step::default()
                },
            )
        }
    }
    pub fn stop_input(&self, running: bool, use_key: bool) -> bool {
        running && self.settings.stop_input && use_key
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn immediate_timed_crossbow_and_cancellation_smoke() {
        let mut m = FastUse::default();
        let c = Context {
            ground: true,
            using: true,
            consumable: true,
            ..Context::default()
        };
        let step = m.game_tick(true, c);
        assert_eq!(step.send, 20);
        assert!(step.release);
        m.settings.delay = 2;
        let id = m.game_tick(true, c).wait.unwrap();
        assert!(m.callback(id, true, c).0);
        m.settings.immediate_speed = 7;
        let (keep, step) = m.callback(id, true, c);
        assert!(!keep && step.release);
        assert_eq!(step.send, 7);
        let id = m.game_tick(true, c).wait.unwrap();
        assert_eq!(
            m.callback(id, true, Context { ground: false, ..c }),
            (false, Step::default())
        );
        m.set_mode(Mode::ItemUseTime);
        assert!(
            !m.game_tick(
                true,
                Context {
                    ticks_using: 14,
                    ..c
                }
            )
            .release
        );
        assert!(
            m.game_tick(
                true,
                Context {
                    ticks_using: 15,
                    ..c
                }
            )
            .release
        );
        m.set_mode(Mode::Crossbow);
        m.settings.tick_cooldown = 3;
        assert!(
            m.game_tick(
                true,
                Context {
                    crossbow: true,
                    player_ticks: 6,
                    ..c
                }
            )
            .crossbow
        );
        assert!(
            !m.game_tick(
                true,
                Context {
                    crossbow: true,
                    player_ticks: 7,
                    ..c
                }
            )
            .crossbow
        );
        m.settings.stop_input = true;
        assert!(m.stop_input(true, true));
        assert!(!m.stop_input(false, true));
    }
}
