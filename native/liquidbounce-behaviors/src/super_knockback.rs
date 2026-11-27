// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSuperKnockback at ae0eb396, with distinct coroutine FIFO waiters.
#[path = "super_knockback_cooldown.rs"]
pub mod cooldown;
#[path = "navigation/java_case.rs"]
mod java_case;
pub const MODES: [&str; 3] = ["Packet", "SprintTap", "WTap"];
pub const CONDITIONS: [&str; 3] = ["OnlyFacing", "OnlyOnGround", "NotInWater"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Packet,
    SprintTap,
    WTap,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(v: &str) -> Option<Self> {
        [Self::Packet, Self::SprintTap, Self::WTap]
            .into_iter()
            .find(|m| java_case::equals_ignore_case(v, m.tag()))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub forward: f32,
    pub sideways: f32,
    pub sprinting: bool,
    pub was_sprinting: bool,
    pub on_ground: bool,
    pub water_or_bubble: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub living: bool,
    pub hurt_time: i32,
    pub facing: bool,
    pub would_critical: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub hurt_time: i32,
    pub chance: f32,
    pub conditions: u32,
    pub only_on_move: bool,
    pub only_forward: bool,
    pub re_sprint: [i32; 2],
    pub until_movement_block: [i32; 2],
    pub until_allowed_movement: [i32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Packet,
            hurt_time: 10,
            chance: 100.0,
            conditions: 4,
            only_on_move: true,
            only_forward: true,
            re_sprint: [0, 1],
            until_movement_block: [0, 1],
            until_allowed_movement: [0, 1],
        }
    }
}
pub trait Random {
    fn unit(&mut self) -> f32;
    fn ticks(&mut self, range: [i32; 2]) -> Result<i32, String>;
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Step {
    pub packets: Vec<bool>,
    pub mark_sprint: bool,
    pub wait: Option<u64>,
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    SprintStop,
    SprintDelay(i32),
    MoveDelay(i32),
    MoveStop,
    MoveAllow(i32),
}
#[derive(Clone, Copy, Debug)]
struct Wait {
    id: u64,
    phase: Phase,
    elapsed: i32,
}
#[derive(Debug, Default)]
pub struct SuperKnockback {
    pub settings: Settings,
    pub cancel_sprint: bool,
    pub in_sequence: bool,
    pub cancel_movement: bool,
    next: u64,
    wait: Option<Wait>,
}
impl SuperKnockback {
    pub fn set_mode(&mut self, mode: Mode) {
        if self.settings.mode != mode {
            self.cancel();
            self.settings.mode = mode;
        }
    }
    pub fn cancel(&mut self) {
        self.cancel_sprint = false;
        self.in_sequence = false;
        self.cancel_movement = false;
        self.wait = None;
    }
    fn wait(&mut self, phase: Phase) -> Step {
        self.next = self
            .next
            .checked_add(1)
            .expect("SuperKnockback waiter exhausted");
        self.wait = Some(Wait {
            id: self.next,
            phase,
            elapsed: 0,
        });
        Step {
            wait: Some(self.next),
            ..Step::default()
        }
    }
    fn chance(&self, r: &mut impl Random) -> bool {
        match self.settings.chance {
            0.0 => false,
            100.0 => true,
            p => {
                let v = r.unit() * 100.0;
                let v = if v >= 100.0 {
                    f32::from_bits(100.0f32.to_bits() - 1)
                } else {
                    v
                };
                v < p
            }
        }
    }
    pub fn should_operate(&self, p: Player, t: Target) -> bool {
        if self.settings.only_on_move {
            let sideways = p.sideways != 0.0;
            if !(p.forward != 0.0 || sideways) || (self.settings.only_forward && sideways) {
                return false;
            }
        }
        (self.settings.conditions & 1 == 0 || t.facing)
            && (self.settings.conditions & 2 == 0 || p.on_ground)
            && (self.settings.conditions & 4 == 0 || !p.water_or_bubble)
    }
    pub fn attack(
        &mut self,
        running: bool,
        p: Player,
        t: Target,
        r: &mut impl Random,
    ) -> Result<Step, String> {
        if !running || !self.should_operate(p, t) {
            return Ok(Step::default());
        }
        if self.settings.mode != Mode::Packet && (!p.sprinting || !p.was_sprinting) {
            return Ok(Step::default());
        }
        // Source evaluates chance before its already-running-sequence guard.
        if !t.living || t.hurt_time > self.settings.hurt_time || !self.chance(r) || t.would_critical
        {
            return Ok(Step::default());
        }
        Ok(match self.settings.mode {
            Mode::Packet => Step {
                packets: if p.sprinting {
                    vec![false, true, false, true]
                } else {
                    vec![true, false, true]
                },
                mark_sprint: true,
                wait: None,
            },
            Mode::SprintTap => {
                if self.cancel_sprint {
                    return Ok(Step::default());
                }
                self.cancel_sprint = true;
                self.wait(Phase::SprintStop)
            }
            Mode::WTap => {
                if self.in_sequence {
                    return Ok(Step::default());
                }
                self.in_sequence = true;
                let ticks = r.ticks(self.settings.until_movement_block)?;
                if ticks == 0 {
                    self.cancel_movement = true;
                    self.wait(Phase::MoveStop)
                } else {
                    self.wait(Phase::MoveDelay(ticks))
                }
            }
        })
    }
    pub fn callback(
        &mut self,
        id: u64,
        running: bool,
        p: Option<Player>,
        r: &mut impl Random,
    ) -> Result<(bool, Step), String> {
        let Some(mut wait) = self.wait.filter(|w| w.id == id) else {
            return Ok((false, Step::default()));
        };
        if !running {
            self.cancel();
            return Ok((false, Step::default()));
        }
        wait.elapsed = wait.elapsed.wrapping_add(1);
        let finished = match wait.phase {
            Phase::SprintStop => p.is_none_or(|p| !p.sprinting && !p.was_sprinting),
            Phase::SprintDelay(t) | Phase::MoveDelay(t) | Phase::MoveAllow(t) => wait.elapsed >= t,
            Phase::MoveStop => p.is_some_and(|p| !(p.forward > 1.0e-5)),
        };
        if !finished {
            self.wait = Some(wait);
            return Ok((true, Step::default()));
        }
        self.wait = None;
        let step = match wait.phase {
            Phase::SprintStop => {
                let ticks = r.ticks(self.settings.re_sprint)?;
                if ticks == 0 {
                    self.cancel_sprint = false;
                    Step::default()
                } else {
                    self.wait(Phase::SprintDelay(ticks))
                }
            }
            Phase::SprintDelay(_) => {
                self.cancel_sprint = false;
                Step::default()
            }
            Phase::MoveDelay(_) => {
                self.cancel_movement = true;
                self.wait(Phase::MoveStop)
            }
            Phase::MoveStop => {
                let ticks = r.ticks(self.settings.until_allowed_movement)?;
                if ticks == 0 {
                    self.cancel();
                    Step::default()
                } else {
                    self.wait(Phase::MoveAllow(ticks))
                }
            }
            Phase::MoveAllow(_) => {
                self.cancel();
                Step::default()
            }
        };
        Ok((false, step))
    }
    pub fn stop_sprint(&self, running: bool) -> bool {
        running && self.settings.mode == Mode::SprintTap && self.cancel_sprint
    }
    pub fn block_movement(&self, running: bool) -> bool {
        running && self.settings.mode == Mode::WTap && self.in_sequence && self.cancel_movement
    }
}
