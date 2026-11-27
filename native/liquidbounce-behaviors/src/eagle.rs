// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleEagle's conditional input state; geometry is supplied by the live simulator.
pub const CONDITIONS: [&str; 7] = [
    "Left",
    "Right",
    "Forwards",
    "Backwards",
    "HoldingBlocks",
    "OnGround",
    "Sneak",
];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub distance: [f32; 2],
    pub conditional: bool,
    pub conditions: u32,
    pub pitch: [f32; 2],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            distance: [0.4, 0.6],
            conditional: true,
            conditions: 1 << 5,
            pitch: [-90., 90.],
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub forwards: bool,
    pub backwards: bool,
    pub holding_blocks: bool,
    pub on_ground: bool,
    pub sneak: bool,
    pub physical_sneak: bool,
    pub flying: bool,
    pub pitch: f32,
}
impl Settings {
    pub fn conditions_met(&self, input: Input) -> bool {
        !self.conditional || {
            let pitch = input.pitch.clamp(-90., 90.);
            pitch >= self.pitch[0]
                && pitch <= self.pitch[1]
                && [
                    input.left,
                    input.right,
                    input.forwards,
                    input.backwards,
                    input.holding_blocks,
                    input.on_ground,
                    input.sneak,
                ]
                .iter()
                .enumerate()
                .all(|(i, value)| self.conditions & (1 << i) == 0 || *value)
        }
    }
}
#[derive(Debug)]
pub struct Eagle {
    pub settings: Settings,
    pub current_distance: f32,
    was_sneaking: bool,
    sneak_captured: bool,
}
impl Default for Eagle {
    fn default() -> Self {
        let settings = Settings::default();
        let current_distance = sample(settings.distance);
        Self {
            settings,
            current_distance,
            was_sneaking: false,
            sneak_captured: false,
        }
    }
}
fn sample(range: [f32; 2]) -> f32 {
    // Kotlin samples [start, endInclusive.nextUp()) with ThreadLocalRandom.nextFloat.
    crate::random::with_thread_random(|r| {
        let from = range[0];
        let to = range[1];
        if from >= to {
            range[0]
        } else {
            r.next_f32_between(from, to.next_up())
        }
    })
}
impl Eagle {
    pub fn configure(&mut self, settings: Settings) {
        if settings.distance != self.settings.distance {
            self.current_distance = sample(settings.distance);
        }
        self.settings = settings;
    }
    pub fn disabled(&mut self) {
        self.was_sneaking = false;
        self.sneak_captured = false;
    }
    pub fn movement_input(&mut self, input: Input, close_to_edge: bool) -> bool {
        let conditions_met = self.settings.conditions_met(input);
        let active = !input.flying && conditions_met && close_to_edge;
        let controls = self.settings.conditional && self.settings.conditions & (1 << 6) != 0;
        if !controls {
            self.sneak_captured = false;
        } else if !self.sneak_captured && active && input.physical_sneak {
            self.sneak_captured = true;
        } else if self.sneak_captured && !input.physical_sneak {
            self.sneak_captured = false;
        }
        let override_sneak = conditions_met && controls && (active || self.sneak_captured);
        let sneak = if override_sneak {
            active
        } else {
            input.physical_sneak || active
        };
        if sneak {
            self.was_sneaking = true;
        } else if self.was_sneaking {
            self.current_distance = sample(self.settings.distance);
            self.was_sneaking = false;
        }
        sneak
    }
}
