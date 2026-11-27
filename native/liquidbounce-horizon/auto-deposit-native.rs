// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use horizon_client::first_person::{Hand, SwingAnimation};
use serde_json::{Value, json};
pub(super) const REQUESTER: u64 = 0x5055_4e43_4844_4550;
fn point_rotation(eyes: [f64; 3], point: [f64; 3]) -> [f32; 2] {
    let d = std::array::from_fn::<_, 3, _>(|i| point[i] - eyes[i]);
    [
        (d[2].atan2(d[0]).to_degrees() - 90.) as f32,
        (-d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees()) as f32,
    ]
}
fn box_value(b: [f64; 6]) -> horizon_core::geometry::Aabb {
    use horizon_core::geometry::{Aabb, Vec3};
    Aabb::new(Vec3::new(b[0], b[1], b[2]), Vec3::new(b[3], b[4], b[5]))
}
fn first_hit(b: [f64; 6], from: [f64; 3], to: [f64; 3]) -> Option<[f64; 3]> {
    use horizon_core::geometry::Vec3;
    let b = box_value(b);
    let from = Vec3::new(from[0], from[1], from[2]);
    let to = Vec3::new(to[0], to[1], to[2]);
    let p = if b.contains(from) {
        b.clip(to, from)
    } else {
        b.clip(from, to)
    }?;
    Some([p.x, p.y, p.z])
}
/// Source PointFinding.projectPointsOnBox: a 256-cell view-plane grid,
/// with float rotation matrices and double world coordinates.
fn projected_points(eyes: [f64; 3], b: [f64; 6]) -> Option<Vec<[f64; 3]>> {
    use horizon_core::geometry::Vec3;
    if box_value(b).contains(Vec3::new(eyes[0], eyes[1], eyes[2])) {
        return None;
    }
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let dir = std::array::from_fn::<_, 3, _>(|i| (b[i] + b[i + 3]) * 0.5 - eyes[i]);
    let len = dot(dir, dir);
    if len < 1e-20 {
        return Some(vec![]);
    }
    let vertices: Vec<[f64; 3]> = (0..8)
        .map(|v| std::array::from_fn(|i| b[i + if v & (1 << i) == 0 { 0 } else { 3 }]))
        .collect();
    let projected: Vec<[f64; 3]> = vertices
        .iter()
        .map(|v| {
            let d = std::array::from_fn(|i| v[i] - eyes[i]);
            let t = dot(d, dir) / len;
            std::array::from_fn(|i| eyes[i] + dir[i] * t)
        })
        .collect();
    let origin = projected.into_iter().min_by(|a, b| {
        let distance = |p: &[f64; 3]| (0..3).map(|i| (p[i] - eyes[i]).powi(2)).sum::<f64>();
        distance(a).total_cmp(&distance(b))
    })?;
    let origin = std::array::from_fn::<_, 3, _>(|i| origin[i] + (eyes[i] - origin[i]) * 0.1);
    let normal = if (len - 1.).abs() <= 1e-4 {
        dir
    } else {
        dir.map(|v| v / len.sqrt())
    };
    let yaw = normal[2].atan2(normal[0]) as f32;
    let pitch = normal[1].atan2((normal[0] * normal[0] + normal[2] * normal[2]).sqrt()) as f32;
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let to = |v: [f32; 3]| -> [f32; 3] {
        [
            cy * cp * v[0] - cy * sp * v[1] - sy * v[2],
            sp * v[0] + cp * v[1],
            sy * cp * v[0] - sy * sp * v[1] + cy * v[2],
        ]
    };
    let back = |v: [f32; 3]| -> [f32; 3] {
        [
            cp * (cy * v[0] + sy * v[2]) + sp * v[1],
            -sp * (cy * v[0] + sy * v[2]) + cp * v[1],
            -sy * v[0] + cy * v[2],
        ]
    };
    let mut min_y = 0f32;
    let mut max_y = 0f32;
    let mut min_z = 0f32;
    let mut max_z = 0f32;
    for vertex in vertices {
        let ray = std::array::from_fn(|i| vertex[i] - eyes[i]);
        let den = dot(ray, normal);
        if den.abs() < 1e-5 {
            continue;
        }
        let numerator = dot(std::array::from_fn(|i| origin[i] - eyes[i]), normal);
        let t = numerator / den;
        let point = std::array::from_fn(|i| (eyes[i] + ray[i] * t - origin[i]) as f32);
        let point = back(point);
        min_y = min_y.min(point[1]);
        max_y = max_y.max(point[1]);
        min_z = min_z.min(point[2]);
        max_z = max_z.max(point[2]);
    }
    let pos = to([0., min_y, min_z]);
    let pos = std::array::from_fn::<_, 3, _>(|i| f64::from(pos[i]) + origin[i]);
    let dy = to([0., max_y - min_y, 0.]).map(f64::from);
    let dz = to([0., 0., max_z - min_z]).map(f64::from);
    let ly = dot(dy, dy).sqrt();
    let lz = dot(dz, dz).sqrt();
    let (ny, nz) = if ly <= 1e-7 && lz <= 1e-7 {
        (1, 1)
    } else if ly <= 1e-7 {
        (1, 256)
    } else if lz <= 1e-7 {
        (256, 1)
    } else {
        let ny = ((256. / (lz / ly)).sqrt().floor() as usize).clamp(1, 256);
        (ny, (256 / ny).max(1))
    };
    let mut points = Vec::new();
    for y in 0..ny {
        for z in 0..nz {
            let ty = (y as f64 + 0.5) / ny as f64;
            let tz = (z as f64 + 0.5) / nz as f64;
            let point = std::array::from_fn::<_, 3, _>(|i| dy[i].mul_add(ty, pos[i]));
            let point = std::array::from_fn::<_, 3, _>(|i| dz[i].mul_add(tz, point[i]));
            let end = std::array::from_fn(|i| point[i] + (eyes[i] - point[i]) * -100.);
            if let Some(p) = first_hit(b, eyes, end) {
                points.push(p);
            }
        }
    }
    Some(points)
}
impl Game<'_> {
    fn auto_deposit_max(&self, stack: &ItemStack) -> i32 {
        let view = horizon_client::item_render::stack_view(stack);
        let components = self.pack.item_renderer.models.components_view(&view);
        components
            .get("minecraft:max_stack_size")
            .and_then(Value::as_i64)
            .unwrap_or(1) as i32
    }
    pub(super) fn auto_deposit_context(&mut self) -> Option<String> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let mut value: Value =
            serde_json::from_str(&self.native_client_chest_cleaner_context()?).ok()?;
        let targets: Vec<_> = self
            .client_inventory_mapping(false)
            .into_iter()
            .filter_map(|internal| {
                if internal.is_none_or(|i| !(i < 36 || (500..=504).contains(&i))) {
                    Some(internal.and_then(|i| self.slot_view(i)))
                } else {
                    None
                }
            })
            .collect();
        let capacities: Vec<i32> = (0..36)
            .map(|slot| {
                let Some(source) = self
                    .inventory
                    .get(slot)
                    .and_then(Option::as_ref)
                    .filter(|s| s.count > 0 && &*s.item != "minecraft:air")
                else {
                    return 0;
                };
                let identity =
                    self.client_inventory_stack_record(Some(source))["components"].clone();
                targets.iter().fold(0i32, |sum, target| {
                    let capacity = match target
                        .as_ref()
                        .filter(|s| s.count > 0 && &*s.item != "minecraft:air")
                    {
                        None => self.auto_deposit_max(source),
                        Some(target)
                            if self.client_inventory_stack_record(Some(target))["components"]
                                == identity =>
                        {
                            self.auto_deposit_max(target)
                                .wrapping_sub(target.count as i32)
                        }
                        _ => 0,
                    };
                    sum.wrapping_add(capacity)
                })
            })
            .collect();
        value["capacity"] = json!(capacities);
        Some(value.to_string())
    }
    fn auto_deposit_valid(&self, value: &Value, pos: [i32; 3]) -> bool {
        let name = &*self
            .res
            .facts()
            .get(self.world.block(pos[0], pos[1], pos[2]))
            .name;
        value["blocks"]
            .as_array()
            .is_some_and(|blocks| blocks.iter().any(|b| b.as_str() == Some(name)))
    }
    fn auto_deposit_rotation_hit(
        &self,
        pos: [i32; 3],
        rotation: [f32; 2],
        range: f64,
    ) -> Option<Hit> {
        let eyes = self.eye_position()?;
        let d = horizon_world::view_direction::view_vector(rotation[1], rotation[0]);
        self.clip_declared_outline(
            self.world.block(pos[0], pos[1], pos[2]),
            pos,
            eyes,
            [d.x, d.y, d.z],
            range,
        )
    }
    pub(super) fn auto_deposit_target(&self, value: &str) -> Option<String> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let value: Value = serde_json::from_str(value).ok()?;
        let range = value["range"].as_f64()?;
        let walls = value["walls"].as_f64()?;
        if !range.is_finite() || range < 0. {
            return None;
        }
        let eyes = self.eye_position()?;
        let radius = range + 1.;
        let from = eyes.map(|v| (v - radius).floor() as i32);
        let to = eyes.map(|v| (v + radius).floor() as i32);
        let mut candidates = Vec::new();
        for x in from[0]..=to[0] {
            for y in from[1]..=to[1] {
                for z in from[2]..=to[2] {
                    let pos = [x, y, z];
                    if !self.auto_deposit_valid(&value, pos) {
                        continue;
                    }
                    let boxes = self.outline_boxes(pos);
                    let distance = boxes
                        .iter()
                        .map(|b| {
                            (0..3)
                                .map(|i| {
                                    let d = eyes[i] - eyes[i].clamp(b[i], b[i + 3]);
                                    d * d
                                })
                                .sum::<f64>()
                        })
                        .fold(f64::INFINITY, f64::min);
                    if distance <= range * range {
                        candidates.push((distance, pos, boxes));
                    }
                }
            }
        }
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, pos, mut boxes) in candidates {
            boxes.sort_by(|a, b| {
                let size = |v: &[f64; 6]| ((v[3] - v[0]) + (v[4] - v[1]) + (v[5] - v[2])) / 3.;
                size(b).total_cmp(&size(a))
            });
            let center = pos.map(|v| f64::from(v) + 0.5);
            let base = point_rotation(eyes, center);
            let preferred = horizon_world::view_direction::view_vector(base[1], base[0]);
            let preferred = [
                eyes[0] + preferred.x * range,
                eyes[1] + preferred.y * range,
                eyes[2] + preferred.z * range,
            ];
            let mut visible: Option<(f32, [f32; 2])> = None;
            let mut hidden: Option<(f32, [f32; 2])> = None;
            for b in boxes {
                let preferred = first_hit(b, eyes, preferred)
                    .filter(|p| {
                        (0..3).map(|i| (p[i] - eyes[i]).powi(2)).sum::<f64>() < range * range
                    })
                    .unwrap_or(preferred);
                let mut points = vec![
                    preferred,
                    std::array::from_fn(|i| eyes[i].clamp(b[i], b[i + 3])),
                ];
                if let Some(projected) = projected_points(eyes, b) {
                    points.extend(projected);
                } else {
                    // Eye inside the box: source scans face centres in Direction order.
                    for axis in [1, 2, 0] {
                        let mut axes = (0..3).filter(|i| *i != axis);
                        let a = axes.next().unwrap();
                        let c = axes.next().unwrap();
                        let spacing =
                            0.1f64.max((((b[a + 3] - b[a]) * (b[c + 3] - b[c])) / 100.).sqrt());
                        let na = ((b[a + 3] - b[a]) / spacing).ceil().max(1.) as usize;
                        let nc = ((b[c + 3] - b[c]) / spacing).ceil().max(1.) as usize;
                        for side in [b[axis], b[axis + 3]] {
                            for ia in 0..na {
                                for ic in 0..nc {
                                    let mut p = [0.; 3];
                                    p[axis] = side;
                                    p[a] = (b[a + 3] - b[a])
                                        .mul_add((ia as f64 + 0.5) / na as f64, b[a]);
                                    p[c] = (b[c + 3] - b[c])
                                        .mul_add((ic as f64 + 0.5) / nc as f64, b[c]);
                                    points.push(p);
                                }
                            }
                        }
                    }
                }
                for point in points {
                    let rotation = point_rotation(eyes, point);
                    let endpoint =
                        std::array::from_fn(|i| (point[i] - eyes[i]).mul_add(2., eyes[i]));
                    let Some(hit) = first_hit(b, eyes, endpoint) else {
                        continue;
                    };
                    let distance = (0..3).map(|i| (hit[i] - eyes[i]).powi(2)).sum::<f64>();
                    let length = distance.sqrt();
                    let direction =
                        std::array::from_fn::<_, 3, _>(|i| (hit[i] - eyes[i]) / length.max(1e-20));
                    use horizon_core::geometry::Vec3;
                    let start = Vec3::new(eyes[0], eyes[1], eyes[2]);
                    let end = Vec3::new(
                        hit[0] + direction[0] * 0.005,
                        hit[1] + direction[1] * 0.005,
                        hit[2] + direction[2] * 0.005,
                    );
                    let seen = horizon_world::world_ray::clip_world(
                        &mut super::no_web_ray::NoWebRay(self),
                        start,
                        end,
                        horizon_world::world_ray::RayBlockMode::Outline,
                        horizon_world::world_ray::RayFluidMode::None,
                        256,
                    )
                    .ok()
                    .is_some_and(|r| {
                        !r.miss && [r.hit.block.x, r.hit.block.y, r.hit.block.z] == pos
                    });
                    if distance >= if seen { range * range } else { walls * walls } {
                        continue;
                    }
                    let dy = ((rotation[0] - base[0] + 180.).rem_euclid(360.) - 180.) as f32;
                    let dp = rotation[1] - base[1];
                    let score = dy * dy + dp * dp;
                    let best = if seen { &mut visible } else { &mut hidden };
                    if best.is_none_or(|b| score < b.0) {
                        *best = Some((score, rotation));
                    }
                }
            }
            if let Some((_, rotation)) = visible.or(hidden) {
                return Some(json!({"pos":pos,"rotation":rotation}).to_string());
            }
        }
        None
    }
    pub(super) fn auto_deposit_punch(&mut self, value: &str) -> bool {
        if !self.native_client_slot_owner() {
            return false;
        }
        let Ok(value) = serde_json::from_str::<Value>(value) else {
            return false;
        };
        if matches!(self.screen, Screen::Inventory | Screen::Creative) {
            return false;
        }
        let hit = if value["rotate"] == true {
            let Some(pos) = value["target"]["pos"]
                .as_array()
                .filter(|p| p.len() == 3)
                .and_then(|p| {
                    Some([
                        p[0].as_i64()? as i32,
                        p[1].as_i64()? as i32,
                        p[2].as_i64()? as i32,
                    ])
                })
            else {
                return false;
            };
            let Some(rotation) = value["rotation"]
                .as_array()
                .filter(|p| p.len() == 2)
                .and_then(|p| Some([p[0].as_f64()? as f32, p[1].as_f64()? as f32]))
            else {
                return false;
            };
            self.auto_deposit_rotation_hit(pos, rotation, value["range"].as_f64().unwrap_or(3.))
        } else {
            self.pick()
                .filter(|h| self.auto_deposit_valid(&value, h.pos))
        };
        let Some(hit) = hit else {
            return false;
        };
        let slot = value["slot"].as_i64().unwrap_or(-1) as i32;
        if !(0..9).contains(&slot) {
            return false;
        }
        let _ = self.native_client_select_slot_silently(REQUESTER, slot, 1);
        use horizon_protocol::packets::play::{PlayerAction, Serverbound as S};
        let direction = match hit.face {
            [0, -1, 0] => 0,
            [0, 1, 0] => 1,
            [0, 0, -1] => 2,
            [0, 0, 1] => 3,
            [-1, 0, 0] => 4,
            _ => 5,
        };
        let pos = hit.pos;
        let sequence = self.next_sequence();
        self.send(S::PlayerAction(PlayerAction {
            action: 0,
            pos,
            direction,
            sequence,
        }));
        self.send(S::PlayerAction(PlayerAction {
            action: 1,
            pos,
            direction: 0,
            sequence: 0,
        }));
        let mode = value["swing"].as_u64().unwrap_or(0);
        if mode == 0 || mode == 3 {
            let mut animation = SwingAnimation::DEFAULT;
            if let Some(stack) = self
                .inventory
                .get(self.client_hotbar_slot())
                .and_then(Option::as_ref)
            {
                let view = horizon_client::item_render::stack_view(stack);
                let components = self.pack.item_renderer.models.components_view(&view);
                if let Some(v) = components.get("minecraft:attack_animation") {
                    animation.kind = match v.get("type").and_then(Value::as_str).unwrap_or("whack")
                    {
                        "none" => 0,
                        "stab" => 2,
                        _ => 1,
                    };
                    animation.duration =
                        v.get("duration").and_then(Value::as_i64).unwrap_or(6) as i32;
                }
            }
            self.swing_hand(Hand::Main, animation);
        }
        if mode == 0 || mode == 2 {
            self.send(S::Punch(horizon_protocol::packets::Empty));
        }
        true
    }
}
