// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::target_strafe_math::{B, V};
use horizon_client::{
    actions::Action,
    motion::MotionWorld,
    target_strafe::{Settings, Target},
    target_strafe_bridge::{self as bridge, Snapshot},
    world_primitives::{PrimitiveShape, WorldPrimitives},
};
use serde_json::{Value, json};
fn v(p: horizon_core::geometry::Vec3) -> V {
    V(p.x, p.y, p.z)
}
fn ring(
    out: &mut WorldPrimitives,
    center: V,
    outer: f32,
    inner: f32,
    outer_color: u32,
    inner_color: u32,
    outline: u32,
) {
    if outer <= 0. {
        return;
    }
    let segments = 128;
    let at = |i: usize, r: f32| {
        let a = (i as f64) * std::f64::consts::TAU / (segments as f64);
        center
            .add(V(a.cos() * f64::from(r), 0., a.sin() * f64::from(r)))
            .array()
    };
    for i in 0..segments {
        let a = at(i, outer);
        let b = at(i + 1, outer);
        out.push(PrimitiveShape::LiquidBounceGradientQuad {
            points: [a, b, at(i + 1, inner), at(i, inner)],
            colors: [outer_color, outer_color, inner_color, inner_color],
        })
        .through_walls();
        out.line(a, b, outline, 1.).through_walls();
    }
}
impl Game<'_> {
    pub(super) fn client_target_strafe_query(&mut self, text: &str) -> String {
        if !self.native_window() || !self.native_client_slot_owner() {
            return "null".into();
        }
        let Ok(q) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        if q["op"] == "facts" {
            let mut facts =
                serde_json::from_str::<Value>(&self.auto_rod_query("{\"op\":\"facts\"}"))
                    .unwrap_or(Value::Null);
            if facts.is_object() {
                facts["jump"] = json!(self.keys.held(Action::Jump));
                facts["ground"] = json!(self.player.as_ref().is_some_and(|p| p.ground.on_ground));
            }
            return facts.to_string();
        }
        if q["op"] != "configure" {
            return "null".into();
        }
        if q["running"] != true {
            bridge::clear();
            return "{}".into();
        }
        let settings = Settings::from_wire(&q["settings"]);
        let input = q["input"].as_u64().unwrap_or(0) as u8;
        let id = q["target"].as_i64().map(|n| n as i32);
        let target = super::auto_bow::entities(self)
            .into_iter()
            .find(|e| Some(e.id) == id)
            .map(|e| Target {
                id: e.id,
                position: v(e.position),
                bounds: B {
                    lo: v(e.bounds.min_position()),
                    hi: v(e.bounds.max_position()),
                },
            });
        let physical = [
            Action::Forward,
            Action::Back,
            Action::Left,
            Action::Right,
        ]
        .into_iter()
        .any(|a| self.keys.held(a));
        let initial_moving = if q["features"] == true {
            input & 15 != 0
        } else {
            physical
        };
        let input_mode = settings.input_mode;
        bridge::configure(Some(Snapshot {
            settings,
            target,
            input,
            initial_moving,
            speed_running: q["speed"] == true,
        }));
        if !input_mode {
            return "{}".into();
        }
        let entities = self.player_entity_collisions();
        let Some(player) = self.player.as_mut() else {
            bridge::clear();
            return "{}".into();
        };
        let next = MotionWorld {
            entities: &entities,
            player,
            scene: &mut self.world,
            assets: &self.pack.assets,
            collision_binding: self.collision,
            resources: self.motion,
            health: &self.health_resources,
        }
        .target_strafe_input(input);
        json!({"input":next}).to_string()
    }
    pub(super) fn client_target_strafe_frame(&mut self, out: &mut WorldPrimitives, partial: f64) {
        if !self.native_window() || !self.native_client_slot_owner() {
            return;
        }
        let Some((s, plan)) = bridge::render().filter(|(s, _)| s.visuals) else {
            return;
        };
        let Some(e) = super::auto_bow::entities(self)
            .into_iter()
            .find(|e| e.id == plan.target.id)
        else {
            bridge::clear();
            return;
        };
        let old = self
            .mobs
            .iter()
            .find(|m| m.id as i32 == e.id)
            .map(|m| m.previous)
            .or_else(|| {
                self.armor_stands
                    .iter()
                    .find(|m| m.id as i32 == e.id)
                    .map(|m| m.previous)
            })
            .or_else(|| {
                self.dragon
                    .as_ref()
                    .filter(|_| self.client_targeting_dragon_id() == e.id)
                    .map(|d| d.previous)
            })
            .map(v)
            .or_else(|| {
                self.lan
                    .iter()
                    .flat_map(|h| h.hub.guests.iter())
                    .find(|g| g.placed && g.dimension == self.dim.id() && g.entity_id == e.id)
                    .map(|g| {
                        V(
                            g.previous_position[0],
                            g.previous_position[1],
                            g.previous_position[2],
                        )
                    })
            })
            .unwrap_or(v(e.position));
        let at = old
            .lerp(v(e.position), partial)
            .add(V(0., f64::from(s.height_offset), 0.));
        ring(
            out,
            at,
            plan.radius + s.width / 2.,
            (plan.radius - s.width / 2.).max(0.),
            s.outer,
            s.inner,
            s.outline,
        );
        if s.show_point {
            ring(
                out,
                plan.next.add(V(0., f64::from(s.height_offset), 0.)),
                s.point_radius,
                0.,
                if plan.valid { s.point } else { s.invalid },
                0,
                if plan.valid {
                    s.point_outline
                } else {
                    s.invalid_outline
                },
            );
        }
    }
}
