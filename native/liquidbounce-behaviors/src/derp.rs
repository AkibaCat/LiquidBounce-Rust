// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleDerp at LiquidBounce ae0eb396. The hidden rotation group
//! has fixed defaults and is deliberately absent from the settings codec.

use crate::{
    no_rotate_set::{FloatRange, RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, Rotation},
};

pub const YAW_MODES: [&str; 5] = ["Static", "Offset", "Random", "Jitter", "Spin"];
pub const PITCH_MODES: [&str; 3] = ["Static", "Offset", "Random"];
pub const ROTATION_PROCESSOR: u64 = 7;
pub const ROTATION_PRIORITY: i32 = -20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum YawMode {
    Static,
    Offset,
    #[default]
    Random,
    Jitter,
    Spin,
}
impl YawMode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Offset => "Offset",
            Self::Random => "Random",
            Self::Jitter => "Jitter",
            Self::Spin => "Spin",
        }
    }
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "Static" => Some(Self::Static),
            "Offset" => Some(Self::Offset),
            "Random" => Some(Self::Random),
            "Jitter" => Some(Self::Jitter),
            "Spin" => Some(Self::Spin),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PitchMode {
    Static,
    Offset,
    #[default]
    Random,
}
impl PitchMode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Offset => "Offset",
            Self::Random => "Random",
        }
    }
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "Static" => Some(Self::Static),
            "Offset" => Some(Self::Offset),
            "Random" => Some(Self::Random),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub yaw_mode: YawMode,
    pub yaw: f32,
    pub yaw_offset: f32,
    pub forward_ticks: i32,
    pub backward_ticks: i32,
    pub spin_speed: i32,
    pub pitch_mode: PitchMode,
    pub pitch: f32,
    pub pitch_offset: f32,
    pub safe_pitch: bool,
    pub not_during_sprint: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            yaw_mode: YawMode::Random,
            yaw: 0.0,
            yaw_offset: 0.0,
            forward_ticks: 2,
            backward_ticks: 2,
            spin_speed: 50,
            pitch_mode: PitchMode::Random,
            pitch: -90.0,
            pitch_offset: 0.0,
            safe_pitch: true,
            not_during_sprint: true,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub yaw: f32,
    pub pitch: f32,
    pub sprint_key: bool,
    pub sprinting: bool,
}
impl Snapshot {
    pub fn parse(value: &serde_json::Value) -> Option<Self> {
        Some(Self {
            yaw: value.get("yaw")?.as_f64()? as f32,
            pitch: value.get("pitch")?.as_f64()? as f32,
            sprint_key: value.get("sprintKey")?.as_bool()?,
            sprinting: value.get("sprinting")?.as_bool()?,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Wait {
    Forward(i32),
    Backward(i32),
    Spin,
}
#[derive(Clone, Debug, Default)]
pub struct Derp {
    pub settings: Settings,
    jitter_yaw: f32,
    spin_yaw: f32,
    wait: Option<Wait>,
    wait_callback: Option<u64>,
    callback_counter: u64,
    wait_registrations: Vec<u64>,
}
impl Derp {
    pub const fn jitter_yaw(&self) -> f32 {
        self.jitter_yaw
    }
    pub const fn spin_yaw(&self) -> f32 {
        self.spin_yaw
    }
    pub const fn busy(&self) -> bool {
        self.wait.is_some()
    }
    /// ModeValueGroup cancels a changed mode's scope even outside a world.
    /// Same-mode writes retain waits; neither path resets stored yaw.
    pub fn set_yaw_mode(&mut self, mode: YawMode) {
        if self.settings.yaw_mode != mode {
            self.settings.yaw_mode = mode;
            self.cancel_sequences();
        }
    }
    pub fn cancel_sequences(&mut self) {
        self.wait = None;
        self.wait_callback = None;
    }
    fn suspend(&mut self, wait: Wait) {
        self.callback_counter = self
            .callback_counter
            .checked_add(1)
            .expect("Derp callback id exhausted");
        self.wait = Some(wait);
        self.wait_callback = Some(self.callback_counter);
        self.wait_registrations.push(self.callback_counter);
    }
    /// Drain immediately after each call, before executing any Host effect.
    /// Cancelled registrations remain observable, as in CoroutineTicker.
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }
    /// Advance only the callback at this position in the global FIFO.
    /// Every Derp suspension is waitTicks(1): this callback always completes;
    /// a subsequent suspend has a fresh id and belongs to next tick's pending.
    pub fn advance_callback(&mut self, callback: u64, running: bool, snapshot: Snapshot) -> bool {
        if self.wait_callback == Some(callback) {
            self.begin_tick(running, snapshot);
        }
        false
    }
    fn backward(&mut self, player_yaw: f32) {
        let count = self.settings.backward_ticks;
        if count > 0 {
            self.jitter_yaw = player_yaw + 180.0;
            self.suspend(Wait::Backward(count - 1));
        }
    }
    /// CoroutineTicker runs before RotationManager's FIRST_PRIORITY handler.
    /// A resumed iteration reads the live yaw. Forward's repeat count was
    /// captured earlier; backward's count is read at its phase transition.
    pub fn begin_tick(&mut self, running: bool, snapshot: Snapshot) {
        let Some(wait) = self.wait.take() else {
            return;
        };
        self.wait_callback = None;
        if !running {
            return;
        }
        match wait {
            Wait::Forward(left) if self.settings.yaw_mode == YawMode::Jitter => {
                if left > 0 {
                    self.jitter_yaw = snapshot.yaw;
                    self.suspend(Wait::Forward(left - 1));
                } else {
                    self.backward(snapshot.yaw);
                }
            }
            Wait::Backward(left) if self.settings.yaw_mode == YawMode::Jitter && left > 0 => {
                self.jitter_yaw = snapshot.yaw + 180.0;
                self.suspend(Wait::Backward(left - 1));
            }
            _ => {}
        }
    }
    /// Earlier mode GameTick handler, exclusive DROP. Sprint only suppresses
    /// the later module handler, so these sequences keep running during it.
    pub fn start_yaw_handler(&mut self, running: bool, snapshot: Snapshot) {
        if !running || self.wait.is_some() {
            return;
        }
        match self.settings.yaw_mode {
            YawMode::Jitter => {
                let count = self.settings.forward_ticks;
                if count > 0 {
                    self.jitter_yaw = snapshot.yaw;
                    self.suspend(Wait::Forward(count - 1));
                } else {
                    self.backward(snapshot.yaw);
                }
            }
            YawMode::Spin => {
                self.spin_yaw += self.settings.spin_speed as f32;
                self.suspend(Wait::Spin);
            }
            _ => {}
        }
    }
    /// Ordinary module GameTick after the mode handlers. Yaw is evaluated
    /// before pitch; skipped requests consume no random values.
    pub fn rotation_target(
        &self,
        running: bool,
        snapshot: Snapshot,
        random: &mut impl FnMut() -> f32,
    ) -> Result<Option<DefaultRotationTarget>, &'static str> {
        if !running
            || self.settings.not_during_sprint && (snapshot.sprint_key || snapshot.sprinting)
        {
            return Ok(None);
        }
        let yaw = match self.settings.yaw_mode {
            YawMode::Static => self.settings.yaw,
            YawMode::Offset => snapshot.yaw + self.settings.yaw_offset,
            YawMode::Random => FloatRange(-180.0, 180.0).sample(random)?,
            YawMode::Jitter => self.jitter_yaw,
            YawMode::Spin => self.spin_yaw,
        };
        let pitch = match self.settings.pitch_mode {
            PitchMode::Static => self.settings.pitch,
            PitchMode::Offset => snapshot.pitch + self.settings.pitch_offset,
            PitchMode::Random => {
                let limit = if self.settings.safe_pitch {
                    90.0
                } else {
                    180.0
                };
                FloatRange(-limit, limit).sample(random)?
            }
        };
        let pitch = if self.settings.safe_pitch {
            pitch.clamp(-90.0, 90.0)
        } else {
            pitch
        };
        Ok(Some(DefaultRotationTarget::sprint(Rotation::new(
            yaw, pitch,
        ))))
    }
    /// The hidden default Linear smoother uses vector component limits.
    /// Reuse its implementation, with independent fixed settings and identity.
    pub fn process_rotation(
        current: Rotation,
        target: Rotation,
        last_managed: Option<Rotation>,
        player_last: Rotation,
        random: &mut impl FnMut() -> f32,
    ) -> Result<Rotation, &'static str> {
        RotationsSettings::default().process(
            SmoothMode::Linear,
            current,
            target,
            last_managed,
            player_last,
            random,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at(yaw: f32) -> Snapshot {
        Snapshot {
            yaw,
            ..Default::default()
        }
    }
    #[test]
    fn jitter_captures_forward_count_reads_backward_live_and_restarts_same_tick() {
        let mut state = Derp::default();
        state.set_yaw_mode(YawMode::Jitter);
        state.start_yaw_handler(true, at(10.0));
        assert_eq!(state.jitter_yaw(), 10.0);
        state.settings.forward_ticks = 99;
        state.settings.backward_ticks = 3;
        state.begin_tick(true, at(20.0));
        state.start_yaw_handler(true, at(200.0));
        assert_eq!(state.jitter_yaw(), 20.0);
        state.begin_tick(true, at(30.0));
        assert_eq!(state.jitter_yaw(), 210.0);
        state.settings.backward_ticks = 99;
        state.begin_tick(true, at(40.0));
        assert_eq!(state.jitter_yaw(), 220.0);
        state.begin_tick(true, at(50.0));
        assert_eq!(state.jitter_yaw(), 230.0);
        state.settings.forward_ticks = 1;
        state.begin_tick(true, at(60.0));
        assert!(!state.busy());
        state.start_yaw_handler(true, at(60.0));
        assert_eq!(state.jitter_yaw(), 60.0);
    }
    #[test]
    fn mode_disable_and_sprint_preserve_state_without_wrapping() {
        let mut state = Derp::default();
        state.set_yaw_mode(YawMode::Spin);
        let sprinting = Snapshot {
            sprint_key: true,
            ..at(0.0)
        };
        state.start_yaw_handler(true, sprinting);
        assert_eq!(state.spin_yaw(), 50.0);
        assert!(
            state
                .rotation_target(true, sprinting, &mut || panic!("sprint must skip RNG"))
                .unwrap()
                .is_none()
        );
        state.set_yaw_mode(YawMode::Spin);
        assert!(state.busy());
        state.begin_tick(false, at(0.0));
        assert!(!state.busy());
        assert_eq!(state.spin_yaw(), 50.0);
        state.settings.spin_speed = i32::MAX;
        state.start_yaw_handler(true, at(0.0));
        assert_eq!(state.spin_yaw(), 50.0 + i32::MAX as f32);
        state.set_yaw_mode(YawMode::Jitter);
        assert!(!state.busy());
        state.settings.forward_ticks = -1;
        state.settings.backward_ticks = 0;
        state.start_yaw_handler(true, at(8.0));
        assert_eq!(state.jitter_yaw(), 0.0);
        assert!(!state.busy());
        state.settings.backward_ticks = 1;
        state.start_yaw_handler(true, at(8.0));
        assert_eq!(state.jitter_yaw(), 188.0);
        state.cancel_sequences();
        assert_eq!(state.jitter_yaw(), 188.0);
    }
    #[test]
    fn every_angle_mode_random_order_safe_pitch_and_fixed_hidden_rotations() {
        let mut state = Derp::default();
        state.settings.not_during_sprint = false;
        let mut samples = [0.25, 0.75].into_iter();
        let target = state
            .rotation_target(true, at(20.0), &mut || samples.next().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(
            target.rotation.yaw.to_bits(),
            crate::random::bounded_f32(0.25, -180.0, 180.0f32.next_up()).to_bits()
        );
        assert_eq!(
            target.rotation.pitch.to_bits(),
            crate::random::bounded_f32(0.75, -90.0, 90.0f32.next_up()).to_bits()
        );
        assert!(!target.rotation.is_normalized);
        assert!(!target.consider_inventory);
        assert_eq!(target.ticks_until_reset, 5);
        assert_eq!(target.reset_threshold, 2.0);
        assert_eq!(
            target.movement_correction,
            crate::rotations::MovementCorrection::Silent
        );
        state.set_yaw_mode(YawMode::Static);
        state.settings.yaw = 500.0;
        state.settings.pitch_mode = PitchMode::Static;
        state.settings.pitch = -150.0;
        let fixed = |state: &Derp| {
            state
                .rotation_target(
                    true,
                    Snapshot {
                        pitch: 80.0,
                        ..at(20.0)
                    },
                    &mut || panic!("fixed modes must not sample"),
                )
                .unwrap()
                .unwrap()
                .rotation
        };
        assert_eq!(fixed(&state), Rotation::new(500.0, -90.0));
        state.settings.safe_pitch = false;
        assert_eq!(fixed(&state), Rotation::new(500.0, -150.0));
        state.set_yaw_mode(YawMode::Offset);
        state.settings.yaw_offset = 340.0;
        state.settings.pitch_mode = PitchMode::Offset;
        state.settings.pitch_offset = 50.0;
        assert_eq!(fixed(&state), Rotation::new(360.0, 130.0));
        state.settings.pitch_mode = PitchMode::Random;
        let pitch = state
            .rotation_target(true, at(0.0), &mut || 0.75)
            .unwrap()
            .unwrap()
            .rotation
            .pitch;
        assert_eq!(
            pitch.to_bits(),
            crate::random::bounded_f32(0.75, -180.0, 180.0f32.next_up()).to_bits()
        );
        let processed = Derp::process_rotation(
            Rotation::ZERO,
            Rotation::new(160.0, 160.0),
            None,
            Rotation::ZERO,
            &mut || panic!("default equal speed does not sample"),
        )
        .unwrap();
        assert!(processed.yaw > 127.0 && processed.yaw < 128.0);
        assert_eq!(processed.yaw, processed.pitch);
        assert!(YawMode::parse("random").is_none());
        assert!(PitchMode::parse("random").is_none());
        assert_eq!(
            Snapshot::parse(
                &serde_json::json!({"yaw":12,"pitch":30,"sprintKey":true,"sprinting":false})
            ),
            Some(Snapshot {
                yaw: 12.0,
                pitch: 30.0,
                sprint_key: true,
                sprinting: false
            })
        );
    }
}
