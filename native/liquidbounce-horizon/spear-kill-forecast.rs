// SPDX-License-Identifier: GPL-3.0-or-later
//! Agent fromOtherPlayer Forecast: guessed fixed inputs and native physics.
//! The world is an owned copy-on-write snapshot; native callbacks are suppressed.
use super::*;
use horizon_client::{effects::PlayerEffects, mobs::Mob, sprint::Keys};
fn guessed(body: &PlayerMotion) -> Keys {
    let d = body.position.subtract(body.old_position);
    let speed = d.x * d.x + d.z * d.z;
    let mut k = Keys::default();
    if speed > 0.05 * 0.05 {
        let yaw = (horizon_core::jmath::atan2(-d.x, d.z) as f32)
            * horizon_core::geometry::trig::RAD_TO_DEG;
        let wrap = |a: f32| {
            let a = a % 360.0;
            if a >= 180.0 {
                a - 360.0
            } else if a < -180.0 {
                a + 360.0
            } else {
                a
            }
        };
        let angle = wrap(yaw - wrap(body.yaw));
        let ahead = angle.abs();
        k.forward = ahead < 70.0;
        k.backward = ahead > 110.0;
        k.left = angle < -20.0 && angle > -160.0;
        k.right = angle > 20.0 && angle < 160.0;
    }
    k.jump = !body.ground.on_ground;
    k.shift = body.shift_key_down;
    k.sprint = speed >= 0.121 * 0.121;
    k
}
fn from_mob(g: &Game<'_>, m: &Mob) -> Option<(PlayerMotion, PlayerEffects)> {
    let mut p = PlayerMotion::new(g.motion, m.position).ok()?;
    let mut values = Vec::new();
    g.attr_sources()
        .mob_attributes(m, &mut Default::default(), &mut values);
    horizon_client::lan::attributes::apply(&mut p.attributes, &values);
    p.position = m.position;
    p.old_position = m.previous;
    p.velocity = m.position.subtract(m.previous);
    p.yaw = m.yaw;
    p.head_yaw = m.head_yaw;
    p.pitch = m.head_pitch;
    p.ground.on_ground = m.on_ground;
    p.ground.horizontal_collision = m.horizontal_collision;
    p.ground.supporting_block = m
        .supporting
        .map(|p| horizon_core::position::BlockPos::new(p[0], p[1], p[2]));
    p.collision.horizontal = m.horizontal_collision;
    p.collision.below = m.vertical_collision_below;
    p.collision.vertical = m.extra.liquidbounce_vertical_collision;
    p.fall_distance = m.fall_distance;
    p.stuck_multiplier = m.stuck_multiplier;
    p.no_jump_delay = m.ai.no_jump_delay;
    p.in_powder_snow = m.in_powder_snow;
    p.was_in_powder_snow = m.extra.liquidbounce_was_in_powder_snow;
    p.pose = m.extra.avatar_pose;
    p.scale = m.model_scale() as f32;
    p.dimensions = p.pose.dimensions(p.scale);
    p.shift_key_down = m.extra.crouching;
    p.local_crouching = m.extra.crouching;
    p.swimming = m.extra.liquidbounce_swimming;
    p.sprinting = m.extra.sprinting;
    p.fall_flying = m.extra.fall_flying;
    p.fall_fly_ticks = m.extra.fall_fly_ticks;
    p.fluids.first_tick = false;
    p.fluids.fast_lava = g.player.as_ref()?.fluids.fast_lava;
    p.fluids.was_touching_water = m.fluids.in_water;
    p.fluids.was_eye_in_water = m.was_eye_in_water;
    p.sleeping = m.extra.avatar_pose == horizon_client::pose::Pose::Sleeping;
    p.is_passenger = false; // SimulatedPlayer is constructed without a vehicle.
    p.entity_id = m.id as i32;
    p.client_side = true;
    p.auto_jump_enabled = false;
    p.border = Some(g.border.geometry().clone());
    let gear = m.equipment_stacks();
    p.leather_boots = gear[2]
        .as_ref()
        .is_some_and(|s| &*s.item == "minecraft:leather_boots");
    p.glider = gear[4].as_ref().is_some_and(|s| {
        let view = horizon_client::item_render::stack_view(s);
        let c = g.pack.item_renderer.models.components_view(&view);
        let chest = horizon_client::elytra_recast_chest::chest(c);
        chest.glider && !chest.next_damage_will_break
    });
    if m.extra.using_item > 0 && !m.extra.passenger {
        let slot = usize::from(m.extra.using_off_hand);
        if let Some(s) = gear[slot].as_ref() {
            let view = horizon_client::item_render::stack_view(s);
            let c = g.pack.item_renderer.models.components_view(&view);
            let effects = c.get("minecraft:use_effects");
            p.item_use_speed = Some(
                effects
                    .and_then(|e| e["speed_multiplier"].as_f64())
                    .unwrap_or(0.2) as f32,
            );
            p.item_use_slows_sprint = !effects
                .and_then(|e| e["can_sprint"].as_bool())
                .unwrap_or(false);
        }
    }
    let mut effects = PlayerEffects::default();
    for index in 0..m.extra.effects.len() {
        effects.add(m.effect_instance(index));
    }
    p.status = effects.status();
    p.mobility_restricted = effects.get("blindness").is_some();
    Some((p, effects))
}
pub(super) fn predict(g: &Game<'_>, id: i32, ticks: i32) -> Option<[f64; 3]> {
    let target = super::spear_kill_target::living(g)
        .into_iter()
        .find(|e| e.id == id)?;
    let mob = g.mobs.iter().find(|m| {
        m.id as i32 == id
            && horizon_client::command::args::normalize(&m.kind.id) == "minecraft:player"
    });
    let guest = g
        .lan
        .iter()
        .flat_map(|l| l.hub.guests.iter())
        .find(|p| p.placed && p.dimension == g.dim.id() && p.entity_id == id);
    if mob.is_none() && guest.is_none() {
        let d = target.position.subtract(target.previous);
        let t = f64::from(ticks);
        return Some([
            t.mul_add(d.x, target.position.x),
            t.mul_add(d.y, target.position.y),
            t.mul_add(d.z, target.position.z),
        ]);
    }
    let (mut body, mut effects) = if let Some(m) = mob {
        from_mob(g, m)?
    } else {
        let guest = guest?;
        let mut p = guest.motion.as_deref()?.clone();
        p.position = target.position;
        p.old_position = target.previous;
        p.velocity = target.position.subtract(target.previous);
        p.yaw = guest.yaw;
        p.pitch = guest.pitch;
        p.is_passenger = false;
        p.auto_jump_enabled = false;
        (p, guest.effects.clone())
    };
    let keys = guessed(&body);
    let mut scene = World::new(g.world.air, 0);
    scene.shape = g.world.shape;
    // Copy every loaded chunk: a target may be 200 blocks from the local body.
    for (&position, chunk) in &g.world.chunks {
        scene.chunks.insert(position, chunk.clone());
    }
    scene.moving_pistons = g.world.moving_pistons.clone();
    scene.shulkers = g.world.shulkers.clone();
    let entities = g.player_entity_collisions();
    let mut motion = horizon_client::motion::MotionWorld {
        entities: &entities,
        player: &mut body,
        scene: &mut scene,
        assets: &g.pack.assets,
        collision_binding: g.collision,
        resources: g.motion,
        health: &g.health_resources,
    };
    motion.liquidbounce_forecast_initialize().ok()?;
    for _ in 0..ticks.clamp(0, 30) {
        motion.player.status = effects.status();
        motion.player.mobility_restricted = effects.get("blindness").is_some();
        motion.liquidbounce_forecast_tick(keys).ok()?;
        let before = effects.modifiers();
        effects.tick();
        for (attr, id, _, _) in before {
            if !effects.modifiers().iter().any(|x| x.1 == id) {
                motion
                    .player
                    .set_modifier(attr, id, 0.0, "add_value", false)
                    .ok()?;
            }
        }
    }
    Some([body.position.x, body.position.y, body.position.z])
}
