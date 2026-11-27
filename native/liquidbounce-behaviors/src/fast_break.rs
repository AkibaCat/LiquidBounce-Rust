// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete ModuleFastBreak, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    None,
    Grim2348,
}
impl Mode {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Grim2348 => "Grim-2.3.48",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "None" => Some(Self::None),
            "Grim-2.3.48" | "AbortAnother" => Some(Self::Grim2348),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FastBreak {
    pub break_damage: f32,
    pub only_tool: bool,
    pub mode: Mode,
}
impl Default for FastBreak {
    fn default() -> Self {
        Self {
            break_damage: 0.8,
            only_tool: false,
            mode: Mode::None,
        }
    }
}
impl FastBreak {
    pub fn tick(self, running: bool, mining_tool: bool, delay: &mut i32, progress: &mut f32) {
        if !running || self.only_tool && !mining_tool {
            return;
        }
        *delay = 0;
        if *progress > self.break_damage {
            *progress = 1.0;
        }
    }
    pub fn abort_above(
        self,
        running: bool,
        mining_tool: bool,
        stop_destroy: bool,
        position: [i32; 3],
    ) -> Option<[i32; 3]> {
        if !running
            || self.mode != Mode::Grim2348
            || !stop_destroy
            || self.only_tool && !mining_tool
        {
            return None;
        }
        Some([position[0], position[1].wrapping_add(1), position[2]])
    }
    pub fn disabled_delay() -> i32 {
        5
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn threshold_is_strict_and_imported_strength_is_not_clamped() {
        let state = FastBreak::default();
        let (mut delay, mut progress) = (4, 0.8);
        state.tick(true, false, &mut delay, &mut progress);
        assert_eq!((delay, progress), (0, 0.8));
        let mut nan = f32::NAN;
        state.tick(true, false, &mut delay, &mut nan);
        assert!(nan.is_nan());
        progress = f32::from_bits(0.8_f32.to_bits() + 1);
        state.tick(true, false, &mut delay, &mut progress);
        assert_eq!(progress, 1.0);
        let state = FastBreak {
            only_tool: true,
            ..state
        };
        delay = 4;
        progress = 2.0;
        state.tick(true, false, &mut delay, &mut progress);
        assert_eq!((delay, progress), (4, 2.0));
        state.tick(true, true, &mut delay, &mut progress);
        assert_eq!((delay, progress), (0, 1.0));
        assert_eq!(FastBreak::disabled_delay(), 5);
    }
    #[test]
    fn abort_mode_alias_and_java_position_overflow() {
        let state = FastBreak {
            mode: Mode::from_tag("AbortAnother").unwrap(),
            ..Default::default()
        };
        assert_eq!(state.mode.tag(), "Grim-2.3.48");
        assert_eq!(
            state.abort_above(true, false, true, [1, i32::MAX, 3]),
            Some([1, i32::MIN, 3])
        );
        assert!(state.abort_above(false, true, true, [0; 3]).is_none());
        assert!(state.abort_above(true, true, false, [0; 3]).is_none());
    }
}
