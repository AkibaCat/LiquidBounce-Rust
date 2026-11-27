// SPDX-License-Identifier: GPL-3.0-or-later
//! Original GlideAbilities capture, with live item getter order preserved.
pub trait GlideStack {
    fn can_glide_chest(&self) -> bool;
    fn is_damageable(&self) -> bool;
    fn max_damage(&self) -> i32;
    fn damage(&self) -> i32;
    fn is_firework_rocket(&self) -> bool;
    fn count(&self) -> i32;
    fn firework_flight_duration(&self) -> Option<i32>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlideAbilities {
    pub rockets: i32,
    pub rocket_ticks: i32,
    pub glide_ticks: i32,
}
impl GlideAbilities {
    pub fn of<'a, S: GlideStack + 'a>(
        elytra: &impl GlideStack,
        stacks: impl IntoIterator<Item = &'a S>,
        use_rockets: bool,
    ) -> Option<Self> {
        if !elytra.can_glide_chest() {
            return None;
        }
        let glide_ticks = if elytra.is_damageable() {
            elytra
                .max_damage()
                .wrapping_sub(elytra.damage())
                .wrapping_sub(1)
                .wrapping_mul(20)
        } else {
            i32::MAX
        };
        let mut rockets: i32 = 0;
        let mut flight = i32::MAX;
        if use_rockets {
            for stack in stacks {
                if stack.is_firework_rocket() {
                    rockets = rockets.wrapping_add(stack.count());
                    flight = flight.min(stack.firework_flight_duration().unwrap_or(1));
                }
            }
        }
        let rocket_ticks = if rockets > 0 {
            10_i32.wrapping_mul(1_i32.wrapping_add(flight))
        } else {
            0
        };
        Some(Self {
            rockets,
            rocket_ticks,
            glide_ticks,
        })
    }
}
