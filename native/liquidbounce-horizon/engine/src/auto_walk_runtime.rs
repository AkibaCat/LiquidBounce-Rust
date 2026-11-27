// SPDX-License-Identifier: GPL-3.0-or-later
//! AutoWalk configuration and RotationManager bridge. The owning navigator is
//! in the Game crate, where actions can synchronously reach real game methods.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{Host, WorldPrimitives};
use liquidbounce_behaviors::{
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{DefaultRotationTarget, MovementCorrection, Rotation, movement_yaw},
};
use serde_json::{Value, json};

pub(crate) const ROTATION_PROVIDER: u64 = 8;
pub(crate) const PROCESSOR_BASE: u64 = 80;
#[derive(Clone, Debug, PartialEq)]
pub struct RenderSettings {
    pub enabled: bool,
    pub can_be_covered: bool,
    pub route: u32,
    pub search: u32,
    pub destination: u32,
    pub breaks: u32,
    pub places: u32,
    pub target: u32,
    pub invalid: u32,
    pub ghost: bool,
    pub fade_out: i32,
}
impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            can_be_covered: false,
            route: 0xff4677ff,
            search: 0xffffc83c,
            destination: 0x3c4677ff,
            breaks: 0xc8ff5555,
            places: 0xc855ff7f,
            target: 0xffff8c1a,
            invalid: 0xffd70909,
            ghost: true,
            fade_out: 800,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct AutoWalkSettings {
    pub mode: String,
    pub destination: [i32; 3],
    pub target: String,
    pub distance: f32,
    pub allow_break: bool,
    pub allow_place: bool,
    pub allow_parkour: bool,
    pub risky_jumps: bool,
    pub max_fall: i32,
    pub path_range: i32,
    pub allow_fly: bool,
    pub allow_elytra: bool,
    pub elytra_rockets: bool,
    pub allow_boat: bool,
    pub allow_eat: bool,
    pub rotations: RotationsSettings,
    pub render: RenderSettings,
    pub(crate) own_target: Option<DefaultRotationTarget>,
}
impl Default for AutoWalkSettings {
    fn default() -> Self {
        Self {
            mode: "Forward".into(),
            destination: [0; 3],
            target: String::new(),
            distance: 3.,
            allow_break: false,
            allow_place: false,
            allow_parkour: true,
            risky_jumps: false,
            max_fall: 3,
            path_range: 64,
            allow_fly: true,
            allow_elytra: true,
            elytra_rockets: true,
            allow_boat: false,
            allow_eat: false,
            rotations: Default::default(),
            render: Default::default(),
            own_target: None,
        }
    }
}
impl AutoWalkSettings {
    fn options(&self) -> Value {
        json!({"allowBreak":self.allow_break,"allowPlace":self.allow_place,"allowParkour":self.allow_parkour,"riskyJumps":self.risky_jumps,"maxFall":self.max_fall,"pathRange":self.path_range,"allowFly":self.allow_fly,"allowElytra":self.allow_elytra,"elytraRockets":self.elytra_rockets,"allowBoat":self.allow_boat,"allowEat":self.allow_eat})
    }
    pub(crate) fn processor_mode(processor: u64) -> Option<SmoothMode> {
        processor
            .checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
}
impl LiquidBounceHooks {
    /// Call after the tick's RotationManager update. The subsequent native
    /// KeyboardInput callback reads its current raw keys and performs navigation.
    pub(crate) fn auto_walk_prepare(&self, host: &mut dyn Host) {
        let [yaw, _] = host.client_player_rotation();
        let config = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            let settings = &modules.auto_walk;
            let mut rotations = modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let active =
                rotations.active_target(|p| modules.rotation_provider_running(&state.registry, p));
            let correction = active.map(|p| p.movement_correction);
            let move_yaw = movement_yaw(yaw, rotations.current, correction);
            let input_yaw =
                if correction == Some(MovementCorrection::Strict) && rotations.current.is_some() {
                    move_yaw
                } else {
                    yaw
                };
            let sent = rotations.server_rotation(false, false);
            let enabled = modules
                .query_ids
                .get("AutoWalk")
                .is_some_and(|id| state.registry.enabled_without_world(*id));
            json!({"enabled":enabled,"running":modules.query_running(&state.registry,"AutoWalk"),"mode":settings.mode,"destination":settings.destination,"target":settings.target,"distance":settings.distance,"inputYaw":input_yaw,"movementYaw":move_yaw,"sent":[sent.yaw,sent.pitch],"current":rotations.current.map(|r|[r.yaw,r.pitch]),"externalRotation":active.is_some()&&active!=settings.own_target,"isFlying":modules.query_running(&state.registry,"Fly"),"recastsElytra":modules.query_running(&state.registry,"ElytraRecast"),"scaffold":modules.query_ids.get("Scaffold").is_some_and(|id|state.registry.enabled_without_world(*id)),"options":settings.options()})
        };
        host.client_auto_walk_config(&config.to_string());
    }
    /// Call at the start of the existing directional-input handler. A true result
    /// asks the outer actor to disable the registry module after Goto arrives.
    pub(crate) fn auto_walk_result(&self, host: &dyn Host) -> bool {
        let Some(result) = host
            .client_auto_walk_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return false;
        };
        let state = self.state();
        let mut modules = state.behaviors.borrow_mut();
        if !modules.query_running(&state.registry, "AutoWalk") {
            return false;
        }
        if let Some(r) = result["request"]["rotation"]
            .as_array()
            .filter(|r| r.len() == 2)
        {
            let settings = &modules.auto_walk;
            let target = DefaultRotationTarget {
                rotation: Rotation::new(
                    r[0].as_f64().unwrap_or(0.) as f32,
                    r[1].as_f64().unwrap_or(0.) as f32,
                ),
                horizontal_turn_speed: 180.,
                vertical_turn_speed: 180.,
                ticks_until_reset: settings.rotations.ticks_until_reset,
                reset_threshold: settings.rotations.reset_threshold,
                consider_inventory: false,
                movement_correction: settings.rotations.movement_correction,
            };
            let processor = PROCESSOR_BASE + settings.rotations.active.processor_id();
            let priority = if result["request"]["purpose"] == "Movement" {
                -20
            } else {
                30
            };
            modules.auto_walk.own_target = Some(target);
            modules
                .rotations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .request_processed(ROTATION_PROVIDER, priority, target, processor, true);
        }
        #[cfg(feature = "webview")]
        if let Some(at) = result["arrived"].as_array() {
            let language = horizon_client::lang::code();
            let key = "liquidbounce.module.autoWalk.messages.arrived";
            let mut message = liquidbounce_interop::localization::describe(&language, key);
            for (i, v) in at.iter().enumerate() {
                message = message
                    .replacen("%s", &v.to_string(), 1)
                    .replace(&format!("{{{i}}}"), &v.to_string());
            }
            state.events.borrow_mut().push((
                "notification".into(),
                json!({"title":"AutoWalk","message":message,"severity":"SUCCESS"}),
            ));
        }
        !result["enabled"].as_bool().unwrap_or(true)
    }
    pub(crate) fn auto_walk_wants_sprint(&self, host: &dyn Host) -> bool {
        host.client_auto_walk_result()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .is_some_and(|v| {
                v["running"] == true && v["wantsSprint"] == true && v["usingItem"] != true
            })
    }
    pub(crate) fn auto_walk_render(
        &self,
        host: &dyn Host,
        partial: f32,
        primitives: &mut WorldPrimitives,
    ) {
        let settings = {
            let state = self.state();
            let modules = state.behaviors.borrow();
            if !modules.query_running(&state.registry, "AutoWalk")
                || !modules.auto_walk.render.enabled
            {
                return;
            }
            modules.auto_walk.render.clone()
        };
        let Some(v) = host
            .client_auto_walk_result()
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
