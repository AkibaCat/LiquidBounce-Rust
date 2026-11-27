// SPDX-License-Identifier: GPL-3.0-or-later
use super::entity::{Entity as _, with_entity, with_entity_mut};
use super::{Game, InteractTarget, Screen};
use horizon_client::chat::ChatSource;
use horizon_client::{actions::Action, first_person::Hand, text::Text};
use horizon_core::geometry::{Axis, Vec3};
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn vehicle_control_id(&self) -> Option<u32> {
        let id = self.riding?;
        if self.boats.iter().any(|b| b.id == id) && self.controls_boat(id) {
            return Some(id);
        }
        self.vehicle_control_mount_id()
    }
    pub(super) fn vehicle_control_state(&mut self, target: i32) -> Option<Value> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let reach=self.interaction_range("minecraft:entity_interaction_range",3.,5.);
        let player = self.player.as_ref()?;
        let controlled = self.vehicle_control_id();
        let vehicle = controlled.and_then(|id| self.native_walk_entity(id));
        let ground = vehicle.is_some_and(|h| with_entity!(self,h,e=>e.on_ground()));
        let water = controlled.is_some_and(|id| {
            self.boats.iter().find(|b| b.id == id).map_or_else(
                || {
                    self.mobs
                        .iter()
                        .find(|m| m.id == id)
                        .is_some_and(|m| m.fluids.in_water)
                },
                |b| b.vehicle_control_in_water,
            )
        });
        let yaw = vehicle.map_or(self.yaw, |h| with_entity!(self,h,e=>e.rotation()[0]));
        let candidate = (target >= 0)
            .then(|| self.native_walk_entity(target as u32))
            .flatten();
        let exists = candidate.is_some_and(|h| with_entity!(self,h,e=>!e.is_removed()));
        // boxedDistanceTo uses the player's eye to the target AABB, not feet-to-centre.
        let distance = candidate.map(|h| {
            let b = with_entity!(self,h,e=>e.bounds());
            let eye = self.eye_position().unwrap_or([
                player.position.x,
                player.position.y,
                player.position.z,
            ]);
            let axes = [Axis::X, Axis::Y, Axis::Z];
            axes.into_iter()
                .enumerate()
                .map(|(i, a)| {
                    let d = eye[i] - eye[i].clamp(b.min(a), b.max(a));
                    d * d
                })
                .sum::<f64>()
                .sqrt()
        });
        Some(
            json!({"controlled":controlled,"passenger":self.riding.is_some(),"ground":ground,"water":water,"yaw":yaw,"playerYaw":self.yaw,
            "sprint":self.keys.held(Action::Sprint),"jump":self.keys.held(Action::Jump),"sneak":self.keys.held(Action::Sneak),"use":self.client_use_key_down(),
            "exists":exists,"reach":distance.is_some_and(|d|d<=reach)}),
        )
    }
    pub(super) fn vehicle_control_apply(&mut self, request: &str) -> bool {
        if !self.native_client_slot_owner() {
            return false;
        }
        let Ok(v) = serde_json::from_str::<Value>(request) else {
            return false;
        };
        if let Some(notice) = v["notice"].as_str() {
            self.vehicle_control_chat(notice);
        }
        if let Some(a) = v["velocity"].as_array() {
            let Some(id) = self.vehicle_control_id() else {
                return false;
            };
            let Some(h) = self.native_walk_entity(id) else {
                return false;
            };
            let xyz =
                std::array::from_fn::<_, 3, _>(|i| a.get(i).and_then(Value::as_f64).unwrap_or(0.));
            with_entity_mut!(self,h,e=>*e.velocity_mut()=Vec3::new(xyz[0],xyz[1],xyz[2]));
            if v["mouse"] == true {
                if let Some(b) = self.boats.iter_mut().find(|b| b.id == id) {
                    b.yaw = self.yaw;
                    b.previous_yaw = self.yaw;
                }
                if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
                    m.yaw = self.yaw;
                    m.previous_yaw = self.yaw;
                }
            }
        }
        if v["detach"] == true {
            // Entity.stopRiding only removes the local rider link; it does not
            // use a dismount-location calculation or synthesize another packet.
            self.riding = None;
        }
        if let Some(id) = v["mount"].as_i64() {
            if self.native_walk_entity(id as u32).is_none() {
                return false;
            }
            if self.remote.is_some() {
                self.riding = Some(id as u32);
            } else {
                self.start_riding(id as u32);
            }
        }
        if let Some(id) = v["interact"].as_i64() {
            let Some(h) = self.native_walk_entity(id as u32) else {
                return false;
            };
            let pos = with_entity!(self,h,e=>e.position());
            if !self.border.contains(pos.x.floor(), pos.z.floor()) {
                return false;
            }
            self.native_client_ensure_carried();
            if self.remote.is_some() {
                self.vehicle_control_remote_interact(id as i32);
            } else {
                let _ = self.with_hand(Hand::Main, |g| {
                    g.interact_entity(Some(InteractTarget {
                        id: id as i32,
                        location: [0.; 3],
                    }))
                });
            }
        }
        true
    }
    fn vehicle_control_chat(&mut self, key: &str) {
        let lang = horizon_client::lang::code()
            .to_ascii_lowercase()
            .replace('-', "_");
        let zh = match lang.as_str() {
            "zh_cn" | "zh_sg" | "zh_hans" => 1,
            "zh_tw" | "zh_hk" | "zh_mo" | "zh_hant" => 2,
            _ => 0,
        };
        let message = match (key, zh) {
            ("quit", 1) => "要从载具上离开，降落后按下潜行键。",
            ("quit", 2) => "要從載具上離開，降落後按下潛行鍵。",
            ("quit", _) => "To exit the vehicle, land on the ground and press the sneak button.",
            ("far", 1) => "载具太远，无法触及。",
            ("far", 2) => "載具太遠，無法觸及。",
            ("far", _) => "The vehicle is too far away to be reached.",
            (_, 1) => "载具消失了。",
            (_, 2) => "載具消失了。",
            (_, _) => "The vehicle disappeared.",
        };
        let focused = self.screen == Screen::Chat;
        let ticks = self.visual_ticks();
        self.ui.chat.add(
            &self.gui.atlas,
            Text::colored(message, 0xffaa00),
            None,
            ChatSource::Client,
            ticks,
            focused,
        );
    }
}
