// SPDX-License-Identifier: GPL-3.0-or-later
//! Runtime bridge for ModuleSpeed.  The movement model stays in
//! `liquidbounce-behaviors`; this file only translates Horizon facts/events.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientDirectionalInputEvent, ClientPlayerJumpEvent, Host};
use liquidbounce_behaviors::speed::{self, Context, Settings};
use serde_json::{Value, json};

#[derive(Default)]
pub struct SpeedRuntime {
    pub settings: Settings,
    pub state: speed::Speed,
}

fn context(value: &Value, input: u8) -> Context {
    let number = |name: &str| value[name].as_f64().unwrap_or(0.0);
    let boolean = |name: &str| value[name].as_bool().unwrap_or(false);
    let velocity = value["velocity"].as_array().map(|v| [
        v.first().and_then(Value::as_f64).unwrap_or(0.0),
        v.get(1).and_then(Value::as_f64).unwrap_or(0.0),
        v.get(2).and_then(Value::as_f64).unwrap_or(0.0),
    ]).unwrap_or([0.0; 3]);
    Context {
        in_game: boolean("inGame"),
        ground: boolean("ground"),
        velocity,
        yaw: number("yaw") as f32,
        input,
        physical_jump: boolean("physicalJump"),
        sprinting: boolean("sprinting"),
        sneaking: boolean("sneaking"),
        using_item: boolean("usingItem"),
        fall_flying: boolean("fallFlying"),
        in_liquid: boolean("inLiquid"),
        fly: boolean("fly"),
        scaffold: boolean("scaffold"),
        scaffold_auto_speed: boolean("scaffoldAutoSpeed"),
        towering: boolean("towering"),
        combat: boolean("combat"),
        speed_level: value["speedLevel"].as_i64().unwrap_or(0) as i32,
        slowness_level: value["slownessLevel"].as_i64().unwrap_or(0) as i32,
        jump_level: value["jumpLevel"].as_i64().unwrap_or(0) as i32,
        hurt: value["hurt"].as_i64().unwrap_or(0) as i32,
        fall_distance: number("fallDistance"),
        fly_distance: number("flyDistance") as f32,
        horizontal_collision: boolean("horizontalCollision"),
        bottom_collision: boolean("bottomCollision"),
        ground_exempt: boolean("groundExempt"),
        ground_within: boolean("groundWithin"),
        jump_blocked: boolean("jumpBlocked"),
        leather_boots: boolean("leatherBoots"),
        collisions: value["collisions"].as_array().map(|v| [
            v.first().and_then(Value::as_i64).unwrap_or(0) as i32,
            v.get(1).and_then(Value::as_i64).unwrap_or(0) as i32,
            v.get(2).and_then(Value::as_i64).unwrap_or(0) as i32,
        ]).unwrap_or([0; 3]),
        now: value["now"].as_i64().unwrap_or(0),
    }
}

fn input(event: &ClientDirectionalInputEvent) -> u8 {
    u8::from(event.forward) | (u8::from(event.backward) << 1)
        | (u8::from(event.left) << 2) | (u8::from(event.right) << 3)
}

impl LiquidBounceHooks {
    pub(crate) fn speed_input(&self, host: &mut dyn Host, event: &mut ClientDirectionalInputEvent) {
        let (settings, running) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            (b.speed.settings.clone(), b.query_running(&state.registry, "Speed"))
        };
        if !running { return; }
        let facts = serde_json::from_str::<Value>(&host.client_speed_facts()).unwrap_or(Value::Null);
        let mut c = context(&facts, input(event));
        c.physical_jump = event.jump;
        let state = self.state();
        let mut b = state.behaviors.borrow_mut();
        b.speed.state.input(&settings, c, &mut event.jump, &mut event.sprint);
    }

    pub(crate) fn speed_tick(&self, host: &mut dyn Host) {
        let (settings, running, combat, scaffold_auto_speed) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let scaffold_running = b.query_running(&state.registry, "Scaffold");
            (b.speed.settings.clone(), b.query_running(&state.registry, "Speed"),
                state.combat.in_combat(), b.scaffold.auto_speed_running(scaffold_running))
        };
        let facts = serde_json::from_str::<Value>(&host.client_speed_facts()).unwrap_or(Value::Null);
        let mut c = context(&facts, facts["input"].as_u64().unwrap_or(0) as u8);
        c.combat = combat;
        c.scaffold_auto_speed = scaffold_auto_speed;
        let (effects, velocity) = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.speed.state.synchronize(&settings, running, &mut c);
            b.speed.state.tick(&settings, &mut c);
            let effects = b.speed.state.effects.clone();
            for timer in &effects.timers {
                b.timer.state.lock().expect("Timer poisoned").timer.request_speed(
                    timer.speed, timer.priority, crate::timer_runtime::TIMER_PROVIDER, timer.ticks,
                );
            }
            (effects, c.velocity)
        };
        horizon_client::target_strafe_bridge::set_hypixel_should_strafe(effects.hypixel_should_strafe);
        let _ = host.client_speed_apply(&json!({
            "velocity": velocity,
            "jumpRepetitions": effects.jump_repetitions,
            "packets": effects.packets.iter().map(|p| json!({"y":p.y_offset,"ground":p.ground,"full":p.full})).collect::<Vec<_>>(),
            "disable": effects.disable_speed,
        }).to_string());
    }

    pub(crate) fn speed_jump(&self, host: &dyn Host, event: &mut ClientPlayerJumpEvent) {
        let (settings, running) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            (b.speed.settings.clone(), b.query_running(&state.registry, "Speed"))
        };
        if !running { return; }
        let facts = serde_json::from_str::<Value>(&host.client_speed_facts()).unwrap_or(Value::Null);
        let mut c = context(&facts, facts["input"].as_u64().unwrap_or(0) as u8);
        c.physical_jump = true;
        let mut power = 0.42_f32;
        let allowed = {
            let state = self.state();
            let mut b = state.behaviors.borrow_mut();
            b.speed.state.synchronize(&settings, running, &mut c);
            let allowed = b.speed.state.jump_before(&settings, &mut c, &mut power);
            b.speed.state.jump_after(&settings, &mut c);
            allowed
        };
        // The 26.3 jump hook exposes only `on_ground` through an immutable
        // Host. Movement application runs in the mutable tick phase.
        let _ = (power, allowed, event.on_ground);
    }
}
