// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, Screen};
use horizon_client::hud::HudBatch;
use serde_json::{Value, json};
#[path = "crosshair_raster.rs"]
mod raster;
fn clocks() -> (i64, i64) {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let now = START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis() as i64
        + 1_000_000;
    let wall = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| v.as_millis() as i64);
    (now, wall)
}
impl Game<'_> {
    pub(super) fn client_crosshair_tick(&self) {
        if !self.native_window() {
            return;
        }
        let target = self.client_crosshair_entity();
        let mut identity = None;
        let mut hurt = 0;
        if let Some(id) = target {
            if let Some(m) = self.mobs.iter().find(|m| m.id as i32 == id && !m.removed) {
                identity = Some(format!("mob:{}", m.join.get()));
                hurt = m.hurt_time;
            } else if let Some(s) = self.armor_stands.iter().find(|s| s.id as i32 == id) {
                identity = Some(format!("stand:{}", s.join.get()));
            } else if let Some(d) = self.dragon.as_ref().filter(|d| d.id as i32 == id) {
                identity = Some(format!("dragon:{}", d.id));
                hurt = d.hurt_time;
            } else if let Some(g) = self
                .lan
                .iter()
                .flat_map(|h| h.hub.guests.iter())
                .find(|g| g.placed && g.entity_id == id)
            {
                if let Some(p) = g.motion.as_ref() {
                    identity = Some(format!("guest:{}", g.conn));
                    hurt = p.health.hurt.hurt_time;
                }
            }
        }
        self.native_client_crosshair(
            &json!({"phase":"tick","now":clocks().0,"target":identity,"hurt":hurt}).to_string(),
            &mut String::new(),
        );
    }
    pub(super) fn client_crosshair_reset(&self) {
        self.native_client_crosshair("{\"phase\":\"reset\"}", &mut String::new());
    }
    pub(super) fn client_crosshair_hide(&self) -> bool {
        let mut out = String::new();
        self.native_client_crosshair("{\"phase\":\"visibility\"}", &mut out);
        serde_json::from_str::<Value>(&out)
            .ok()
            .is_some_and(|v| v["cancelVanilla"] == true)
    }
    pub(super) fn client_crosshair_overlay(&self, batch: &mut HudBatch) {
        if !self.native_window() || self.player.is_none() {
            return;
        }
        let partial = self.clock.fraction() as f32;
        let attack = self
            .hand_strength
            .scale(partial, self.main_hand_attack_speed());
        let charge = self.using.and_then(|(slot, ticks)| {
            let s = self.stack_in(slot)?;
            let ticks = ticks as f32 + partial;
            Some(match &*s.item {
                "minecraft:bow" => {
                    let f = ticks as i32 as f32 / 20.;
                    ((f * f + f * 2.) / 3.).min(1.)
                }
                "minecraft:crossbow" => (ticks / Self::charge_ticks(s) as f32).clamp(0., 1.),
                "minecraft:trident" => (ticks / 10.).clamp(0., 1.),
                _ => return None,
            })
        });
        let item = 1.
            - [self.held(), self.offhand.as_ref()]
                .into_iter()
                .flatten()
                .map(|s| {
                    let info = self.res.items().get(&s.item);
                    let group = horizon_client::cooldowns::group(
                        s,
                        info.and_then(|i| i.use_cooldown.as_ref()),
                    );
                    self.cooldowns.percent(&group, partial)
                })
                .fold(0f32, f32::max);
        let menu = self
            .client_container_menu_text(self.screen)
            .and_then(|(m, _)| m);
        // InventoryScreen and ContainerScreen only; other station/creative screens retain the overlay.
        let hidden = self.screen == Screen::Inventory
            && self.mount_menu.opened_mob().is_none()
            && (menu.is_none()
                || menu
                    .as_deref()
                    .is_some_and(|m| m.starts_with("minecraft:generic_9x")));
        let (now, wall) = clocks();
        let input=json!({"phase":"render","now":now,"wall":wall,"width":batch.width,"height":batch.height,"firstPerson":self.camera_type.first_person(),"hiddenScreen":hidden,"attack":attack,"charge":charge,"item":item}).to_string();
        let mut out = String::new();
        self.native_client_crosshair(&input, &mut out);
        let Ok(v) = serde_json::from_str::<Value>(&out) else {
            return;
        };
        if let Some(c) = v.get("circle").filter(|c| !c.is_null()) {
            if let (Ok(center), Ok(lut), Some(inner), Some(outer)) = (
                serde_json::from_value::<[f32; 2]>(c["center"].clone()),
                serde_json::from_value::<Vec<i32>>(c["lut"].clone()),
                c["inner"].as_f64(),
                c["outer"].as_f64(),
            ) {
                for p in raster::circle(center, inner as f32, outer as f32, &lut, batch.scale) {
                    let [a, b, c, d] = p.bounds;
                    self.native_fill(batch, [a, b, c - a, d - b], p.color);
                }
            }
        }
        for q in v["quads"].as_array().into_iter().flatten() {
            if let (Some(a), Some(b), Some(c), Some(d), Some(color)) = (
                q[0].as_f64(),
                q[1].as_f64(),
                q[2].as_f64(),
                q[3].as_f64(),
                q[4].as_i64(),
            ) {
                self.native_fill(
                    batch,
                    [a as f32, b as f32, (c - a) as f32, (d - b) as f32],
                    color as u32,
                );
            }
        }
    }
}
