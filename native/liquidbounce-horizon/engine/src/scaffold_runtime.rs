// SPDX-License-Identifier: GPL-3.0-or-later
//! ModuleScaffold and ModuleClutch event adapter over real native placement.
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    clicking::{Clicker, Cooldown, Math, Settings as ClickSettings, Technique},
    crosshair_clicker::Id,
    no_rotate_set::{RotationsSettings, SmoothMode},
    placement_renderer::{FULL, PlacementRenderer},
    rotations::{DefaultRotationTarget, Rotation},
    scaffold_features as features, scaffold_lifecycle as lifecycle,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
pub const ROTATION_PROVIDER: u64 = 29;
pub const PROCESSOR_BASE: u64 = 420;
pub const REQUESTER: u64 = 0x5343_4146_464f_4c44;
pub const TIMER_PROVIDER: u64 = REQUESTER;
const USE_COOLDOWN: Cooldown = Cooldown {
    real_ticks: 0.,
    reset_ticks: 0.,
    server_tick_speed: 1.,
    attack_delay: 1.,
    has_cooldown: false,
};
pub(super) fn click_math() -> Math {
    Math {
        log: horizon_client::clicker_math::log,
        exp: horizon_client::clicker_math::exp,
        strict_log: horizon_client::clicker_math::fdlibm::log,
    }
}
pub(super) fn number(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or(0.)
}
pub(super) fn array<T: serde::de::DeserializeOwned>(v: &Value) -> Option<T> {
    serde_json::from_value(v.clone()).ok()
}
pub(super) fn feature_query(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_scaffold_features(&v.to_string())).unwrap_or(Value::Null)
}
fn action(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_scaffold_action(&v.to_string())).unwrap_or(Value::Null)
}
fn prediction(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_scaffold_prediction(&v.to_string())).unwrap_or(Value::Null)
}
fn planner(host: &mut dyn Host, v: Value) -> Value {
    serde_json::from_str(&host.client_scaffold_planner(&v.to_string())).unwrap_or(Value::Null)
}
fn technique(host: &mut dyn Host, v: Value) -> Value {
    let text = match v["technique"].as_str() {
        Some("Clutch") => host.client_scaffold_clutch(&v.to_string()),
        Some("GodBridge" | "Breezily") => host.client_scaffold_special_technique(&v.to_string()),
        _ => host.client_scaffold_technique(&v.to_string()),
    };
    serde_json::from_str(&text).unwrap_or(Value::Null)
}
pub struct ScaffoldRuntime {
    pub settings: BTreeMap<String, Value>,
    pub rotations: RotationsSettings,
    pub render: PlacementRenderer,
    pub height: lifecycle::State,
    pub eagle: features::Eagle,
    pub telly: features::Telly,
    pub tower: features::Tower,
    pub blink: Arc<ScaffoldBlink>,
    pub clicker: Clicker,
    pub target: Value,
    pub line: Value,
    pub raw_input: [bool; 4],
    pub facts: Value,
    pub next_block: Option<String>,
    pub block_count: i32,
    pub head_cooldown: i32,
    pub move_ticks: i32,
    pub air_ticks: i32,
    pub was_placed: bool,
    pub strafe_disable_pending: bool,
    pub prediction_reset_pending: bool,
    pub reset_pending: bool,
    pub enable_pending: bool,
    pub delay: i32,
    pub waiting: bool,
    pub wait_until_ground: bool,
}
impl Default for ScaffoldRuntime {
    fn default() -> Self {
        let mut r=Self {
            settings: crate::config::scaffold::defaults(),
            rotations: Default::default(),
            render: PlacementRenderer::new(false, 0x5a00ff00),
            height: Default::default(),
            eagle: Default::default(),
            telly: Default::default(),
            tower: Default::default(),
            blink: Arc::new(ScaffoldBlink::default()),
            clicker: Clicker::new(
                ClickSettings::use_key(),
                false,
                Default::default(),
                click_math(),
            ),
            target: Value::Null,
            line: Value::Null,
            raw_input: [false; 4],
            facts: Value::Null,
            next_block: None,
            block_count: 0,
            head_cooldown: 0,
            move_ticks: 0,
            air_ticks: 0,
            was_placed: false,
            strafe_disable_pending: false,
            prediction_reset_pending: false,
            reset_pending: false,
            enable_pending: false,
            delay: 0,
            waiting: false,
            wait_until_ground: false,
        };
        r.eagle.blocks=r.sample_int("Technique.Normal.Eagle.BlocksToEagle");
        r.eagle.edge=r.sample_float("Technique.Normal.Eagle.EdgeDistance");
        r.telly.jump_ticks=r.sample_int("Technique.Normal.Telly.Jump");
        r
    }
}
impl ScaffoldRuntime {
    pub fn flag(&self, k: &str) -> bool {
        self.settings
            .get(k)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
    pub fn int(&self, k: &str) -> i32 {
        self.settings.get(k).and_then(Value::as_i64).unwrap_or(0) as i32
    }
    pub fn float(&self, k: &str) -> f32 {
        self.settings.get(k).and_then(Value::as_f64).unwrap_or(0.) as f32
    }
    pub fn tag(&self, k: &str) -> &str {
        self.settings.get(k).and_then(Value::as_str).unwrap_or("")
    }
    pub fn ints(&self, k: &str) -> [i32; 2] {
        self.settings.get(k).and_then(array).unwrap_or([0; 2])
    }
    pub fn floats(&self, k: &str) -> [f32; 2] {
        self.settings.get(k).and_then(array).unwrap_or([0.; 2])
    }
    pub fn sample_int(&self, k: &str) -> i32 {
        let r = self.ints(k);
        liquidbounce_behaviors::random::random_i32_inclusive(r[0], r[1])
    }
    pub fn sample_float(&self, k: &str) -> f32 {
        let r = self.floats(k);
        liquidbounce_behaviors::no_rotate_set::FloatRange(r[0], r[1])
            .sample(&mut liquidbounce_behaviors::random::random_unit_f32)
            .expect("Scaffold float range")
    }
    pub fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn reset(&mut self) {
        self.height.reset();
        self.target = Value::Null;
        self.line = Value::Null;
        self.next_block = None;
        self.delay = 0;
        self.waiting = false;
        self.wait_until_ground = false;
        self.reset_pending = true;
        self.render
            .clear_silently(crate::placement_esp_runtime::placement_esp_now());
    }
    pub fn towering(&mut self) -> bool {
        let selected = self.tag("Tower") != "None";
        let jump = self.facts["physicalJump"] == true;
        self.height.is_towering(selected, jump)
    }
    pub fn auto_speed_running(&self, running: bool) -> bool {
        running && self.flag("AutoSpeed")
    }
    pub fn down(&self) -> bool {
        self.flag("Technique.Normal.Down.Enabled") && self.facts["physicalSneak"] == true
    }
    pub fn down_falloff(&self) -> bool {
        self.down() && self.facts["downSupport"] == true
    }
    pub fn same_y(&self) -> lifecycle::SameY {
        match self.tag("SameY") {
            "On" => lifecycle::SameY::On,
            "JumpKey" => lifecycle::SameY::JumpKey,
            "Falling" => lifecycle::SameY::Falling,
            "Hypixel" => lifecycle::SameY::Hypixel,
            _ => lifecycle::SameY::Off,
        }
    }
    pub fn normal_running(&self) -> bool {
        self.tag("Technique") == "Normal"
    }
    pub fn sprint_value(&self, source: u8, mut value: i8, moving: bool, ground: bool) -> i8 {
        if !self.flag("SprintControl.Enabled") {
            return value;
        }
        for (enabled, key) in [
            (source == 0 || source == 1, "SprintControl.Client"),
            (source == 0 || source == 2, "SprintControl.Server"),
        ] {
            if enabled {
                match self.tag(key) {
                    "ForceSprint" if moving => value = 1,
                    "ForceNoSprint" => value = 0,
                    "NoSprintOnPlace" if self.was_placed => value = 0,
                    "NoSprintOnGround" => value = i8::from(!ground),
                    _ => {}
                }
            }
        }
        value
    }
}
pub struct ClutchRuntime {
    pub min_fall_distance: f32,
    pub path: Value,
    pub guarding: bool,
    pub release_pending: bool,
}
impl Default for ClutchRuntime {
    fn default() -> Self {
        Self {
            min_fall_distance: 5.,
            path: Value::Null,
            guarding: false,
            release_pending: false,
        }
    }
}
impl ClutchRuntime {
    pub fn disabled(&mut self) {
        self.guarding = false;
        self.release_pending |= !self.path.is_null();
        self.path = Value::Null;
    }
}
#[derive(Default)]
pub struct ScaffoldBlink {
    pub state: Mutex<BlinkState>,
}
#[derive(Default)]
pub struct BlinkState {
    pub running: bool,
    pub pulse_time: i64,
    pub started: i64,
    pub flush: Vec<String>,
    pub ground: bool,
    pub sneak: bool,
    pub towering: bool,
}
impl ScaffoldBlink {
    /// BlinkPacketEvent's no-action return leaves other providers' decisions intact.
    pub fn queues(&self, outgoing: bool, place: bool) -> bool {
        if !outgoing {
            return false;
        }
        let mut s = self.state.lock().expect("Scaffold Blink");
        if !s.running {
            return false;
        }
        let elapsed =
            crate::behaviors::current_time_millis().wrapping_sub(s.started) >= s.pulse_time;
        let flush = s.flush.iter().any(|tag| match tag.as_str() {
            "Place" => place,
            "Towering" => s.towering,
            "Sneaking" => s.sneak,
            "NotSneaking" => !s.sneak,
            "OnGround" => s.ground,
            "InAir" => !s.ground,
            _ => false,
        });
        if elapsed || flush {
            s.started = crate::behaviors::current_time_millis();
            false
        } else {
            true
        }
    }
}
impl LiquidBounceHooks {
    pub(crate) fn scaffold_active(&self) -> bool {
        let s = self.state();
        let b = s.behaviors.borrow();
        b.query_running(&s.registry, "Scaffold")
            || (b.query_running(&s.registry, "Clutch") && !b.clutch.path.is_null())
    }
    fn scaffold_rotations(&self, host: &dyn Host) -> (Rotation, Rotation) {
        let p = host.client_player_rotation();
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = b.rotations.lock().expect("RotationManager");
        (
            r.current.unwrap_or(Rotation::new(p[0], p[1])),
            r.server_rotation(
                b.criticals.is_lagging(),
                b.query_running(&s.registry, "Freeze"),
            ),
        )
    }
    fn scaffold_selection(&self, host: &mut dyn Host) -> Value {
        let q = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.scaffold;
            json!({"autoBlock":r.flag("AutoBlock.Enabled"),"doNotUseBelowCount":r.int("AutoBlock.DoNotUseBelowCount"),"disallowed":r.settings["BlockItemSelection.Disallowed"],"unfavorable":r.settings["BlockItemSelection.Unfavorable"]})
        };
        serde_json::from_str(&host.client_scaffold_selection(&q.to_string())).unwrap_or(Value::Null)
    }
    fn scaffold_current_technique(&self) -> String {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        if b.query_running(&s.registry, "Clutch") && !b.clutch.path.is_null() {
            "Clutch".into()
        } else if b.scaffold.towering() {
            "Normal".into()
        } else {
            b.scaffold.tag("Technique").into()
        }
    }
    pub(crate) fn scaffold_drain(&self, host: &mut dyn Host) {
        let facts = feature_query(host, json!({"op":"facts"}));
        let (reset, enable, predict, strafe) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let release = std::mem::take(&mut b.clutch.release_pending)
                && !b.query_running(&s.registry, "Scaffold");
            if release {
                b.scaffold.reset();
            }
            let r = &mut b.scaffold;
            if facts.is_object() {
                r.facts = facts;
            }
            let enable = std::mem::take(&mut r.enable_pending);
            if enable {
                r.height.enabled(number(&r.facts, "positionY") as i32);
                if let Some(p) = array::<[f64; 3]>(&r.facts["position"]) {
                    r.height.enabled(p[1].floor() as i32);
                }
            }
            (
                std::mem::take(&mut r.reset_pending),
                enable,
                std::mem::take(&mut r.prediction_reset_pending),
                std::mem::take(&mut r.strafe_disable_pending),
            )
        };
        if reset || enable {
            planner(host, json!({"op":"reset"}));
        }
        if reset || predict {
            prediction(host, json!({"op":"reset"}));
        }
        if reset {
            host.client_reset_silent_slot(REQUESTER);
            self.scaffold_count_event(None);
        }
        if strafe {
            feature_query(host, json!({"op":"motion","horizontalMultiplier":0.5}));
        }
    }
    pub(crate) fn scaffold_prepare(&self, host: &mut dyn Host) {
        self.scaffold_drain(host);
        let selection = self.scaffold_selection(host);
        let (enabled, clutch_running, minimum, was) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.query_ids
                    .get("Scaffold")
                    .is_some_and(|id| s.registry.enabled_without_world(*id)),
                b.query_running(&s.registry, "Clutch"),
                b.clutch.min_fall_distance,
                !b.clutch.path.is_null(),
            )
        };
        if clutch_running {
            let q = json!({"scaffoldEnabled":enabled,"blockCount":selection["blockCount"],"minFallDistance":minimum});
            let forecast: Value = serde_json::from_str(&host.client_clutch_query(&q.to_string()))
                .unwrap_or(Value::Null);
            let release = {
                let s = self.state();
                let mut b = s.behaviors.borrow_mut();
                b.clutch.guarding = forecast["guarding"] == true;
                b.clutch.path = forecast["path"].clone();
                let release = was && b.clutch.path.is_null() && !enabled;
                if release {
                    b.scaffold.reset();
                }
                release
            };
            if release {
                self.scaffold_drain(host);
            }
        }
        if !self.scaffold_active() {
            return;
        }
        let (_, server) = self.scaffold_rotations(host);
        let name = self.scaffold_current_technique();
        let (line, predict, settings, position, slot) = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.scaffold;
            r.wait_until_ground = true;
            r.block_count = number(&selection, "blockCount") as i32;
            r.next_block = selection["bestSlot"].as_u64().and_then(|i| {
                selection["hotbar"][i as usize]["block"]
                    .as_str()
                    .map(str::to_owned)
            });
            (
                r.line.clone(),
                r.flag("Prediction.Enabled"),
                json!({"bootstrapBackoff":r.float("Prediction.BootstrapBackoff"),"cutoffDistance":r.float("Prediction.PredictionCutoffDistance"),"warmupPlacements":r.int("Prediction.WarmupPlacements")}),
                r.facts["position"].clone(),
                selection["bestSlot"].clone(),
            )
        };
        let predicted = prediction(
            host,
            json!({"op":"predict","enabled":predict,"line":line,"settings":settings}),
        );
        let predicted = if predicted.is_null() {
            position
        } else {
            predicted
        };
        let raw = { self.state().behaviors.borrow().scaffold.raw_input };
        let eagle = self.scaffold_eagle(host, raw);
        let request = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let path = b.clutch.path.clone();
            let freeze = b.query_running(&s.registry, "Freeze");
            let r = &mut b.scaffold;
            let towering = r.towering();
            json!({"op":"plan","technique":name,"position":predicted,"pose":if eagle{"Crouching"}else{"Standing"},"slot":slot,"serverRotation":[server.yaw,server.pitch],"aimsAhead":r.tag("Rotations.RotationTiming")=="Normal","path":path,"optimalLine":line,"freeze":freeze,"aimMode":r.tag("Technique.Normal.RotationMode"),"length":r.int("Technique.Expand.Length"),"height":{"state":{"placementY":r.height.placement_y,"startY":r.height.start_y,"jumps":r.height.jumps,"wasTowering":r.height.was_towering},"sameY":r.tag("SameY"),"towering":towering,"downRunning":r.flag("Technique.Normal.Down.Enabled"),"shouldGoDown":r.down(),"ceiling":r.flag("Technique.Normal.Ceiling.Enabled")&&r.facts["belowAir"]==false}})
        };
        let found = technique(host, request);
        {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.scaffold;
            if name == "Clutch" {
                r.target = found;
            } else {
                r.target = found["target"].clone();
                if let Some(h) = found.get("heightState") {
                    r.height.placement_y = number(h, "placementY") as i32;
                    r.height.start_y = number(h, "startY") as i32;
                    r.height.jumps = number(h, "jumps") as i32;
                    r.height.was_towering = h["wasTowering"] == true;
                }
            }
        }
        if self
            .state()
            .behaviors
            .borrow()
            .scaffold
            .tag("Rotations.RotationTiming")
            == "Normal"
        {
            if let Some(rotation) = self.scaffold_technique_rotation(host, &name) {
                self.scaffold_request_rotation(rotation);
            }
        }
    }
    fn scaffold_technique_rotation(&self, host: &mut dyn Host, name: &str) -> Option<Rotation> {
        let request = {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.scaffold;
            if name == "Clutch" {
                return array::<[f32; 2]>(&r.target["rotation"]).map(|a| Rotation::new(a[0], a[1]));
            }
            let towering = r.towering();
            json!({"op":"rotation","technique":name,"target":r.target,"rawInput":r.raw_input,"requiresSight":r.flag("Technique.Normal.RequiresSight"),"telly":{"enabled":r.flag("Technique.Normal.Telly.Enabled"),"doNotAim":r.telly.do_not_aim(r.air_ticks,r.int("Technique.Normal.Telly.Straight"),towering,r.flag("Technique.Normal.Telly.AimOnTower")),"resetMode":r.tag("Technique.Normal.Telly.ResetMode")}})
        };
        array::<[f32; 2]>(&technique(host, request)).map(|a| Rotation::new(a[0], a[1]))
    }
    fn scaffold_request_rotation(&self, rotation: Rotation) {
        let s = self.state();
        let b = s.behaviors.borrow();
        let r = &b.scaffold;
        let settings = &r.rotations;
        b.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(
                ROTATION_PROVIDER,
                40,
                DefaultRotationTarget {
                    rotation,
                    horizontal_turn_speed: 180.,
                    vertical_turn_speed: 180.,
                    ticks_until_reset: settings.ticks_until_reset,
                    reset_threshold: settings.reset_threshold,
                    consider_inventory: r.flag("Rotations.ConsiderInventory"),
                    movement_correction: settings.movement_correction,
                },
                PROCESSOR_BASE + settings.active.processor_id(),
                true,
            );
    }
    fn scaffold_select(&self, host: &mut dyn Host, selection: &Value) -> bool {
        let (auto, reset) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.scaffold.flag("AutoBlock.Enabled"),
                b.scaffold.int("AutoBlock.SlotResetDelay"),
            )
        };
        let decision = lifecycle::select_block(
            auto,
            selection["rawMainValid"] == true,
            selection["offhandValid"] == true,
            selection["bestSlot"].as_u64().map(|i| i as usize),
        );
        match decision.action {
            lifecycle::SlotAction::Select(slot) => {
                host.client_select_slot_silently(REQUESTER, slot as i32, reset);
            }
            lifecycle::SlotAction::Reset => {
                host.client_reset_silent_slot(REQUESTER);
            }
        }
        decision.has_main
    }
    pub(crate) fn scaffold_tick(&self, host: &mut dyn Host) {
        if !self.scaffold_active() {
            return;
        }
        if self.state().behaviors.borrow().scaffold.waiting {
            return;
        }
        let selection = self.scaffold_selection(host);
        let facts = feature_query(host, json!({"op":"facts"}));
        {
            let s = self.state();
            let mut b = s.behaviors.borrow_mut();
            let r = &mut b.scaffold;
            r.facts = facts.clone();
            r.block_count = number(&selection, "blockCount") as i32;
            let p: [f64; 3] = array(&facts["position"]).unwrap_or([0.; 3]);
            r.height.tick_begin(
                p[1].floor() as i32,
                facts["onGround"] == true,
                facts["physicalJump"] == true,
            );
            if r.waiting {
                return;
            }
        }
        self.scaffold_count_event(Some(number(&selection, "blockCount") as i32));
        let (current, server) = self.scaffold_rotations(host);
        let name = self.scaffold_current_technique();
        let (target, timing, simulate) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            (
                b.scaffold.target.clone(),
                b.scaffold.tag("Rotations.RotationTiming").to_owned(),
                b.scaffold.flag("SimulatePlacementAttempts.Enabled"),
            )
        };
        let rotation = if timing != "Normal" && !target.is_null() {
            self.scaffold_technique_rotation(host, &name)
                .unwrap_or(current)
        } else {
            current
        };
        let rotation = rotation
            .normalize(
                current,
                liquidbounce_behaviors::rotations::modern_mouse_gcd(
                    host.client_mouse_sensitivity(),
                ),
            )
            .unwrap_or(rotation);
        let crosshair = technique(
            host,
            json!({"op":"crosshair","technique":name,"target":target,"rotation":[rotation.yaw,rotation.pitch],"serverRotation":[server.yaw,server.pitch],"aimsAhead":timing=="Normal","shouldGoDown":self.state().behaviors.borrow().scaffold.down()}),
        );
        let clicker_hit = if simulate && crosshair.is_null() {
            feature_query(
                host,
                json!({"op":"rawCrosshair","rotation":[rotation.yaw,rotation.pitch]}),
            )
        } else {
            crosshair.clone()
        };
        let (delay, always, swing, min_dist) = {
            let s = self.state();
            let b = s.behaviors.borrow();
            let r = &b.scaffold;
            (
                r.sample_int("Delay"),
                r.flag("AutoBlock.Always"),
                r.tag("Swing").to_owned(),
                r.float("MinDist"),
            )
        };
        let mut main = selection["rawMainValid"] == true;
        let off = selection["offhandValid"] == true;
        if always {
            main = self.scaffold_select(host, &selection);
        }
        let effective = self.scaffold_selection(host);
        let suitable = if effective["mainValid"] == true {
            Some("Main")
        } else if effective["offhandValid"] == true {
            Some("Off")
        } else {
            None
        };
        if simulate && !clicker_hit.is_null() && facts["moving"] == true {
            if let Some(hand) = suitable {
                let can = action(
                    host,
                    json!({"op":"checkPlacement","hit":clicker_hit,"hand":hand}),
                ) == true;
                let allowed = {
                    let s = self.state();
                    let b = s.behaviors.borrow();
                    let r = &b.scaffold;
                    lifecycle::simulate_attempt(lifecycle::Attempt {
                        enabled: true,
                        failed_only: r.flag("SimulatePlacementAttempts.FailedAttemptsOnly"),
                        suitable_hand: true,
                        block_hit: true,
                        can_place_on_face: can,
                        same_y: r.same_y(),
                        physical_jump: facts["physicalJump"] == true,
                        clicked_y: clicker_hit["block"][1].as_i64().unwrap_or(0) as i32,
                        clicked_up: clicker_hit["face"][1] == 1,
                        placement_y: r.height.placement_y,
                        player_block_y: array::<[f64; 3]>(&facts["position"]).unwrap_or([0.; 3])[1]
                            .floor() as i32,
                    })
                };
                if allowed {
                    let mut clicks = 0;
                    loop {
                        if !self
                            .state()
                            .behaviors
                            .borrow_mut()
                            .scaffold
                            .clicker
                            .take_attempt(USE_COOLDOWN)
                        {
                            break;
                        }
                        let _ = action(
                            host,
                            json!({"op":"execute","hit":clicker_hit,"rotation":[rotation.yaw,rotation.pitch],"hand":hand,"swing":swing,"simulated":true}),
                        );
                        self.state()
                            .behaviors
                            .borrow_mut()
                            .scaffold
                            .clicker
                            .record_success(None);
                        clicks += 1;
                    }
                    self.state()
                        .behaviors
                        .borrow_mut()
                        .scaffold
                        .clicker
                        .finish_click(clicks);
                }
            }
        }
        if target.is_null() || crosshair.is_null() {
            return;
        }
        let Some(point) = array(&crosshair["point"]) else {
            return;
        };
        let Some(face) = array(&crosshair["face"]) else {
            return;
        };
        let eye = action(host, json!({"op":"facts"}));
        let Some(eye) = array(&eye["eye"]) else {
            return;
        };
        if !lifecycle::valid_crosshair(point, eye, face, min_dist) {
            return;
        }
        if !always {
            main = self.scaffold_select(host, &selection);
        }
        if !main && !off {
            return;
        }
        let hand = if main { "Main" } else { "Off" };
        if timing != "Normal" {
            if rotation != server {
                action(
                    host,
                    json!({"op":"sendRotation","rotation":[rotation.yaw,rotation.pitch]}),
                );
            }
            if timing == "OnTickSnap" {
                self.scaffold_request_rotation(rotation);
            }
        }
        let line = self.state().behaviors.borrow().scaffold.line.clone();
        let falloff = if line.is_null() {
            Value::Null
        } else {
            prediction(host, json!({"op":"falloff","line":line}))
        };
        let success = action(
            host,
            json!({"op":"execute","hit":crosshair,"placed":target["placed"],"rotation":[rotation.yaw,rotation.pitch],"hand":hand,"swing":swing,"simulated":false}),
        ) == true;
        let (_, new_server) = self.scaffold_rotations(host);
        let player = host.client_player_rotation();
        if timing == "OnTick" && [new_server.yaw, new_server.pitch] != player {
            let yaw = rotation.yaw
                + liquidbounce_behaviors::rotations::wrap_degrees(player[0] - rotation.yaw);
            action(
                host,
                json!({"op":"sendRotation","rotation":[yaw,player[1]]}),
            );
        }
        if success {
            let enabled = self
                .state()
                .behaviors
                .borrow()
                .scaffold
                .flag("Prediction.Enabled");
            prediction(
                host,
                json!({"op":"record","enabled":enabled,"line":line,"falloff":falloff}),
            );
            if delay > 0 {
                let s = self.state();
                let mut b = s.behaviors.borrow_mut();
                b.scaffold.delay = delay;
                b.scaffold.waiting = true;
                b.coroutine_ticker
                    .register(crate::coroutine_ticker::WaitOwner::Scaffold, 0);
            }
        }
    }
    pub(crate) fn scaffold_wait(&self, in_game: bool) -> bool {
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let running =
            in_game && (b.query_running(&s.registry, "Scaffold") || !b.clutch.path.is_null());
        let r = &mut b.scaffold;
        if !running || !r.waiting {
            r.waiting = false;
            return false;
        }
        r.delay -= 1;
        if r.delay <= 0 {
            r.waiting = false;
            false
        } else {
            true
        }
    }
    /// Native dispatch occurs inside doPlacement's successful callback, before swing.
    pub(crate) fn scaffold_success(&self, host: &mut dyn Host, placed: [i32; 3], simulated: bool) {
        planner(host, json!({"op":"track","block":placed}));
        let bounds = feature_query(host, json!({"op":"outline","block":placed}));
        let bounds = array(&bounds).unwrap_or(FULL);
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let r = &mut b.scaffold;
        r.render.add(
            placed,
            bounds,
            crate::placement_esp_runtime::placement_esp_now(),
            true,
        );
        if r.flag("Technique.Normal.Eagle.Enabled") {
            if r.eagle.placed.wrapping_add(1) > r.eagle.blocks {
                let blocks = r.sample_int("Technique.Normal.Eagle.BlocksToEagle");
                let edge = r.sample_float("Technique.Normal.Eagle.EdgeDistance");
                r.eagle.placement(|| (blocks, edge));
            } else {
                r.eagle
                    .placement(|| unreachable!("refresh only after exceeding BlocksToEagle"));
            }
        }
        r.blink.state.lock().expect("Scaffold Blink").pulse_time =
            i64::from(r.sample_int("Blink.Time"));
        r.was_placed = true;
        if !simulated {
            r.target = Value::Null;
        }
    }
    pub(crate) fn scaffold_count_event(&self, count: Option<i32>) {
        #[cfg(feature = "webview")]
        {
            let s = self.state();
            let b = s.behaviors.borrow();
            s.events.borrow_mut().push((
                "blockCountChange".into(),
                json!({"block":b.scaffold.next_block,"count":count}),
            ));
        }
        #[cfg(not(feature = "webview"))]
        let _ = count;
    }
    pub(crate) fn scaffold_other_click(&self, id: Id) {
        if id != Id::Use || !self.scaffold_active() {
            return;
        }
        let s = self.state();
        let mut b = s.behaviors.borrow_mut();
        let plan = b.crosshair_clicker.clicker(id).plan.clone();
        b.scaffold
            .clicker
            .plan
            .pressed_by(&plan, self.started.elapsed().as_millis() as i64);
    }
    pub(crate) fn scaffold_player_press(&self, use_key: bool, open: bool, pressed: bool) {
        if use_key && !open && pressed && self.scaffold_active() {
            self.state()
                .behaviors
                .borrow_mut()
                .scaffold
                .clicker
                .plan
                .pressed_by_player();
        }
    }
}
