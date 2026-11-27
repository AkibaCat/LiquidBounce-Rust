// SPDX-License-Identifier: GPL-3.0-or-later
//! Rotation/config bridge for the original Parkour and SafeWalk game handlers.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{Host, WorldPrimitives};
use liquidbounce_behaviors::{
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, MovementCorrection, Rotation, movement_yaw},
};
use serde_json::{Value, json};
pub(crate) const ROTATION_PROVIDER: u64 = 9;
pub(crate) const PROCESSOR_BASE: u64 = 90;
#[derive(Clone, Debug, PartialEq)]
pub struct ParkourRender {
    pub enabled: bool,
    pub can_be_covered: bool,
    pub arc: u32,
    pub takeoff: u32,
    pub landing: u32,
    pub aim: u32,
}
impl Default for ParkourRender {
    fn default() -> Self {
        Self {
            enabled: true,
            can_be_covered: false,
            arc: 0xdcffffff,
            takeoff: 0xffffc83c,
            landing: 0xff55ff7f,
            aim: 0xff4677ff,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParkourSettings {
    pub assist: bool,
    pub max_angle: f32,
    pub rotations: RotationsSettings,
    pub render: ParkourRender,
}
impl Default for ParkourSettings {
    fn default() -> Self {
        Self {
            assist: true,
            max_angle: 35.,
            rotations: Default::default(),
            render: Default::default(),
        }
    }
}
impl ParkourSettings {
    pub(crate) fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SafeWalkSettings {
    pub mode: String,
    pub distance: [f32; 2],
    pub keep: [i32; 2],
    pub on_edge_mode: String,
    pub sneak: [i32; 2],
    pub jump: bool,
}
impl Default for SafeWalkSettings {
    fn default() -> Self {
        Self {
            mode: "Safe".into(),
            distance: [0.1, 0.15],
            keep: [1, 2],
            on_edge_mode: "Stop".into(),
            sneak: [0, 0],
            jump: false,
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn edge_assistance_prepare(&self, host: &mut dyn Host) {
        let [yaw, _] = host.client_player_rotation();
        let physical_jump = host.client_motion_state()[2] != 0.;
        let value = {
            let state = self.state();
            let m = state.behaviors.borrow();
            let mut r = m
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let correction = r
                .active_target(|p| m.rotation_provider_running(&state.registry, p))
                .map(|t| t.movement_correction);
            let p = &m.parkour;
            let s = &m.safe_walk;
            json!({"movementYaw":movement_yaw(yaw,r.current,correction),"currentYaw":r.current.map(|r|r.yaw),"silent":correction==Some(MovementCorrection::Silent),"physicalJump":physical_jump,"parkour":{"running":m.query_running(&state.registry,"Parkour"),"assist":p.assist,"maxAngle":p.max_angle,"render":p.render.enabled},"eagle":{"running":m.query_running(&state.registry,"Eagle"),"distance":m.eagle.distance,"conditional":m.eagle.conditional,"conditions":m.eagle.conditions,"pitch":m.eagle.pitch},"safeWalk":{"running":m.query_running(&state.registry,"SafeWalk"),"mode":s.mode,"distance":s.distance,"keep":s.keep,"onEdgeMode":s.on_edge_mode,"sneak":s.sneak,"jump":s.jump}})
        };
        host.client_edge_assistance_config(&value.to_string());
    }
    pub(crate) fn edge_assistance_result(&self, host: &dyn Host) -> bool {
        let Some(v) = host
            .client_edge_assistance_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return false;
        };
        let [_, pitch] = host.client_player_rotation();
        let state = self.state();
        let m = state.behaviors.borrow();
        if !m.query_running(&state.registry, "Parkour") {
            return false;
        }
        if let Some(yaw) = v["aimYaw"].as_f64() {
            let s = &m.parkour.rotations;
            let target = DefaultRotationTarget {
                rotation: Rotation::new(yaw as f32, pitch),
                horizontal_turn_speed: 180.,
                vertical_turn_speed: 180.,
                ticks_until_reset: s.ticks_until_reset,
                reset_threshold: s.reset_threshold,
                consider_inventory: false,
                movement_correction: s.movement_correction,
            };
            m.rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .request_processed(
                    ROTATION_PROVIDER,
                    -20,
                    target,
                    PROCESSOR_BASE + s.active.processor_id(),
                    true,
                );
        }
        v["sprint"] == true
    }
    pub(crate) fn edge_assistance_render(&self, host: &dyn Host, primitives: &mut WorldPrimitives) {
        let settings = {
            let state = self.state();
            let m = state.behaviors.borrow();
            if !m.query_running(&state.registry, "Parkour")
                || !m.parkour.assist
                || !m.parkour.render.enabled
            {
                return;
            }
            m.parkour.render.clone()
        };
        let Some(v) = host
            .client_edge_assistance_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        fn point(v: &Value) -> Option<[f64; 3]> {
            let p = v.as_array()?;
            Some([
                p.first()?.as_f64()?,
                p.get(1)?.as_f64()?,
                p.get(2)?.as_f64()?,
            ])
        }
        let Some(aim) = point(&v["target"]) else {
            return;
        };
        let through = !settings.can_be_covered;
        let eye = [aim[0], aim[1] + f64::from(1.62_f32), aim[2]];
        for (a, b) in [
            (aim, eye),
            (
                [eye[0] - 0.15, eye[1], eye[2]],
                [eye[0] + 0.15, eye[1], eye[2]],
            ),
            (
                [eye[0], eye[1], eye[2] - 0.15],
                [eye[0], eye[1], eye[2] + 0.15],
            ),
        ] {
            primitives.line(a, b, settings.aim, 1.).through_walls = through;
        }
        for (index, arc) in v["arcs"].as_array().into_iter().flatten().enumerate() {
            let path = arc["path"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(point)
                .collect::<Vec<_>>();
            for line in path.windows(2) {
                let mut a = line[0];
                let mut b = line[1];
                a[1] += 0.05;
                b[1] += 0.05;
                primitives.line(a, b, settings.arc, 1.).through_walls = through;
            }
            if index == 0
                && v["runUp"] == true
                && let (Some(start), Some(position)) = (path.first(), point(&v["position"]))
            {
                primitives
                    .line(
                        [position[0], position[1] + 0.05, position[2]],
                        [start[0], start[1] + 0.05, start[2]],
                        settings.takeoff & 0xffffff | ((settings.takeoff >> 24) / 2) << 24,
                        1.,
                    )
                    .through_walls = through;
                primitives
                    .filled_box(
                        [start[0] - 0.15, start[1], start[2] - 0.15],
                        [start[0] + 0.15, start[1] + 0.02, start[2] + 0.15],
                        settings.takeoff & 0xffffff | 60 << 24,
                    )
                    .through_walls = through;
                primitives
                    .box_outline(
                        [start[0] - 0.15, start[1], start[2] - 0.15],
                        [start[0] + 0.15, start[1] + 0.02, start[2] + 0.15],
                        settings.takeoff,
                        1.,
                    )
                    .through_walls = through;
            }
            if let Some(takeoff) = arc["takeoff"].as_u64().filter(|i| *i > 0) {
                let mut offset = 0.75;
                for line in path.windows(2).take(takeoff as usize) {
                    let from = [line[0][0], line[0][1] + 0.05, line[0][2]];
                    let delta: [f64; 3] = std::array::from_fn(|i| line[1][i] - line[0][i]);
                    let length = delta.iter().map(|v| v * v).sum::<f64>().sqrt();
                    if length < 1e-4 {
                        continue;
                    }
                    let d = delta.map(|v| v / length);
                    let horizontal = (d[0] * d[0] + d[2] * d[2]).sqrt();
                    let side = if horizontal * horizontal < 1e-4 {
                        [1., 0., 0.]
                    } else {
                        [-d[2] / horizontal, 0., d[0] / horizontal]
                    };
                    while offset < length {
                        let tip: [f64; 3] =
                            std::array::from_fn(|i| from[i] + (offset + 0.2) * d[i]);
                        for sign in [-1., 1.] {
                            primitives
                                .line(
                                    std::array::from_fn(|i| {
                                        from[i] + (offset - 0.2) * d[i] + sign * 0.2 * side[i]
                                    }),
                                    tip,
                                    settings.arc,
                                    1.,
                                )
                                .through_walls = through;
                        }
                        offset += 1.5;
                    }
                    offset -= length;
                }
            }
            if let Some(p) = point(&arc["landing"]) {
                let max = [p[0] + 1., p[1] + 0.02, p[2] + 1.];
                primitives
                    .filled_box(p, max, settings.landing & 0xffffff | 60 << 24)
                    .through_walls = through;
                primitives
                    .box_outline(p, max, settings.landing, 1.)
                    .through_walls = through;
            }
            if let Some(p) = path.last() {
                primitives
                    .box_outline(
                        [p[0] - 0.3, p[1], p[2] - 0.3],
                        [p[0] + 0.3, p[1] + 0.02, p[2] + 0.3],
                        settings.landing,
                        1.,
                    )
                    .through_walls = through;
            }
            if let Some(index) = arc["takeoff"].as_u64().map(|n| n as usize)
                && let (Some(p), Some(next)) = (path.get(index), path.get(index + 1))
            {
                let dx = next[0] - p[0];
                let dz = next[2] - p[2];
                let length = (dx * dx + dz * dz).sqrt();
                if length >= 1e-4 {
                    primitives
                        .line(
                            [
                                p[0] - dz / length * 0.5,
                                p[1] + 0.05,
                                p[2] + dx / length * 0.5,
                            ],
                            [
                                p[0] + dz / length * 0.5,
                                p[1] + 0.05,
                                p[2] - dx / length * 0.5,
                            ],
                            settings.takeoff,
                            1.,
                        )
                        .through_walls = through;
                    primitives
                        .filled_box(
                            [p[0] - 0.15, p[1], p[2] - 0.15],
                            [p[0] + 0.15, p[1] + 0.02, p[2] + 0.15],
                            settings.takeoff & 0xffffff | 60 << 24,
                        )
                        .through_walls = through;
                    primitives
                        .box_outline(
                            [p[0] - 0.15, p[1], p[2] - 0.15],
                            [p[0] + 0.15, p[1] + 0.02, p[2] + 0.15],
                            settings.takeoff,
                            1.,
                        )
                        .through_walls = through;
                }
            }
        }
    }
}
