// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleMaceKill and LocalPlayer.warp, LiquidBounce ae0eb396.
#[path = "mace_kill_height.rs"]
pub mod height;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaceKill {
    pub fall_height: i32,
}
impl Default for MaceKill {
    fn default() -> Self {
        Self { fall_height: 22 }
    }
}
pub trait Host {
    fn holding_mace(&mut self) -> bool;
    fn determine_height(&mut self, maximum: i32) -> i32;
    fn on_ground(&mut self) -> bool;
    fn position(&mut self) -> [f64; 3];
    /// None is StatusOnly on foot. A rider instead moves/sends its vehicle.
    fn warp(&mut self, position: Option<[f64; 3]>, on_ground: bool);
}
impl MaceKill {
    pub fn attack(self, running: bool, host: &mut impl Host) {
        if !running || !host.holding_mace() {
            return;
        }
        let height = host.determine_height(self.fall_height);
        if height > 10 {
            let count = (f64::from(height) / 10.0).abs().ceil() as i32;
            for _ in 0..count {
                host.warp(None, false);
            }
        } else {
            for _ in 0..2 {
                let ground = host.on_ground();
                host.warp(None, ground);
            }
        }
        let [x, y, z] = host.position();
        host.warp(Some([x, y + f64::from(height), z]), false);
        // The source re-reads the player here, after packet listeners ran.
        let position = host.position();
        host.warp(Some(position), false);
    }
}
