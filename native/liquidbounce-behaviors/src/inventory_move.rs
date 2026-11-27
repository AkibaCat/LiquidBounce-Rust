// SPDX-License-Identifier: GPL-3.0-or-later
//! InventoryMove policies from LiquidBounce ae0eb396. Shared by the native adapter.
pub const BEHAVIORS: [&str; 4] = ["Normal", "Safe", "Undetectable", "StopOnAction"];
pub const SPRINT: [&str; 3] = ["DoNotChange", "ForceSprint", "ForceNoSprint"];
pub const SNEAK: [&str; 3] = ["DoNotChange", "ForceSneak", "ForceNoSneak"];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub behavior: u8,
    pub passthrough_sneak: bool,
    pub sprint_control: bool,
    pub client_sprint: u8,
    pub server_sprint: u8,
    pub sneak_control: bool,
    pub client_sneak: u8,
    pub timer: bool,
    pub speed: f32,
    pub blink: bool,
    pub maximum_time: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            behavior: 0,
            passthrough_sneak: false,
            sprint_control: false,
            client_sprint: 0,
            server_sprint: 0,
            sneak_control: false,
            client_sneak: 0,
            timer: false,
            speed: 1.,
            blink: false,
            maximum_time: 10000,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Screen {
    pub open: bool,
    pub container: bool,
    pub inventory: bool,
    pub editing: bool,
}
impl Settings {
    pub fn handles(self, running: bool, screen: Screen, key: usize) -> bool {
        if !running {
            return false;
        }
        if !screen.open {
            return true;
        }
        if key > 6 || (key == 5 && !self.passthrough_sneak) || screen.editing {
            return false;
        }
        match self.behavior {
            1 => !screen.container || screen.inventory,
            2 => !screen.container,
            _ => true,
        }
    }
    pub fn handled_screen(screen: Screen, server_open: bool) -> bool {
        screen.container || server_open
    }
    fn force(mode: u8, moving: bool, old: bool) -> bool {
        match mode {
            1 if moving => true,
            2 => false,
            _ => old,
        }
    }
    pub fn sprint(
        self,
        running: bool,
        screen: Screen,
        server_open: bool,
        source: u8,
        moving: bool,
        old: bool,
    ) -> bool {
        if !running || !self.sprint_control || !Self::handled_screen(screen, server_open) {
            return old;
        }
        let v = if source != 2 {
            Self::force(self.client_sprint, moving, old)
        } else {
            old
        };
        if source != 1 {
            Self::force(self.server_sprint, moving, v)
        } else {
            v
        }
    }
    pub fn sneak(self, running: bool, screen: Screen, server_open: bool, old: bool) -> bool {
        if !running || !self.sneak_control || !Self::handled_screen(screen, server_open) {
            return old;
        }
        match self.client_sneak {
            1 => true,
            2 => false,
            _ => old,
        }
    }
    pub fn safe_click(
        self,
        running: bool,
        screen: Screen,
        container: i32,
        pressed: [bool; 6],
    ) -> bool {
        running
            && self.behavior == 1
            && container == 0
            && (0..6).any(|i| pressed[i] && self.handles(running, screen, i))
    }
}
#[derive(Default)]
pub struct State {
    pub settings: Settings,
    pub running: bool,
    pub screen: Screen,
    pub server_open: bool,
    pub pressed: [bool; 6],
    pub input: [bool; 7],
    pub serial: Option<u64>,
    pub started: i64,
}
impl State {
    pub fn configure(&mut self, running: bool, s: Settings, server_open: bool) {
        self.running = running;
        self.settings = s;
        self.server_open = server_open;
    }
    pub fn keyboard(&mut self, key: usize, pressed: bool) -> bool {
        if !self.running || self.settings.behavior != 1 || key >= 6 {
            return false;
        }
        let pressed = pressed && self.settings.handles(self.running, self.screen, key);
        self.pressed[key] = pressed;
        pressed && self.screen.inventory && self.server_open
    }
    pub fn observe(&mut self, screen: Screen, serial: u64, now: i64) -> bool {
        let replaced = self.serial != Some(serial) || self.screen.container != screen.container;
        self.serial = Some(serial);
        self.screen = screen;
        if replaced && self.running && self.settings.blink && screen.container {
            self.started = now;
            return true;
        }
        false
    }
    pub fn expired(&self, now: i64) -> bool {
        self.running
            && self.settings.blink
            && self.screen.container
            && now.wrapping_sub(self.started) >= i64::from(self.settings.maximum_time)
    }
    pub fn blink(&self, container_packet: bool) -> u8 {
        if self.running && self.settings.blink && self.screen.container {
            if container_packet { 1 } else { 2 }
        } else {
            0
        }
    }
    pub fn stop_packet(&self) -> bool {
        self.running && self.settings.behavior == 3 && self.input.iter().any(|v| *v)
    }
}
