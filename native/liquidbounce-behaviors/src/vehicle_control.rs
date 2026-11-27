// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleVehicleControl.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub horizontal: f32,
    pub vertical: f32,
    pub sprint_enabled: bool,
    pub sprint_horizontal: f32,
    pub sprint_vertical: f32,
    pub glide: f32,
    pub mouse_control: bool,
    pub no_glide_on_sprint: bool,
    pub rehook: bool,
    pub unhook_after: i32,
    pub hook_after: i32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            horizontal: 0.5,
            vertical: 0.35,
            sprint_enabled: true,
            sprint_horizontal: 5.,
            sprint_vertical: 2.,
            glide: -0.15,
            mouse_control: false,
            no_glide_on_sprint: false,
            rehook: false,
            unhook_after: 4,
            hook_after: 2,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub moving: bool,
    pub sprint: bool,
    pub jump: bool,
    pub sneak: bool,
    pub use_down: bool,
    pub yaw: f32,
    pub vehicle_yaw: f32,
    pub vehicle_water: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RehookAction {
    Interact(i32),
    ForceMount(i32),
    TooFar,
    Gone,
}
#[derive(Clone, Copy, Debug)]
pub struct Wait {
    pub generation: u64,
    pub elapsed: i32,
    pub ticks: i32,
    pub hook: bool,
}
pub struct VehicleControl {
    pub direction: [bool; 4],
    pub settings: Settings,
    pub was_in_vehicle: bool,
    pub vehicle_id: i32,
    pub force_attempt: bool,
    pub detach_on_input: bool,
    pub waiter: Option<Wait>,
    generation: u64,
}
impl Default for VehicleControl {
    fn default() -> Self {
        Self {
            direction: [false; 4],
            settings: Settings::default(),
            was_in_vehicle: false,
            vehicle_id: -1,
            force_attempt: false,
            detach_on_input: false,
            waiter: None,
            generation: 0,
        }
    }
}
impl VehicleControl {
    pub fn velocity(&self, input: Input, sin_cos: [f64; 2], spear: Option<[f64; 3]>) -> [f64; 3] {
        if let Some(v) = spear {
            return v;
        }
        let sprint = input.sprint && self.settings.sprint_enabled;
        let h = if input.moving {
            if sprint {
                self.settings.sprint_horizontal
            } else {
                self.settings.horizontal
            }
        } else {
            0.
        };
        let v = if sprint {
            self.settings.sprint_vertical
        } else {
            self.settings.vertical
        };
        let y = if input.jump {
            v
        } else if input.sneak {
            -v
        } else if !input.vehicle_water && !(sprint && self.settings.no_glide_on_sprint) {
            self.settings.glide
        } else {
            0.
        };
        [
            -sin_cos[0] * f64::from(h),
            f64::from(y),
            sin_cos[1] * f64::from(h),
        ]
    }
    pub fn movement_notice(&mut self, vehicle: bool, use_down: bool) -> bool {
        if !vehicle {
            self.was_in_vehicle = false;
            return false;
        }
        if !self.was_in_vehicle && use_down {
            self.was_in_vehicle = true;
            true
        } else {
            false
        }
    }
    pub fn input(&mut self, vehicle: bool, safe: bool, sneak: bool) -> (bool, bool) {
        let mut shift = sneak;
        if vehicle || self.vehicle_id >= 0 {
            shift &= safe;
            if shift {
                self.vehicle_id = -1;
            }
        }
        let detach = std::mem::take(&mut self.detach_on_input);
        (if detach { true } else { shift }, detach)
    }
    pub fn cancel(&mut self) {
        self.waiter = None;
        self.generation = self.generation.wrapping_add(1);
    }
    fn wait(&mut self, ticks: i32, hook: bool) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.waiter = Some(Wait {
            generation: self.generation,
            elapsed: 0,
            ticks,
            hook,
        });
        self.generation
    }
    pub fn start_rehook(
        &mut self,
        passenger: bool,
        exists: bool,
        within_reach: bool,
    ) -> (Option<RehookAction>, Option<u64>) {
        if !self.settings.rehook || self.waiter.is_some() {
            return (None, None);
        }
        if self.vehicle_id >= 0 && !passenger {
            if !exists {
                self.vehicle_id = -1;
                return (Some(RehookAction::Gone), None);
            }
            if !within_reach {
                self.vehicle_id = -1;
                return (Some(RehookAction::TooFar), None);
            }
            if !self.force_attempt {
                self.force_attempt = true;
                (Some(RehookAction::Interact(self.vehicle_id)), None)
            } else {
                (Some(RehookAction::ForceMount(self.vehicle_id)), None)
            }
        } else {
            self.force_attempt = false;
            let id = self.wait(self.settings.unhook_after, false);
            (None, Some(id))
        }
    }
    /// Returns retained callback plus an optional newly registered hook wait.
    pub fn callback(&mut self, id: u64, controlled: Option<i32>) -> (bool, Option<u64>) {
        let Some(mut w) = self.waiter.filter(|w| w.generation == id) else {
            return (false, None);
        };
        w.elapsed = w.elapsed.saturating_add(1);
        if w.elapsed < w.ticks {
            self.waiter = Some(w);
            return (true, None);
        }
        self.waiter = None;
        if w.hook {
            return (false, None);
        }
        let Some(vehicle) = controlled else {
            return (false, None);
        };
        self.vehicle_id = vehicle;
        self.detach_on_input = true;
        let delay = self.settings.hook_after - 1;
        if delay <= 0 {
            (false, None)
        } else {
            let id = self.wait(delay, true);
            (false, Some(id))
        }
    }
}
