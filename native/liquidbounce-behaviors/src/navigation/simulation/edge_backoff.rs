// SPDX-License-Identifier: GPL-3.0-or-later
//! Minecraft Agent EdgeBackOff; the caller supplies the real canFall operation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeBackOff {
    pub x: f64,
    pub z: f64,
}
impl EdgeBackOff {
    pub fn back_off(&mut self, dx: f64, dz: f64, mut can_fall: impl FnMut(f64, f64, f64) -> bool) {
        self.x = dx;
        self.z = dz;
        let depth = f64::from(0.6_f32);
        while self.x != 0.0 && can_fall(self.x, 0.0, depth) {
            self.x = shortened(self.x);
        }
        while self.z != 0.0 && can_fall(0.0, self.z, depth) {
            self.z = shortened(self.z);
        }
        while self.x != 0.0 && self.z != 0.0 && can_fall(self.x, self.z, depth) {
            self.x = shortened(self.x);
            self.z = shortened(self.z);
        }
    }
}
fn shortened(delta: f64) -> f64 {
    if delta.abs() <= 0.05 {
        0.0
    } else {
        delta - delta.signum() * 0.05
    }
}
