// SPDX-License-Identifier: GPL-3.0-or-later
//! Original FreeCam events, independent Navigator and RotationManager provider.
use crate::{
    auto_walk_runtime::AutoWalkSettings,
    runtime::{LiquidBounceHooks, raw_options},
};
use horizon_client::mod_api::{Host, WorldPrimitives};
use liquidbounce_behaviors::{
    combat_rotations,
    free_cam::FreeCam,
    rotations::{DefaultRotationTarget, MovementCorrection, Rotation, movement_yaw},
};
use serde_json::{Value, json};
pub const ROTATION_PROVIDER: u64 = 30;
pub const PROCESSOR_BASE: u64 = 5000;
#[derive(Clone, Debug, PartialEq)]
pub struct FreeCamRuntime {
    pub behavior: FreeCam,
    pub rotations: combat_rotations::Settings,
    pub navigation: AutoWalkSettings,
    pub navigation_enabled: bool,
    pub navigation_key: i32,
}
impl Default for FreeCamRuntime {
    fn default() -> Self {
        let mut navigation = AutoWalkSettings::default();
        navigation.allow_break = true;
        navigation.allow_place = true;
        Self {
            behavior: Default::default(),
            rotations: Default::default(),
            navigation,
            navigation_enabled: false,
            navigation_key: 341,
        }
    }
}
impl FreeCamRuntime {
    pub fn clear(&mut self) {
        self.behavior.disabled();
        self.navigation.own_target = None;
    }
}
fn array<const N: usize>(v: &Value) -> Option<[f64; N]> {
    let a = v.as_array().filter(|a| a.len() == N)?;
    let mut out = [0.; N];
    for (i, v) in a.iter().enumerate() {
        out[i] = v.as_f64()?;
    }
    Some(out)
}
impl LiquidBounceHooks {
    fn free_cam_process(&self, input: &Value) -> Value {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let running = b.query_running(&s.registry, "FreeCam");
        let f = &mut b.free_cam.behavior;
        if running {
            if let (Some(eye), Some(rotation)) = (
                array::<3>(&input["eye"]),
                array::<2>(&input["playerRotation"]),
            ) {
                f.initialize(eye, rotation.map(|v| v as f32));
            }
        }
        match input["phase"].as_str().unwrap_or("state") {
            "tick" if f.active(running) => {
                let keys = std::array::from_fn(|i| input["keys"][i].as_bool().unwrap_or(false));
                if !f.cancel(running, 3, input["liquid"] == true) {
                    f.tick(keys, input["grabbed"] == true);
                }
            }
            "damage" => {
                f.cancel(
                    running,
                    0,
                    input["health"]
                        .as_f64()
                        .zip(input["previous"].as_f64())
                        .is_some_and(|(a, b)| a < b),
                );
            }
            "teleport" => {
                f.cancel(running, 1, true);
            }
            "move" => {
                f.cancel(
                    running,
                    2,
                    input["x"].as_f64().unwrap_or(0.).abs() > 0.
                        || input["z"].as_f64().unwrap_or(0.).abs() > 0.,
                );
            }
            "world" => {
                if running {
                    f.position.clear();
                    f.initialize = false;
                    f.cancel_pending = true;
                }
            }
            "middle" => {
                if let Some(target) = array::<3>(&input["target"]) {
                    f.middle_click(running, target);
                }
            }
            _ => {}
        }
        let f = &b.free_cam;
        let active = f.behavior.active(running);
        let partial = input["partial"].as_f64().unwrap_or(1.) as f32;
        let p = f.behavior.position;
        let manager = b
            .rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let server = manager.server_rotation(
            b.criticals.is_lagging(),
            b.query_running(&s.registry, "Freeze"),
        );
        json!({"active":active,"position":p.interpolate(partial),"rotation":p.interpolate_rotation(partial),
            "currentPosition":p.pos,"lastPosition":p.last_pos,"currentRotation":p.rot,"lastRotation":p.last_rot,
            "cameraInteract":f.behavior.settings.camera_interact,"keepSneaking":f.behavior.settings.keep_sneaking,
            "midClick":f.behavior.settings.mid_click_camera_teleport,"navigation":f.navigation_enabled,
            "key":f.navigation_key,"managed":manager.current.is_some(),"serverRotation":[server.yaw,server.pitch]})
    }
    pub(crate) fn free_cam(&self, input: &str, output: &mut String) {
        let Ok(input) = serde_json::from_str::<Value>(input) else {
            return;
        };
        *output = self.free_cam_process(&input).to_string();
    }
    pub(crate) fn free_cam_policy(&self, kind: u8, x: f64, z: f64, flags: &mut u32) {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let running = b.query_running(&s.registry, "FreeCam");
        if kind == 1 {
            b.free_cam
                .behavior
                .cancel(running, 2, x.abs() > 0. || z.abs() > 0.);
        }
        if !b.free_cam.behavior.active(running) {
            return;
        }
        *flags |= 1;
        if b.free_cam.behavior.settings.camera_interact {
            *flags |= 2;
        }
        if b.rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current
            .is_some()
        {
            *flags |= 4;
        }
    }
    pub(crate) fn free_cam_prepare(&self, host: &mut dyn Host) {
        let context = host
            .client_free_cam_context()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({"phase":"state"}));
        let state = self.free_cam_process(&context);
        let cancel = self
            .state()
            .behaviors
            .borrow()
            .free_cam
            .behavior
            .cancel_pending;
        if cancel {
            let options = raw_options(host);
            let effects = {
                let mut s = self.state();
                let id = s.behaviors.borrow().query_ids.get("FreeCam").copied();
                if let Some(id) = id {
                    let _ = s.with_registry(options, |r, c| r.set_enabled_sync(id, false, c));
                }
                s.take_effects()
            };
            crate::runtime::apply_effects(self, host, effects);
            return;
        }
        if state["active"] != true {
            return;
        }
        let going = state["navigation"] == true
            && host.client_keyboard_scan_down(state["key"].as_i64().unwrap_or(-1) as i32);
        let nav = host
            .client_free_cam_navigation_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
        let target = if going {
            nav.as_ref()
                .and_then(|v| array::<2>(&v["movementRotation"]))
                .map(|r| Rotation::new(r[0] as f32, r[1] as f32))
        } else {
            let look = {
                let s = self.state();
                let b = s.behaviors.borrow();
                b.free_cam.behavior.settings.camera_interact && b.free_cam.behavior.settings.look_at
            };
            if !look {
                None
            } else {
                array::<3>(&context["crosshair"])
                    .zip(array::<3>(&context["eye"]))
                    .map(|(to, from)| {
                        let [x, y, z] = std::array::from_fn(|i| to[i] - from[i]);
                        Rotation::new(
                            (z.atan2(x).to_degrees() - 90.) as f32,
                            -y.atan2((x * x + z * z).sqrt()).to_degrees() as f32,
                        )
                    })
            }
        };
        if let Some(rotation) = target {
            self.free_cam_request(rotation);
        }
    }
    fn free_cam_request(&self, rotation: Rotation) {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &b.free_cam.rotations.basic;
        let target = DefaultRotationTarget {
            rotation,
            horizontal_turn_speed: 180.,
            vertical_turn_speed: 180.,
            ticks_until_reset: r.ticks_until_reset,
            reset_threshold: r.reset_threshold,
            consider_inventory: true,
            movement_correction: r.movement_correction,
        };
        let processor = PROCESSOR_BASE + b.free_cam.rotations.processor_id();
        b.free_cam.navigation.own_target = Some(target);
        b.rotations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .request_processed(ROTATION_PROVIDER, -100, target, processor, true);
    }
    pub(crate) fn free_cam_navigation(&self, host: &mut dyn Host) {
        let state = self.free_cam_process(&json!({"phase":"state"}));
        let going = state["active"] == true
            && state["navigation"] == true
            && host.client_keyboard_scan_down(state["key"].as_i64().unwrap_or(-1) as i32);
        let goal = if going {
            array::<3>(&state["currentPosition"])
                .zip(array::<2>(&state["currentRotation"]))
                .and_then(|(p, r)| host.client_free_cam_trace(p, r.map(|v| v as f32)))
        } else {
            None
        };
        let [yaw, _] = host.client_player_rotation();
        let config = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let f = &b.free_cam;
            let n = &f.navigation;
            let mut rotations = b
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let active = rotations.active_target(|p| b.rotation_provider_running(&s.registry, p));
            let correction = active.map(|p| p.movement_correction);
            let move_yaw = movement_yaw(yaw, rotations.current, correction);
            let input_yaw =
                if correction == Some(MovementCorrection::Strict) && rotations.current.is_some() {
                    move_yaw
                } else {
                    yaw
                };
            let sent = rotations.server_rotation(
                b.criticals.is_lagging(),
                b.query_running(&s.registry, "Freeze"),
            );
            json!({"enabled":state["active"]==true&&f.navigation_enabled,"running":going,"goal":goal,"inputYaw":input_yaw,"movementYaw":move_yaw,"sent":[sent.yaw,sent.pitch],"current":rotations.current.map(|r|[r.yaw,r.pitch]),"externalRotation":active.is_some()&&active!=n.own_target,"isFlying":b.query_running(&s.registry,"Fly"),"recastsElytra":b.query_running(&s.registry,"ElytraRecast"),"scaffold":b.query_ids.get("Scaffold").is_some_and(|id|s.registry.enabled_without_world(*id)),"options":{"allowBreak":n.allow_break,"allowPlace":n.allow_place,"allowParkour":n.allow_parkour,"riskyJumps":n.risky_jumps,"maxFall":n.max_fall,"pathRange":n.path_range,"allowFly":n.allow_fly,"allowElytra":n.allow_elytra,"elytraRockets":n.elytra_rockets,"allowBoat":n.allow_boat,"allowEat":n.allow_eat}})
        };
        host.client_free_cam_navigation_config(&config.to_string());
    }
    pub(crate) fn free_cam_navigation_result(&self, host: &dyn Host) -> bool {
        let Some(result) = host
            .client_free_cam_navigation_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return false;
        };
        if result["running"] != true {
            return false;
        }
        if let Some(r) = array::<2>(&result["request"]["rotation"]) {
            self.free_cam_request(Rotation::new(r[0] as f32, r[1] as f32));
        }
        result["wantsSprint"] == true && result["usingItem"] != true
    }
}

impl LiquidBounceHooks {
    pub(crate) fn free_cam_render(
        &self,
        host: &dyn Host,
        partial: f32,
        primitives: &mut WorldPrimitives,
    ) {
        let settings = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            if !modules.query_running(&state.registry, "FreeCam")
                || !modules.free_cam.navigation_enabled
                || !modules.free_cam.navigation.render.enabled
            {
                return;
            }
            modules.free_cam.navigation.render.clone()
        };
        let Some(v) = host
            .client_free_cam_navigation_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let route = &v["route"];
        let through = !settings.can_be_covered;
        fn point(v: &Value) -> Option<[f64; 3]> {
            let a = v.as_array()?;
            Some([
                a.first()?.as_f64()?,
                a.get(1)?.as_f64()?,
                a.get(2)?.as_f64()?,
            ])
        }
        fn alpha(c: u32, a: u32) -> u32 {
            c & 0x00ffffff | (a.min(255) << 24)
        }
        fn fade(c: u32, f: f32) -> u32 {
            alpha(c, (((c >> 24) as f32) * f).clamp(0., 255.) as u32)
        }
        fn poly(
            p: &mut WorldPrimitives,
            points: &[[f64; 3]],
            color: u32,
            through: bool,
            arrows: bool,
        ) {
            let mut offset = 0.75;
            for line in points.windows(2) {
                let mut from = line[0];
                let mut to = line[1];
                from[1] += 0.05;
                to[1] += 0.05;
                p.line(from, to, color, 1.).through_walls = through;
                if arrows {
                    let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
                    let length = delta.iter().map(|x| x * x).sum::<f64>().sqrt();
                    if length < 1e-4 {
                        continue;
                    }
                    let d = delta.map(|x| x / length);
                    let horizontal = (d[0] * d[0] + d[2] * d[2]).sqrt();
                    let side = if horizontal * horizontal < 1e-4 {
                        [1., 0., 0.]
                    } else {
                        [-d[2] / horizontal, 0., d[0] / horizontal]
                    };
                    while offset < length {
                        let tip = std::array::from_fn(|i| from[i] + (offset + 0.2) * d[i]);
                        let back: [f64; 3] =
                            std::array::from_fn(|i| from[i] + (offset - 0.2) * d[i]);
                        for sign in [-1., 1.] {
                            p.line(
                                std::array::from_fn(|i| back[i] + sign * 0.2 * side[i]),
                                tip,
                                color,
                                1.,
                            )
                            .through_walls = through;
                        }
                        offset += 1.5;
                    }
                    offset -= length;
                }
            }
        }
        let state = route["state"].as_str().unwrap_or("");
        if state.is_empty() {
            return;
        }
        let now = crate::behaviors::current_time_millis();
        let factor = if matches!(state, "DONE" | "INVALID") {
            if settings.fade_out == 0 {
                0.
            } else {
                1. - (now - route["since"].as_i64().unwrap_or(now)) as f32
                    / settings.fade_out as f32
            }
        } else {
            1.
        };
        if factor <= 0. {
            return;
        }
        let path = route["points"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(point)
            .collect::<Vec<_>>();
        let color = fade(
            if state == "INVALID" {
                settings.invalid
            } else {
                settings.route
            },
            factor,
        );
        if state == "PENDING" {
            if let (Some(start), Some(goal)) = (point(&route["start"]), point(&route["goal"])) {
                let reached = path.last().copied().unwrap_or(start);
                poly(
                    primitives,
                    &[reached, goal],
                    alpha(settings.search, 90),
                    through,
                    false,
                );
                poly(primitives, &path, settings.search, through, true);
                let phase = (now % 800) as f64 / 800. * std::f64::consts::TAU;
                let pulse = ((phase.sin() + 1.) / 2. * 160.) as u32;
                let min = [reached[0] - 0.15, reached[1], reached[2] - 0.15];
                let max = [reached[0] + 0.15, reached[1] + 0.3, reached[2] + 0.15];
                primitives
                    .filled_box(min, max, alpha(settings.search, pulse))
                    .through_walls = through;
                primitives
                    .box_outline(min, max, settings.search, 1.)
                    .through_walls = through;
            }
        } else {
            poly(
                primitives,
                &path,
                alpha(color, (color >> 24) / 3),
                through,
                false,
            );
            let mut ahead = Vec::new();
            if let Some(mut head) = point(&route["head"]) {
                if let Some(old) = point(&route["previousHead"]) {
                    for i in 0..3 {
                        head[i] = old[i] + (head[i] - old[i]) * f64::from(partial);
                    }
                }
                ahead.push(head);
            }
            ahead.extend(
                route["ahead"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(point),
            );
            if !ahead.is_empty() {
                poly(primitives, &ahead, color, through, true);
            } else if state == "ACTIVE" {
                poly(primitives, &path, color, through, true);
            }
        }
        if state == "ACTIVE" {
            for (key, color) in [("breaks", settings.breaks), ("places", settings.places)] {
                for p in route[key]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(point)
                {
                    let max = [p[0] + 1., p[1] + 1., p[2] + 1.];
                    primitives
                        .filled_box(p, max, alpha(color, (color >> 24) / 4))
                        .through_walls = through;
                    primitives.box_outline(p, max, color, 1.).through_walls = through;
                }
            }
        }
        if let Some(p) = point(&route["goal"]) {
            let marker = fade(
                if state == "INVALID" {
                    settings.invalid
                } else {
                    settings.destination
                },
                factor,
            );
            let min = [p[0] - 0.5, p[1], p[2] - 0.5];
            let max = [p[0] + 0.5, p[1] + 0.02, p[2] + 0.5];
            primitives.filled_box(min, max, marker).through_walls = through;
            primitives
                .box_outline(min, max, alpha(marker, 255), 1.)
                .through_walls = through;
            primitives
                .box_outline(
                    [p[0] - 0.3, p[1], p[2] - 0.3],
                    [p[0] + 0.3, p[1] + 1.8, p[2] + 0.3],
                    alpha(marker, 160),
                    1.,
                )
                .through_walls = through;
        }
    }
}
