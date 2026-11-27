// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoAccount ae0eb396: message sequence and current-setting command.
use java_regex::{PatternSyntaxError, Regex};

pub const SOURCES: [&str; 3] = ["Chat", "Title", "Subtitle"];
#[derive(Clone, Debug)]
pub struct Pattern(Regex);
impl Pattern {
    pub fn new(source: &str) -> Result<Self, PatternSyntaxError> {
        Regex::new(source).map(Self)
    }
    pub fn source(&self) -> &str {
        self.0.pattern()
    }
}
impl PartialEq for Pattern {
    fn eq(&self, rhs: &Self) -> bool {
        self.source() == rhs.source()
    }
}
impl Eq for Pattern {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Register,
    Login,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Connection,
    Delay { elapsed: i32, ticks: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pending {
    id: u64,
    operation: Operation,
    phase: Phase,
}
#[derive(Clone, Debug)]
pub struct AutoAccount {
    pub password: String,
    pub delay: [i32; 2],
    pub register_command: String,
    pub login_command: String,
    pub register_regex: Pattern,
    pub login_regex: Pattern,
    pub repeat_password_on_register: bool,
    pub message_sources: u32,
    sending: bool,
    pending: Option<Pending>,
    next_id: u64,
}
impl Default for AutoAccount {
    fn default() -> Self {
        Self {
            password: "a1b2c3d4".into(),
            delay: [3, 5],
            register_command: "register".into(),
            login_command: "login".into(),
            register_regex: Pattern::new("/register").unwrap(),
            login_regex: Pattern::new("/login").unwrap(),
            repeat_password_on_register: true,
            message_sources: 7,
            sending: false,
            pending: None,
            next_id: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    pub keep: bool,
    pub wait: Option<u64>,
    pub operation: Option<Operation>,
}
impl AutoAccount {
    pub fn cancel(&mut self) {
        self.sending = false;
        self.pending = None;
    }
    pub fn message(&mut self, running: bool, source: u8, message: &str) -> Option<u64> {
        if !running || self.sending || source >= 3 || self.message_sources & (1 << source) == 0 {
            return None;
        }
        let operation = if self.register_regex.0.contains_match(message) {
            Operation::Register
        } else if self.login_regex.0.contains_match(message) {
            Operation::Login
        } else {
            return None;
        };
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("AutoAccount callback exhausted");
        self.pending = Some(Pending {
            id: self.next_id,
            operation,
            phase: Phase::Connection,
        });
        self.sending = true;
        // tickUntil always suspends at least once, including a live connection.
        Some(self.next_id)
    }
    pub fn callback(
        &mut self,
        id: u64,
        running: bool,
        connected: bool,
        random: impl FnOnce(i32, i32) -> i32,
    ) -> Step {
        let Some(mut pending) = self.pending.filter(|v| v.id == id) else {
            return Step::default();
        };
        if !running {
            self.cancel();
            return Step::default();
        }
        match pending.phase {
            Phase::Connection if !connected => {
                return Step {
                    keep: true,
                    ..Step::default()
                };
            }
            Phase::Connection => {
                if self.delay[0] > self.delay[1] {
                    self.pending = None;
                    return Step::default();
                }
                let ticks = random(self.delay[0], self.delay[1]);
                if ticks != 0 {
                    self.next_id = self
                        .next_id
                        .checked_add(1)
                        .expect("AutoAccount callback exhausted");
                    pending.id = self.next_id;
                    pending.phase = Phase::Delay { elapsed: 0, ticks };
                    self.pending = Some(pending);
                    return Step {
                        wait: Some(pending.id),
                        ..Step::default()
                    };
                }
            }
            Phase::Delay { elapsed, ticks } => {
                let elapsed = elapsed.wrapping_add(1);
                if elapsed < ticks {
                    pending.phase = Phase::Delay { elapsed, ticks };
                    self.pending = Some(pending);
                    return Step {
                        keep: true,
                        ..Step::default()
                    };
                }
            }
        }
        self.pending = None;
        // Keep sending set while the host sends, to reject reentrant messages.
        Step {
            operation: Some(pending.operation),
            ..Step::default()
        }
    }
    pub fn sent(&mut self) {
        self.sending = false;
    }
    pub fn command(&self, operation: Operation) -> String {
        match operation {
            Operation::Login => format!("{} {}", self.login_command, self.password),
            Operation::Register if self.repeat_password_on_register => format!(
                "{} {} {}",
                self.register_command, self.password, self.password
            ),
            Operation::Register => format!("{} {}", self.register_command, self.password),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn message_connection_delay_and_cancel_smoke() {
        let mut module = AutoAccount::default();
        let id = module.message(true, 0, "/register /login").unwrap();
        assert!(module.message(true, 1, "/login").is_none());
        assert!(module.callback(id, true, false, |_, _| panic!()).keep);
        module.delay = [1, 1];
        let wait = module
            .callback(id, true, true, |a, b| {
                assert_eq!((a, b), (1, 1));
                a
            })
            .wait
            .unwrap();
        module.password = "changed".into();
        assert_eq!(
            module.callback(wait, true, true, |_, _| panic!()).operation,
            Some(Operation::Register)
        );
        assert_eq!(
            module.command(Operation::Register),
            "register changed changed"
        );
        assert!(module.message(true, 0, "/login").is_none());
        module.sent();
        module.delay = [0, 0];
        let id = module.message(true, 2, "/login").unwrap();
        assert_eq!(
            module.callback(id, true, true, |_, _| 0).operation,
            Some(Operation::Login)
        );
        module.cancel();
        assert_eq!(
            module.callback(id, true, true, |_, _| panic!()),
            Step::default()
        );
        assert!(module.message(false, 0, "/login").is_none());
    }
}
