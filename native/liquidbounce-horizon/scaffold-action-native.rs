// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleScaffold's real player facts, direct placement predicate and dispatch.
use super::{BuildingObstruction, Game, GameLevel, Hand, Hit};
use horizon_core::{
    geometry::{Direction, Vec3},
    position::BlockPos,
};
use horizon_world::{
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_obstruction::EntityIdentity,
    level::PlaceContext,
    registry::BlockStateId,
};
use serde_json::{Value, json};

fn hit(value: &Value) -> Option<Hit> {
    Some(Hit {
        pos: serde_json::from_value(value["block"].clone()).ok()?,
        face: serde_json::from_value(value["face"].clone()).ok()?,
        point: serde_json::from_value(value["point"].clone()).ok()?,
    })
}
fn inside(request: &Value) -> Option<bool> {
    request["hit"]["inside"]
        .as_bool()
        .or_else(|| request["inside"].as_bool())
}
fn hand(value: &Value) -> Option<Hand> {
    match value.as_str()? {
        "Main" => Some(Hand::Main),
        "Off" => Some(Hand::Off),
        _ => None,
    }
}
fn unobstructed(
    level: &GameLevel<'_>,
    position: BlockPos,
    state: BlockStateId,
    context: &CollisionContext<'_>,
    subject: Option<horizon_client::collisions::PowderSnowSubject>,
) -> bool {
    let mut scene = horizon_client::collisions::SceneCollisions {
        world: &*level.world,
        binding: level.collision,
        subject,
    };
    let shape = level
        .collision
        .collision_shape_with_context(&mut scene, state, position, context)
        .expect("Scaffold getPlacementState collision shape")
        .moved(Vec3::new(
            f64::from(position.x),
            f64::from(position.y),
            f64::from(position.z),
        ));
    horizon_world::entity_obstruction::is_unobstructed(
        &mut BuildingObstruction(level.building_blocker_boxes()),
        None,
        &shape,
    )
    .expect("Scaffold getPlacementState entity obstruction")
}
impl Game<'_> {
    pub(super) fn scaffold_action_query(&mut self, text: &str) -> Option<String> {
        let request = serde_json::from_str::<Value>(text).ok()?;
        match request["op"].as_str()? {
            "facts" => {
                let p = self.player.as_ref()?;
                let keys = horizon_client::reverse_step_bridge::keys();
                Some(json!({"position":[p.position.x,p.position.y,p.position.z],"eye":self.eye_position()?,
                    "blockY":p.position.y.floor() as i32,"velocityY":p.velocity.y,"onGround":p.ground.on_ground,
                    "horizontalCollision":p.collision.horizontal,"moving":p.move_vector != [0.0,0.0],
                    "inputJump":keys[4] || p.was_auto_jump,"physicalJump":self.keys.held(horizon_client::actions::Action::Jump),"yaw":self.yaw,"pitch":self.pitch}).to_string())
            }
            "checkPlacement" => {
                let hit = hit(&request["hit"])?;
                let hand = hand(&request["hand"])?;
                let stack = self.with_hand(hand, |g| g.held().cloned())?;
                let player = self.player.as_ref()?;
                let face = Direction::ALL
                    .into_iter()
                    .find(|d| d.normal() == hit.face)?;
                let context = PlaceContext {
                    clicked_pos: BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]),
                    face,
                    hit: Vec3::new(hit.point[0], hit.point[1], hit.point[2]),
                    inside: inside(&request)?,
                    yaw: player.yaw,
                    pitch: player.pitch,
                    sneaking: player.shift_key_down,
                    item: stack.item.to_string(),
                    can_use_game_master_blocks: self.can_use_game_master_blocks(),
                };
                // getPlacementState uses placementContext(player) for ordinary
                // BlockItem, but StandingAndWallBlockItem uses empty(). Capture
                // the effective main hand independently of the hand being used.
                let main = self
                    .inventory
                    .get(self.client_hotbar_slot())
                    .and_then(Option::as_ref)
                    .cloned();
                let collision = CollisionContext::placement(Some(CollisionEntity {
                    identity: EntityIdentity(
                        self.remote
                            .as_ref()
                            .map_or(0, |r| r.entity_id as u32 as u64),
                    ),
                    kind: EntityContextKind::Player,
                    y: player.position.y,
                    descending: player.shift_key_down,
                    main_hand: Some(main.as_ref().map_or(CollisionItem::EMPTY, |s| {
                        CollisionItem {
                            item: &s.item,
                            count: s.count as i32,
                        }
                    })),
                }));
                let mut subject = player.powder_snow_subject();
                subject.leather_boots = self.armor[3]
                    .as_ref()
                    .is_some_and(|s| s.count > 0 && &*s.item == "minecraft:leather_boots");
                let can = self.with_level(|b, l, t| {
                    b.liquidbounce_scaffold_placement_state(
                        l,
                        t,
                        &context,
                        |level, pos, state, placement| {
                            if placement {
                                unobstructed(level, pos, state, &collision, Some(subject))
                            } else {
                                unobstructed(
                                    level,
                                    pos,
                                    state,
                                    &CollisionContext::empty(false),
                                    None,
                                )
                            }
                        },
                    )
                    .is_some()
                });
                Some(json!(can).to_string())
            }
            "execute" => {
                let hit = hit(&request["hit"])?;
                let rotation = serde_json::from_value(request["rotation"].clone()).ok()?;
                let hand = hand(&request["hand"])?;
                let swing = request["swing"].as_str()?;
                let simulated = request["simulated"].as_bool()?;
                let placed: [i32; 3] = if simulated {
                    std::array::from_fn(|i| hit.pos[i].wrapping_add(hit.face[i]))
                } else {
                    serde_json::from_value(request["placed"].clone()).ok()?
                };
                let accepted = self.client_scaffold_do_placement(
                    hit,
                    inside(&request)?,
                    rotation,
                    hand,
                    swing,
                    |g, _| {
                        g.native_client_scaffold_success(placed, simulated);
                        true
                    },
                );
                Some(json!(accepted).to_string())
            }
            "sendRotation" => {
                let [yaw, pitch]: [f32; 2] =
                    serde_json::from_value(request["rotation"].clone()).ok()?;
                let p = self.player.as_ref()?;
                let at = p.position;
                let flags = u8::from(p.ground.on_ground) | (u8::from(p.collision.horizontal) << 1);
                self.send(
                    horizon_protocol::packets::play::Serverbound::MovePlayerPosRot(
                        horizon_protocol::packets::play::MovePlayerPosRot {
                            x: at.x,
                            y: at.y,
                            z: at.z,
                            yaw,
                            pitch,
                            flags,
                        },
                    ),
                );
                Some("true".into())
            }
            _ => None,
        }
    }
}
