// SPDX-License-Identifier: GPL-3.0-or-later
//! Offhand 20-tick FallingPlayer landing position over the actual scoped player.
use super::*;
use horizon_client::{crosshair_falling_bridge as bridge, sprint::Keys};
pub(super) fn collision(
    g: &mut Game<'_>,
    ticks: i32,
    add_y: f64,
) -> std::result::Result<Option<horizon_core::position::BlockPos>, String> {
    if !g.native_client_slot_owner() {
        return Err("no local player for falling simulation".into());
    }
    let p = g.player.as_ref().ok_or("no local player")?;
    let mut source = p.clone();
    let mut body = p.clone();
    let mut raw = horizon_client::reverse_step_bridge::keys();
    raw[4] = false;
    raw[5] = p.shift_key_down;
    raw[6] |= p.sprinting;
    let keys = Keys {
        forward: raw[0],
        backward: raw[1],
        left: raw[2],
        right: raw[3],
        jump: false,
        shift: raw[5],
        sprint: raw[6],
        analog: None,
    };
    body.auto_jump_enabled = false;
    body.auto_jump_time = 0;
    body.is_passenger = false;
    body.velocity.y += add_y;
    let (mut air, mut ground, mut strict, mut input, mut apply) = (1.0, 1.0, false, 0, false);
    g.native_client_strafe(&mut air, &mut ground, &mut strict, &mut input, &mut apply);
    source.client_strafe = apply.then_some((
        horizon_client::strafe::Strafe {
            strength_in_air: air,
            strength_on_ground: ground,
            strict_movement: strict,
        },
        input,
    ));
    let mut scene = World::new(g.world.air, 0);
    scene.shape = g.world.shape;
    for (&pos, chunk) in &g.world.chunks {
        scene.chunks.insert(pos, chunk.clone());
    }
    scene.moving_pistons = g.world.moving_pistons.clone();
    scene.shulkers = g.world.shulkers.clone();
    let entities = g.player_entity_collisions();
    let terrain = horizon_client::terrain_speed_bridge::enter(g.native_terrain_snapshot());
    let scope = bridge::Scope::enter(source);
    let mut effects = g.potions.effects.clone();
    let result = (|| {
        let mut motion = horizon_client::motion::MotionWorld {
            entities: &entities,
            player: &mut body,
            scene: &mut scene,
            assets: &g.pack.assets,
            collision_binding: g.collision,
            resources: g.motion,
            health: &g.health_resources,
        };
        // fromClientPlayer is called by GameTick before LocalPlayer movement;
        // it begins a full next tick, preserving prior fluid/crouch observations.
        for _tick in 0..ticks.max(0) {
            motion.player.status = effects.status();
            motion.player.mobility_restricted = effects.get("blindness").is_some();
            let (step, _) = horizon_client::navigation_physics::with_prediction(
                horizon_client::navigation_physics::module_safe_walk(),
                false,
                || motion.liquidbounce_forecast_tick(keys),
            );
            bridge::flush();
            step.map_err(|e| e.to_string())?;
            if motion.player.ground.on_ground {
                return Ok(Some(motion.player.ground.supporting_block.unwrap_or_else(
                    || {
                        horizon_core::position::BlockPos::new(
                            motion.player.position.x.floor() as i32,
                            (motion.player.position.y - 0.2).floor() as i32,
                            motion.player.position.z.floor() as i32,
                        )
                    },
                )));
            }
            let before = effects.modifiers();
            effects.tick();
            for (attr, id, _, _) in before {
                if !effects.modifiers().iter().any(|e| e.1 == id) {
                    motion
                        .player
                        .set_modifier(attr, id, 0.0, "add_value", false)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(None)
    })();
    bridge::flush();
    let source = scope.finish();
    drop(terrain);
    // These are effects of the source module callbacks themselves. The
    // speculative body's position, velocity and packets never reach Game.
    if let Some(p) = g.player.as_mut() {
        p.position = source.position;
        p.old_position = source.old_position;
        p.velocity = source.velocity;
    }
    result
}
