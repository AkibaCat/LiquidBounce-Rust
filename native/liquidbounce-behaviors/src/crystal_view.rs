// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleCrystalView and both EndCrystal mixins at LiquidBounce ae0eb396.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrystalView {
    pub size: f32,
    pub y_translate: f32,
    pub spin_speed: f32,
    pub bounce: f32,
}

impl Default for CrystalView {
    fn default() -> Self {
        Self {
            size: 0.3,
            y_translate: -0.5,
            spin_speed: 0.0,
            bounce: 0.25,
        }
    }
}

impl CrystalView {
    /// The two renderer and two model parameters, with vanilla identities while disabled.
    /// Imported Float values retain their original arithmetic; the UI owns slider bounds.
    pub fn parameters(self, running: bool) -> [f32; 4] {
        if running {
            [self.size, self.y_translate, self.spin_speed, self.bounce]
        } else {
            [1.0, 0.0, 1.0, 1.0]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_defaults_and_disabled_identity_are_exact() {
        let module = CrystalView::default();
        assert_eq!(module.parameters(true), [0.3, -0.5, 0.0, 0.25]);
        assert_eq!(module.parameters(false), [1.0, 0.0, 1.0, 1.0]);
    }
    #[test]
    fn raw_imported_values_are_not_silently_clamped() {
        let module = CrystalView {
            size: 3.0,
            y_translate: -9.0,
            spin_speed: -2.0,
            bounce: -3.0,
        };
        assert_eq!(module.parameters(true), [3.0, -9.0, -2.0, -3.0]);
    }
}
