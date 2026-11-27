// SPDX-License-Identifier: GPL-3.0-or-later
//! PlaceWater's real world planning and vanilla predicted bucket actions.
use super::*;
use horizon_client::no_web_model as model;
#[path = "no_web_wire.rs"]
mod wire;
use serde_json::{Value, json};
use std::cell::RefCell;
pub(super) const REQUESTER: u64 = 0x4e4f_5745_42;
struct NativeWorld<'a, 'r>(RefCell<&'a mut Game<'r>>);
impl model::World for NativeWorld<'_, '_> {
    fn block(&self, pos: model::Pos) -> model::Block {
        let mut g = self.0.borrow_mut();
        let state = g.world.block(pos[0], pos[1], pos[2]);
        let facts = g.res.facts().get(state);
        let web = &*facts.name == "minecraft:cobweb";
        let air = g.res.facts().is_air(state);
        let replaceable = facts.replaceable;
        let fluid = facts.fluid;
        let source = g
            .fluid_height_of(pos[0], pos[1], pos[2], None)
            .is_some_and(|(_, source)| source);
        let outline = g
            .outline_boxes(pos)
            .iter()
            .map(|b| std::array::from_fn(|i| b[i] - pos[i % 3] as f64))
            .collect();
        let center_support = g.with_level(|b, level, ticks| {
            b.no_web_center_support(
                level,
                ticks,
                horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]),
            )
        });
        model::Block {
            web,
            air,
            replaceable,
            lava_source: source && fluid == Fluid::Lava,
            water_source: source && fluid == Fluid::Water,
            center_support,
            outline,
        }
    }
}
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_millis() as u64)
}
fn nearest(g: &Game<'_>, item: &str) -> Option<i32> {
    let selected = g.client_hotbar_slot() as i32;
    std::iter::once(40)
        .chain(0..9)
        .filter(|slot| {
            if *slot == 40 {
                g.offhand.as_ref()
            } else {
                g.inventory[*slot as usize].as_ref()
            }
            .is_some_and(|s| s.count > 0 && &*s.item == item)
        })
        .min_by_key(|slot| {
            if *slot == selected {
                i32::MIN
            } else if *slot == 40 {
                i32::MIN + 1
            } else {
                (*slot - selected).abs()
            }
        })
}
impl Game<'_> {
    pub(super) fn no_web_plan(&mut self, value: &str) -> Option<String> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let value: Value = serde_json::from_str(value).ok()?;
        let mut state = model::NoWeb::default();
        wire::read_state(&value, &mut state);
        let eyes = self.eye_position()?;
        let look = horizon_world::view_direction::view_vector(self.pitch, self.yaw);
        let player = model::Player {
            eyes,
            look: [look.x, look.y, look.z],
            reach: self.block_reach,
            evaporates: self.dimension_type().water_evaporates,
            water_slot: nearest(self, "minecraft:water_bucket"),
            bucket_slot: nearest(self, "minecraft:bucket"),
        };
        let world = NativeWorld(RefCell::new(self));
        let action = state.plan(&world, player, now());
        Some(json!({"state":wire::state(&state),"action":action.map(wire::action),"rotation":action.map(|a|a.rotation(eyes))}).to_string())
    }
    fn no_web_trace(&self, rotation: [f32; 2], fluid: bool) -> Option<model::Hit> {
        let eyes = self.eye_position()?;
        let start = Vec3::new(eyes[0], eyes[1], eyes[2]);
        let dir = horizon_world::view_direction::view_vector(rotation[1], rotation[0]);
        let result = horizon_world::world_ray::clip_world(
            &mut super::no_web_ray::NoWebRay(self),
            start,
            start.add_vector(dir.scale(self.block_reach)),
            horizon_world::world_ray::RayBlockMode::Outline,
            if fluid {
                horizon_world::world_ray::RayFluidMode::SourceOnly
            } else {
                horizon_world::world_ray::RayFluidMode::None
            },
            256,
        )
        .ok()?;
        if result.miss {
            return None;
        }
        use horizon_core::geometry::Direction as D;
        let h = result.hit;
        Some(model::Hit {
            pos: [h.block.x, h.block.y, h.block.z],
            point: [h.location.x, h.location.y, h.location.z],
            face: match h.direction {
                D::Down => [0, -1, 0],
                D::Up => [0, 1, 0],
                D::North => [0, 0, -1],
                D::South => [0, 0, 1],
                D::West => [-1, 0, 0],
                D::East => [1, 0, 0],
            },
        })
    }
    pub(super) fn no_web_execute(&mut self, value: &str) -> Option<String> {
        if !self.native_client_slot_owner() {
            return None;
        }
        let value: Value = serde_json::from_str(value).ok()?;
        let action = wire::read_action(&value["action"])?;
        let rotation = [
            value["rotation"][0].as_f64()? as f32,
            value["rotation"][1].as_f64()? as f32,
        ];
        let Some(hit) = action.resolve(
            self.no_web_trace(rotation, false),
            self.no_web_trace(rotation, true),
        ) else {
            return Some(json!({"attempted":false}).to_string());
        };
        if action.slot != 40
            && !self
                .native_client_select_slot_silently(REQUESTER, action.slot, 1)
                .ok()?
        {
            return Some(json!({"attempted":true,"success":false}).to_string());
        }
        self.remote_send_carried();
        let hand = if action.slot == 40 {
            horizon_client::first_person::Hand::Off
        } else {
            horizon_client::first_person::Hand::Main
        };
        let old = self.forced_hit;
        self.forced_hit = Some(Hit {
            pos: hit.pos,
            face: hit.face,
            point: hit.point,
        });
        let used = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.native_with_item_rotation(rotation[0], rotation[1], |g| {
                if g.remote_use_item_on(hand) {
                    true
                } else {
                    let used = g.remote_use_item_rotated(hand, rotation[0], rotation[1]);
                    if used {
                        g.heights.item_used(hand);
                    }
                    used
                }
            })
        }));
        self.forced_hit = old;
        let used = match used {
            Ok(v) => v,
            Err(e) => std::panic::resume_unwind(e),
        };
        let mut state = model::NoWeb::default();
        wire::read_state(&value["state"], &mut state);
        if used {
            let world = NativeWorld(RefCell::new(self));
            state.success(action, hit, now(), &world);
        }
        Some(json!({"attempted":true,"success":used,"state":wire::state(&state)}).to_string())
    }
}
