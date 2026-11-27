// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleFakeLag at LiquidBounce ae0eb396; the shared BlinkManager owns packets.
use crate::clock::Chronometer;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub range: [f32; 2],
    pub delay: [i32; 2],
    pub recoil: i32,
    pub constant: bool,
    pub flush_on: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            range: [2., 5.],
            delay: [300, 600],
            recoil: 250,
            constant: false,
            flush_on: 7,
        }
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum PacketKind {
    #[default]
    Other,
    EntityInteract,
    BlockInteract,
    Action,
    ResourcePack,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct Context {
    pub dead: bool,
    pub in_water: bool,
    pub screen: bool,
    pub using_consumable: bool,
    pub shoot_constant: bool,
    pub no_shoot_target: bool,
    pub old_position: bool,
    pub has_entities: bool,
    pub intersects: bool,
    pub server_distance: f64,
    pub client_distance: f64,
}
pub struct FakeLag {
    pub settings: Settings,
    pub next_delay: i32,
    pub chronometer: Chronometer,
    pub enemy_nearby: bool,
}
impl FakeLag {
    pub fn new(next_delay: i32) -> Self {
        Self {
            settings: Settings::default(),
            next_delay,
            chronometer: Chronometer::default(),
            enemy_nearby: false,
        }
    }
    pub fn disabled(&mut self) {
        self.enemy_nearby = false;
    }
    /// Incoming correction/damage cases in the original switch are unreachable
    /// behind its OUTGOING guard, and intentionally cannot reset this clock.
    pub fn queues(
        &mut self,
        running: bool,
        outgoing: bool,
        packet: PacketKind,
        now: i64,
        oldest: Option<i64>,
        c: Context,
        mut random: impl FnMut([i32; 2]) -> i32,
    ) -> bool {
        if !running || !outgoing || c.dead || c.in_water || c.screen {
            return false;
        }
        if !self
            .chronometer
            .has_at_least_elapsed(i64::from(self.settings.recoil), &mut || now)
        {
            return false;
        }
        if oldest.is_some_and(|t| now.wrapping_sub(t) >= i64::from(self.next_delay)) {
            self.next_delay = random(self.settings.delay);
            return false;
        }
        let mask = match packet {
            PacketKind::EntityInteract => 1,
            PacketKind::BlockInteract => 2,
            PacketKind::Action => 4,
            _ => 0,
        };
        if self.settings.flush_on & mask != 0 || packet == PacketKind::ResourcePack {
            self.chronometer.reset_at(now);
            return false;
        }
        if c.using_consumable {
            return false;
        }
        if c.shoot_constant && c.no_shoot_target {
            return true;
        }
        if self.settings.constant {
            return true;
        }
        self.enemy_nearby
            && (!c.old_position
                || c.has_entities && !c.intersects && c.server_distance >= c.client_distance)
    }
}
