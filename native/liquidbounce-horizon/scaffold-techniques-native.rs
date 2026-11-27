// SPDX-License-Identifier: GPL-3.0-or-later
//! Complete source Normal/Expand target, rotation and crosshair operations.
use super::{
    Game, ItemStack,
    block_placement::{self as placement, Target},
    scaffold_aim_rules as aim, scaffold_height_rules as height, scaffold_technique_rules as rules,
};
use horizon_client::pose::Pose;
use horizon_core::{
    geometry::{
        Vec3,
        trig::{DEG_TO_RAD, RAD_TO_DEG},
    },
    jmath,
};
use serde_json::{Value, json};

struct Math;
fn point(value: Vec3) -> [f64; 3] {
    [value.x, value.y, value.z]
}
impl aim::Math for Math {
    fn rotation_direction(&mut self, r: [f32; 2]) -> aim::Point {
        point(Vec3::direction_from_rotation(r[1], r[0]))
    }
    fn yaw_axis(&mut self, yaw: f32) -> aim::Point {
        point(Vec3::new(0.0, 0.0, 1.0).rotate_y(yaw * DEG_TO_RAD))
    }
    fn point_yaw(&mut self, p: aim::Point) -> f32 {
        (jmath::atan2(-p[0], p[2]) as f32) * RAD_TO_DEG
    }
    fn random_between(&mut self, from: f64, to: f64) -> f64 {
        super::edge_random::with_thread_random(|random| random.next_f64_between(from, to))
    }
}
fn mode(value: &Value) -> Option<aim::Mode> {
    Some(match value.as_str()? {
        "Center" => aim::Mode::Center,
        "Random" => aim::Mode::Random,
        "Stabilized" => aim::Mode::Stabilized,
        "NearestRotation" => aim::Mode::NearestRotation,
        "ReverseYaw" => aim::Mode::ReverseYaw,
        "DiagonalYaw" => aim::Mode::DiagonalYaw,
        "AngleYaw" => aim::Mode::AngleYaw,
        "EdgePoint" => aim::Mode::EdgePoint,
        _ => return None,
    })
}
fn line(value: &Value) -> Option<aim::Linear> {
    Some(aim::Linear {
        anchor: serde_json::from_value(value["position"].clone()).ok()?,
        direction: serde_json::from_value(value["direction"].clone()).ok()?,
        segment: false,
    })
}
fn write_target(target: &Target) -> Value {
    json!({"interacted":target.interacted,"placed":target.placed,"direction":target.direction,
        "point":target.point,"minY":target.min_y,"rotation":target.rotation})
}
fn target(value: &Value) -> Option<Target> {
    Some(Target {
        interacted: serde_json::from_value(value["interacted"].clone()).ok()?,
        placed: serde_json::from_value(value["placed"].clone()).ok()?,
        direction: serde_json::from_value(value["direction"].clone()).ok()?,
        point: serde_json::from_value(value["point"].clone()).ok()?,
        min_y: value["minY"].as_f64()?,
        rotation: serde_json::from_value(value["rotation"].clone()).ok()?,
    })
}
fn pose(value: &Value) -> Option<Pose> {
    Some(match value.as_str()? {
        "Standing" | "STANDING" => Pose::Standing,
        "Crouching" | "CROUCHING" => Pose::Crouching,
        "Swimming" | "SWIMMING" => Pose::Swimming,
        "FallFlying" | "FALL_FLYING" => Pose::FallFlying,
        "SpinAttack" | "SPIN_ATTACK" => Pose::SpinAttack,
        "Sleeping" | "SLEEPING" => Pose::Sleeping,
        _ => return None,
    })
}
fn same_y(value: &Value) -> Option<height::SameY> {
    Some(match value.as_str()? {
        "Off" => height::SameY::Off,
        "On" => height::SameY::On,
        "JumpKey" => height::SameY::JumpKey,
        "Falling" => height::SameY::Falling,
        "Hypixel" => height::SameY::Hypixel,
        _ => return None,
    })
}
struct Height {
    state: height::State,
    same_y: height::SameY,
    facts: height::PositionFacts,
    towering: bool,
    down: bool,
    ceiling: bool,
}
impl Height {
    fn read(game: &Game<'_>, value: &Value) -> Option<Self> {
        let player = game.player.as_ref()?;
        let keys = horizon_client::reverse_step_bridge::keys();
        let integer = |key: &str| i32::try_from(value["state"][key].as_i64()?).ok();
        Some(Self {
            state: height::State {
                placement_y: integer("placementY")?,
                start_y: integer("startY")?,
                jumps: integer("jumps")?,
                was_towering: value["state"]["wasTowering"].as_bool()?,
                force_sneak: 0,
            },
            same_y: same_y(&value["sameY"])?,
            facts: height::PositionFacts {
                block: [0; 3],
                velocity_y: player.velocity.y,
                input_jump: keys[4] || player.was_auto_jump,
                physical_jump: game.keys.held(horizon_client::actions::Action::Jump),
                moving: player.move_vector != [0.0, 0.0],
                horizontal_collision: player.collision.horizontal,
            },
            towering: value["towering"].as_bool()?,
            down: value["downRunning"].as_bool()? && value["shouldGoDown"].as_bool()?,
            ceiling: value["ceiling"].as_bool()?,
        })
    }
    fn target(&mut self, mut block: [i32; 3]) -> [i32; 3] {
        if self.towering || self.state.was_towering {
            if self.towering {
                self.state.was_towering = true;
            }
            block[1] = block[1].wrapping_sub(1);
            block
        } else if self.down {
            block[1] = block[1].wrapping_sub(2);
            block
        } else if self.ceiling {
            block[1] = block[1].wrapping_add(3);
            block
        } else {
            self.state.targeted_position(
                height::PositionFacts {
                    block,
                    ..self.facts
                },
                self.same_y,
            )
        }
    }
    fn write(&self) -> Value {
        json!({"placementY":self.state.placement_y,"startY":self.state.start_y,
            "jumps":self.state.jumps,"wasTowering":self.state.was_towering})
    }
}
impl Game<'_> {
    pub(super) fn scaffold_technique_query(&mut self, text: &str) -> Option<String> {
        let request = serde_json::from_str::<Value>(text).ok()?;
        let technique = request["technique"].as_str()?;
        if !matches!(technique, "Normal" | "Expand") {
            return None;
        }
        match request["op"].as_str()? {
            "plan" => {
                let predicted: [f64; 3] =
                    serde_json::from_value(request["position"].clone()).ok()?;
                let rotation: [f32; 2] =
                    serde_json::from_value(request["serverRotation"].clone()).ok()?;
                let player = self.player.as_ref()?;
                let yaw = player.yaw;
                let eye_height = pose(&request["pose"])?.dimensions(player.scale).eye_height;
                let mut location = placement::capture_location(self, Some(predicted), rotation)?;
                location.eye_height = eye_height;
                let stack = if request["slot"].is_null() {
                    ItemStack::new("minecraft:sandstone", 64)
                } else {
                    let slot = usize::try_from(request["slot"].as_u64()?).ok()?;
                    if slot > 8 {
                        return None;
                    }
                    self.inventory.get(slot)?.as_ref()?.clone()
                };
                let mut height = Height::read(self, &request["height"])?;
                let found = if technique == "Normal" {
                    let aim_mode = mode(&request["aimMode"])?;
                    let optimal = if request["optimalLine"].is_null() {
                        None
                    } else {
                        Some(line(&request["optimalLine"])?)
                    };
                    let down = request["height"]["shouldGoDown"].as_bool()?;
                    let mut factory =
                        |_: &Game<'_>,
                         face: placement::Face,
                         block: [i32; 3],
                         location: placement::Location| {
                            Some(aim::sample(
                                aim_mode,
                                aim::Face {
                                    from: face.from,
                                    to: face.to,
                                },
                                block.map(f64::from),
                                aim::Location {
                                    position: location.position,
                                    eye: location.eye(),
                                    rotation: location.rotation,
                                    moving: location.moving,
                                },
                                optimal,
                                &mut Math,
                            ))
                        };
                    let mut options = placement::Options::center(&stack, location);
                    options.offsets = placement::offsets(rules::normal_radius(
                        request["freeze"].as_bool()?,
                        down,
                    ));
                    options.priority =
                        optimal.map_or(placement::Priority::Position(predicted), |line| {
                            placement::Priority::Line {
                                position: line.anchor,
                                direction: line.direction,
                            }
                        });
                    options.face = placement::FaceFactory::Custom(&mut factory);
                    options.consider_facing_away = down;
                    let block = height.target(predicted.map(|v| v.floor() as i32));
                    placement::find(self, block, &mut options)
                } else {
                    let length = i32::try_from(request["length"].as_i64()?).ok()?;
                    if !(1..=10).contains(&length) {
                        return None;
                    }
                    let radians = yaw * DEG_TO_RAD;
                    let xz = [
                        -(jmath::sin(f64::from(radians)) as f32),
                        jmath::cos(f64::from(radians)) as f32,
                    ];
                    let mut options = placement::Options::center(&stack, location);
                    options.consider_facing_away = true;
                    // BlockOffsetOptions.Default compares against the live player,
                    // though its sole zero offset makes the ordering unobservable.
                    let p = self.player.as_ref()?;
                    options.priority =
                        placement::Priority::Position([p.position.x, p.position.y, p.position.z]);
                    rules::find_expand(predicted, xz, length, |candidate| {
                        placement::find(self, height.target(candidate), &mut options)
                    })
                };
                Some(
                    json!({"target":found.as_ref().map(write_target),"heightState":height.write()})
                        .to_string(),
                )
            }
            "rotation" => {
                if technique == "Normal"
                    && request["telly"]["enabled"].as_bool()?
                    && request["telly"]["doNotAim"].as_bool()?
                {
                    return Some(match request["telly"]["resetMode"].as_str()? {
                        "Reverse" => {
                            let p = self.player.as_ref()?;
                            json!(rules::reverse_rotation(p.yaw, p.pitch)).to_string()
                        }
                        "Reset" => "null".into(),
                        _ => return None,
                    });
                }
                let target = target(&request["target"])?;
                let rotation = if technique == "Expand" {
                    placement::looking_at(
                        target.placed.map(|v| f64::from(v) + 0.5),
                        self.eye_position()?,
                    )
                } else {
                    if request["requiresSight"].as_bool()? {
                        let range = self
                            .interaction_range("minecraft:block_interaction_range", 4.5, 5.0)
                            .max(self.interaction_range(
                                "minecraft:entity_interaction_range",
                                3.0,
                                5.0,
                            ));
                        // Original checks sight, then falls through to the same target
                        // rotation even when the check fails.
                        let _sight = placement::trace_world(self, target.rotation, range)
                            .is_some_and(|hit| hit.pos == target.interacted);
                    }
                    target.rotation
                };
                Some(json!(rotation).to_string())
            }
            "crosshair" => {
                let target = target(&request["target"])?;
                let rotation = serde_json::from_value(request["rotation"].clone()).ok()?;
                let range = self
                    .interaction_range("minecraft:block_interaction_range", 4.5, 5.0)
                    .max(self.interaction_range("minecraft:entity_interaction_range", 3.0, 5.0));
                let fallback = technique == "Expand" || request["shouldGoDown"].as_bool()?;
                let click = placement::verify_click(
                    self,
                    &target,
                    rotation,
                    range,
                    placement::ClickTrace::World,
                    if fallback {
                        placement::FailedClick::PlannedHit
                    } else {
                        placement::FailedClick::Nothing
                    },
                )?;
                Some(json!({"block":click.hit.pos,"face":click.hit.face,"point":click.hit.point,"inside":click.inside}).to_string())
            }
            _ => None,
        }
    }
}
