// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual canStep block collisions and withStrafe body effects.
use super::*;
use horizon_core::geometry::Vec3;
use horizon_world::{
    block_collision_scan::BlockCollisionScan,
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
};
impl Game<'_> {
    fn step_can(&self, height: f64) -> bool {
        let Some(p) = self.player.as_ref() else {
            return false;
        };
        if !p.collision.horizontal || p.shift_key_down || !p.ground.on_ground {
            return false;
        }
        let angle = f64::from(p.yaw).to_radians();
        let offset = p.bounds().move_by(Vec3::new(
            -horizon_core::jmath::sin(angle) * 0.1,
            0.,
            horizon_core::jmath::cos(angle) * 0.1,
        ));
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(0),
                kind: EntityContextKind::Player,
                y: p.position.y,
                descending: p.shift_key_down,
                main_hand: Some(CollisionItem::EMPTY),
            },
            false,
        );
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: Some(p.powder_snow_subject()),
        };
        let clear = !BlockCollisionScan::any_collision(
            &mut scene,
            context,
            offset.move_by(Vec3::new(0., height, 0.)),
            false,
            1_000_000,
        )
        .expect("Step raised block collision");
        clear
            && BlockCollisionScan::any_collision(&mut scene, context, offset, false, 1_000_000)
                .expect("Step forward block collision")
    }
    pub(super) fn step_input(&mut self, jump: &mut bool) {
        if !self.native_client_slot_owner() {
            return;
        }
        let can_one = self.step_can(1.);
        let can_one_quarter = self.step_can(1.25);
        let can_one_half = self.step_can(1.5);
        let mut effects = String::new();
        self.native_client_step_input(can_one, can_one_quarter, can_one_half, jump, &mut effects);
        self.step_effects(&effects);
    }
    pub(super) fn step_effects(&mut self, value: &str) {
        if !self.native_client_slot_owner() {
            return;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(value) else {
            return;
        };
        let Some(p) = self.player.as_mut() else {
            return;
        };
        if let Some(y) = v["setY"].as_f64() {
            p.velocity.y = y;
        }
        p.velocity.y += v["addY"].as_f64().unwrap_or(0.);
        if let Some(speed) = v["strafe"].as_f64() {
            let k = horizon_client::reverse_step_bridge::keys();
            let forward = i32::from(k[0]) - i32::from(k[1]);
            let side = i32::from(k[2]) - i32::from(k[3]);
            if forward == 0 && side == 0 {
                p.velocity.x = 0.;
                p.velocity.z = 0.;
            } else {
                let mut yaw = p.yaw;
                if forward < 0 {
                    yaw += 180.;
                }
                let factor = if forward < 0 {
                    -0.5
                } else if forward > 0 {
                    0.5
                } else {
                    1.
                };
                if side > 0 {
                    yaw -= 90. * factor;
                } else if side < 0 {
                    yaw += 90. * factor;
                }
                let radians = f64::from(yaw).to_radians();
                p.velocity.x = -horizon_core::jmath::sin(radians) * speed;
                p.velocity.z = horizon_core::jmath::cos(radians) * speed;
            }
        }
        if v["subtractFraction"] == true {
            p.velocity.y -= p.position.y % 1.;
        }
        if v["ground"] == true {
            p.ground.on_ground = true;
        }
    }
}
