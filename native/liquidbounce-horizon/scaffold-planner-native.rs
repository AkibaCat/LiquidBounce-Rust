// SPDX-License-Identifier: GPL-3.0-or-later
//! Real empty-context support shapes for ScaffoldMovementPlanner.
use super::{Game, scaffold_planner_rules as rules};
use horizon_core::position::BlockPos;
use horizon_world::collision_context::CollisionContext;
use serde_json::{Value, json};

impl Game<'_> {
    fn scaffold_support_candidates(&self) -> Option<([f64; 3], Vec<rules::Candidate>)> {
        let player = self.player.as_ref()?;
        let position = [player.position.x, player.position.y, player.position.z];
        let bounds = player.bounds();
        let min = bounds.min_position();
        let max = bounds.max_position();
        let bounds = [min.x, min.y, min.z, max.x, max.y, max.z];
        let support_y = (min.y.floor() as i32).wrapping_sub(1);
        let mut candidates = Vec::new();
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        for x in min.x.floor() as i32..=max.x.floor() as i32 {
            for z in min.z.floor() as i32..=max.z.floor() as i32 {
                let block = [x, support_y, z];
                let state = self.world.block(x, support_y, z);
                let shape = self
                    .collision
                    .collision_shape_with_context(
                        &mut scene,
                        state,
                        BlockPos::new(x, support_y, z),
                        &CollisionContext::empty(false),
                    )
                    .expect("Scaffold planner support shape");
                if shape.is_empty() {
                    continue;
                }
                let boxes: Vec<_> = shape
                    .boxes()
                    .iter()
                    .map(|b| {
                        let a = b.min_position();
                        let z = b.max_position();
                        [a.x, a.y, a.z, z.x, z.y, z.z]
                    })
                    .collect();
                candidates.push(rules::Candidate::from_boxes(
                    block, &boxes, position, bounds,
                ));
            }
        }
        Some((position, candidates))
    }
    pub(super) fn scaffold_planner_query(&mut self, text: &str) -> Option<String> {
        let request: Value = serde_json::from_str(text).ok()?;
        match request["op"].as_str()? {
            "reset" => self.scaffold_planner_state.reset(),
            "track" => self
                .scaffold_planner_state
                .track(serde_json::from_value(request["block"].clone()).ok()?),
            "support" => {}
            "line" => {
                let input: [bool; 4] = serde_json::from_value(request["input"].clone()).ok()?;
                let line = if input == [false; 4] {
                    None
                } else {
                    let angle = rules::movement_angle(self.player.as_ref()?.yaw, input);
                    let (position, candidates) = self.scaffold_support_candidates()?;
                    self.scaffold_planner_state
                        .line(position, angle, &candidates, |yaw| {
                            let d = horizon_core::geometry::Vec3::direction_from_rotation(0.0, yaw);
                            [d.x, d.y, d.z]
                        })
                };
                return Some(json!({"line":line.map(|l|json!({"position":l.position,"direction":l.direction})),
                    "support":self.scaffold_planner_state.support.map(|s|json!({"block":s.block,"offsetX":s.offset_x,"offsetZ":s.offset_z}))}).to_string());
            }
            _ => return None,
        }
        Some(json!({"support":self.scaffold_planner_state.support.map(|s|json!({"block":s.block,"offsetX":s.offset_x,"offsetZ":s.offset_z}))}).to_string())
    }
}
