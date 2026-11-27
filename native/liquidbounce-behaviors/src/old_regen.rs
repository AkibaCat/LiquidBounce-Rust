// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleOldRegen ae0eb396. Health speed intentionally has no maximum clamp.
#[path = "navigation/java_case.rs"]
mod java_case;
pub const CONDITIONS: [&str; 4] = [
    "NotInTheAir",
    "NotDuringMove",
    "NotDuringRegeneration",
    "DoNotCauseHunger",
];
pub const PACKET_TYPES: [&str; 4] = [
    "OnGroundOnly",
    "PositionAndOnGround",
    "LookAndOnGround",
    "Full",
];
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OldRegen {
    pub health_enabled: bool,
    pub health: i32,
    pub increment: i32,
    pub bad_effects: bool,
    pub fire: bool,
    pub maximum_speed: i32,
    pub timer: f32,
    pub conditions: u32,
    pub packet_type: u8,
}
impl Default for OldRegen {
    fn default() -> Self {
        Self {
            health_enabled: true,
            health: 16,
            increment: 20,
            bad_effects: false,
            fire: false,
            maximum_speed: 100,
            timer: 1.0,
            conditions: 1,
            packet_type: 3,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub creative: bool,
    pub dead: bool,
    pub health: f32,
    pub ground: bool,
    pub moving: bool,
    pub regeneration: bool,
    pub food: i32,
    pub burning: bool,
    pub wet: bool,
    pub longest_harmful: Option<i32>,
}
pub fn packet_type(tag: &str) -> Option<u8> {
    PACKET_TYPES
        .iter()
        .position(|name| java_case::equals_ignore_case(name, tag))
        .map(|index| index as u8)
}
impl OldRegen {
    pub fn tick_speed(self, running: bool, context: Context) -> i32 {
        if !running || context.creative || context.dead {
            return 0;
        }
        let blocked = [
            !context.ground,
            context.moving,
            context.regeneration,
            context.food < 20,
        ];
        if blocked
            .into_iter()
            .enumerate()
            .any(|(index, value)| value && self.conditions & (1 << index) != 0)
        {
            return 0;
        }
        let mut speed = 0;
        if self.health_enabled {
            speed =
                speed.max(((self.health as f32 - context.health) * self.increment as f32) as i32);
        }
        if self.fire && context.burning && !context.wet {
            speed = speed.max(9);
        }
        if self.bad_effects
            && let Some(duration) = context.longest_harmful
        {
            speed = speed.max((duration / 20).min(self.maximum_speed));
        }
        speed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn health_fire_effects_and_guards_smoke() {
        let context = Context {
            creative: false,
            dead: false,
            health: 10.0,
            ground: true,
            moving: false,
            regeneration: false,
            food: 20,
            burning: true,
            wet: false,
            longest_harmful: Some(2400),
        };
        let mut module = OldRegen::default();
        assert_eq!(module.tick_speed(true, context), 120);
        assert_eq!(module.tick_speed(false, context), 0);
        assert_eq!(
            module.tick_speed(
                true,
                Context {
                    ground: false,
                    ..context
                }
            ),
            0
        );
        module.health_enabled = false;
        module.fire = true;
        module.bad_effects = true;
        assert_eq!(module.tick_speed(true, context), 100);
        module.maximum_speed = 5;
        assert_eq!(module.tick_speed(true, context), 9);
        assert_eq!(
            module.tick_speed(
                true,
                Context {
                    wet: true,
                    ..context
                }
            ),
            5
        );
        module.conditions = 15;
        assert_eq!(
            module.tick_speed(
                true,
                Context {
                    food: 19,
                    ..context
                }
            ),
            0
        );
        assert_eq!(packet_type("FULL"), Some(3));
    }
}
