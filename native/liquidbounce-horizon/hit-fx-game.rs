// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::particles::{ParticleOptions, ParticleType};
impl Game<'_> {
    pub(super) fn client_hit_fx_self_sound(&self, original: &str) -> String {
        let mut output = String::new();
        self.native_client_hit_fx("{\"action\":\"self\"}", &mut output);
        serde_json::from_str::<serde_json::Value>(&output)
            .ok()
            .and_then(|v| v["sound"].as_str().map(str::to_owned))
            .unwrap_or_else(|| original.into())
    }
    pub(super) fn client_hit_fx_packet(
        &mut self,
        name: &str,
        source: i32,
        at: [f64; 3],
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> bool {
        let source_name = usize::try_from(source)
            .ok()
            .and_then(|i| horizon_client::settings::SOUND_SOURCES.get(i))
            .copied()
            .unwrap_or("");
        let mut output = String::new();
        self.native_client_hit_fx(
            &serde_json::json!({"action":"sound","name":name,"source":source_name,"at":at})
                .to_string(),
            &mut output,
        );
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&output) else {
            return false;
        };
        if let Some(id) = v["target"].as_i64() {
            for particle in v["particles"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
            {
                if let Some((target, body)) = self.remote_tracking_body(id as i32) {
                    if particle == "Blood" {
                        if let Some(state) = self
                            .pack
                            .assets
                            .registry
                            .default_state("minecraft:redstone_block")
                        {
                            let pos = [
                                body.feet[0].floor() as i32,
                                (body.feet[1].floor() as i32).wrapping_add(1),
                                body.feet[2].floor() as i32,
                            ];
                            self.destroy_particles(pos, state);
                        }
                        continue;
                    }
                    let kind = match particle {
                        "Fire" => ParticleType::Lava,
                        "Heart" => ParticleType::Heart,
                        "Water" => ParticleType::FallingWater,
                        "Smoke" => ParticleType::Smoke,
                        "Magic" => ParticleType::EnchantedHit,
                        "Crits" => ParticleType::Crit,
                        _ => continue,
                    };
                    self.particles
                        .tracking_emitter(ParticleOptions::simple(kind), target, body);
                }
            }
        }
        if let Some(sound) = v["sound"].as_str() {
            self.audio
                .play_on_seeded(source as usize, sound, Some(at), volume, pitch, seed);
            true
        } else {
            false
        }
    }
}
