// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    runtime::LiquidBounceHooks,
    scaffold_runtime::{self as runtime, ScaffoldRuntime, array, feature_query, number},
};
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host};
use liquidbounce_behaviors::{
    clicking::Technique,
    no_rotate_set::{RotationsSettings, SmoothMode},
    rotations::{Rotation, wrap_degrees},
    scaffold_features as features,
};
use serde_json::{Value, json};
fn input(e: &ClientDirectionalInputEvent) -> [bool; 4] {
    [e.forward, e.backward, e.left, e.right]
}
fn set_input(e: &mut ClientDirectionalInputEvent, i: [bool; 4]) {
    [e.forward, e.backward, e.left, e.right] = i;
}
fn moving_yaw(mut yaw: f32, i: [bool; 4]) -> f32 {
    let m = if i[1] && !i[0] {
        yaw += 180.;
        -0.5
    } else if i[0] && !i[1] {
        0.5
    } else {
        1.
    };
    if i[2] && !i[3] {
        yaw -= 90. * m
    }
    if i[3] && !i[2] {
        yaw += 90. * m
    }
    yaw
}
fn strafe(yaw: f32, i: [bool; 4], speed: f64, x: &mut f64, z: &mut f64) {
    if !features::moving(i) {
        *x = 0.;
        *z = 0.;
        return;
    }
    let a = f64::from(moving_yaw(yaw, i)).to_radians();
    *x = -horizon_client::scaffold_math::sin(a) * speed;
    *z = horizon_client::scaffold_math::cos(a) * speed;
}
fn tower_settings(r: &ScaffoldRuntime) -> features::TowerSettings {
    use features::TowerMode as M;
    let mode = match r.tag("Tower") {
        "Motion" => M::Motion,
        "Pulldown" => M::Pulldown,
        "Karhu-2024-05-10" => M::Karhu,
        "Vulcan-2.8.8" => M::Vulcan,
        "Hypixel-2026-07-12" => M::Hypixel,
        _ => M::None,
    };
    features::TowerSettings {
        mode,
        motion: r.float("Tower.Motion.Motion"),
        height: r.float("Tower.Motion.TriggerHeight"),
        slow: r.float("Tower.Motion.Slow"),
        trigger: r.float(if mode == M::Karhu {
            "Tower.Karhu-2024-05-10.Trigger"
        } else {
            "Tower.Pulldown.Trigger"
        }),
        timer: r.float("Tower.Karhu-2024-05-10.Timer"),
        pulldown: r.flag("Tower.Karhu-2024-05-10.Pulldown"),
    }
}
fn tower_facts(r: &ScaffoldRuntime) -> features::TowerFacts {
    let p: [f64; 3] = array(&r.facts["position"]).unwrap_or([0.; 3]);
    let v: [f64; 3] = array(&r.facts["velocity"]).unwrap_or([0.; 3]);
    features::TowerFacts {
        y: p[1],
        velocity_y: v[1],
        physical_jump: r.facts["physicalJump"] == true,
        blocks: r.block_count,
        block_below: r.facts["blockBelow"] == true,
        ground: r.facts["onGround"] == true,
        moving: r.facts["moving"] == true,
        horizontal_speed: (v[0] * v[0] + v[2] * v[2]).sqrt(),
        tick: number(&r.facts, "tick") as i32,
        protocol_1_8: false,
    }
}
fn ticks(
    settings: &RotationsSettings,
    mut current: Rotation,
    target: Rotation,
    last: Option<Rotation>,
    player_last: Rotation,
) -> i32 {
    if settings.active == SmoothMode::Acceleration {
        let diff = current.delta_to(target);
        if diff[0].abs() < 1.0e-5 && diff[1].abs() < 1.0e-5 {
            return 0;
        }
        let next = settings
            .process(
                settings.active,
                current,
                target,
                last,
                player_last,
                &mut liquidbounce_behaviors::random::random_unit_f32,
            )
            .unwrap_or(current);
        let change = [next.yaw - current.yaw, next.pitch - current.pitch];
        if (change[0].abs() < 1.0e-5 && change[1].abs() < 1.0e-5)
            || (diff[0].abs() < change[0].abs() && diff[1].abs() < change[1].abs())
        {
            return 0;
        }
        let a = (diff[0].abs() / change[0].abs()).floor();
        let b = (diff[1].abs() / change[1].abs()).floor();
        return if a.is_nan() || b.is_nan() {
            0
        } else {
            a.max(b) as i32
        };
    }
    let mut result = -1;
    loop {
        let speed = if settings.active == SmoothMode::Linear {
            settings.linear
        } else {
            settings.sigmoid.speeds
        };
        let mut h = speed.horizontal_turn_speed.0;
        let mut v = speed.vertical_turn_speed.0;
        if settings.active == SmoothMode::Sigmoid {
            let delta = current.delta_to(target);
            let difference = f64::from(delta[0] * delta[0] + delta[1] * delta[1]).sqrt() as f32;
            let scaled = difference.min(180.) / 120.;
            let exponent = -settings.sigmoid.steepness * (scaled - settings.sigmoid.midpoint);
            let factor = 1. / (1. + f64::from(exponent).exp());
            h = (factor * f64::from(h)) as f32;
            v = (factor * f64::from(v)) as f32;
        }
        current = current.towards_linear(target, h, v);
        result += 1;
        if current.delta_length_squared_to(target) <= 4. || result >= 80 {
            return result;
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn scaffold_first_tick(&self) {
        let active = self.scaffold_active();
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.scaffold;
        r.render.tick();
        if !active {
            return;
        }
        if r.flag("SprintControl.Enabled") {
            r.was_placed = false;
        }
        let mut click = r.clicker.settings;
        click.cps = r.ints("SimulatePlacementAttempts.Clicker.CPS");
        click.max_per_tick = r.int("SimulatePlacementAttempts.Clicker.MaxPerTick");
        click.technique = if r.tag("SimulatePlacementAttempts.Clicker.Technique") == "Constant" {
            Technique::Constant
        } else {
            Technique::Human
        };
        r.clicker.settings = click;
        r.clicker.plan.set_math(runtime::click_math());
        r.clicker.tick(self.started.elapsed().as_millis() as i64);
    }
    pub(crate) fn scaffold_eagle(&self, host: &mut dyn Host, i: [bool; 4]) -> bool {
        let distance = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.scaffold;
            if !r.flag("Technique.Normal.Eagle.Enabled")
                || !r.eagle.eligible(
                    r.down_falloff(),
                    r.facts["onGround"] == true,
                    r.flag("Technique.Normal.Eagle.OnlyOnGround"),
                    r.facts["flying"] == true,
                )
            {
                return false;
            }
            r.eagle.edge
        };
        serde_json::from_str::<Value>(&host.client_scaffold_prediction(
            &json!({"op":"close","distance":distance,"input":i}).to_string(),
        ))
        .unwrap_or(Value::Null)
            == true
    }
    /// Default MovementInput handlers, before the module's MODEL_STATE snapshot.
    pub(crate) fn scaffold_input_early(
        &self,
        _host: &mut dyn Host,
        e: &mut ClientDirectionalInputEvent,
    ) {
        if !self.scaffold_active() {
            return;
        }
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.scaffold;
        if r.normal_running() && r.flag("Technique.Normal.Telly.Enabled") {
            let none = b
                .rotations
                .lock()
                .expect("RotationManager")
                .current
                .is_none();
            e.jump |= r.telly.input(
                r.facts["moving"] == true,
                r.block_count,
                r.facts["onGround"] == true,
                r.tag("Technique.Normal.Telly.ResetMode") == "Reverse",
                none,
                r.int("Technique.Normal.Telly.Straight"),
            );
        }
    }
    pub(crate) fn scaffold_input_model(
        &self,
        host: &mut dyn Host,
        e: &mut ClientDirectionalInputEvent,
    ) {
        if !self.scaffold_active() {
            return;
        }
        let i = input(e);
        let line: Value = serde_json::from_str(
            &host.client_scaffold_planner(&json!({"op":"line","input":i}).to_string()),
        )
        .unwrap_or(Value::Null);
        let stabilize = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.scaffold;
            r.raw_input = i;
            r.line = line["line"].clone();
            r.normal_running()
                && r.flag("Technique.Normal.StabilizeMovement.Enabled")
                && !(e.jump && r.facts["onGround"] == true)
        };
        if stabilize {
            let line = self.state().behaviors.borrow().scaffold.line.clone();
            let v = feature_query(host, json!({"op":"stabilize","input":input(e),"line":line}));
            if let Some(i) = array(&v) {
                set_input(e, i);
            }
        }
    }
    pub(crate) fn scaffold_input_safety(
        &self,
        host: &mut dyn Host,
        e: &mut ClientDirectionalInputEvent,
    ) {
        if !self.scaffold_active() {
            return;
        }
        let name = {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.scaffold.tag("Technique").to_owned()
        };
        if name == "Normal" && !e.shift && self.scaffold_eagle(host, input(e)) {
            e.shift = true;
        }
        if name == "Breezily" {
            let distance = {
                self.state()
                    .behaviors
                    .borrow()
                    .scaffold
                    .floats("Technique.Breezily.EdgeDistance")
            };
            let v: Value = serde_json::from_str(&host.client_scaffold_special_technique(
                &json!({"op":"breezilyInput","input":input(e),"edgeDistance":distance}).to_string(),
            ))
            .unwrap_or(Value::Null);
            if let Some(i) = array(&v) {
                set_input(e, i);
            }
        }
        let (ledge, blocks, target, raw, minimum, force, modes, sneak_time, rotation, tick_count) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let (rotation, tick_count) = {
                let r = b.rotations.lock().expect("RotationManager");
                let p = host.client_player_last_rotation();
                let player_last = Rotation::new(p[0], p[1]);
                let player = host.client_player_rotation();
                let rotation = r.current.unwrap_or(Rotation::new(player[0], player[1]));
                (
                    rotation,
                    ticks(
                        &b.scaffold.rotations,
                        r.actual_server,
                        rotation,
                        r.last,
                        player_last,
                    ),
                )
            };
            let r = &mut b.scaffold;
            if r.height.take_forced_sneak() {
                e.shift = true;
            }
            let v: [f64; 3] = array(&r.facts["velocity"]).unwrap_or([0.; 3]);
            if r.flag("SpeedLimiter.Enabled")
                && (v[0] * v[0] + v[2] * v[2]).sqrt()
                    > f64::from(r.float("SpeedLimiter.SpeedLimit"))
            {
                set_input(e, [false; 4]);
            }
            (
                r.flag("Ledge"),
                r.block_count,
                r.target.clone(),
                r.raw_input,
                r.float("MinDist"),
                r.int("Technique.GodBridge.ForceSneakBelowCount"),
                r.settings["Technique.GodBridge.Modes"].clone(),
                r.ints("Technique.GodBridge.SneakTime"),
                rotation,
                tick_count,
            )
        };
        if !ledge {
            return;
        }
        let close: Value = serde_json::from_str(
            &host.client_scaffold_prediction(&json!({"op":"close","distance":0.1}).to_string()),
        )
        .unwrap_or(Value::Null);
        let mut a = features::ledge(close == true, blocks, tick_count).unwrap_or_default();
        if a == features::LedgeAction::default() && name == "GodBridge" {
            let v:Value=serde_json::from_str(&host.client_scaffold_special_technique(&json!({"op":"godbridgeLedge","input":raw,"target":target,"rotation":[rotation.yaw,rotation.pitch],"minDist":minimum,"blockCount":blocks,"forceSneakBelowCount":force,"modes":modes,"sneakTime":sneak_time}).to_string())).unwrap_or(Value::Null);
            a = features::LedgeAction {
                jump: v["jump"] == true,
                sneak: number(&v, "sneakTime") as i32,
                stop: v["stopInput"] == true,
                backwards: v["stepBack"] == true,
            };
        }
        let mut i = input(e);
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let old = b.scaffold.height.force_sneak;
        features::apply_ledge(a, &mut i, &mut e.jump, &mut b.scaffold.height.force_sneak);
        if a.sneak > old {
            e.shift = true;
        }
        set_input(e, i);
    }
    pub(crate) fn scaffold_input_final(
        &self,
        host: &mut dyn Host,
        e: &mut ClientDirectionalInputEvent,
    ) {
        let active = self.scaffold_active();
        let q = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.scaffold;
            let running = active && b.clutch.path.is_null();
            json!({"op":"safeWalkInput","input":[e.forward,e.backward,e.left,e.right,e.jump,e.shift,e.sprint],"running":running,"mode":r.tag("SafeWalk"),"distance":r.floats("SafeWalk.OnEdge.Distance"),"keep":r.ints("SafeWalk.OnEdge.Keep"),"onEdgeMode":r.tag("SafeWalk.OnEdge.Mode"),"sneak":r.ints("SafeWalk.OnEdge.Sneak"),"jump":r.flag("SafeWalk.OnEdge.Jump"),"down":active&&r.normal_running()&&r.down_falloff()})
        };
        if let Some(i) = array::<[bool; 7]>(&feature_query(host, q)) {
            [
                e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
            ] = i;
        }
    }
    pub(crate) fn scaffold_feature_tick(&self, host: &mut dyn Host) {
        let active = self.scaffold_active();
        let facts = feature_query(host, json!({"op":"facts"}));
        let (effect, acceleration, strafe_speed, head, timer) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            b.timer
                .set_provider_running(runtime::TIMER_PROVIDER, active);
            let r = &mut b.scaffold;
            if facts.is_object() {
                r.facts = facts.clone();
            }
            let towering = r.towering();
            {
                let mut blink = r.blink.state.lock().expect("Scaffold Blink");
                blink.running = active && r.flag("Blink.Enabled");
                blink.ground = facts["onGround"] == true;
                blink.sneak = facts["shift"] == true;
                blink.towering = towering;
                blink.flush = array(&r.settings["Blink.FlushOn"]).unwrap_or_default();
            }
            if !active {
                return;
            }
            let ground = facts["onGround"] == true;
            if ground {
                r.air_ticks = 0;
            } else {
                r.air_ticks = r.air_ticks.wrapping_add(1);
            }
            if r.normal_running() && r.flag("Technique.Normal.Telly.Enabled") {
                r.telly.tick(ground);
            }
            let settings = tower_settings(r);
            let tower_facts = tower_facts(r);
            let effect = r.tower.tick(settings, tower_facts);
            let acceleration = (r.flag("Acceleration.Enabled")
                && (!r.flag("Acceleration.OnlyOnGround") || ground))
                .then(|| r.float("Acceleration.SpeedMultiplier"));
            let strafe = if r.flag("Strafe.Enabled") {
                if facts["moving"] == true {
                    r.move_ticks += 1
                } else {
                    r.move_ticks = 0
                }
                (!r.flag("Strafe.OnlyOnGround") || ground).then(|| {
                    features::strafe_speed(
                        r.flag("Strafe.Hypixel"),
                        r.float("Strafe.Speed"),
                        facts["hasSpeed"] == true,
                        number(&facts, "tick") as i32,
                        r.move_ticks,
                    )
                })
            } else {
                None
            };
            let head = if r.normal_running() && r.flag("Technique.Normal.HeadHitter.Enabled") {
                if r.head_cooldown > 0 {
                    r.head_cooldown -= 1;
                    false
                } else if ground && facts["headHit"] == true && facts["moving"] == true {
                    r.head_cooldown = r.sample_int("Technique.Normal.HeadHitter.JumpDelay");
                    true
                } else {
                    false
                }
            } else {
                false
            };
            (effect, acceleration, strafe, head, r.float("Timer"))
        };
        self.scaffold_tower_effect(host, effect);
        if let Some(m) = acceleration {
            feature_query(host, json!({"op":"motion","horizontalMultiplier":m}));
        }
        if let Some(speed) = strafe_speed {
            feature_query(host, json!({"op":"strafe","speed":speed}));
        }
        if head {
            feature_query(host, json!({"op":"jump"}));
        }
        if timer != 1. {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.timer.state.lock().expect("Timer").timer.request_speed(
                timer,
                20,
                runtime::TIMER_PROVIDER,
                1,
            );
        }
    }
    fn scaffold_tower_effect(&self, host: &mut dyn Host, e: features::TowerEffect) {
        if e.y.is_some()
            || e.velocity_y.is_some()
            || e.horizontal_multiplier.is_some()
            || e.award_jump
        {
            feature_query(
                host,
                json!({"op":"motion","y":e.y,"velocityY":e.velocity_y,"horizontalMultiplier":e.horizontal_multiplier,"awardJump":e.award_jump}),
            );
        }
        if let Some(timer) = e.timer {
            let s = self.state();
            let b = s.behaviors.borrow();
            b.timer.state.lock().expect("Timer").timer.request_speed(
                timer,
                20,
                runtime::TIMER_PROVIDER,
                1,
            );
        }
        if e.wrong_protocol {
            self.blink_notify(
                "The Hypixel tower requires Minecraft 1.8 protocol.",
                "Hypixel 搭高需要 Minecraft 1.8 协议。",
                "Hypixel 搭高需要 Minecraft 1.8 協定。",
            );
        }
        if e.stationary {
            self.blink_notify(
                "Stand still to use the Hypixel tower.",
                "使用 Hypixel 搭高时需要保持静止。",
                "使用 Hypixel 搭高時需要保持靜止。",
            );
        }
    }
    pub(crate) fn scaffold_jump(&self, y: f64, motion: f32) {
        if !self.scaffold_active() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let mode = tower_settings(&b.scaffold).mode;
        let count = b.scaffold.tower.pending.len();
        b.scaffold.tower.jump(mode, y, motion, false);
        if b.scaffold.tower.pending.len() > count && count == 0 {
            b.coroutine_ticker
                .register(crate::coroutine_ticker::WaitOwner::ScaffoldTower, 0);
        }
    }
    pub(crate) fn scaffold_tower_wait(&self, host: &mut dyn Host, in_game: bool) -> bool {
        if !in_game || !self.scaffold_active() {
            self.state()
                .behaviors
                .borrow_mut()
                .scaffold
                .tower
                .pending
                .clear();
            return false;
        }
        let facts = feature_query(host, json!({"op":"facts"}));
        let (effect, keep) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.scaffold;
            r.facts = facts;
            let settings = tower_settings(r);
            let facts = tower_facts(r);
            let e = r.tower.advance_sequences(settings, facts);
            (e, !r.tower.pending.is_empty())
        };
        self.scaffold_tower_effect(host, effect);
        keep
    }
    pub(crate) fn scaffold_after_jump(
        &self,
        yaw: f32,
        i: [bool; 4],
        x: &mut f64,
        _y: &mut f64,
        z: &mut f64,
    ) {
        if !self.scaffold_active() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.scaffold;
        if r.normal_running() && r.flag("Technique.Normal.Telly.Enabled") {
            let next = r.sample_int("Technique.Normal.Telly.Jump");
            r.telly.after_jump(next);
        }
        if r.flag("StrafeOnJump.Enabled") {
            let angle = ((moving_yaw(yaw, i) + 180.) / 45.).round_ties_even() * 45.;
            let key = if angle % 90. == 0. {
                "StrafeOnJump.StraightSpeed"
            } else {
                "StrafeOnJump.DiagonalSpeed"
            };
            strafe(yaw, i, f64::from(r.sample_float(key)), x, z);
        }
    }
    pub(crate) fn scaffold_sprint(
        &self,
        source: u8,
        original: bool,
        moving: bool,
        ground: bool,
    ) -> bool {
        if !self.scaffold_active() {
            return original;
        }
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.scaffold;
        if !r.flag("SprintControl.Enabled") {
            return original;
        }
        let apply = |key, original| {
            let mode = match r.tag(key) {
                "ForceSprint" => features::SprintMode::Force,
                "ForceNoSprint" => features::SprintMode::Never,
                "NoSprintOnPlace" => features::SprintMode::NoPlace,
                "NoSprintOnGround" => features::SprintMode::NoGround,
                _ => features::SprintMode::Keep,
            };
            features::sprint(mode, original, moving, ground, r.was_placed)
        };
        let mut result = original;
        if source == 0 || source == 1 {
            result = apply("SprintControl.Client", result)
        }
        if source == 0 || source == 2 {
            result = apply("SprintControl.Server", result)
        }
        result
    }
    pub(crate) fn scaffold_omnidirectional(&self) -> bool {
        if !self.scaffold_active() {
            return false;
        }
        let s = self.state();
        let b = s.behaviors.borrow();
        b.scaffold.flag("SprintControl.Enabled")
            && b.scaffold.tag("SprintControl.Client") == "ForceSprint"
    }
}
