// SPDX-License-Identifier: GPL-3.0-or-later
//! Actual player replay and vanilla view vectors for AutoWindCharge.
use super::{
    Game,
    auto_walk_player::NativePlayer,
    auto_walk_simulation::NativeSimulation,
    auto_walk_world::{self as world, NativeLevel, NavigationRegistry},
};
use crate::navigation::{
    input::Input,
    walk_control::body::{SimulatedInput, WalkSimulation},
};
use serde_json::{Value, json};
impl Game<'_> {
    pub(super) fn auto_wind_charge_query(&mut self, text: &str) -> String {
        if self.player.is_none() {
            return "null".into();
        }
        let Ok(v) = serde_json::from_str::<Value>(text) else {
            return "null".into();
        };
        let rotation = |field: &str| -> Option<[f32; 2]> {
            Some([v[field][0].as_f64()? as f32, v[field][1].as_f64()? as f32])
        };
        let (Some(server), Some(target)) = (rotation("server"), rotation("target")) else {
            return "null".into();
        };
        let a = super::client_rotation_direction(server[0], server[1]);
        let b = super::client_rotation_direction(target[0], target[1]);
        let cross = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let length = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let angle = length
            .atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
            .to_degrees() as f32;
        let ticks = v["ticks"].as_i64().unwrap_or(0) as i32;
        if ticks <= 0 {
            return json!({"angle":angle,"landing":null}).to_string();
        }
        world::with_scope(self, || {
            let epoch = world::read(world::current_epoch);
            let registry = world::read(NavigationRegistry::capture);
            let player = NativePlayer::new(NativeLevel::new(registry, epoch));
            let key = |i: usize| v["keys"][i] == true;
            let mut input = SimulatedInput::new(Input {
                forward: key(0),
                backward: key(1),
                left: key(2),
                right: key(3),
                jump: false,
                shift: key(5),
                sprint: key(6),
            });
            input.force_stay_on_ground_surface =
                horizon_client::navigation_physics::staying_on_surface(false);
            let yaw = v["movementYaw"].as_f64().unwrap_or(0.) as f32;
            let mut simulation = NativeSimulation::new(&player, input, yaw);
            for tick in 0..ticks {
                let before = simulation.predicted_position();
                simulation.tick();
                if simulation.player().on_ground() {
                    let block = simulation
                        .player()
                        .read(|p| p.ground.supporting_block.map(|p| [p.x, p.y, p.z]));
                    return json!({"angle":angle,"landing":{"block":block,"tick":tick,"before":[before.x,before.y,before.z]}}).to_string();
                }
            }
            json!({"angle":angle,"landing":null}).to_string()
        })
    }
}
