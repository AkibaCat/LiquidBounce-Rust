// SPDX-License-Identifier: GPL-3.0-or-later
use crate::runtime::LiquidBounceHooks;
use horizon_client::mod_api::{ClientDirectionalInputEvent, Host};
use liquidbounce_behaviors::{
    anti_void::{Exempt, Rescue, State},
    rotations::movement_yaw,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
pub const REQUESTER: u64 = 0x414e_5449_564f_4944;
/// Shared Clutch.guard service. Clutch must write its real canCatch result;
/// merely enabling Clutch never implies guarding. Unregistered Clutch reads false.
#[derive(Default)]
pub struct RescueGuards {
    clutch_guarding: AtomicBool,
}
impl RescueGuards {
    pub fn set_clutch_guarding(&self, value: bool) {
        self.clutch_guarding.store(value, Ordering::Release);
    }
    pub fn clutch_guarding(&self, clutch_running: bool) -> bool {
        clutch_running && self.clutch_guarding.load(Ordering::Acquire)
    }
}
#[derive(Default)]
pub struct AntiVoidService {
    pub state: Mutex<State>,
    pub input: Mutex<[bool; 7]>,
    running: AtomicBool,
    flags: AtomicU8,
    dead: AtomicBool,
    guards: Arc<RescueGuards>,
}
impl AntiVoidService {
    pub fn new(guards: Arc<RescueGuards>) -> Self {
        Self {
            guards,
            ..Default::default()
        }
    }
    pub fn set_context(&self, running: bool, fly: bool, clutch: bool, scaffold: bool) {
        self.running.store(running, Ordering::Release);
        self.flags.store(
            u8::from(fly) | u8::from(clutch) << 1 | u8::from(scaffold) << 2,
            Ordering::Release,
        );
    }
    fn exempt(&self) -> Exempt {
        let bits = self.flags.load(Ordering::Acquire);
        Exempt {
            dead: self.dead.load(Ordering::Acquire),
            fly: bits & 1 != 0,
            clutch_guarding: self.guards.clutch_guarding(bits & 2 != 0),
            scaffold: bits & 4 != 0,
        }
    }
    pub fn observe_dead(&self, dead: bool) {
        self.dead.store(dead, Ordering::Release);
    }
    pub fn queues(&self, outgoing: bool) -> bool {
        outgoing
            && self
                .state
                .lock()
                .expect("AntiVoid")
                .requires_lag(self.running.load(Ordering::Acquire), self.exempt())
    }
    pub fn record_input(&self, e: &ClientDirectionalInputEvent) {
        *self.input.lock().expect("AntiVoid input") = [
            e.forward, e.backward, e.left, e.right, e.jump, e.shift, e.sprint,
        ];
    }
}
impl LiquidBounceHooks {
    pub(crate) fn anti_void_shape(&self, active: &mut bool, floor: &mut f64) {
        let state = self.state();
        let b = state.behaviors.borrow();
        let exempt = Exempt {
            dead: false,
            fly: b.query_running(&state.registry, "Fly"),
            clutch_guarding: b
                .rescue_guards
                .clutch_guarding(b.query_running(&state.registry, "Clutch")),
            scaffold: b.query_running(&state.registry, "Scaffold"),
        };
        if let Some(y) = b
            .anti_void
            .state
            .lock()
            .expect("AntiVoid")
            .ghost_floor(b.query_running(&state.registry, "AntiVoid"), exempt)
        {
            *active = true;
            *floor = y;
        }
    }
    pub(crate) fn anti_void_network(&self, _host: &mut dyn Host, pre: bool, y: &mut f64) {
        let state = self.state();
        let b = state.behaviors.borrow();
        let mut model = b.anti_void.state.lock().expect("AntiVoid");
        let active = b.query_running(&state.registry, "AntiVoid") && model.settings.mode == 1;
        model.network(active, pre, y);
    }
    pub(crate) fn anti_void_tick(&self, host: &mut dyn Host) {
        let [yaw, _] = host.client_player_rotation();
        let (service, manager, query) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            if !b.query_running(&state.registry, "AntiVoid") {
                return;
            }
            let service = b.anti_void.clone();
            let settings = service.state.lock().expect("AntiVoid").settings.clone();
            let input = *service.input.lock().expect("AntiVoid input");
            let mut r = b.rotations.lock().expect("RotationManager");
            let correction = r
                .active_target(|provider| b.rotation_provider_running(&state.registry, provider))
                .map(|t| t.movement_correction);
            let query =
                serde_json::json!({"voidLevel":settings.void_level,"useItem":settings.mode==3,
                "thresholdBits":settings.y_motion_threshold.to_bits(),"keys":input,
                "movementYaw":movement_yaw(yaw,r.current,correction)})
                .to_string();
            (service, b.criticals.clone(), query)
        };
        let Some(v) = host
            .client_anti_void_query(&query)
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        else {
            return;
        };
        service.dead.store(v["dead"] == true, Ordering::Release);
        let Some(position) = serde_json::from_value::<[f64; 3]>(v["position"].clone()).ok() else {
            return;
        };
        let (running, exempt) = {
            let state = self.state();
            let b = state.behaviors.borrow();
            let running = b.query_running(&state.registry, "AntiVoid");
            service.set_context(
                running,
                b.query_running(&state.registry, "Fly"),
                b.query_running(&state.registry, "Clutch"),
                b.query_running(&state.registry, "Scaffold"),
            );
            (running, service.exempt())
        };
        let (eligible, void_level) = {
            let mut model = service.state.lock().expect("AntiVoid");
            model.observe(v["likely"] == true, position);
            (model.eligible(running, exempt), model.settings.void_level)
        };
        if !eligible {
            return;
        }
        // Native BlockShapeEvent may reenter the engine; hold no state/model lock.
        let empty = host.client_anti_void_clear(void_level);
        let positions = !manager.positions().is_empty();
        let slots =
            serde_json::from_value::<Vec<(i32, String)>>(v["slots"].clone()).unwrap_or_default();
        let action = service.state.lock().expect("AntiVoid").rescue(
            running,
            exempt,
            empty,
            v["fallDistance"].as_f64().unwrap_or(0.),
            v["motionY"].as_f64().unwrap_or(0.),
            v["reachesGround"] == true,
            positions,
            &slots,
            v["selected"].as_i64().unwrap_or(0) as i32,
            |r| liquidbounce_behaviors::random::random_i32_inclusive(r[0], r[1]),
        );
        let Some(action) = action else {
            return;
        };
        match action {
            Rescue::Raise(height) => host.client_anti_void_raise(height),
            Rescue::Silent => {}
            Rescue::CancelBlink => {
                if let Some(position) = manager.cancel_shared() {
                    host.client_blink_control("reset", &position);
                }
                self.criticals_flush_ready(host);
            }
            Rescue::Use { slot, delay } => {
                if slot != 40 {
                    let _ = host.client_select_slot_silently(REQUESTER, slot, delay);
                }
                let [yaw, pitch] = host.client_player_rotation();
                let current = self
                    .state()
                    .behaviors
                    .borrow()
                    .rotations
                    .lock()
                    .expect("RotationManager")
                    .current;
                let (yaw, pitch) = current.map_or((yaw, pitch), |r| (r.yaw, r.pitch));
                host.client_use_item(i32::from(slot == 40), yaw, pitch);
            }
        }
        let language = horizon_client::lang::code();
        let message = if language.starts_with("zh_tw")
            || language.starts_with("zh_hk")
            || language.starts_with("zh_hant")
        {
            "已採取措施防止墜入虛空"
        } else if language.starts_with("zh") {
            "已采取措施防止坠入虚空"
        } else {
            "Action taken to prevent void fall"
        };
        #[cfg(feature = "webview")]
        self.state().events.borrow_mut().push((
            "notification".into(),
            serde_json::json!({
            "title":liquidbounce_interop::localization::display_module_name(&language,"AntiVoid"),
            "message":message,"severity":"INFO"}),
        ));
        #[cfg(not(feature = "webview"))]
        eprintln!("[LiquidBounce] AntiVoid: {message}");
    }
}
