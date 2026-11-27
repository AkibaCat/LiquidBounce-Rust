// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleReverseStep modes and bounded Instant landing replay, ae0eb396.
pub const MODES: [&str; 3] = ["Instant", "Strict", "Accelerator"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Instant,
    Strict,
    Accelerator,
}
impl Mode {
    pub fn tag(self) -> &'static str {
        MODES[self as usize]
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "Instant" => Some(Self::Instant),
            "Strict" => Some(Self::Strict),
            "Accelerator" => Some(Self::Accelerator),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub maximum_fall_distance: f32,
    pub ticks: i32,
    pub simulate_falling: bool,
    pub motion: f32,
    pub factor: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::Instant,
            maximum_fall_distance: 1.,
            ticks: 20,
            simulate_falling: false,
            motion: 1.,
            factor: 1.,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ReverseStep {
    pub running: bool,
    pub initiated_jump: bool,
}
impl ReverseStep {
    pub fn jump(&mut self) {
        if self.running {
            self.initiated_jump = true;
        }
    }
    pub fn observe(&mut self, running: bool, vertical: f64, on_ground: bool) {
        self.running = running;
        if running {
            if vertical > 0. {
                self.initiated_jump = true;
            } else if on_ground {
                self.initiated_jump = false;
            }
        }
    }
    pub fn eligible(
        self,
        s: Settings,
        on_ground: bool,
        fall_distance: f64,
        unwanted: bool,
        collision_below: bool,
    ) -> bool {
        self.running
            && !self.initiated_jump
            && !on_ground
            && !unwanted
            && fall_distance <= f64::from(s.maximum_fall_distance)
            && collision_below
    }
}
/// Accelerator deliberately zeros horizontal motion, as Vec3.multiply(0,f,0).
pub fn velocity(s: Settings, v: [f64; 3]) -> Option<[f64; 3]> {
    match s.mode {
        Mode::Strict => Some([v[0], -f64::from(s.motion), v[2]]),
        Mode::Accelerator if v[1] < 0. => Some([0., v[1] * f64::from(s.factor), 0.]),
        _ => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub position: [f64; 3],
    pub on_ground: bool,
    pub horizontal_collision: bool,
}
pub trait Simulation {
    fn sample(&self) -> Sample;
    fn tick(&mut self);
}
#[derive(Clone, Debug, PartialEq)]
pub struct Landing {
    pub position: [f64; 3],
    pub packets: Vec<Sample>,
}
pub fn instant(s: Settings, simulation: &mut impl Simulation) -> Option<Landing> {
    let mut packets = Vec::new();
    // Original 0..ticks is inclusive: the final movement may land on tick N.
    for _ in 0..=s.ticks {
        let sample = simulation.sample();
        if sample.on_ground {
            return Some(Landing {
                position: sample.position,
                packets,
            });
        }
        simulation.tick();
        if s.simulate_falling {
            packets.push(simulation.sample());
        }
    }
    None
}
