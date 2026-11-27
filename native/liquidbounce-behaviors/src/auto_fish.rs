// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleAutoFish at ae0eb396: an exclusive DROP sequence with dynamic guards.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FishHand {
    Main,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelayRange {
    pub first: i32,
    pub last: i32,
}

impl DelayRange {
    pub const fn new(first: i32, last: i32) -> Self {
        Self { first, last }
    }

    fn sample(self, random: &mut impl FnMut(i32, i32) -> i32) -> Result<i32, &'static str> {
        // Imported IntRanges are not UI-clamped. Kotlin random() throws for an
        // empty range and terminates this run; it does not disable the module.
        if self.first > self.last {
            return Err("AutoFish delay range is empty");
        }
        Ok(random(self.first, self.last))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutoFishSettings {
    pub reel_delay: DelayRange,
    pub recast_rod: bool,
    pub recast_delay: DelayRange,
    pub auto_cast_rod: bool,
    pub auto_cast_delay: DelayRange,
    /// None is the registry comparator's null key. An unknown imported sound
    /// deserializes to null and can therefore select unregistered inline sound.
    pub sounds: BTreeSet<Option<String>>,
    pub sound_distance: bool,
    pub max_distance: f32,
}

impl Default for AutoFishSettings {
    fn default() -> Self {
        Self {
            reel_delay: DelayRange::new(5, 8),
            recast_rod: true,
            recast_delay: DelayRange::new(15, 20),
            auto_cast_rod: false,
            auto_cast_delay: DelayRange::new(15, 20),
            sounds: BTreeSet::from([Some("minecraft:entity.fishing_bobber.splash".into())]),
            sound_distance: true,
            max_distance: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Stage {
    #[default]
    Idle,
    Trigger,
    Reel(i32),
    AfterReel(FishHand),
    Recast(i32),
    AutoCast(i32),
}

#[derive(Debug, Default)]
pub struct AutoFish {
    pub settings: AutoFishSettings,
    caught_fish: bool,
    stage: Stage,
    wait_callback: Option<u64>,
    callback_counter: u64,
    wait_registrations: Vec<u64>,
}

impl AutoFish {
    pub const fn caught_fish(&self) -> bool {
        self.caught_fish
    }

    pub fn busy(&self) -> bool {
        self.stage != Stage::Idle
    }

    fn register_wait(&mut self) {
        self.callback_counter = self
            .callback_counter
            .checked_add(1)
            .expect("AutoFish callback id exhausted");
        self.wait_callback = Some(self.callback_counter);
        self.wait_registrations.push(self.callback_counter);
    }

    /// Drain after every state call, before performing the returned useItem.
    pub fn take_wait_registrations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.wait_registrations)
    }

    /// Evaluate this waiter at its actual global FIFO position. A countdown
    /// or an unsatisfied tickUntil retains its id; any next wait has a new id
    /// and is pending until the following tick, even when it is also busy.
    pub fn advance_callback(
        &mut self,
        callback: u64,
        running: bool,
        rod_hand: Option<FishHand>,
        hook_present: bool,
        random: &mut impl FnMut(i32, i32) -> i32,
    ) -> Result<(bool, Option<FishHand>), &'static str> {
        if self.wait_callback != Some(callback) {
            return Ok((false, None));
        }
        let effect = self.advance_wait(running, rod_hand, hook_present, random)?;
        Ok((self.wait_callback == Some(callback), effect))
    }

    /// The selected sound must be the registry object represented by the
    /// packet's holder, not merely an inline SoundEvent with the same location.
    pub fn sound_packet(
        &mut self,
        running: bool,
        registered_sound: Option<&str>,
        hook: Option<[f64; 3]>,
        sound_position: [f64; 3],
    ) -> bool {
        if !running {
            return false;
        }
        let Some(hook) = hook else { return false };
        if !self
            .settings
            .sounds
            .contains(&registered_sound.map(str::to_owned))
        {
            return false;
        }
        if self.settings.sound_distance {
            let dx = hook[0] - sound_position[0];
            let dy = hook[1] - sound_position[1];
            let dz = hook[2] - sound_position[2];
            // Kotlin Float.sq() rounds before the Double comparison. Imported
            // negative/NaN/infinite values preserve JVM comparison behavior.
            let threshold = self.settings.max_distance * self.settings.max_distance;
            if dx * dx + dy * dy + dz * dz > f64::from(threshold) {
                return false;
            }
        }
        self.caught_fish = true;
        true
    }

    /// CoroutineTicker runs before GameTickEvent. Call once per actual client
    /// tick, including menus, then execute returned use outside all state locks.
    /// A fresh rod hand is required on every waiting tick, not the original hand.
    pub fn advance_wait(
        &mut self,
        running: bool,
        rod_hand: Option<FishHand>,
        hook_present: bool,
        random: &mut impl FnMut(i32, i32) -> i32,
    ) -> Result<Option<FishHand>, &'static str> {
        if matches!(self.stage, Stage::Idle | Stage::AfterReel(_)) {
            return Ok(None);
        }
        let Some(hand) = rod_hand.filter(|_| running) else {
            self.stage = Stage::Idle;
            self.wait_callback = None;
            return Ok(None);
        };
        let result = match self.stage {
            Stage::Trigger if self.caught_fish => {
                self.caught_fish = false;
                self.wait_callback = None;
                self.begin_delay(self.settings.reel_delay, Stage::Reel, hand, true, random)
            }
            Stage::Trigger if !hook_present => {
                self.stage = Stage::Idle;
                self.wait_callback = None;
                if self.settings.auto_cast_rod {
                    self.begin_delay(
                        self.settings.auto_cast_delay,
                        Stage::AutoCast,
                        hand,
                        false,
                        random,
                    )
                } else {
                    Ok(None)
                }
            }
            Stage::Trigger => Ok(None),
            Stage::Reel(ticks) if ticks <= 1 => {
                self.stage = Stage::AfterReel(hand);
                self.wait_callback = None;
                Ok(Some(hand))
            }
            Stage::Reel(ticks) => {
                self.stage = Stage::Reel(ticks - 1);
                Ok(None)
            }
            Stage::Recast(ticks) | Stage::AutoCast(ticks) if ticks <= 1 => {
                self.stage = Stage::Idle;
                self.wait_callback = None;
                Ok(Some(hand))
            }
            Stage::Recast(ticks) => {
                self.stage = Stage::Recast(ticks - 1);
                Ok(None)
            }
            Stage::AutoCast(ticks) => {
                self.stage = Stage::AutoCast(ticks - 1);
                Ok(None)
            }
            Stage::Idle | Stage::AfterReel(_) => unreachable!(),
        };
        if result.is_err() {
            self.stage = Stage::Idle;
            self.wait_callback = None;
        }
        result
    }

    /// Resume the synchronous code after reeling, after the real use completed.
    /// RecastRod.enabled and its delay are read here, so callbacks from useItem
    /// may change them. A zero delay reuses the guard's last hand without a new
    /// suspension/requirement check, even if the first use broke the rod.
    pub fn after_use(
        &mut self,
        random: &mut impl FnMut(i32, i32) -> i32,
    ) -> Result<Option<FishHand>, &'static str> {
        let Stage::AfterReel(hand) = self.stage else {
            return Ok(None);
        };
        self.stage = Stage::Idle;
        self.wait_callback = None;
        if !self.settings.recast_rod {
            return Ok(None);
        }
        self.begin_delay(
            self.settings.recast_delay,
            Stage::Recast,
            hand,
            false,
            random,
        )
    }

    fn begin_delay(
        &mut self,
        delay: DelayRange,
        waiting: fn(i32) -> Stage,
        hand: FishHand,
        reel: bool,
        random: &mut impl FnMut(i32, i32) -> i32,
    ) -> Result<Option<FishHand>, &'static str> {
        let ticks = delay.sample(random)?;
        if ticks == 0 {
            self.stage = if reel {
                Stage::AfterReel(hand)
            } else {
                Stage::Idle
            };
            Ok(Some(hand))
        } else {
            // waitTicks only special-cases zero: negatives still suspend until
            // the next tick, where elapsed=1 already satisfies elapsed>=ticks.
            self.stage = waiting(ticks.max(1));
            self.register_wait();
            Ok(None)
        }
    }

    /// Dispatch the tickHandler after existing waits/effects. Busy runs DROP
    /// this event. Even an already true trigger suspends until the next tick.
    pub fn start_tick_handler(&mut self, running: bool, rod_hand: Option<FishHand>) {
        if self.stage == Stage::Idle && running && rod_hand.is_some() {
            self.stage = Stage::Trigger;
            self.register_wait();
        }
    }

    /// Call for actual in-game CancelSequences/OnDisabled lifecycle callbacks.
    /// Out-of-game enabled-value edits bypass these callbacks in ClientModule.
    pub fn cancel_sequences(&mut self) {
        self.stage = Stage::Idle;
        self.wait_callback = None;
    }

    pub fn on_disabled(&mut self) {
        self.stage = Stage::Idle;
        self.wait_callback = None;
        self.caught_fish = false;
    }
}
