// SPDX-License-Identifier: GPL-3.0-or-later
//! Real feature facts and mutations. Decisions remain in the source runtime.
use super::Game;
use horizon_core::{geometry::Vec3, position::BlockPos};
use horizon_world::{
    block_collision_scan::BlockCollisionScan,
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
    level::data::state_flags,
    living_motion,
};
use serde_json::{Value, json};
fn wrap(mut yaw: f32) -> f32 {
    yaw %= 360.;
    if yaw >= 180. {
        yaw -= 360.;
    }
    if yaw < -180. {
        yaw += 360.;
    }
    yaw
}

impl Game<'_> {
    fn scaffold_shape_nonempty(&self, block: BlockPos) -> bool {
        let state = self.world.block(block.x, block.y, block.z);
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        !self
            .collision
            .collision_shape_with_context(&mut scene, state, block, &CollisionContext::empty(false))
            .expect("Scaffold block collision shape")
            .is_empty()
    }
    pub(super) fn scaffold_with_motion<R>(
        &mut self,
        run: impl FnOnce(&mut horizon_client::motion::MotionWorld<'_, '_, '_>) -> R,
    ) -> Option<R> {
        let entities = self.player_entity_collisions();
        let p = self.player.as_mut()?;
        let mut motion = horizon_client::motion::MotionWorld {
            entities: &entities,
            player: p,
            scene: &mut self.world,
            assets: &self.pack.assets,
            collision_binding: self.collision,
            resources: self.motion,
            health: &self.health_resources,
        };
        Some(run(&mut motion))
    }
    fn scaffold_block_below(&self) -> Option<bool> {
        let p = self.player.as_ref()?;
        let held = self
            .inventory
            .get(self.client_hotbar_slot())
            .and_then(Option::as_ref);
        let identity = EntityIdentity(
            self.remote
                .as_ref()
                .map_or(0, |r| r.entity_id as u32 as u64),
        );
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity,
                kind: EntityContextKind::Player,
                y: p.position.y,
                descending: p.shift_key_down,
                main_hand: Some(held.map_or(CollisionItem::EMPTY, |s| CollisionItem {
                    item: &s.item,
                    count: s.count as i32,
                })),
            },
            false,
        );
        let mut subject = p.powder_snow_subject();
        subject.leather_boots = self.armor[3]
            .as_ref()
            .is_some_and(|s| s.count > 0 && &*s.item == "minecraft:leather_boots");
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: Some(subject),
        };
        let bounds = p
            .bounds()
            .inflate(Vec3::new(0.5, 0.0, 0.5))
            .move_by(Vec3::new(0.0, -1.05, 0.0));
        Some(
            BlockCollisionScan::any_collision(&mut scene, context, bounds, false, 1_000_000)
                .expect("Scaffold real block collisions below"),
        )
    }
    pub(super) fn scaffold_features_query(&mut self, text: &str) -> Option<String> {
        let request: Value = serde_json::from_str(text).ok()?;
        match request["op"].as_str()? {
            "safeWalkInput" => {
                let input = serde_json::from_value(request["input"].clone()).ok()?;
                Some(
                    json!(super::edge_assistance::scaffold_input(
                        self, &request, input
                    ))
                    .to_string(),
                )
            }
            "outline" => {
                let pos = serde_json::from_value(request["block"].clone()).ok()?;
                let boxes = super::block_placement::outline(self, pos, true);
                let bounds = if boxes.is_empty() {
                    [0., 0., 0., 1., 1., 1.]
                } else {
                    std::array::from_fn(|i| {
                        if i < 3 {
                            boxes.iter().map(|b| b[i]).fold(f64::INFINITY, f64::min)
                        } else {
                            boxes.iter().map(|b| b[i]).fold(f64::NEG_INFINITY, f64::max)
                        }
                    })
                };
                Some(json!(bounds).to_string())
            }
            "strafe" => {
                let speed = request["speed"].as_f64()?;
                let k = horizon_client::reverse_step_bridge::keys();
                let input = [k[0], k[1], k[2], k[3]];
                let p = self.player.as_mut()?;
                if super::scaffold_feature_rules::moving(input) {
                    let yaw = super::scaffold_planner_rules::movement_angle(p.yaw, input);
                    let a = f64::from(yaw).to_radians();
                    p.velocity.x = -horizon_core::jmath::sin(a) * speed;
                    p.velocity.z = horizon_core::jmath::cos(a) * speed;
                } else {
                    p.velocity.x = 0.;
                    p.velocity.z = 0.;
                }
                Some("true".into())
            }
            "stabilize" => {
                let input = serde_json::from_value(request["input"].clone()).ok()?;
                if request["line"].is_null() {
                    return Some(json!(input).to_string());
                }
                let line = super::scaffold_planner_rules::Line {
                    position: serde_json::from_value(request["line"]["position"].clone()).ok()?,
                    direction: serde_json::from_value(request["line"]["direction"].clone()).ok()?,
                };
                let p = self.player.as_ref()?;
                let at = [p.position.x, p.position.y, p.position.z];
                let nearest = line.nearest(at);
                let delta = std::array::from_fn(|i| nearest[i] - at[i]);
                let degrees = wrap(
                    (horizon_core::jmath::atan2(delta[2], delta[0]).to_degrees() as f32 - 90.)
                        - wrap(p.yaw),
                );
                let correction = [
                    degrees > -90. && degrees < 90.,
                    degrees < -90. || degrees > 90.,
                    degrees > -180. && degrees < 0.,
                    degrees > 0. && degrees < 180.,
                ];
                Some(
                    json!(super::scaffold_feature_rules::stabilize(
                        input,
                        delta,
                        [p.velocity.x, 0., p.velocity.z],
                        correction
                    ))
                    .to_string(),
                )
            }
            "facts" => {
                let p = self.player.as_ref()?;
                let at = p.position;
                let pos = BlockPos::new(
                    at.x.floor() as i32,
                    at.y.floor() as i32,
                    at.z.floor() as i32,
                );
                let state = self.world.block(pos.x, pos.y - 1, pos.z);
                let below_air = self.res.behaviours().facts(state).flags & state_flags::AIR != 0;
                let keys = horizon_client::reverse_step_bridge::keys();
                let speed = self.potions.effects.get("speed").is_some();
                let down = self.keys.held(horizon_client::actions::Action::Sneak);
                let jump = self.keys.held(horizon_client::actions::Action::Jump);
                let head = self.scaffold_shape_nonempty(BlockPos::new(pos.x, pos.y + 2, pos.z));
                let block_below = self.scaffold_block_below()?;
                let down_support = self.with_level(|b, l, t| {
                    b.liquidbounce_placement_holds_solid(
                        l,
                        t,
                        BlockPos::new(pos.x, pos.y - 2, pos.z),
                    )
                });
                let power = self.scaffold_with_motion(|motion| {
                    living_motion::jump_power(motion, 1.0).expect("Scaffold real jump power")
                })?;
                let p = self.player.as_ref()?;
                Some(json!({"position":[at.x,at.y,at.z],"velocity":[p.velocity.x,p.velocity.y,p.velocity.z],"yaw":p.yaw,"pitch":p.pitch,
                    "onGround":p.ground.on_ground,"flying":p.flying,"shift":p.shift_key_down,"physicalSneak":down,"physicalJump":jump,
                    "moving":p.move_vector!=[0.0,0.0],"tick":p.health.tick_count,"hasSpeed":speed,"belowAir":below_air,"headHit":head,
                    "downSupport":down_support,"blockBelow":block_below,"jumpPower":power,"input":[keys[0],keys[1],keys[2],keys[3]],"inputJump":keys[4]}).to_string())
            }
            "motion" => {
                let p = self.player.as_mut()?;
                if !request["y"].is_null() {
                    p.position.y = request["y"].as_f64()?;
                }
                if !request["velocity"].is_null() {
                    let v: [f64; 3] = serde_json::from_value(request["velocity"].clone()).ok()?;
                    p.velocity = Vec3::new(v[0], v[1], v[2]);
                }
                if !request["velocityY"].is_null() {
                    p.velocity.y = request["velocityY"].as_f64()?;
                }
                if !request["horizontalMultiplier"].is_null() {
                    let m = request["horizontalMultiplier"].as_f64()?;
                    p.velocity.x *= m;
                    p.velocity.z *= m;
                }
                if request["awardJump"] == true {
                    p.events.push(horizon_client::motion::MotionEvent::Health(
                        horizon_client::health::HealthEvent::Statistic {
                            name: "jump",
                            amount: 1,
                        },
                    ));
                }
                Some("true".into())
            }
            "jump" => {
                self.scaffold_with_motion(|motion| {
                    motion.jump_from_ground().expect("Scaffold HeadHitter jump")
                })?;
                Some("true".into())
            }
            "rawCrosshair" => {
                let r = serde_json::from_value(request["rotation"].clone()).ok()?;
                let range = self
                    .interaction_range("minecraft:block_interaction_range", 4.5, 5.0)
                    .max(self.interaction_range("minecraft:entity_interaction_range", 3.0, 5.0));
                let c = super::block_placement::trace_world_click(self, r, range)?;
                Some(json!({"block":c.hit.pos,"face":c.hit.face,"point":c.hit.point,"inside":c.inside}).to_string())
            }
            _ => None,
        }
    }
}
