// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn client_free_cam_state(&self, partial: f32) -> Value {
        let mut output = String::new();
        let input = json!({"phase":"state","partial":partial,"eye":self.eye_position(),"playerRotation":[self.yaw,self.pitch]});
        self.native_client_free_cam(&input.to_string(), &mut output);
        serde_json::from_str(&output).unwrap_or(Value::Null)
    }
    pub(super) fn client_free_cam_event(&self, input: Value) {
        self.native_client_free_cam(&input.to_string(), &mut String::new());
    }
    pub(super) fn client_free_cam_active(&self) -> bool {
        self.native_window()
            && self.lan_acting.is_none()
            && self.lan_acting_uuid.is_none()
            && self
                .player
                .as_ref()
                .is_some_and(|p| p.native_client_queries_allowed())
            && horizon_client::free_cam_bridge::active()
    }
    pub(super) fn client_free_cam_aim(&self) -> Option<([f64; 3], [f64; 3])> {
        let state = self.client_free_cam_state(1.);
        if state["active"] != true {
            return None;
        }
        let interact = state["cameraInteract"] == true;
        let p = if interact {
            &state["currentPosition"]
        } else {
            return Some((
                self.eye_position()?,
                super::client_rotation_direction(
                    state["serverRotation"][0].as_f64()? as f32,
                    state["serverRotation"][1].as_f64()? as f32,
                ),
            ));
        };
        Some((
            [p[0].as_f64()?, p[1].as_f64()?, p[2].as_f64()?],
            super::pick_view_vector(
                state["currentRotation"][0].as_f64()? as f32,
                state["currentRotation"][1].as_f64()? as f32,
            ),
        ))
    }
    pub(super) fn client_free_cam_camera(&self, partial: f32) -> Option<([f64; 3], [f32; 2])> {
        if self.camera_entity().is_some() {
            return None;
        }
        let s = self.client_free_cam_state(partial);
        if s["active"] != true {
            return None;
        }
        Some((
            [
                s["position"][0].as_f64()?,
                s["position"][1].as_f64()?,
                s["position"][2].as_f64()?,
            ],
            [
                s["rotation"][0].as_f64()? as f32,
                s["rotation"][1].as_f64()? as f32,
            ],
        ))
    }
    pub(super) fn free_cam_context(&self) -> Option<String> {
        if !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some() {
            return None;
        }
        let p = self
            .player
            .as_ref()
            .filter(|p| p.native_client_queries_allowed())?;
        use winit::keyboard::KeyCode;
        let keys = [
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::Space,
            KeyCode::ShiftLeft,
            KeyCode::ControlLeft,
        ]
        .map(|k| self.keys.raw(&k));
        let block = self.pick().map(|h| h.point);
        let crosshair = self
            .mob_hit()
            .and_then(|(index, distance, offset)| {
                if block.is_some_and(|point| {
                    self.aim().is_some_and(|(eye, _)| {
                        point
                            .iter()
                            .zip(eye)
                            .map(|(a, b)| (a - b).powi(2))
                            .sum::<f64>()
                            < distance * distance
                    })
                }) {
                    return None;
                }
                let m = self.mobs.get(index)?;
                Some([
                    m.position.x + offset[0],
                    m.position.y + offset[1],
                    m.position.z + offset[2],
                ])
            })
            .or(block)
            .or_else(|| {
                self.aim().map(|(p, d)| {
                    std::array::from_fn(|i| p[i] + d[i] * self.block_reach.max(self.entity_reach))
                })
            });
        Some(json!({"phase":"tick","eye":self.eye_position()?,"playerRotation":[self.yaw,self.pitch],
            "keys":keys,"grabbed":self.captured||self.agent_pointer,"liquid":p.fluids.was_touching_water||p.is_in_lava(),
            "crosshair":crosshair}).to_string())
    }
    pub(super) fn free_cam_trace(
        &self,
        position: [f64; 3],
        rotation: [f32; 2],
    ) -> Option<[f64; 3]> {
        use horizon_core::geometry::Vec3;
        let direction = super::pick_view_vector(rotation[0], rotation[1]);
        let end: [f64; 3] = std::array::from_fn(|i| position[i] + direction[i] * 200.);
        let owner = self.remote.as_ref().map_or(0, |r| r.entity_id);
        super::free_cam_ray::clip(
            self,
            Vec3::new(position[0], position[1], position[2]),
            Vec3::new(end[0], end[1], end[2]),
            owner,
            true,
        )
        .map(|(_, p)| [p.x, p.y, p.z])
    }
    pub(super) fn client_free_cam_middle(&self) {
        let s = self.client_free_cam_state(1.);
        if s["active"] != true || s["midClick"] != true {
            return;
        }
        let Some((p, r)) = self.client_free_cam_camera(1.) else {
            return;
        };
        if let Some(target) = self.free_cam_trace(p, r) {
            self.client_free_cam_event(json!({"phase":"middle","target":target}));
        }
    }
    pub(super) fn client_free_cam_keep_sneaking(&self) -> bool {
        let s = self.client_free_cam_state(1.);
        s["active"] == true && s["keepSneaking"] == true
    }
    pub(super) fn client_free_cam_hide_crosshair(&self) -> bool {
        let flags = if self.client_free_cam_active() {
            horizon_client::free_cam_bridge::flags()
        } else {
            0
        };
        flags & 1 != 0 && flags & 2 == 0
    }
}
