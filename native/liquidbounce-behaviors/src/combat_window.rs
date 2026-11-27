// SPDX-License-Identifier: GPL-3.0-or-later
//! CombatManager's AttackEntityEvent / GameTickEvent combat window.
//! Attack cooldown clocks and KillAura target tracking have separate owners.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CombatWindow {
    remaining: i32,
    pause_combat: i32,
    pause_rotation: i32,
    pause_blocking: i32,
}
impl CombatWindow {
    pub fn tick(&mut self) {
        if self.remaining > 0 {
            self.remaining -= 1;
        }
        for pause in [&mut self.pause_combat, &mut self.pause_rotation, &mut self.pause_blocking] {
            if *pause > 0 {
                *pause -= 1;
            }
        }
    }
    /// The source handler still observes a cancelled AttackEntityEvent.
    pub fn attack(&mut self, living: bool, should_be_attacked: bool) {
        if living && should_be_attacked {
            self.remaining = 40;
        }
    }
    pub fn in_combat(&self) -> bool {
        self.remaining > 0
    }
    pub fn remaining(&self) -> i32 {
        self.remaining
    }
    pub fn pause_combat_for_at_least(&mut self, ticks: i32) {
        self.pause_combat = self.pause_combat.max(ticks);
    }
    pub fn pause_rotation_for_at_least(&mut self, ticks: i32) {
        self.pause_rotation = self.pause_rotation.max(ticks);
    }
    pub fn pause_blocking_for_at_least(&mut self, ticks: i32) {
        self.pause_blocking = self.pause_blocking.max(ticks);
    }
    pub fn should_pause_combat(&self) -> bool { self.pause_combat > 0 }
    pub fn should_pause_rotation(&self) -> bool { self.pause_rotation > 0 }
    pub fn should_pause_blocking(&self) -> bool { self.pause_blocking > 0 }
    pub fn combat_pause_ticks(&self) -> i32 { self.pause_combat }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eligible_attacks_refresh_exactly_forty_game_ticks() {
        let mut window = CombatWindow::default();
        window.attack(false, true);
        window.attack(true, false);
        assert!(!window.in_combat());
        window.attack(true, true);
        for left in (1..=40).rev() {
            assert_eq!(window.remaining(), left);
            assert!(window.in_combat());
            window.tick();
        }
        window.tick();
        assert_eq!(window.remaining(), 0);
        window.attack(true, true);
        window.tick();
        window.attack(true, true);
        assert_eq!(window.remaining(), 40);
    }

    #[test]
    fn pauses_extend_independently_from_attack_tracking() {
        let mut window = CombatWindow::default();
        window.pause_combat_for_at_least(-1);
        assert!(!window.should_pause_combat());
        window.pause_combat_for_at_least(2);
        window.pause_rotation_for_at_least(3);
        window.pause_blocking_for_at_least(1);
        assert!(!window.in_combat());
        window.attack(true, true);
        window.tick();
        assert!(!window.should_pause_blocking());
        assert!(window.should_pause_rotation());
        window.pause_combat_for_at_least(0);
        assert_eq!(window.combat_pause_ticks(), 1);
        window.pause_combat_for_at_least(4);
        for _ in 0..4 { window.tick(); }
        assert!(!window.should_pause_combat());
        assert!(!window.should_pause_rotation());
        assert_eq!(window.remaining(), 35);
    }
}
