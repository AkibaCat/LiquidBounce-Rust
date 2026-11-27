// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoQueue from LiquidBounce ae0eb396. Native observations and effects are
//! supplied by the actor; tick waits share its CoroutineTicker FIFO.
#[path = "navigation/java_case_data.rs"]
mod java_case_data;
pub const PRESETS: [&str; 3] = ["HypixelSW", "GommeDuels", "Custom"];
// The pinned Subtitle object is also named Title. Gson overwrites the first
// choice and setByString selects the first. Preserve this upstream collision.
pub const TRIGGERS: [&str; 6] = [
    "Title",
    "Title",
    "Message",
    "Item",
    "TabHeader",
    "TabFooter",
];
pub const ACTIONS: [&str; 2] = ["Chat", "UseItem"];
pub const PAUSES: [&str; 4] = [
    "ClickGuiOpen",
    "ChatScreenOpen",
    "ContainerScreenOpen",
    "PauseScreenOpen",
];
pub const CHAT_TYPES: [&str; 3] = ["ChatMessage", "DisguisedChatMessage", "GameMessage"];
pub const MATCHES: [&str; 8] = [
    "Equals",
    "EqualsIgnoreCase",
    "Contains",
    "ContainsIgnoreCase",
    "StartsWith",
    "StartsWithIgnoreCase",
    "EndsWith",
    "EndsWithIgnoreCase",
];
pub const GAMES: [&str; 2] = ["Normal", "Insane"];
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    pub mode: usize,
    pub names: Vec<String>,
    pub match_mode: usize,
    pub items: Vec<String>,
}
impl Default for Pick {
    fn default() -> Self {
        Self {
            mode: 0,
            names: vec!["Paper".into()],
            match_mode: 0,
            items: vec!["minecraft:paper".into()],
        }
    }
}
fn folded(s: &str) -> Vec<u32> {
    s.chars()
        .map(|c| {
            let p = u32::from(c);
            java_case_data::SIMPLE_FOLD
                .binary_search_by_key(&p, |&(a, _)| a)
                .map_or(p, |i| java_case_data::SIMPLE_FOLD[i].1)
        })
        .collect()
}
impl Pick {
    pub fn matches(&self, slot: &Slot) -> bool {
        if self.mode == 1 {
            return self.items.iter().any(|v| v == &slot.item);
        }
        self.names.iter().any(|name| {
            if self.match_mode & 1 == 1 {
                let a = folded(&slot.name);
                let b = folded(name);
                match self.match_mode / 2 {
                    0 => a == b,
                    1 => b.is_empty() || a.windows(b.len()).any(|w| w == b),
                    2 => a.starts_with(&b),
                    _ => a.ends_with(&b),
                }
            } else {
                match self.match_mode / 2 {
                    0 => slot.name == *name,
                    1 => slot.name.contains(name),
                    2 => slot.name.starts_with(name),
                    _ => slot.name.ends_with(name),
                }
            }
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub preset: usize,
    pub pause: u32,
    pub game: usize,
    pub win: String,
    pub lose: String,
    pub control_aura: bool,
    pub trigger: usize,
    pub action: usize,
    pub title: Vec<String>,
    pub subtitle: Vec<String>,
    pub messages: Vec<String>,
    pub chat_types: u32,
    pub trigger_pick: Pick,
    pub header: String,
    pub footer: String,
    pub start_delay: [i32; 2],
    pub message_delay: [i32; 2],
    pub sends: Vec<String>,
    pub action_pick: Pick,
    pub control: bool,
    pub kill_aura: bool,
    pub speed: bool,
    pub wait_world: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            preset: 0,
            pause: 15,
            game: 0,
            win: "GG, nice try".into(),
            lose: "GG, bist wohl besser als ich!".into(),
            control_aura: true,
            trigger: 0,
            action: 0,
            title: vec!["Win".into(), "胜利".into()],
            subtitle: vec!["恭喜".into()],
            messages: vec!["Новая игра".into(), "游戏结束".into()],
            chat_types: 4,
            trigger_pick: Pick::default(),
            header: "Duels".into(),
            footer: "Duels".into(),
            start_delay: [0, 0],
            message_delay: [0, 0],
            sends: vec!["/play solo_normal".into()],
            action_pick: Pick::default(),
            control: true,
            kill_aura: true,
            speed: false,
            wait_world: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub slot: i32,
    pub item: String,
    pub name: String,
}
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub now: i64,
    pub pause: u32,
    pub selected: i32,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub slots: Vec<Slot>,
    pub npc: Option<i32>,
    pub aura_running: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Command(String),
    Chat(String),
    ChatOrCommand(String),
    Select { slot: i32, owner: bool },
    Use(i32),
    RawUse,
    Interact(i32),
    Control(&'static str, bool),
    Notice(&'static str, bool),
    EnableInfo,
    ResetGomme,
    Register(u64),
}
#[derive(Clone, Debug)]
enum Stage {
    Cooldown,
    GommeHead,
    CustomChat { messages: Vec<String>, index: usize },
    CustomUse(i32),
    CustomWorld,
    CustomDone,
    Result(bool),
}
#[derive(Clone, Debug)]
enum Wait {
    Ticks(i32),
    Millis(i64),
    Async(i64),
    World,
}
#[derive(Clone, Debug)]
struct Task {
    id: u64,
    wait: Wait,
    stage: Stage,
    guard: Option<i32>,
    main: bool,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub settings: Settings,
    pub in_match: bool,
    pub was_in_queue: bool,
    pub latches: [bool; 3],
    tasks: Vec<Task>,
    next: u64,
    pub running: bool,
    pub reset_gomme: bool,
    pub info: bool,
}
impl State {
    pub fn enabled(&mut self) {
        if self.settings.preset == 1 {
            self.info = true;
        }
    }
    pub fn cancel(&mut self) {
        self.tasks.clear();
    }
    pub fn disabled(&mut self) {
        self.cancel();
        if self.settings.preset == 1 {
            self.in_match = false;
            self.reset_gomme = true;
        }
        self.running = false;
    }
    pub fn set_preset(&mut self, preset: usize) {
        if preset != self.settings.preset {
            self.disabled();
            self.settings.preset = preset;
            self.enabled();
        }
    }
    pub fn set_trigger(&mut self, trigger: usize) {
        self.settings.trigger = trigger;
    }
    fn add(
        &mut self,
        wait: Wait,
        stage: Stage,
        guard: Option<i32>,
        main: bool,
        e: &mut Vec<Effect>,
    ) {
        self.next = self
            .next
            .checked_add(1)
            .expect("AutoQueue task identity exhausted");
        let id = self.next;
        let async_wait = matches!(wait, Wait::Async(_));
        self.tasks.push(Task {
            id,
            wait,
            stage,
            guard,
            main,
        });
        if !async_wait {
            e.push(Effect::Register(id));
        }
    }
    fn control(&self, in_queue: bool, e: &mut Vec<Effect>) {
        if self.settings.control {
            if self.settings.kill_aura {
                e.push(Effect::Control("KillAura", !in_queue));
            }
            if self.settings.speed {
                e.push(Effect::Control("Speed", !in_queue));
            }
        }
    }
    fn guard(&self, task: &Task, c: &Context) -> bool {
        task.guard.is_none_or(|slot| {
            c.slots
                .iter()
                .find(|s| s.slot == slot)
                .is_some_and(|s| self.settings.action_pick.matches(s))
        })
    }
    fn custom_done(&mut self, guard: Option<i32>, e: &mut Vec<Effect>) {
        if self.settings.wait_world {
            self.add(Wait::World, Stage::CustomWorld, guard, true, e);
        } else {
            self.add(Wait::Ticks(20), Stage::CustomDone, guard, true, e);
        }
    }
    fn chat_next(
        &mut self,
        messages: Vec<String>,
        index: usize,
        c: &Context,
        random: &mut dyn FnMut(i32, i32) -> i32,
        e: &mut Vec<Effect>,
    ) {
        if index >= messages.len() {
            self.custom_done(None, e);
            return;
        }
        let range = if index == 0 {
            self.settings.start_delay
        } else {
            self.settings.message_delay
        };
        let delay = random(range[0], range[1]).max(0);
        if delay == 0 {
            e.push(Effect::ChatOrCommand(messages[index].clone()));
            self.chat_next(messages, index + 1, c, random, e);
        } else {
            self.add(
                Wait::Async(c.now.saturating_add(i64::from(delay))),
                Stage::CustomChat { messages, index },
                None,
                true,
                e,
            );
        }
    }
    pub fn event(
        &mut self,
        running: bool,
        source: u8,
        kind: u8,
        message: &str,
        now: i64,
    ) -> Vec<Effect> {
        let mut e = Vec::new();
        if !running {
            return e;
        }
        if self.settings.preset == 2 {
            let (index, keywords) = match (source, self.settings.trigger) {
                (1, 0) => (0, &self.settings.title),
                (2, 1) => (1, &self.settings.subtitle),
                (0, 2) if self.settings.chat_types & (1 << kind) != 0 => {
                    (2, &self.settings.messages)
                }
                _ => return e,
            };
            if keywords.iter().any(|v| message.contains(v)) {
                self.latches[index] = true;
            }
        } else if self.settings.preset == 1 && source == 0 && kind == 2 {
            let notice =
                if message.contains("Du hast deine zuletzt genutzte Warteschlange für das Kit") {
                    Some(("Queue started", false))
                } else if message.contains("Du bist bereits in einer Warteschlange") {
                    Some(("Already in queue", true))
                } else if message.contains("Das Match kann beginnen") {
                    self.in_match = true;
                    Some(("Match started", false))
                } else if message.contains("und das Match gewonnen") {
                    self.in_match = false;
                    self.add(
                        Wait::Millis(now.saturating_add(2000)),
                        Stage::Result(true),
                        None,
                        false,
                        &mut e,
                    );
                    Some(("Match won", false))
                } else if message.contains("Du wurdest von") && message.contains("getötet") {
                    self.in_match = false;
                    self.add(
                        Wait::Millis(now.saturating_add(2000)),
                        Stage::Result(false),
                        None,
                        false,
                        &mut e,
                    );
                    Some(("Match lost", false))
                } else {
                    None
                };
            if let Some((text, error)) = notice {
                e.insert(0, Effect::Notice(text, error));
            }
        }
        e
    }
    pub fn tick(
        &mut self,
        running: bool,
        c: &Context,
        mut random: impl FnMut(i32, i32) -> i32,
    ) -> Vec<Effect> {
        let mut e = Vec::new();
        self.running = running;
        if self.reset_gomme {
            e.push(Effect::ResetGomme);
            self.reset_gomme = false;
        }
        if self.info {
            if running && self.settings.preset == 1 {
                e.push(Effect::EnableInfo);
            }
            self.info = false;
        }
        if !running || self.tasks.iter().any(|t| t.main) || self.settings.pause & c.pause != 0 {
            return e;
        }
        match self.settings.preset {
            0 => {
                if c.slots
                    .iter()
                    .any(|s| s.slot >= 0 && s.item == "minecraft:paper")
                {
                    e.push(Effect::Command(format!(
                        "play {}",
                        if self.settings.game == 0 {
                            "solo_normal"
                        } else {
                            "solo_insane"
                        }
                    )));
                    self.add(Wait::Ticks(20), Stage::Cooldown, None, true, &mut e);
                }
            }
            1 => {
                let Some(header) = c.header.as_ref() else {
                    self.in_match = false;
                    return e;
                };
                if !header.contains("GommeHD.net") {
                    self.in_match = false;
                    e.push(Effect::Notice("Not on GommeHD.net", true));
                    self.add(Wait::Ticks(20), Stage::Cooldown, None, true, &mut e);
                } else if header.contains("Lobby") {
                    self.in_match = false;
                    if let Some(npc) = c.npc {
                        e.push(Effect::Interact(npc));
                        e.push(Effect::Notice("Interacted with Duels NPC", false));
                    } else {
                        e.push(Effect::Notice("Could not find Duels NPC", true));
                    }
                    self.add(
                        Wait::Millis(c.now.saturating_add(5000)),
                        Stage::Cooldown,
                        None,
                        true,
                        &mut e,
                    );
                } else if header.contains("Duels") {
                    if !self.in_match {
                        if self.settings.control_aura {
                            e.push(Effect::Control("KillAura", false));
                        }
                        if let Some(slot) = c
                            .slots
                            .iter()
                            .find(|s| s.slot >= 0 && s.item == "minecraft:player_head")
                            .map(|s| s.slot)
                        {
                            if slot != c.selected {
                                e.push(Effect::Select { slot, owner: true });
                            }
                            self.add(Wait::Ticks(5), Stage::GommeHead, None, true, &mut e);
                        }
                    } else if !c.aura_running && self.settings.control_aura {
                        e.push(Effect::Control("KillAura", true));
                    }
                } else {
                    self.in_match = false;
                }
            }
            _ => {
                let triggered = match self.settings.trigger {
                    0..=2 => std::mem::take(&mut self.latches[self.settings.trigger]),
                    3 => c
                        .slots
                        .iter()
                        .any(|s| self.settings.trigger_pick.matches(s)),
                    4 => c
                        .header
                        .as_ref()
                        .is_some_and(|h| h.contains(&self.settings.header)),
                    _ => c
                        .footer
                        .as_ref()
                        .is_some_and(|h| h.contains(&self.settings.footer)),
                };
                if triggered {
                    self.was_in_queue = true;
                    self.control(true, &mut e);
                    if self.settings.action == 0 {
                        self.chat_next(self.settings.sends.clone(), 0, c, &mut random, &mut e);
                    } else if let Some(slot) = c
                        .slots
                        .iter()
                        .find(|s| self.settings.action_pick.matches(s))
                        .map(|s| s.slot)
                    {
                        e.push(Effect::Select { slot, owner: false });
                        self.add(
                            Wait::Ticks(1),
                            Stage::CustomUse(slot),
                            Some(slot),
                            true,
                            &mut e,
                        );
                    } else {
                        self.custom_done(None, &mut e);
                    }
                } else if self.settings.control && self.was_in_queue {
                    self.was_in_queue = false;
                    self.control(false, &mut e);
                }
            }
        }
        e
    }
    fn resume(
        &mut self,
        task: Task,
        c: &Context,
        random: &mut dyn FnMut(i32, i32) -> i32,
    ) -> Vec<Effect> {
        let mut e = Vec::new();
        match task.stage {
            Stage::Cooldown | Stage::CustomDone => {}
            Stage::GommeHead => {
                e.push(Effect::RawUse);
                self.add(Wait::Ticks(20), Stage::Cooldown, None, true, &mut e);
            }
            Stage::Result(win) => e.push(Effect::Chat(if win {
                self.settings.win.clone()
            } else {
                self.settings.lose.clone()
            })),
            Stage::CustomUse(slot) => {
                e.push(Effect::Use(slot));
                self.custom_done(task.guard, &mut e);
            }
            Stage::CustomChat { messages, index } => {
                e.push(Effect::ChatOrCommand(messages[index].clone()));
                self.chat_next(messages, index + 1, c, random, &mut e);
            }
            Stage::CustomWorld => {
                self.add(Wait::Ticks(20), Stage::CustomDone, task.guard, true, &mut e)
            }
        }
        e
    }
    pub fn callback(
        &mut self,
        id: u64,
        running: bool,
        c: &Context,
        mut random: impl FnMut(i32, i32) -> i32,
    ) -> (bool, Vec<Effect>) {
        let Some(index) = self.tasks.iter().position(|t| t.id == id) else {
            return (false, vec![]);
        };
        if !running || !self.guard(&self.tasks[index], c) {
            self.tasks.remove(index);
            return (false, vec![]);
        }
        let ready = match &mut self.tasks[index].wait {
            Wait::Ticks(n) => {
                *n -= 1;
                *n <= 0
            }
            Wait::Millis(at) => c.now >= *at,
            Wait::World | Wait::Async(_) => false,
        };
        if !ready {
            return (true, vec![]);
        }
        let task = self.tasks.remove(index);
        (false, self.resume(task, c, &mut random))
    }
    pub fn wake(
        &mut self,
        running: bool,
        c: &Context,
        mut random: impl FnMut(i32, i32) -> i32,
    ) -> Vec<Effect> {
        let mut e = Vec::new();
        let ids = self
            .tasks
            .iter()
            .filter(|t| matches!(t.wait,Wait::Async(at)if c.now>=at))
            .map(|t| t.id)
            .collect::<Vec<_>>();
        for id in ids {
            if let Some(i) = self.tasks.iter().position(|t| t.id == id) {
                let task = self.tasks.remove(i);
                if running && self.guard(&task, c) {
                    e.extend(self.resume(task, c, &mut random));
                }
            }
        }
        e
    }
    pub fn next_wake(&self) -> Option<i64> {
        self.tasks
            .iter()
            .filter_map(|t| match t.wait {
                Wait::Async(at) => Some(at),
                _ => None,
            })
            .min()
    }
    pub fn world_change(&mut self) -> Vec<Effect> {
        let mut e = Vec::new();
        let ids = self
            .tasks
            .iter()
            .filter(|t| matches!(t.wait, Wait::World))
            .map(|t| t.id)
            .collect::<Vec<_>>();
        for id in ids {
            let i = self.tasks.iter().position(|t| t.id == id).unwrap();
            let task = self.tasks.remove(i);
            self.add(Wait::Ticks(20), Stage::CustomDone, task.guard, true, &mut e);
        }
        e
    }
}
