// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleHitbox and its Entity/AttackRange/ProjectileUtil/debug mixins.
// Copyright (c) 2015-2026 CCBlueX, LiquidBounce ae0eb396.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hitbox {
    pub size: f32,
    pub apply_to_debug_hitbox: bool,
    pub apply_to_component: bool,
}
impl Default for Hitbox {
    fn default() -> Self {
        Self {
            size: 0.1,
            apply_to_debug_hitbox: true,
            apply_to_component: true,
        }
    }
}
impl Hitbox {
    /// Entity.getPickRadius RETURN replaces the original radius for valid targets.
    pub fn pick_margin(self, original: f32, should_be_attacked: bool) -> f32 {
        if should_be_attacked {
            self.size
        } else {
            original
        }
    }
    pub fn debug_margin(self, should_be_attacked: bool) -> f32 {
        if self.apply_to_debug_hitbox && should_be_attacked {
            self.size
        } else {
            0.0
        }
    }
    /// These two mixins only check attacker === localPlayer; no target filter.
    pub fn component_margin(self, original: f32) -> f32 {
        if self.apply_to_component {
            self.size + original
        } else {
            original
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pick_replaces_but_component_adds_float_margin() {
        let module = Hitbox::default();
        assert_eq!(module.pick_margin(1.0, true), 0.1);
        assert_eq!(module.pick_margin(1.0, false), 1.0);
        assert_eq!(
            module.component_margin(0.125).to_bits(),
            (0.1f32 + 0.125).to_bits()
        );
    }
    #[test]
    fn independent_flags_and_raw_import_values_remain_effective() {
        let module = Hitbox {
            size: -0.2,
            apply_to_debug_hitbox: false,
            apply_to_component: false,
        };
        assert_eq!(module.pick_margin(0.3, true), -0.2);
        assert_eq!(module.debug_margin(true), 0.0);
        assert_eq!(module.component_margin(0.125), 0.125);
    }
}
