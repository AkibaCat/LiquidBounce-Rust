// SPDX-License-Identifier: GPL-3.0-or-later
// Derived from LiquidBounce ae0eb396 ModuleFastPlace.kt, CCBlueX 2015-2026.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FastPlace {
    pub cooldown: (i32, i32),
    pub projectiles: bool,
    pub blocks: bool,
    pub start_delay: i32,
}
impl Default for FastPlace {
    fn default() -> Self {
        Self {
            cooldown: (0, 0),
            projectiles: true,
            blocks: true,
            start_delay: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemClasses {
    pub projectile: bool,
    pub block: bool,
}
impl FastPlace {
    /// The random supplier is called only after the original item and time
    /// guards, including when the existing cooldown and sampled range are zero.
    /// An empty imported range propagates the event handler's sampling failure.
    pub fn use_cooldown<E>(
        self,
        running: bool,
        hands: [ItemClasses; 2],
        since_press: i64,
        old: i32,
        mut sample: impl FnMut(i32, i32) -> Result<i32, E>,
    ) -> Result<i32, E> {
        let applies = (self.projectiles && hands.iter().any(|i| i.projectile))
            || (self.blocks && hands.iter().any(|i| i.block));
        if !running
            || !applies
            || (self.start_delay > 0 && since_press < i64::from(self.start_delay))
        {
            return Ok(old);
        }
        let next = sample(self.cooldown.0, self.cooldown.1)?;
        Ok(if next > 0 || old > 0 { next } else { old })
    }
}

/// The pinned InputTracker has a mouse event listener but no keyboard updater.
/// Its keyboard map therefore remains empty, even for a rebound use mapping.
#[derive(Clone, Debug, Default)]
pub struct PressTimes {
    mouse: [i64; 32],
}
impl PressTimes {
    pub fn mouse_event(&mut self, button: i32, pressed: bool, now: i64) {
        if pressed
            && let Some(slot) = usize::try_from(button)
                .ok()
                .and_then(|i| self.mouse.get_mut(i))
        {
            *slot = now;
        }
    }
    pub fn since_use_press(&self, mouse_button: i32, now: i64) -> i64 {
        let time = usize::try_from(mouse_button)
            .ok()
            .and_then(|i| self.mouse.get(i))
            .copied()
            .unwrap_or(0);
        if time > 0 {
            now.wrapping_sub(time)
        } else {
            i64::MAX
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const BLOCK: ItemClasses = ItemClasses {
        projectile: false,
        block: true,
    };
    const PROJECTILE: ItemClasses = ItemClasses {
        projectile: true,
        block: false,
    };
    #[test]
    fn checks_either_hand_and_preserves_source_sampling_order() {
        let mut calls = 0;
        let mut sample = |a, b| {
            calls += 1;
            assert_eq!((a, b), (0, 0));
            Ok::<_, ()>(0)
        };
        let fast = FastPlace::default();
        assert_eq!(
            fast.use_cooldown(false, [BLOCK, BLOCK], 0, 4, &mut sample),
            Ok(4)
        );
        assert_eq!(
            fast.use_cooldown(true, [ItemClasses::default(); 2], 0, 4, &mut sample),
            Ok(4)
        );
        assert_eq!(
            fast.use_cooldown(true, [ItemClasses::default(), BLOCK], 0, 4, &mut sample),
            Ok(0)
        );
        assert_eq!(
            fast.use_cooldown(true, [PROJECTILE, BLOCK], 0, 0, &mut sample),
            Ok(0)
        );
        assert_eq!(calls, 2);
    }
    #[test]
    fn delay_filters_selected_types_and_imported_negative_values() {
        let fast = FastPlace {
            projectiles: false,
            start_delay: 100,
            cooldown: (-2, -2),
            ..Default::default()
        };
        assert_eq!(
            fast.use_cooldown(true, [PROJECTILE; 2], 100, 4, |_, _| Err::<i32, _>(
                "sampled"
            )),
            Ok(4)
        );
        assert_eq!(
            fast.use_cooldown(true, [BLOCK; 2], 99, 4, |_, _| Err::<i32, _>("sampled")),
            Ok(4)
        );
        assert_eq!(
            fast.use_cooldown(true, [BLOCK; 2], 100, 4, |_, _| Ok::<_, ()>(-2)),
            Ok(-2)
        );
        assert_eq!(
            fast.use_cooldown(true, [BLOCK; 2], 100, -1, |_, _| Ok::<_, ()>(-2)),
            Ok(-1)
        );
        assert_eq!(
            fast.use_cooldown(true, [BLOCK; 2], i64::MAX, 0, |_, _| Ok::<_, ()>(2)),
            Ok(2)
        );
        let invalid = FastPlace {
            cooldown: (3, 2),
            ..Default::default()
        };
        assert_eq!(
            invalid.use_cooldown(true, [BLOCK; 2], 0, 4, |a, b| if a > b {
                Err("empty")
            } else {
                Ok(a)
            }),
            Err("empty")
        );
    }
    #[test]
    fn input_tracker_keeps_last_press_across_release_and_reads_rebound_mouse() {
        let mut presses = PressTimes::default();
        assert_eq!(presses.since_use_press(1, 100), i64::MAX);
        presses.mouse_event(1, true, 100);
        presses.mouse_event(1, false, 110);
        assert_eq!(presses.since_use_press(1, 150), 50);
        assert_eq!(presses.since_use_press(0, 150), i64::MAX);
        presses.mouse_event(0, true, 120);
        assert_eq!(presses.since_use_press(0, 150), 30);
        assert_eq!(presses.since_use_press(-1, 150), i64::MAX);
        assert_eq!(presses.since_use_press(1, 90), -10);
    }
}
