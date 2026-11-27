// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleMacros at ae0eb396. Real KeyEvent actions are intentionally unfiltered.
pub use crate::auto_queue::{MATCHES, Pick, Slot};
pub const COUNT: usize = 5;
pub const ACTIONS: [&str; 2] = ["UseOnly", "PlaceOrUse"];
pub const SWINGS: [&str; 2] = ["DoNotHide", "HideForClient"];
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trigger {
    pub key: String,
    pub delay: [i32; 2],
    pub current: i32,
    pub last: i64,
}
impl Default for Trigger {
    fn default() -> Self {
        Self {
            key: "key.keyboard.unknown".into(),
            delay: [0, 0],
            current: 0,
            last: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chat {
    pub trigger: Trigger,
    pub delay: [i32; 2],
    pub messages: Vec<String>,
    pub client_command: bool,
}
impl Default for Chat {
    fn default() -> Self {
        Self {
            trigger: Trigger::default(),
            delay: [0, 0],
            messages: vec![],
            client_command: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub trigger: Trigger,
    pub action: usize,
    pub pick: Pick,
    pub swing: usize,
    pub hold: [i32; 2],
}
impl Default for Item {
    fn default() -> Self {
        Self {
            trigger: Trigger::default(),
            action: 0,
            pick: Pick::default(),
            swing: 0,
            hold: [1, 1],
        }
    }
}
#[derive(Clone, Debug)]
struct Task {
    id: u64,
    event: u64,
    index: usize,
    messages: Vec<String>,
    list: u64,
    revision: u64,
    next: usize,
    deadline: Option<i64>,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub chats: [Chat; COUNT],
    pub items: [Item; COUNT],
    tasks: Vec<Task>,
    next_id: u64,
    next_event: u64,
    event_failed: bool,
    lists: [u64; COUNT],
    revisions: [u64; COUNT],
    pub text_values: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Start {
    Chat(u64),
    Item {
        slot: i32,
        hold: i32,
        action: usize,
        swing: usize,
    },
}
impl State {
    /// Human text setters replace the ArrayList, leaving its captured iterator.
    pub fn replace_messages(&mut self, index: usize, messages: Vec<String>) {
        self.lists[index] = self.lists[index].wrapping_add(1);
        self.revisions[index] = 0;
        self.chats[index].messages = messages;
    }
    /// Gson mutates the current ListValue collection in place. Java's iterator
    /// checks its modification count on next(), after the hasNext size check.
    pub fn import_messages(&mut self, index: usize, messages: Vec<String>) {
        if self.text_values {
            self.replace_messages(index, messages);
        } else {
            self.revisions[index] = self.revisions[index].wrapping_add(1);
            self.chats[index].messages = messages;
        }
    }
    pub fn trigger_mut(&mut self, index: usize) -> &mut Trigger {
        if index < COUNT {
            &mut self.chats[index].trigger
        } else {
            &mut self.items[index - COUNT].trigger
        }
    }
    pub fn trigger(&self, index: usize) -> &Trigger {
        if index < COUNT {
            &self.chats[index].trigger
        } else {
            &self.items[index - COUNT].trigger
        }
    }
    pub fn new_event(&mut self) -> u64 {
        self.next_event = self.next_event.wrapping_add(1);
        self.event_failed = false;
        self.next_event
    }
    pub fn begin(
        &mut self,
        index: usize,
        event: u64,
        key: &str,
        now: i64,
        slots: &[Slot],
        mut random: impl FnMut(i32, i32) -> i32,
    ) -> Result<Option<Start>, &'static str> {
        let trigger = self.trigger_mut(index);
        if trigger.key != key || now.wrapping_sub(trigger.last) <= i64::from(trigger.current) {
            return Ok(None);
        }
        trigger.last = now;
        if trigger.delay[0] > trigger.delay[1] {
            return Err("empty TriggerDelay range");
        }
        trigger.current = random(trigger.delay[0], trigger.delay[1]);
        if event == self.next_event && self.event_failed {
            return Ok(None);
        }
        if index < COUNT {
            let messages = self.chats[index].messages.clone();
            self.next_id = self.next_id.wrapping_add(1);
            let id = self.next_id;
            self.tasks.push(Task {
                id,
                event,
                index,
                messages,
                list: self.lists[index],
                revision: self.revisions[index],
                next: 0,
                deadline: None,
            });
            Ok(Some(Start::Chat(id)))
        } else {
            let item = &self.items[index - COUNT];
            let Some(slot) = slots.iter().find(|s| item.pick.matches(s)).map(|s| s.slot) else {
                return Ok(None);
            };
            if item.hold[0] > item.hold[1] {
                return Err("empty HoldTime range");
            }
            Ok(Some(Start::Item {
                slot,
                hold: random(item.hold[0], item.hold[1]),
                action: item.action,
                swing: item.swing,
            }))
        }
    }
    pub fn message(&mut self, id: u64) -> Result<Option<(String, bool)>, &'static str> {
        let Some(task) = self.tasks.iter().find(|t| t.id == id) else {
            return Ok(None);
        };
        if task.list == self.lists[task.index] && task.revision != self.revisions[task.index] {
            if task.next == self.chats[task.index].messages.len() {
                self.tasks.retain(|t| t.id != id);
                return Ok(None);
            }
            return Err("ConcurrentModificationException: macro message list changed");
        }
        if let Some(message) = task.messages.get(task.next) {
            Ok(Some((
                message.clone(),
                self.chats[task.index].client_command,
            )))
        } else {
            self.tasks.retain(|t| t.id != id);
            Ok(None)
        }
    }
    /// Delay is sampled after every message, including the final one. Zero and
    /// negative kotlinx delay values continue synchronously without suspension.
    pub fn sent(
        &mut self,
        id: u64,
        now: i64,
        mut random: impl FnMut(i32, i32) -> i32,
    ) -> Result<bool, &'static str> {
        let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) else {
            return Ok(false);
        };
        task.next += 1;
        let delay = self.chats[task.index].delay;
        if delay[0] > delay[1] {
            return Err("empty Delay range");
        }
        let ms = random(delay[0], delay[1]);
        task.deadline = (ms > 0).then(|| now.saturating_add(i64::from(ms)));
        Ok(ms <= 0)
    }
    pub fn due(&mut self, now: i64) -> Vec<u64> {
        let mut ids = Vec::new();
        for task in &mut self.tasks {
            if task.deadline.is_some_and(|at| at <= now) {
                task.deadline = None;
                ids.push(task.id);
            }
        }
        ids
    }
    pub fn next_wake(&self) -> Option<i64> {
        self.tasks.iter().filter_map(|t| t.deadline).min()
    }
    pub fn failed(&mut self, id: u64) {
        if let Some(event) = self.tasks.iter().find(|t| t.id == id).map(|t| t.event) {
            self.failed_event(event);
        }
    }
    pub fn failed_event(&mut self, event: u64) {
        if event == self.next_event {
            self.event_failed = true;
        }
        self.tasks.retain(|t| t.event != event);
    }
    pub fn cancel(&mut self) {
        self.tasks.clear();
    }
}
