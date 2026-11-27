// SPDX-License-Identifier: GPL-3.0-or-later
// ModuleHitFX and HitFXRegistry, LiquidBounce ae0eb396, CCBlueX 2015-2026.
use crate::random::ThreadRandom;
pub const PARTICLES: [&str; 7] = ["Blood", "Fire", "Heart", "Water", "Smoke", "Magic", "Crits"];
include!("hit_fx_sounds.rs");
pub const VANILLA_HITS: [&str; 9] = [
    "entity.arrow.hit_player",
    "entity.player.attack.nodamage",
    "entity.player.attack.knockback",
    "entity.player.attack.crit",
    "entity.player.attack.strong",
    "entity.player.attack.sweep",
    "entity.player.attack.weak",
    "item.spear.hit",
    "entity.player.hurt",
];
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub particles: u32,
    pub amount: [i32; 2],
    pub other_sound: u32,
    pub self_sound: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            particles: 1 << 1,
            amount: [1, 1],
            other_sound: 1 << 17,
            self_sound: 1 << 3,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitFx {
    pub settings: Settings,
    pub last_target: Option<i32>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    pub target: i32,
    pub particles: Vec<&'static str>,
    pub sound: Option<&'static str>,
}
fn choose(mask: u32, n: usize, r: &mut ThreadRandom) -> Option<usize> {
    let choices = (0..n).filter(|i| mask & (1 << i) != 0).collect::<Vec<_>>();
    if choices.is_empty() {
        None
    } else {
        Some(choices[r.next_i32_bound(choices.len() as i32) as usize])
    }
}
pub fn sound(mask: u32, r: &mut ThreadRandom) -> Option<&'static str> {
    let i = choose(mask, SOUNDS.len(), r)?;
    let variants = SOUNDS[i].1;
    Some(variants[r.next_i32_bound(variants.len() as i32) as usize])
}
impl HitFx {
    pub fn attack(&mut self, id: i32, living: bool, alive: bool) {
        if living && alive {
            self.last_target = Some(id);
        }
    }
    pub fn packet(
        &mut self,
        name: &str,
        source: &str,
        target: i32,
        alive: bool,
        position: [f64; 3],
        sound_position: [f64; 3],
        r: &mut ThreadRandom,
    ) -> Option<Effect> {
        if !matches!(source, "player" | "neutral" | "hostile")
            || !VANILLA_HITS.contains(&name.trim_start_matches("minecraft:"))
            || self.last_target != Some(target)
            || !alive
            || position
                .into_iter()
                .zip(sound_position)
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f64>()
                > 16.
        {
            return None;
        }
        let mut particles = vec![];
        if self.settings.particles != 0 {
            let count = r.next_i32_inclusive(self.settings.amount[0], self.settings.amount[1]);
            for _ in 0..count {
                particles
                    .push(PARTICLES[choose(self.settings.particles, PARTICLES.len(), r).unwrap()]);
            }
        }
        let sound = sound(self.settings.other_sound, r);
        self.last_target = None;
        Some(Effect {
            target,
            particles,
            sound,
        })
    }
}
