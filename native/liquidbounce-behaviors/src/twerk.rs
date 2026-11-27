// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleTwerk at LiquidBounce ae0eb396, priority 10 MovementInputEvent.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Twerk {
    /// Raw Int imports are preserved. 1..20 is UI metadata only.
    pub delay: i32,
}

impl Default for Twerk {
    fn default() -> Self {
        Self { delay: 2 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwerkError {
    /// Java's % throws ArithmeticException before assigning event.sneak.
    DivisionByZero,
}

impl Twerk {
    /// Use the actual player's signed Entity.tickCount for this input phase.
    /// This is a direct overwrite; physical sneak does not bypass the module.
    /// Later lower-priority MovementInputEvent handlers can overwrite it.
    pub fn movement_input(
        &self,
        running: bool,
        tick_count: i32,
        sneak: &mut bool,
    ) -> Result<(), TwerkError> {
        if !running {
            return Ok(());
        }
        let period = self.delay.wrapping_mul(2);
        if period == 0 {
            return Err(TwerkError::DivisionByZero);
        }
        *sneak = tick_count.wrapping_rem(period) < self.delay;
        Ok(())
    }
}
