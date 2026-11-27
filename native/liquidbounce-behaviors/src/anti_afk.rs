// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleAntiAFK at ae0eb396. Waits precede the new GameTickEvent;
//! effects execute outside module locks and resume their synchronous tail.

use crate::{
    no_rotate_set::{FloatRange, RotationsSettings, SmoothMode},
    random::ThreadRandom,
    rotations::{DefaultRotationTarget, Rotation},
};

pub const INTERACTIONS: [&str; 6] = [
    "Jump",
    "SwingHand",
    "ChangeSlot",
    "Yaw",
    "Pitch",
    "RandomDirection",
];
pub const MODES: [&str; 3] = ["Old", "RandomInteraction", "Custom"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Old,
    #[default]
    RandomInteraction,
    Custom,
}
impl Mode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Old => "Old",
            Self::RandomInteraction => "RandomInteraction",
            Self::Custom => "Custom",
        }
    }
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "Old" => Some(Self::Old),
            "RandomInteraction" => Some(Self::RandomInteraction),
            "Custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Direction {
    pub forwards: bool,
    pub backwards: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub interactions: u8,
    pub delay: [i32; 2],
    pub rotate: bool,
    pub ignore_open_inventory: bool,
    pub rotations: RotationsSettings,
    pub rotate_delay: i32,
    pub angle: f32,
    pub swing: bool,
    pub swing_delay: i32,
    pub jump: bool,
    pub move_forward: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: Mode::RandomInteraction,
            interactions: (1 << 1) | (1 << 3) | (1 << 4),
            delay: [4, 7],
            rotate: true,
            ignore_open_inventory: true,
            rotations: Default::default(),
            rotate_delay: 5,
            angle: 1.0,
            swing: true,
            swing_delay: 5,
            jump: true,
            move_forward: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub swinging: bool,
    pub on_ground: bool,
    pub yaw: f32,
    pub pitch: f32,
}
impl Snapshot {
    pub fn parse(value: &serde_json::Value) -> Option<Self> {
        Some(Self {
            swinging: value.get("swinging")?.as_bool()?,
            on_ground: value.get("onGround")?.as_bool()?,
            yaw: value.get("yaw")?.as_f64()? as f32,
            pitch: value.get("pitch")?.as_f64()? as f32,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotationTarget {
    pub target: DefaultRotationTarget,
    pub processor: SmoothMode,
}
impl RotationTarget {
    /// Separate mode instances from NoRotateSet, despite sharing algorithms.
    pub const fn processor_id(self) -> u64 {
        self.processor.processor_id() + 3
    }
    pub const fn processor_mode(id: u64) -> Option<SmoothMode> {
        match id {
            4 => Some(SmoothMode::Linear),
            5 => Some(SmoothMode::Sigmoid),
            6 => Some(SmoothMode::Acceleration),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    /// Options.keyUp.isDown = true, retained until the real mapping releases.
    ForwardKey,
    /// Main-hand attack animation and an unconditional ordinary Punch packet.
    SwingHand,
    /// Raw Inventory.selected; no immediate carried synchronization.
    SelectedSlot(i32),
    Yaw(f32),
    Pitch(f32),
    Rotation(RotationTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lane {
    Old,
    Random,
    CustomSwing,
    CustomRotate,
}
#[derive(Clone, Copy, Debug)]
struct Wait {
    lane: Lane,
    remaining: i32,
    order: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RandomStage {
    #[default]
    Idle,
    Direction,
    Outer,
    AfterEffect,
}

#[derive(Clone, Debug, Default)]
pub struct AntiAfk {
    pub settings: Settings,
    random_direction: Direction,
    waits: Vec<Wait>,
    order: u64,
    wait_registrations: Vec<u64>,
    random_stage: RandomStage,
    generation: u64,
    pending_random_tail: Option<u64>,
    pending_random_jump: usize,
    pending_custom_jump: usize,
}
impl AntiAfk {
    pub const fn random_direction(&self) -> Direction {
        self.random_direction
    }
    pub fn busy(&self) -> bool {
        !self.waits.is_empty() || self.random_stage == RandomStage::AfterEffect
    }
    fn waiting(&self, lane: Lane) -> bool {
        self.waits.iter().any(|w| w.lane == lane)
    }
    fn wait(&mut self, lane: Lane, ticks: i32) -> bool {
        if ticks == 0 {
            return false;
        }
        self.order = self
            .order
            .checked_add(1)
            .expect("AntiAFK callback id exhausted");
        self.waits.push(Wait {
            lane,
            remaining: ticks.max(1),
            order: self.order,
        });
        self.wait_registrations.push(self.order);
        true
    }
    /// Enqueue these ids in CoroutineTicker immediately after every call and
    /// before native effects or another module's continuation can run.
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }
    fn lane_running(&self, lane: Lane, running: bool) -> bool {
        running
            && match lane {
                Lane::Old => self.settings.mode == Mode::Old,
                Lane::Random => self.settings.mode == Mode::RandomInteraction,
                Lane::CustomSwing | Lane::CustomRotate => self.settings.mode == Mode::Custom,
            }
    }
    /// The global FIFO calls exactly one waiter. Its guard and snapshot are
    /// read at that position, after all earlier callbacks' real Host effects.
    /// keep=true retains only this id; newly registered waits are always new
    /// ids, even if a completed random-direction wait starts its outer delay.
    pub fn advance_callback(
        &mut self,
        callback: u64,
        running: bool,
        snapshot: Snapshot,
        server_rotation: Rotation,
        random: &mut ThreadRandom,
    ) -> Result<(bool, Option<Effect>), &'static str> {
        let Some(index) = self.waits.iter().position(|wait| wait.order == callback) else {
            return Ok((false, None));
        };
        if !self.lane_running(self.waits[index].lane, running) {
            let wait = self.waits.remove(index);
            if wait.lane == Lane::Random {
                self.random_stage = RandomStage::Idle;
            }
            return Ok((false, None));
        }
        self.waits[index].remaining -= 1;
        if self.waits[index].remaining > 0 {
            return Ok((true, None));
        }
        let wait = self.waits.remove(index);
        self.resume_wait(wait, snapshot, server_rotation, random)
            .map(|effect| (false, effect))
    }
    fn resume_wait(
        &mut self,
        wait: Wait,
        snapshot: Snapshot,
        server_rotation: Rotation,
        random: &mut ThreadRandom,
    ) -> Result<Option<Effect>, &'static str> {
        match wait.lane {
            Lane::Old => Ok(Some(Effect::Yaw(snapshot.yaw + 180.0))),
            Lane::CustomSwing => Ok(Some(Effect::SwingHand)),
            Lane::CustomRotate => Ok(Some(self.rotation(server_rotation, random))),
            Lane::Random => {
                match self.random_stage {
                    RandomStage::Direction => {
                        self.random_direction = Direction::default();
                        self.outer_delay(random)?;
                    }
                    RandomStage::Outer => self.random_stage = RandomStage::Idle,
                    _ => unreachable!("only suspended random stages own a wait"),
                }
                Ok(None)
            }
        }
    }
    fn sample_delay(&self, random: &mut ThreadRandom) -> Result<i32, &'static str> {
        let [first, last] = self.settings.delay;
        if first > last {
            return Err("AntiAFK delay range is empty");
        }
        Ok(random.next_i32_inclusive(first, last))
    }
    fn outer_delay(&mut self, random: &mut ThreadRandom) -> Result<(), &'static str> {
        self.random_stage = RandomStage::Idle;
        let delay = self.sample_delay(random)?;
        if self.wait(Lane::Random, delay) {
            self.random_stage = RandomStage::Outer;
        }
        Ok(())
    }
    /// Call once at CoroutineTicker before this tick's ready continuations.
    /// Dynamic mode/running guards cancel waits. They do not reset the mode's
    /// stored direction or unregister once<MovementInputEvent> listeners.
    pub fn begin_tick(&mut self, running: bool) {
        let mode = self.settings.mode;
        self.waits.retain(|w| {
            running
                && match w.lane {
                    Lane::Old => mode == Mode::Old,
                    Lane::Random => mode == Mode::RandomInteraction,
                    Lane::CustomSwing | Lane::CustomRotate => mode == Mode::Custom,
                }
        });
        if !running || mode != Mode::RandomInteraction {
            self.random_stage = RandomStage::Idle;
        }
        for wait in &mut self.waits {
            wait.remaining -= 1;
        }
    }
    /// Drain until None, executing each effect and after_effect outside locks
    /// before obtaining another live snapshot. Ready waits retain registration
    /// order, even when the two independent Custom coroutines started apart.
    pub fn next_ready(
        &mut self,
        snapshot: Snapshot,
        server_rotation: Rotation,
        random: &mut ThreadRandom,
    ) -> Result<Option<Effect>, &'static str> {
        loop {
            let Some(index) = self
                .waits
                .iter()
                .enumerate()
                .filter(|(_, w)| w.remaining <= 0)
                .min_by_key(|(_, w)| w.order)
                .map(|(i, _)| i)
            else {
                return Ok(None);
            };
            let wait = self.waits.remove(index);
            if let Some(effect) = self.resume_wait(wait, snapshot, server_rotation, random)? {
                return Ok(Some(effect));
            }
        }
    }
    /// GameTick's earlier, independent Custom swingRepeatable. Check Swing
    /// and isSwinging only when starting; changes during delay do not recheck.
    pub fn start_swing_handler(&mut self, running: bool, snapshot: Snapshot) -> Option<Effect> {
        if !running
            || self.settings.mode != Mode::Custom
            || self.waiting(Lane::CustomSwing)
            || !self.settings.swing
            || snapshot.swinging
        {
            return None;
        }
        (!self.wait(Lane::CustomSwing, self.settings.swing_delay)).then_some(Effect::SwingHand)
    }
    /// GameTick's main exclusive DROP handler, after earlier Custom effects.
    pub fn start_main_handler(
        &mut self,
        running: bool,
        snapshot: Snapshot,
        server_rotation: Rotation,
        random: &mut ThreadRandom,
    ) -> Result<Option<Effect>, &'static str> {
        if !running {
            return Ok(None);
        }
        match self.settings.mode {
            Mode::Old => {
                if !self.waiting(Lane::Old) {
                    self.wait(Lane::Old, 10);
                }
                Ok(None)
            }
            Mode::RandomInteraction => self.start_random(snapshot, random),
            Mode::Custom => {
                if self.waiting(Lane::CustomRotate) {
                    return Ok(None);
                }
                if self.settings.move_forward {
                    // Resume the synchronous tail after applying the real key.
                    return Ok(Some(Effect::ForwardKey));
                }
                self.custom_main_tail(snapshot, server_rotation, random)
            }
        }
    }
    fn custom_main_tail(
        &mut self,
        snapshot: Snapshot,
        server_rotation: Rotation,
        random: &mut ThreadRandom,
    ) -> Result<Option<Effect>, &'static str> {
        if self.settings.jump && snapshot.on_ground {
            self.pending_custom_jump += 1;
        }
        if self.settings.rotate && !self.wait(Lane::CustomRotate, self.settings.rotate_delay) {
            return Ok(Some(self.rotation(server_rotation, random)));
        }
        Ok(None)
    }
    fn start_random(
        &mut self,
        snapshot: Snapshot,
        random: &mut ThreadRandom,
    ) -> Result<Option<Effect>, &'static str> {
        if self.random_stage != RandomStage::Idle {
            return Ok(None);
        }
        let mask = self.settings.interactions & 0x3f;
        let count = mask.count_ones() as i32;
        if count == 0 {
            return Ok(None);
        }
        let index = random.next_i32_bound(count) as usize;
        let interaction = (0..6)
            .filter(|bit| mask & (1 << bit) != 0)
            .nth(index)
            .unwrap();
        let effect = match interaction {
            0 => {
                self.pending_random_jump += 1;
                None
            }
            1 => (!snapshot.swinging).then_some(Effect::SwingHand),
            2 => Some(Effect::SelectedSlot(random.next_i32_between(0, 9))),
            3 => Some(Effect::Yaw(
                snapshot.yaw + FloatRange(-180.0, 180.0).sample(&mut || random.next_unit())?,
            )),
            4 => Some(Effect::Pitch(
                (snapshot.pitch + FloatRange(-5.0, 5.0).sample(&mut || random.next_unit())?)
                    .clamp(-90.0, 90.0),
            )),
            5 => {
                self.random_direction = Direction {
                    forwards: random.next_bool(),
                    backwards: random.next_bool(),
                    left: random.next_bool(),
                    right: random.next_bool(),
                };
                // A reversed delay throws after the direction assignment, so
                // the already assigned value survives that terminated run.
                let delay = self.sample_delay(random)?;
                if self.wait(Lane::Random, delay) {
                    self.random_stage = RandomStage::Direction;
                    return Ok(None);
                }
                self.random_direction = Direction::default();
                None
            }
            _ => unreachable!(),
        };
        if effect.is_some() {
            self.random_stage = RandomStage::AfterEffect;
            self.pending_random_tail = Some(self.generation);
        } else {
            self.outer_delay(random)?;
        }
        Ok(effect)
    }
    /// Resume after the effect really executed, so an ordinary swing's packet
    /// callbacks may edit Delay before its random sample. No mode/running guard
    /// is invented between synchronous statements (only suspension checks it).
    pub fn after_effect(
        &mut self,
        effect: Effect,
        snapshot: Snapshot,
        server_rotation: Rotation,
        random: &mut ThreadRandom,
    ) -> Result<Option<Effect>, &'static str> {
        if let Some(generation) = self.pending_random_tail.take() {
            self.outer_delay(random)?;
            // Cancelling the Job inside an ordinary Punch callback does not
            // interrupt the synchronous Delay.random() expression. Its next
            // nonzero suspension immediately observes that cancellation.
            if generation != self.generation {
                self.waits.retain(|wait| wait.lane != Lane::Random);
                self.random_stage = RandomStage::Idle;
            }
        }
        if effect == Effect::ForwardKey {
            self.custom_main_tail(snapshot, server_rotation, random)
        } else {
            Ok(None)
        }
    }
    fn rotation(&self, current: Rotation, random: &mut ThreadRandom) -> Effect {
        let jitter = random.next_f64_between(-5.0, 5.0) as f32;
        Effect::Rotation(RotationTarget {
            target: DefaultRotationTarget {
                rotation: Rotation {
                    yaw: current.yaw + self.settings.angle,
                    pitch: (current.pitch + jitter).clamp(-90.0, 90.0),
                    is_normalized: false,
                },
                horizontal_turn_speed: 180.0,
                vertical_turn_speed: 180.0,
                ticks_until_reset: self.settings.rotations.ticks_until_reset,
                reset_threshold: self.settings.rotations.reset_threshold,
                consider_inventory: !self.settings.ignore_open_inventory,
                movement_correction: self.settings.rotations.movement_correction,
            },
            processor: self.settings.rotations.active,
        })
    }
    /// Source's normal mode handler then its dynamic once handlers. EventManager
    /// skips inactive listeners, so pending once survives disable/mode changes.
    pub fn movement_input(&mut self, running: bool, direction: &mut Direction, jump: &mut bool) {
        if !running {
            return;
        }
        match self.settings.mode {
            Mode::Old => direction.forwards = true,
            Mode::RandomInteraction => {
                *direction = self.random_direction;
                if self.pending_random_jump != 0 {
                    *jump = true;
                    self.pending_random_jump = 0;
                }
            }
            Mode::Custom => {
                if self.pending_custom_jump != 0 {
                    *jump = true;
                    self.pending_custom_jump = 0;
                }
            }
        }
    }
    /// Actual CancelSequences lifecycle only; source retains direction/keyUp
    /// and EventHooks. Imported enabled edits outside a world skip lifecycle.
    pub fn cancel_sequences(&mut self) {
        self.waits.clear();
        self.random_stage = RandomStage::Idle;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rng() -> ThreadRandom {
        ThreadRandom::seeded(123, 1)
    }
    fn context() -> Snapshot {
        Snapshot {
            on_ground: true,
            yaw: 12.0,
            pitch: 87.0,
            ..Default::default()
        }
    }
    #[test]
    fn old_waits_ten_ticks_and_preserves_other_direction_flags() {
        let mut state = AntiAfk::default();
        state.settings.mode = Mode::Old;
        let mut r = rng();
        assert_eq!(
            state.start_main_handler(true, context(), Rotation::default(), &mut r),
            Ok(None)
        );
        for _ in 0..9 {
            state.begin_tick(true);
            assert_eq!(
                state.next_ready(context(), Rotation::default(), &mut r),
                Ok(None)
            );
        }
        state.begin_tick(true);
        assert_eq!(
            state.next_ready(context(), Rotation::default(), &mut r),
            Ok(Some(Effect::Yaw(192.0)))
        );
        let mut direction = Direction {
            backwards: true,
            right: true,
            ..Default::default()
        };
        let mut jump = false;
        state.movement_input(true, &mut direction, &mut jump);
        assert!(direction.forwards && direction.backwards && direction.right);
        assert!(!jump);
    }
    #[test]
    fn random_direction_has_two_independent_delays_and_survives_cancellation() {
        let mut state = AntiAfk::default();
        state.settings.interactions = 1 << 5;
        state.settings.delay = [2, 2];
        let mut r = rng();
        state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap();
        let direction = state.random_direction();
        assert!(state.busy());
        state.begin_tick(true);
        state
            .next_ready(context(), Rotation::default(), &mut r)
            .unwrap();
        assert_eq!(state.random_direction(), direction);
        state.cancel_sequences();
        assert_eq!(state.random_direction(), direction);
        state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap();
        for _ in 0..2 {
            state.begin_tick(true);
            state
                .next_ready(context(), Rotation::default(), &mut r)
                .unwrap();
        }
        assert_eq!(state.random_direction(), Direction::default());
        assert!(state.busy());
        for _ in 0..2 {
            state.begin_tick(true);
            state
                .next_ready(context(), Rotation::default(), &mut r)
                .unwrap();
        }
        assert!(!state.busy());
    }
    #[test]
    fn empty_selection_draws_nothing_and_once_jump_survives_inactive_mode() {
        let mut state = AntiAfk::default();
        state.settings.interactions = 0;
        let mut r = rng();
        let mut reference = r;
        state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap();
        assert_eq!(r.next_i32(), reference.next_i32());
        state.settings.interactions = 1;
        state.settings.delay = [0, 0];
        state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap();
        state.cancel_sequences();
        state.settings.mode = Mode::Old;
        let mut d = Direction::default();
        let mut jump = false;
        state.movement_input(true, &mut d, &mut jump);
        assert!(!jump);
        state.settings.mode = Mode::RandomInteraction;
        state.movement_input(false, &mut d, &mut jump);
        assert!(!jump);
        state.movement_input(true, &mut d, &mut jump);
        assert!(jump);
        jump = false;
        state.movement_input(true, &mut d, &mut jump);
        assert!(!jump);
    }
    #[test]
    fn swing_outer_delay_reads_after_native_effect_and_negative_waits_suspend() {
        let mut state = AntiAfk::default();
        state.settings.interactions = 1 << 1;
        let mut r = rng();
        let effect = state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap()
            .unwrap();
        assert_eq!(effect, Effect::SwingHand);
        state.settings.delay = [-9, -9];
        state
            .after_effect(effect, context(), Rotation::default(), &mut r)
            .unwrap();
        assert!(state.busy());
        state.begin_tick(true);
        assert_eq!(
            state.next_ready(context(), Rotation::default(), &mut r),
            Ok(None)
        );
        assert!(!state.busy());
        state.settings.delay = [3, 3];
        let effect = state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap()
            .unwrap();
        state.cancel_sequences();
        let mut expected = r;
        expected.next_i32_inclusive(3, 3);
        state
            .after_effect(effect, context(), Rotation::default(), &mut r)
            .unwrap();
        assert!(!state.busy());
        assert_eq!(r.next_i32(), expected.next_i32());
        state.settings.delay = [5, 4];
        assert_eq!(
            state.start_main_handler(
                true,
                Snapshot {
                    swinging: true,
                    ..context()
                },
                Rotation::default(),
                &mut r
            ),
            Err("AntiAFK delay range is empty")
        );
        assert!(!state.busy());
    }
    #[test]
    fn custom_independent_waits_capture_delay_but_read_rotation_values_live() {
        let mut state = AntiAfk::default();
        state.settings.mode = Mode::Custom;
        state.settings.swing_delay = 1;
        state.settings.rotate_delay = 2;
        let mut r = rng();
        assert_eq!(state.start_swing_handler(true, context()), None);
        let forward = state
            .start_main_handler(true, context(), Rotation::default(), &mut r)
            .unwrap()
            .unwrap();
        assert_eq!(forward, Effect::ForwardKey);
        state
            .after_effect(forward, context(), Rotation::default(), &mut r)
            .unwrap();
        state.settings.swing = false;
        state.settings.rotate = false;
        state.settings.angle = 27.0;
        state.settings.rotate_delay = 100;
        state.settings.rotations.active = SmoothMode::Sigmoid;
        state.begin_tick(true);
        assert_eq!(
            state.next_ready(context(), Rotation::default(), &mut r),
            Ok(Some(Effect::SwingHand))
        );
        assert_eq!(
            state.next_ready(context(), Rotation::default(), &mut r),
            Ok(None)
        );
        state.begin_tick(true);
        let Some(Effect::Rotation(target)) = state
            .next_ready(
                context(),
                Rotation {
                    yaw: 50.0,
                    pitch: 90.0,
                    is_normalized: false,
                },
                &mut r,
            )
            .unwrap()
        else {
            panic!("rotation after captured two ticks")
        };
        assert_eq!(target.target.rotation.yaw, 77.0);
        assert!((85.0..=90.0).contains(&target.target.rotation.pitch));
        assert_eq!(target.processor_id(), 5);
        assert!(!target.target.consider_inventory);
        let mut d = Direction::default();
        let mut jump = false;
        state.movement_input(true, &mut d, &mut jump);
        assert!(jump);
        assert!(!d.forwards);
    }
    #[test]
    fn all_random_actions_and_jvm_sampling_vectors() {
        let mut state = AntiAfk::default();
        let mut r = rng();
        state.settings.delay = [0, 0];
        for (bit, expected) in [(2, "slot"), (3, "yaw"), (4, "pitch")] {
            state.settings.interactions = 1 << bit;
            let effect = state
                .start_main_handler(true, context(), Rotation::default(), &mut r)
                .unwrap()
                .unwrap();
            match (expected, effect) {
                ("slot", Effect::SelectedSlot(slot)) => assert!((0..9).contains(&slot)),
                ("yaw", Effect::Yaw(yaw)) => assert!((-168.0..=192.0).contains(&yaw)),
                ("pitch", Effect::Pitch(pitch)) => assert!((82.0..=90.0).contains(&pitch)),
                _ => panic!("wrong action"),
            }
            state
                .after_effect(effect, context(), Rotation::default(), &mut r)
                .unwrap();
            assert!(!state.busy());
        }
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("java-random-vectors.json");
        if path.exists() {
            let vectors: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            for case in vectors.as_array().unwrap() {
                let mut r = ThreadRandom::seeded(
                    case["seed"].as_u64().unwrap(),
                    case["thread"].as_u64().unwrap(),
                );
                for draw in case["draws"].as_array().unwrap() {
                    assert_eq!(r.next_i64().to_string(), draw["long"].as_str().unwrap());
                    assert_eq!(r.next_bool(), draw["boolean"].as_bool().unwrap());
                    assert_eq!(
                        r.next_f64_between(-5.0, 5.0).to_bits().to_string(),
                        draw["doubleBits"].as_str().unwrap()
                    );
                    assert_eq!(
                        r.next_i32_between(0, 9),
                        draw["slot"].as_i64().unwrap() as i32
                    );
                    assert_eq!(
                        r.next_i32_bound(draw["bound"].as_i64().unwrap() as i32),
                        draw["index"].as_i64().unwrap() as i32
                    );
                }
            }
        }
    }
}
