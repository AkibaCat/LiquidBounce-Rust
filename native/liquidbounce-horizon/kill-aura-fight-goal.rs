// SPDX-License-Identifier: GPL-3.0-or-later
// Appended to game::kill_aura; original KillAuraFightBot at ae0eb396.
mod fight_goal {
    use super::Game;
    use horizon_core::geometry::Vec3;
    use horizon_world::{
        block_collision_scan::BlockCollisionScan,
        collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
        entity_obstruction::EntityIdentity,
    };
    use serde_json::Value;
    use std::cmp::Ordering;

    type Point = [f64; 3];
    #[derive(Clone, Copy)]
    struct Opponent {
        position: Point,
        yaw: f32,
        range: f32,
        required_yaw: f32,
        out_of_distance: bool,
        out_of_danger: bool,
    }
    #[derive(Clone, Copy)]
    struct Settings {
        dangerous_yaw: f32,
        runaway: bool,
        leader: bool,
        leader_radius: f32,
    }
    fn point(p: Vec3) -> Point {
        [p.x, p.y, p.z]
    }
    fn distance_squared(a: Point, b: Point) -> f64 {
        let x = a[0] - b[0];
        let y = a[1] - b[1];
        let z = a[2] - b[2];
        x * x + y * y + z * z
    }
    fn wrap(mut angle: f32) -> f32 {
        angle %= 360.;
        if angle >= 180. {
            angle -= 360.;
        }
        if angle < -180. {
            angle += 360.;
        }
        angle
    }
    fn offset(at: Point, scale: f32, direction: Point) -> Point {
        let scale = f64::from(scale);
        std::array::from_fn(|i| scale.mul_add(direction[i], at[i]))
    }
    fn compare(a: f64, b: f64) -> Ordering {
        match (a.is_nan(), b.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => a.total_cmp(&b),
        }
    }
    fn closest(points: impl IntoIterator<Item = Point>, player: Point) -> Option<Point> {
        let mut nearest = None;
        for at in points {
            if nearest.is_none_or(|old| {
                compare(distance_squared(at, player), distance_squared(old, player))
                    == Ordering::Less
            }) {
                nearest = Some(at);
            }
        }
        nearest
    }
    // Pure source selection. Native calls supply vanilla directions and real
    // contextual block collisions; the batch harness supplies exact fixtures.
    fn select(
        player: Point,
        opponent: Option<Opponent>,
        leader: Option<Point>,
        settings: Settings,
        will_click: bool,
        direction: impl Fn(f32) -> Point,
        mut collides: impl FnMut(Point) -> bool,
    ) -> Option<(Point, bool)> {
        let goal = if let Some(leader) = leader {
            closest(
                (-180..=180)
                    .step_by(45)
                    .map(|yaw| offset(leader, settings.leader_radius, direction(yaw as f32))),
                player,
            )
            .unwrap_or(leader)
        } else {
            let target = opponent?;
            if settings.runaway && !target.out_of_distance && !will_click {
                offset(player, target.range, direction(target.required_yaw))
            } else {
                let looking_at = offset(target.position, target.range, direction(target.yaw));
                let mut candidates = Vec::new();
                for yaw in (-180..=180).step_by(10) {
                    let yaw = yaw as f32;
                    let at = offset(target.position, target.range, direction(yaw));
                    // Source queries collision before rejecting dangerous angles.
                    if collides(at) {
                        continue;
                    }
                    if wrap(target.yaw - yaw).abs() <= settings.dangerous_yaw {
                        continue;
                    }
                    candidates.push(at);
                }
                // Kotlin sortedBy is stable. Its order decides equal-distance
                // minBy ties; reducing directly to the player's distance loses it.
                candidates.sort_by(|a, b| {
                    compare(
                        distance_squared(*a, looking_at),
                        distance_squared(*b, looking_at),
                    )
                });
                closest(candidates, player).unwrap_or(looking_at)
            }
        };
        let context_jump = opponent.is_some_and(|o| o.out_of_distance && !o.out_of_danger);
        // Leader.running also assists combat movement when its named player was
        // not found. Entity.closerThan uses strict distance < radius squared.
        let leader_jump = settings.leader
            && !(distance_squared(player, goal)
                < f64::from(settings.leader_radius) * f64::from(settings.leader_radius));
        Some((goal, context_jump || leader_jump))
    }

    // NATIVE_FIGHT_GOAL
    impl Game<'_> {
        /// Entity.doesNotCollideBelow() defaults to -64 even in dimensions with
        /// another minimum build height. The queried entity supplies the full
        /// collision context; using the local player changes powder snow and
        /// scaffolding behavior for remote targets.
        pub(in crate::game) fn kill_aura_ground_below(&self, id: i32, b: super::B) -> bool {
            let Some(entity) = self.native_entity(id as u32) else {
                return false;
            };
            let Some(position) = self.native_entity_position(entity) else {
                return false;
            };
            if position[1] < -64. || b.lo.1 < -64. {
                return false;
            }
            let living = self.native_living(entity);
            let held = living.and_then(|e| {
                self.native_living_hand_item(e, horizon_client::first_person::Hand::Main)
            });
            let boots = living.and_then(|e| {
                self.native_living_equipment(e, horizon_client::mod_api::EquipmentSlot::Feet)
            });
            let local = self.remote.as_ref().map_or(0, |r| r.entity_id);
            let mut descending = false;
            let mut subject = None;
            let kind = if id == local {
                if let Some(player) = self.player.as_ref() {
                    descending = player.shift_key_down;
                    subject = Some(player.powder_snow_subject());
                }
                EntityContextKind::Player
            } else if let Some(guest) = self
                .lan
                .iter()
                .flat_map(|l| l.hub.guests.iter())
                .find(|g| g.placed && g.entity_id == id && g.dimension == self.dim.id())
            {
                descending = guest.sneaking;
                subject = guest.motion.as_ref().map(|p| p.powder_snow_subject());
                EntityContextKind::Player
            } else if let Some(mob) = self.mobs.iter().find(|m| m.id as i32 == id && !m.removed) {
                descending = mob.extra.crouching;
                subject = Some(horizon_client::collisions::PowderSnowSubject {
                    fall_distance: mob.fall_distance,
                    leather_boots: boots.as_ref().is_some_and(|stack| {
                        stack.count > 0 && stack.item.as_ref() == "minecraft:leather_boots"
                    }),
                    walkable_type: self
                        .res
                        .mob_data()
                        .entity_has("minecraft:powder_snow_walkable_mobs", &mob.kind.id),
                    lava_walker: mob.kind.id.as_str() == "minecraft:strider",
                    falling_block: false,
                });
                if horizon_client::command::args::normalize(&mob.kind.id) == "minecraft:player" {
                    EntityContextKind::Player
                } else if living.is_some() {
                    EntityContextKind::Living
                } else {
                    EntityContextKind::Other
                }
            } else if living.is_some() {
                EntityContextKind::Living
            } else {
                EntityContextKind::Other
            };
            let context = CollisionContext::for_entity(
                CollisionEntity {
                    identity: EntityIdentity(u64::from(id as u32)),
                    kind,
                    y: position[1],
                    descending,
                    main_hand: living.map(|_| {
                        held.as_ref()
                            .map_or(CollisionItem::EMPTY, |stack| CollisionItem {
                                item: &stack.item,
                                count: stack.count as i32,
                            })
                    }),
                },
                false,
            );
            let bounds = horizon_core::geometry::Aabb::new(
                Vec3::new(b.lo.0, -64., b.lo.2),
                Vec3::new(b.hi.0, b.hi.1, b.hi.2),
            );
            let mut scene = horizon_client::collisions::SceneCollisions {
                world: &self.world,
                binding: self.collision,
                subject,
            };
            BlockCollisionScan::any_collision(&mut scene, context, bounds, false, usize::MAX)
                .expect("FightBot target block collision column")
        }
        fn fight_goal_collides(&self, at: Point) -> bool {
            let Some(player) = self.player.as_ref() else {
                return false;
            };
            let bounds = player.bounds().move_by(Vec3::new(
                at[0] - player.position.x,
                at[1] - player.position.y,
                at[2] - player.position.z,
            ));
            let held = self
                .inventory
                .get(self.client_hotbar_slot())
                .and_then(Option::as_ref);
            let context = CollisionContext::for_entity(
                CollisionEntity {
                    identity: EntityIdentity(
                        self.remote
                            .as_ref()
                            .map_or(0, |r| r.entity_id as u32 as u64),
                    ),
                    kind: EntityContextKind::Player,
                    y: player.position.y,
                    descending: player.shift_key_down,
                    main_hand: Some(held.map_or(CollisionItem::EMPTY, |stack| CollisionItem {
                        item: &stack.item,
                        count: stack.count as i32,
                    })),
                },
                false,
            );
            let mut scene = horizon_client::collisions::SceneCollisions {
                world: &self.world,
                binding: self.collision,
                subject: Some(player.powder_snow_subject()),
            };
            BlockCollisionScan::any_collision(&mut scene, context, bounds, false, usize::MAX)
                .expect("FightBot actual target-position block collision")
        }
        fn fight_goal_leader(&self, username: &str) -> Option<Point> {
            if username.is_empty() {
                return None;
            }
            if self.own_profile_name() == username {
                return self.player.as_ref().map(|p| point(p.position));
            }
            if let Some(mob) = self.mobs.iter().find(|m| {
                !m.removed
                    && horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player"
                    && self.remote_player_name(m.id) == Some(username)
            }) {
                return Some(point(mob.position));
            }
            self.lan
                .iter()
                .flat_map(|h| h.hub.guests.iter())
                .find(|g| g.placed && g.dimension == self.dim.id() && g.name == username)
                .map(|g| g.position)
        }
        pub(in crate::game) fn kill_aura_fight_goal(&self, value: &Value) -> Option<(Point, bool)> {
            let player = point(self.player.as_ref()?.position);
            let number =
                |key: &str, default: f32| value[key].as_f64().map_or(default, |v| v as f32);
            let settings = Settings {
                dangerous_yaw: number("dangerousYaw", 55.),
                runaway: value["runawayOnCooldown"].as_bool().unwrap_or(true),
                leader: value["leader"] == true,
                leader_radius: number("leaderRadius", 5.),
            };
            let opponent = value["targetId"].as_i64().and_then(|id| {
                let id = id as i32;
                let dragon = self
                    .dragon
                    .as_ref()
                    .filter(|d| d.id as i32 == id || self.client_targeting_dragon_id() == id);
                let (at, yaw) = if let Some(dragon) = dragon {
                    (point(dragon.position), dragon.yaw)
                } else {
                    let entity = self.native_entity(id as u32)?;
                    (
                        self.native_entity_position(entity)?,
                        self.native_entity_rotation(entity)?[0],
                    )
                };
                let distance = distance_squared(player, at).sqrt();
                let distance_float = distance as f32;
                let interaction = number("range", 3.);
                let range = if interaction.is_nan() || distance_float.is_nan() {
                    f32::NAN
                } else {
                    interaction.min(distance_float)
                };
                // Rotation.lookingAt(player feet, target eye). Pitch is then
                // overwritten to zero; target eye height does not enter its yaw.
                let required_yaw = wrap(
                    (horizon_core::jmath::atan2(player[2] - at[2], player[0] - at[0])
                        * f64::from(horizon_core::geometry::trig::RAD_TO_DEG))
                        as f32
                        - 90.,
                );
                Some(Opponent {
                    position: at,
                    yaw,
                    range,
                    required_yaw,
                    out_of_distance: distance > f64::from(number("opponentRange", 3.)),
                    out_of_danger: wrap(required_yaw - yaw).abs() > settings.dangerous_yaw,
                })
            });
            let leader = settings
                .leader
                .then(|| self.fight_goal_leader(value["username"].as_str().unwrap_or("")))
                .flatten();
            select(
                player,
                opponent,
                leader,
                settings,
                value["willClick"] == true,
                |yaw| point(Vec3::direction_from_rotation(0., yaw)),
                |at| self.fight_goal_collides(at),
            )
        }
    }
}
