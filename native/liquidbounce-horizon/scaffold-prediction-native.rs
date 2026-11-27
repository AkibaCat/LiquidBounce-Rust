// SPDX-License-Identifier: GPL-3.0-or-later
//! Real edge, next-body and world-collision queries for Scaffold prediction.
use super::{Game, scaffold_planner_rules::Line, scaffold_prediction_rules as rules};
use horizon_core::geometry::Vec3;
use horizon_world::{
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
    world_movement::{self, CollisionCheckRequest},
};
use serde_json::{Value, json};

fn array(value: Vec3) -> [f64; 3] {
    [value.x, value.y, value.z]
}
fn vector(value: [f64; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}
fn line(value: &Value) -> Option<Line> {
    Some(Line {
        position: serde_json::from_value(value["position"].clone()).ok()?,
        direction: serde_json::from_value(value["direction"].clone()).ok()?,
    })
}
fn settings(value: &Value) -> Option<rules::Settings> {
    Some(rules::Settings {
        bootstrap_backoff: value["bootstrapBackoff"].as_f64()? as f32,
        cutoff_distance: value["cutoffDistance"].as_f64()? as f32,
        warmup_placements: i32::try_from(value["warmupPlacements"].as_i64()?).ok()?,
    })
}
fn angle(line: Line) -> f32 {
    horizon_core::jmath::atan2(line.direction[2], line.direction[0]) as f32
}
fn rotate(point: [f64; 3], angle: f32) -> [f64; 3] {
    array(vector(point).rotate_y(angle))
}

impl Game<'_> {
    fn scaffold_falloff(&self, line: Line) -> Option<[f64; 3]> {
        let player = self.player.as_ref()?;
        let nearest = line.nearest(array(player.position));
        let from = Vec3::new(nearest[0], nearest[1] - 0.1, nearest[2]);
        let delta = rules::with_length(line.direction, 3.0);
        let to = from.add_vector(vector(delta));
        let edge = super::scaffold_edge::find(self, from, to, 0.5)?;
        Some([edge.x, player.position.y, edge.z])
    }
    fn scaffold_would_fall(&mut self, position: Vec3) -> Option<bool> {
        let player = self.player.as_mut()?;
        let step = player.attribute("minecraft:step_height").ok()?;
        let bounds = player
            .dimensions
            .bounds(position)
            .inflate(Vec3::new(-0.05, 0.0, -0.05))
            .move_by(Vec3::new(0.0, player.fall_distance - step, 0.0));
        let entities = self.player_entity_collisions();
        let player = self.player.as_ref()?;
        let held = self
            .inventory
            .get(self.client_hotbar_slot())
            .and_then(Option::as_ref);
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(0),
                kind: EntityContextKind::Player,
                y: player.position.y,
                descending: player.shift_key_down,
                main_hand: Some(held.map_or(CollisionItem::EMPTY, |s| CollisionItem {
                    item: &s.item,
                    count: s.count as i32,
                })),
            },
            false,
        );
        let mut subject = player.powder_snow_subject();
        subject.leather_boots = self.armor[3]
            .as_ref()
            .is_some_and(|s| s.count > 0 && &*s.item == "minecraft:leather_boots");
        let scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: Some(subject),
        };
        let mut collision = horizon_client::collisions::EntityCollisions {
            blocks: scene,
            entities: &entities,
            border: player
                .border
                .as_ref()
                .map(|border| (border, [player.position.x, player.position.z])),
        };
        Some(
            world_movement::no_world_collision(
                &mut collision,
                CollisionCheckRequest {
                    entity: EntityIdentity(0),
                    context,
                    bounds,
                    candidate_budget: 1_000_000,
                },
            )
            .expect("Scaffold wouldBeCloseToFallOff"),
        )
    }
    fn scaffold_close_to_edge(&mut self, distance: f64, directional:Option<[bool;4]>) -> Option<bool> {
        let player = self.player.as_ref()?;
        let position = player.position;
        let yaw = player.yaw;
        let keys = horizon_client::reverse_step_bridge::keys();
        let directional=directional.unwrap_or([keys[0],keys[1],keys[2],keys[3]]);
        let input = horizon_client::sprint::Keys {
            forward: directional[0],
            backward: directional[1],
            left: directional[2],
            right: directional[3],
            jump: false,
            shift: false,
            sprint: player.sprinting || keys[6],
            analog: None,
        };
        let mut next = None;
        super::falling_forecast::visit_with_keys(self, 1, 0.0, input, |body, _| {
            next = Some((body.position, body.velocity));
            true
        })
        .ok()?;
        let (predicted, velocity) = next?;
        let horizontal = velocity.horizontal();
        let direction = if horizontal.length_squared() > 0.003 * 0.003 {
            horizontal.normalize()
        } else {
            Vec3::direction_from_rotation(
                0.0,
                super::scaffold_planner_rules::movement_angle(
                    yaw,
                    directional,
                ),
            )
        };
        let from = position.add_vector(Vec3::new(0.0, -0.1, 0.0));
        let to = Vec3::new(
            distance.mul_add(direction.x, from.x),
            distance.mul_add(direction.y, from.y),
            distance.mul_add(direction.z, from.z),
        );
        if super::scaffold_edge::find(self, from, to, 0.5).is_some() {
            return Some(true);
        }
        if self.scaffold_would_fall(position)? {
            return Some(true);
        }
        self.scaffold_would_fall(predicted.add_vector(horizontal))
    }
    pub(super) fn scaffold_prediction_query(&mut self, text: &str) -> Option<String> {
        let request: Value = serde_json::from_str(text).ok()?;
        let result = match request["op"].as_str()? {
            "reset" => {
                self.scaffold_prediction_state.reset();
                Value::Bool(true)
            }
            "average" => json!(self.scaffold_prediction_state.average()),
            "falloff" => json!(self.scaffold_falloff(line(&request["line"])?)),
            "close" => json!(self.scaffold_close_to_edge(request["distance"].as_f64()?,serde_json::from_value(request["input"].clone()).ok())?),
            "record" => {
                let enabled = request["enabled"].as_bool()?;
                let optimal = line(&request["line"]);
                let falloff = if request["falloff"].is_null() {
                    None
                } else {
                    Some(serde_json::from_value(request["falloff"].clone()).ok()?)
                };
                let position = array(self.player.as_ref()?.position);
                self.scaffold_prediction_state.record(
                    enabled,
                    position,
                    falloff,
                    optimal.map(angle),
                    rotate,
                );
                Value::Bool(true)
            }
            "predict" => {
                if !request["enabled"].as_bool()? || request["line"].is_null() {
                    return Some("null".into());
                }
                let optimal = line(&request["line"])?;
                let settings = settings(&request["settings"])?;
                if self.scaffold_close_to_edge(f64::from(settings.cutoff_distance),None)? {
                    return Some("null".into());
                }
                let Some(falloff) = self.scaffold_falloff(optimal) else {
                    return Some("null".into());
                };
                let player = array(self.player.as_ref()?.position);
                let support = self
                    .scaffold_planner_state
                    .support
                    .map(|s| [s.offset_x, s.offset_z]);
                json!(self.scaffold_prediction_state.predict(
                    player,
                    falloff,
                    angle(optimal),
                    support,
                    settings,
                    rotate
                ))
            }
            _ => return None,
        };
        Some(result.to_string())
    }
}
