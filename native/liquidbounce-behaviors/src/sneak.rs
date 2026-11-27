// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleSneak at the native 26.3 protocol (without ViaFabricPlus).
pub const MODES: [&str; 3] = ["Legit", "Vanilla", "Switch"];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sneak {
    pub mode: u8,
    pub not_during_move: bool,
    pub magma_only: bool,
}
impl Default for Sneak {
    fn default() -> Self {
        Self {
            mode: 1,
            not_during_move: false,
            magma_only: false,
        }
    }
}
impl Sneak {
    pub fn allowed(self, running: bool, moving: bool) -> bool {
        running && !(moving && self.not_during_move)
    }
    pub fn legit(self, running: bool, moving: bool, immune: bool, on_magma: bool) -> bool {
        self.mode == 0
            && self.allowed(running, moving)
            && (!self.magma_only || (!immune && on_magma))
    }
    pub fn force_packet(self, running: bool, moving: bool, flags: u8) -> u8 {
        if self.mode == 1 && self.allowed(running, moving) {
            flags | 32
        } else {
            flags
        }
    }
    pub fn protocol_warning(self) -> bool {
        self.mode == 2
    }
    pub fn tag(self) -> &'static str {
        MODES[usize::from(self.mode.min(2))]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_input_packet_and_mode_smoke() {
        let mut s = Sneak::default();
        assert_eq!(s.force_packet(true, false, 65), 97);
        assert_eq!(s.force_packet(false, false, 65), 65);
        s.not_during_move = true;
        assert_eq!(s.force_packet(true, true, 1), 1);
        s.mode = 0;
        s.magma_only = true;
        assert!(s.legit(true, false, false, true));
        assert!(!s.legit(true, false, true, true));
        assert!(!s.legit(true, true, false, true));
        assert!(!s.legit(true, false, false, false));
        s.mode = 2;
        assert!(s.protocol_warning());
        assert_eq!(s.force_packet(true, false, 1), 1);
    }
}
