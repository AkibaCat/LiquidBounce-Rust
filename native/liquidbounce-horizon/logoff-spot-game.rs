// SPDX-License-Identifier: GPL-3.0-or-later
// LiquidBounce ae0eb396 ModuleLogoffSpot: real, client-owned RemotePlayer equivalents.
use super::Game;
use horizon_client::mobs::Mob;
use serde_json::{Value, json};
pub(super) struct LoggedOffPlayer {
    pub id: u32,
    pub original_id: i32,
    pub original_join: u64,
    pub name: String,
    pub _time: std::time::SystemTime,
}
impl Game<'_> {
    fn client_logoff_options(&mut self) -> bool {
        let mut out = String::new();
        self.native_client_logoff_spot(r#"{"action":"options"}"#, &mut out);
        serde_json::from_str::<Value>(&out).is_ok_and(|v| v["running"] == true)
    }
    fn client_logoff_sync(&mut self) {
        let ids: Vec<i32> = self
            .client_logoff_spots
            .values()
            .map(|p| p.id as i32)
            .collect();
        self.native_client_logoff_spot(
            &json!({"action":"ids","ids":ids}).to_string(),
            &mut String::new(),
        );
    }
    fn client_logoff_message(&mut self, kind: &str, args: Value) {
        self.native_client_logoff_spot(
            &json!({"action":"message","kind":kind,"args":args}).to_string(),
            &mut String::new(),
        );
    }
    fn client_logoff_remove(&mut self, id: u32) {
        self.mobs.retain(|m| m.id != id);
        self.persist.uuids.remove(&id);
        self.client_logoff_remove_identity(id as i32);
    }
    pub(super) fn client_logoff_clear(&mut self) {
        let old = std::mem::take(&mut self.client_logoff_spots);
        for p in old.into_values() {
            self.client_logoff_remove(p.id);
        }
        self.native_client_logoff_spot(r#"{"action":"reset"}"#, &mut String::new());
    }
    fn client_logoff_health(&self, name: &str, native: f32) -> f32 {
        let Some(o) = self.score.board.display_objective("below_name") else {
            return native;
        };
        if !["❤", "HP", "Health", "Здоровья", "Здоровье"]
            .iter()
            .any(|s| o.display_text().plain().contains(s))
        {
            return native;
        }
        self.score
            .board
            .score(name, &o.name)
            .map_or(native, |s| s.value as f32)
    }
    // Called before RemoveEntities erases its profile maps, and on direct client removals.
    // The origin's join token suppresses the second call for the same removal.
    pub(super) fn client_logoff_capture(&mut self, ids: &[i32]) {
        // All playable windows, including single player/LAN, are remote clients of
        // their server. Never insert these into an authoritative server Game.
        if !self.native_window() || self.remote.is_none() || self.lan_acting_uuid.is_some() {
            return;
        }
        if !self.client_logoff_options() {
            self.client_logoff_clear();
            return;
        }
        for &original_id in ids {
            if self.native_client_blink_dummy() == Some(original_id)
                || self
                    .client_logoff_spots
                    .values()
                    .any(|p| p.id == original_id as u32)
            {
                continue;
            }
            let Some(source) = self
                .mobs
                .iter()
                .find(|m| m.id == original_id as u32 && &*m.kind.id == "minecraft:player")
                .cloned()
            else {
                continue;
            };
            let Some(profile) = self.remote_player_uuid(source.id) else {
                continue;
            };
            let uuid = self
                .persist
                .uuids
                .get(&source.id)
                .map_or(profile, |u| horizon_client::lan::playerdata::ints_uuid(*u));
            let join = source.join.get();
            if self
                .client_logoff_spots
                .get(&uuid)
                .is_some_and(|p| p.original_id == original_id && p.original_join == join)
            {
                continue;
            }
            let Some(name) = self.remote_player_name(source.id).map(str::to_owned) else {
                continue;
            };
            let Some(kind) = self.res.player_type().clone() else {
                continue;
            };
            static NEXT: std::sync::atomic::AtomicI32 =
                std::sync::atomic::AtomicI32::new(-1_000_000_000);
            let id = loop {
                let id = NEXT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) as u32;
                if self.native_entity(id).is_none() {
                    break id;
                }
            };
            let mut fake = Mob::new(id, kind, source.position, source.yaw);
            super::remote_mob_data::client_defaults(&mut fake);
            fake.previous = source.position;
            fake.head_yaw = source.head_yaw;
            fake.previous_head_yaw = source.head_yaw;
            fake.head_pitch = source.head_pitch;
            fake.previous_head_pitch = source.head_pitch;
            fake.health = self.client_logoff_health(&name, source.health);
            fake.skin = source.skin.clone();
            fake.slim = source.slim;
            fake.skin_3d = source.skin_3d.clone();
            fake.cape = source.cape.clone();
            // Fresh player defaults; copy inventory without stale pose/effects/hurt state.
            let gear = source.equipment_stacks();
            fake.extra.set_main_hand_stack(gear[0].clone());
            fake.extra.off_hand = source.extra.off_hand.clone();
            fake.extra.head = source.extra.head.clone();
            fake.extra.chest = source.extra.chest.clone();
            fake.extra.legs = source.extra.legs.clone();
            fake.extra.feet = source.extra.feet.clone();
            fake.extra.armor = source.extra.armor.clone();
            self.persist
                .uuids
                .insert(id, horizon_client::save::random_uuid());
            self.client_logoff_add_identity(original_id, id as i32, profile, name.clone());
            self.mobs.push(fake);
            self.client_logoff_spots.insert(
                uuid,
                LoggedOffPlayer {
                    id,
                    original_id,
                    original_join: join,
                    name: name.clone(),
                    _time: std::time::SystemTime::now(),
                },
            );
            self.client_logoff_message(
                "disappeared",
                json!([
                    name,
                    source.position.x.floor() as i32,
                    source.position.y.floor() as i32,
                    source.position.z.floor() as i32
                ]),
            );
        }
        self.client_logoff_sync();
    }
    pub(super) fn client_logoff_tick(&mut self) {
        if !self.native_window() {
            return;
        }
        if !self.client_logoff_options() {
            self.client_logoff_clear();
            return;
        }
        let mut removed = vec![];
        for (&uuid, p) in &self.client_logoff_spots {
            let unloaded = self.mobs.iter().find(|m| m.id == p.id).is_none_or(|m| {
                !self
                    .world
                    .loaded(m.position.x.floor() as i32, m.position.z.floor() as i32)
            });
            // Profile UUIDs are kept for skin lookup; exclude the random-UUID fakes.
            let reappeared = self
                .mobs
                .iter()
                .filter(|m| !m.removed && &*m.kind.id == "minecraft:player")
                .any(|m| {
                    self.persist
                        .uuids
                        .get(&m.id)
                        .is_some_and(|u| horizon_client::lan::playerdata::ints_uuid(*u) == uuid)
                });
            if unloaded || reappeared {
                removed.push((uuid, if unloaded { "unloaded" } else { "reappeared" }));
            }
        }
        for (uuid, kind) in removed {
            if let Some(p) = self.client_logoff_spots.remove(&uuid) {
                self.client_logoff_remove(p.id);
                self.client_logoff_message(kind, json!([p.name]));
            }
        }
        self.client_logoff_sync();
    }
}
