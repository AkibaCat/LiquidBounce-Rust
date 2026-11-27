// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, view_vector};
use horizon_client::{
    hud::HudBatch,
    mesher::Vertex,
    mob_owners as owners,
    mobs::{Mob, Who},
    text::Text,
};
impl Game<'_> {
    fn client_owner_uuid(&self, id: u32) -> Option<u128> {
        if id == self.remote.as_ref().map_or(0, |r| r.entity_id as u32) && self.player.is_some() {
            return Some(horizon_client::lan::playerdata::ints_uuid(
                self.acting_uuid(),
            ));
        }
        if let Some(uuid) = self.remote_player_uuid(id) {
            return Some(uuid);
        }
        if let Some(g) = self
            .lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .find(|g| g.placed && g.entity_id as u32 == id)
        {
            return Some(g.uuid);
        }
        self.mobs
            .iter()
            .find(|m| m.id == id && !m.removed)
            .and_then(|_| self.persist.uuids.get(&id))
            .map(|u| horizon_client::lan::playerdata::ints_uuid(*u))
    }
    fn client_owner_who(&self, owner: Option<Who>, guest: Option<u32>) -> Option<u128> {
        match owner {
            Some(Who::Mob(id)) => self.client_owner_uuid(id),
            Some(Who::Player) => match guest {
                Some(conn) => self
                    .lan
                    .iter()
                    .flat_map(|h| h.hub.guests.iter())
                    .find(|g| g.conn == conn && g.placed)
                    .map(|g| g.uuid),
                None => self
                    .player
                    .as_ref()
                    .map(|_| horizon_client::lan::playerdata::ints_uuid(self.acting_uuid())),
            },
            None => None,
        }
    }
    pub(super) fn client_record_projectile_owner(
        &self,
        id: u32,
        owner: Option<Who>,
        guest: Option<u32>,
        at: [f64; 3],
        height: f32,
    ) {
        if !owners::enabled(true) {
            return;
        }
        let uuid = owners::remote_owner(id)
            .and_then(|id| self.client_owner_uuid(id))
            .or_else(|| self.client_owner_who(owner, guest));
        if let Some(owner) = uuid {
            self.client_record_owner(owner, at, height);
        }
    }
    fn client_record_owner(&self, owner: u128, at: [f64; 3], height: f32) {
        let light = self.world.light(
            at[0].floor() as i32,
            (at[1] + f64::from(height) * 0.85).floor() as i32,
            at[2].floor() as i32,
        );
        owners::record(owners::Label {
            owner,
            at,
            height,
            light: horizon_client::block_entity_render::vertex_light(light),
        });
    }
    pub(super) fn client_record_mob_owner(&self, m: &Mob, alpha: f64) {
        if !owners::enabled(false) || !owners::OWNABLE.contains(&&*m.kind.id) {
            return;
        }
        let Some(uuid) = m.extra.owner.or_else(|| {
            if &*m.kind.id == "minecraft:vex" {
                self.persist.uuids.get(&(m.extra.counter2 as u32)).copied()
            } else {
                None
            }
        }) else {
            return;
        };
        let at = [
            m.previous.x + (m.position.x - m.previous.x) * alpha,
            m.previous.y + (m.position.y - m.previous.y) * alpha,
            m.previous.z + (m.position.z - m.previous.z) * alpha,
        ];
        self.client_record_owner(
            horizon_client::lan::playerdata::ints_uuid(uuid),
            at,
            m.height(),
        );
    }
    fn client_owner_name(&self, uuid: u128) -> Option<String> {
        if self.player.is_some()
            && uuid == horizon_client::lan::playerdata::ints_uuid(self.acting_uuid())
        {
            return Some(self.own_profile_name());
        }
        for (id, name) in self.remote_player_names() {
            if self.remote_player_uuid(id) == Some(uuid) {
                return Some(name);
            }
        }
        self.lan
            .iter()
            .flat_map(|h| h.hub.guests.iter())
            .find(|g| g.placed && g.uuid == uuid)
            .map(|g| g.name.clone())
    }
    pub(super) fn mob_owner_geometry(&mut self, camera: glam::DVec3, out: &mut Vec<Vertex>) {
        let (yaw, pitch) = self.posed_rotation.unwrap_or_else(|| {
            if self.camera_type == super::perspective::CameraType::ThirdPersonFront {
                (self.yaw + 180.0, -self.pitch)
            } else {
                (self.yaw, self.pitch)
            }
        });
        let forward = glam::DVec3::from_array(view_vector(yaw, pitch));
        let right = forward.cross(glam::DVec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        for label in owners::labels() {
            let name = self
                .client_owner_name(label.owner)
                .unwrap_or_else(|| owners::cached(label.owner));
            let pos = glam::DVec3::from_array(label.at);
            if pos.distance_squared(camera) > 4096.0 {
                continue;
            }
            let text = Text::literal(name).resolve_legacy();
            let o = pos + glam::DVec3::Y * f64::from(label.height / 2.0) - camera;
            let mut batch = HudBatch::new([1, 1]);
            let width = self.gui.atlas.font.width_text(&text) as i32;
            let x = (-width / 2) as f32;
            // The pinned mixin submits one NORMAL, shadowed text pass with opaque black background.
            batch.fill(
                &self.gui.atlas,
                x - 1.0,
                -1.0,
                width as f32 + 1.0,
                10.0,
                0xff000000,
            );
            let background = batch.vertices.len();
            batch.text_styled_world(&self.gui.atlas, &text, x, 0.0, 0xffffffff, true);
            self.mark_entity(out.len(), "minecraft:name_tag");
            for (i, quad) in batch.vertices.as_chunks::<6>().0.iter().enumerate() {
                for v in [quad[5], quad[2], quad[1], quad[0]] {
                    let depth = if i * 6 < background {
                        -0.002
                    } else {
                        -f64::from(v.depth) * 0.025
                    };
                    let p = o + right * f64::from(v.pos[0]) * 0.025
                        - up * f64::from(v.pos[1]) * 0.025
                        + forward * depth;
                    out.push(Vertex {
                        normal: [0; 4],
                        pos: p.as_vec3().to_array(),
                        uv: v.uv,
                        layer: 0,
                        color: v.color,
                        light: label.light,
                        glint_uv: [0.0; 2],
                    });
                }
            }
            self.map_patches.extend(batch.patches);
        }
    }
}
