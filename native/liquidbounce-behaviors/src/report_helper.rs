// SPDX-License-Identifier: GPL-3.0-or-later
//! ReportHelper, AutoReport and AutoConfirm from LiquidBounce ae0eb396.
//! Each source sequence owns one callback in the shared CoroutineTicker.
#[path = "report_helper_format.rs"]
mod formatter;
#[path = "navigation/java_case.rs"]
mod java_case;
pub use formatter::format_command;

pub const MODES: [&str; 2] = ["Hypixel", "Heypixel"];
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub report: bool,
    pub delay: [i32; 2],
    pub chance: i32,
    pub pattern: String,
    pub confirm: bool,
    pub mode: usize,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            report: false,
            delay: [1, 3],
            chance: 100,
            pattern: "report %s".into(),
            confirm: false,
            mode: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub name: String,
    pub local: bool,
    pub friend: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    Wait(u64),
    Send(String),
}
#[derive(Clone, Debug)]
struct PendingReport {
    id: u64,
    name: String,
    ticks: i32,
    elapsed: i32,
}
#[derive(Clone, Debug)]
struct PendingConfirm {
    id: u64,
    screen: u64,
    mode: usize,
    elapsed: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirm {
    Keep,
    Stop,
    Inspect { screen: u64, mode: usize },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub index: i32,
    pub item: String,
    pub empty: bool,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub settings: Settings,
    reported: Vec<String>,
    reports: Vec<PendingReport>,
    confirms: Vec<PendingConfirm>,
    next: u64,
    session: Option<u64>,
}
impl State {
    fn next(&mut self) -> u64 {
        self.next = self
            .next
            .checked_add(1)
            .expect("ReportHelper callback exhausted");
        self.next
    }
    pub fn cancel(&mut self) {
        self.reports.clear();
        self.confirms.clear();
    }
    pub fn disabled(&mut self) {
        self.reported.clear();
        self.cancel();
    }
    pub fn session(&mut self, identity: u64) {
        if self
            .session
            .replace(identity)
            .is_some_and(|old| old != identity)
        {
            self.reported.clear();
        }
    }
    pub fn set_report(&mut self, enabled: bool) {
        if self.settings.report && !enabled {
            self.reported.clear();
            self.reports.clear();
        }
        self.settings.report = enabled;
    }
    pub fn set_confirm(&mut self, enabled: bool) {
        if self.settings.confirm && !enabled {
            self.confirms.clear();
        }
        self.settings.confirm = enabled;
    }
    pub fn set_mode(&mut self, mode: usize) {
        if self.settings.mode != mode {
            self.confirms.clear();
        }
        self.settings.mode = mode;
    }
    pub fn message(
        &mut self,
        running: bool,
        message: &str,
        self_name: &str,
        players: &[Player],
        mut random: impl FnMut(i32, i32) -> i32,
    ) -> Option<Report> {
        if !running || !self.settings.report || !message.contains(self_name) {
            return None;
        }
        // world.players() order and exact case-sensitive FriendManager names.
        let name = &players
            .iter()
            .find(|p| !p.local && p.name != self_name && message.contains(&p.name) && !p.friend)?
            .name;
        // Kotlin consumes chance randomness before the case-insensitive set insertion.
        if random(0, 99) >= self.settings.chance
            || self
                .reported
                .iter()
                .any(|old| java_case::equals_ignore_case(old, name))
        {
            return None;
        }
        self.reported.push(name.clone());
        let ticks = random(self.settings.delay[0], self.settings.delay[1]);
        if ticks == 0 {
            return Some(Report::Send(name.clone()));
        }
        let id = self.next();
        self.reports.push(PendingReport {
            id,
            name: name.clone(),
            ticks,
            elapsed: 0,
        });
        Some(Report::Wait(id))
    }
    pub fn report_callback(
        &mut self,
        id: u64,
        running: bool,
        in_game: bool,
    ) -> (bool, Option<String>) {
        let Some(index) = self.reports.iter().position(|p| p.id == id) else {
            return (false, None);
        };
        if !running || !in_game || !self.settings.report {
            self.reports.remove(index);
            return (false, None);
        }
        let pending = &mut self.reports[index];
        pending.elapsed += 1;
        if pending.elapsed < pending.ticks {
            return (true, None);
        }
        (false, Some(self.reports.remove(index).name))
    }
    pub fn screen(&mut self, running: bool, screen: u64) -> Option<u64> {
        if !running || !self.settings.confirm {
            return None;
        }
        let id = self.next();
        self.confirms.push(PendingConfirm {
            id,
            screen,
            mode: self.settings.mode,
            elapsed: 0,
        });
        Some(id)
    }
    pub fn confirm_screen(&self, id: u64) -> Option<u64> {
        self.confirms.iter().find(|p| p.id == id).map(|p| p.screen)
    }
    pub fn confirm_callback(
        &mut self,
        id: u64,
        running: bool,
        in_game: bool,
        screen_is_current: bool,
    ) -> Confirm {
        let Some(index) = self.confirms.iter().position(|p| p.id == id) else {
            return Confirm::Stop;
        };
        let pending = &mut self.confirms[index];
        if !running || !in_game || !self.settings.confirm || pending.mode != self.settings.mode {
            self.confirms.remove(index);
            return Confirm::Stop;
        }
        pending.elapsed += 1;
        // tickUntil increments before the predicate; tickConditional returns
        // elapsed >= 5 even when the captured object first matches at tick 5.
        if pending.elapsed >= 5 {
            self.confirms.remove(index);
            return Confirm::Stop;
        }
        if !screen_is_current {
            return Confirm::Keep;
        }
        let pending = self.confirms.remove(index);
        Confirm::Inspect {
            screen: pending.screen,
            mode: pending.mode,
        }
    }
}
pub fn confirm_slot(mode: usize, slots: &[Slot]) -> Option<i32> {
    match mode {
        0 => {
            const EMPTY: [usize; 24] = [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
                26,
            ];
            if slots.len() != 27
                || EMPTY.iter().any(|&index| !slots[index].empty)
                || slots[11].empty
                || slots[13].empty
                || slots[15].empty
                || slots[11].item != "minecraft:green_terracotta"
                || slots[13].item != "minecraft:player_head"
                || slots[15].item != "minecraft:red_terracotta"
            {
                return None;
            }
            Some(11)
        }
        1 if slots.len() == 9 => slots
            .iter()
            .find(|s| !s.empty && s.item == "minecraft:diamond_sword")
            .map(|s| s.index),
        _ => None,
    }
}
