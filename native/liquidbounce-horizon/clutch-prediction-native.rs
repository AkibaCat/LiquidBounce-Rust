// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleClutch.canCatch/predictFall over the real native player and outlines.
use super::Game;
use serde_json::{Value, json};

const MAX_FALL_TICKS: i32 = 60;

/// searchBlocksInRangeSorted(range).isNotEmpty(): sorted order does not affect
/// this Boolean. The source range and its square are both float operations.
fn in_reach(game: &Game<'_>, range: f32) -> bool {
    let Some(eye) = game.eye_position() else {
        return false;
    };
    let radius = f64::from(range + 1.0);
    let minimum: [i32; 3] = std::array::from_fn(|i| (eye[i] - radius).floor() as i32);
    let maximum: [i32; 3] = std::array::from_fn(|i| (eye[i] + radius).floor() as i32);
    let lower_y = minimum[1].max(game.world.shape.min_y);
    let upper_y = maximum[1].min(game.world.shape.max_y() - 1);
    let distance_limit = f64::from(range * range);
    // LevelReader.findBlocksIn only visits loaded chunks inside world height.
    for &(cx, cz) in game.world.chunks.keys() {
        let x = cx.saturating_mul(16);
        let z = cz.saturating_mul(16);
        let lower_x = minimum[0].max(x);
        let upper_x = maximum[0].min(x.saturating_add(15));
        let lower_z = minimum[2].max(z);
        let upper_z = maximum[2].min(z.saturating_add(15));
        for y in lower_y..=upper_y {
            for z in lower_z..=upper_z {
                for x in lower_x..=upper_x {
                    let pos = [x, y, z];
                    let state = game.world.block(x, y, z);
                    if game.res.facts().get(state).replaceable {
                        continue;
                    }
                    let boxes = super::block_placement::outline(game, pos, true);
                    // ShapeExtensions.distanceToSqr(empty) is +infinity.
                    // TargetFinding's separate center fallback is not used.
                    if !boxes.is_empty()
                        && super::placement_core::distance_squared(&boxes, pos, eye)
                            <= distance_limit
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

impl Game<'_> {
    fn clutch_forecast(
        &mut self,
        scaffold_enabled: bool,
        block_count: i32,
        minimum_fall: f32,
    ) -> Result<Value, String> {
        let player = self
            .player
            .as_ref()
            .ok_or("Clutch requires a local player")?;
        let passenger = player.is_passenger || self.riding.is_some();
        let fall_flying = player.fall_flying;
        let flying = player.flying;
        let on_ground = player.ground.on_ground;
        let input_steered = horizon_client::reverse_step_bridge::keys()[..4]
            .iter()
            .any(|down| *down);
        let initial_horizontal =
            player.velocity.x * player.velocity.x + player.velocity.z * player.velocity.z;
        let mut reachable = false;
        let mut landing = false;
        let allowed = !passenger && !fall_flying && !flying && !scaffold_enabled && block_count > 0;
        if allowed && !on_ground {
            let range =
                self.interaction_range("minecraft:block_interaction_range", 4.5, 5.0) as f32;
            reachable = in_reach(self, range);
            if !reachable {
                horizon_world::block_walk::without_shape_changes(|| {
                    super::falling_forecast::visit(self, MAX_FALL_TICKS, 0.0, |player, _| {
                        landing = player.ground.on_ground;
                        landing
                    })
                })?;
            }
        }
        let guarding = allowed && (on_ground || reachable || landing);
        let mut positions = Vec::new();
        let mut must_catch = false;
        if guarding {
            // canCatch's replay can run source simulation callbacks. As the
            // source does, predictFall starts again from the actual player.
            let player = self.player.as_ref().ok_or("Clutch lost its local player")?;
            let keys = horizon_client::reverse_step_bridge::keys();
            let steered = keys[..4].iter().any(|down| *down);
            let horizontal =
                player.velocity.x * player.velocity.x + player.velocity.z * player.velocity.z;
            if !(player.ground.on_ground && (steered || horizontal == 0.0)) {
                horizon_world::block_walk::without_shape_changes(|| {
                    super::falling_forecast::visit(self, MAX_FALL_TICKS, 0.0, |player, _| {
                        positions.push([player.position.x, player.position.y, player.position.z]);
                        // FallingPlayer.fallPath observes the post-movement
                        // fall threshold before considering grounded safety.
                        if player.fall_distance > f64::from(minimum_fall) {
                            must_catch = true;
                            return true;
                        }
                        player.ground.on_ground
                            && (steered
                                || player.velocity.x * player.velocity.x
                                    + player.velocity.z * player.velocity.z
                                    == 0.0)
                    })
                })?;
            }
        }
        Ok(json!({
            "passenger":passenger,"fallFlying":fall_flying,"flying":flying,
            "scaffoldEnabled":scaffold_enabled,"blockCount":block_count,
            "steered":input_steered,"horizontalVelocitySquared":initial_horizontal,
            "onGround":on_ground,"inReach":reachable,"landing":landing,
            "guarding":guarding,"path":must_catch.then_some(positions)
        }))
    }

    pub(super) fn clutch_query(&mut self, request: &str) -> String {
        let Ok(request) = serde_json::from_str::<Value>(request) else {
            return "null".into();
        };
        // These must come from the real Scaffold state and block selector;
        // there is deliberately no successful default for either dependency.
        let (Some(scaffold), Some(blocks), Some(minimum)) = (
            request["scaffoldEnabled"].as_bool(),
            request["blockCount"].as_i64(),
            request["minFallDistance"].as_f64(),
        ) else {
            return "null".into();
        };
        match self.clutch_forecast(scaffold, blocks as i32, minimum as f32) {
            Ok(forecast) => forecast.to_string(),
            Err(error) => {
                eprintln!("[LiquidBounce] Clutch forecast: {error}");
                "null".into()
            }
        }
    }
}
