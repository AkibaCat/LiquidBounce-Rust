// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{coroutine_ticker::WaitOwner, runtime::LiquidBounceHooks};
use horizon_client::mod_api::Host;
use liquidbounce_behaviors::{
    no_rotate_set::{RotationsSettings, SmoothMode},
    no_web as model,
    rotations::{DefaultRotationTarget, Rotation},
};
use serde_json::{Value, json};
#[path = "no_web_wire.rs"]
mod wire;
pub(crate) const ROTATION_PROVIDER: u64 = 10;
pub(crate) const PROCESSOR_BASE: u64 = 100;
pub(crate) const REQUESTER: u64 = 0x4e4f_5745_42;
#[derive(Default)]
pub struct NoWebRuntime {
    pub behavior: model::NoWeb,
    pub rotations: RotationsSettings,
    pub action: Option<Value>,
    pub active_mode: Option<model::Mode>,
    pub waiting: bool,
    pub wait: i32,
    pub callback: u64,
}
impl NoWebRuntime {
    pub(crate) fn processor_mode(id: u64) -> Option<SmoothMode> {
        id.checked_sub(PROCESSOR_BASE)
            .and_then(SmoothMode::from_processor_id)
    }
    pub fn reset(&mut self) {
        self.behavior.reset();
        self.action = None;
    }
    pub fn start(&mut self) -> u64 {
        self.callback = self.callback.wrapping_add(1);
        self.waiting = true;
        self.wait = 0;
        self.callback
    }
    pub fn cancel(&mut self) {
        self.callback = self.callback.wrapping_add(1);
        self.waiting = false;
        self.wait = 0;
        self.reset();
    }
}
impl LiquidBounceHooks {
    pub(crate) fn no_web_collision(
        &self,
        pos: [i32; 3],
        moving: bool,
        ground: bool,
        tick: i32,
        result: &mut String,
    ) {
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        let running = m.query_running(&state.registry, "NoWeb");
        let e = m
            .no_web
            .behavior
            .collision(running, pos, moving, ground, tick);
        *result=json!({"cancel":e.cancel,"breakWorld":e.break_world,"destroyPackets":e.destroy_packets,"jump":e.jump,"speed":e.speed,"blend":e.blend,"y":e.y}).to_string();
    }
    pub(crate) fn no_web_prepare(&self, host: &mut dyn Host) {
        let (request, release) = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            let running = m.query_running(&state.registry, "NoWeb");
            let s = &mut m.no_web;
            let mode = s.behavior.settings.mode;
            let release = s.active_mode.is_some() && (!running || s.active_mode != Some(mode));
            if release {
                s.reset();
            }
            s.active_mode = running.then_some(mode);
            s.action = None;
            if running && !s.waiting {
                let id = s.start();
                m.coroutine_ticker.register(WaitOwner::NoWeb, id);
            }
            let request = (running && mode == model::Mode::PlaceWater)
                .then(|| wire::state(&m.no_web.behavior).to_string());
            (request, release)
        };
        if release {
            host.client_reset_silent_slot(REQUESTER);
        }
        let Some(request) = request else {
            return;
        };
        let Some(result) = host
            .client_no_web_plan(&request)
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        let s = &mut m.no_web;
        wire::read_state(&result["state"], &mut s.behavior);
        s.action = (!result["action"].is_null()).then(|| result["action"].clone());
        let (Some(yaw), Some(pitch)) = (
            result["rotation"][0].as_f64(),
            result["rotation"][1].as_f64(),
        ) else {
            return;
        };
        let r = &s.rotations;
        let target = DefaultRotationTarget {
            rotation: Rotation::new(yaw as f32, pitch as f32),
            horizontal_turn_speed: 180.,
            vertical_turn_speed: 180.,
            ticks_until_reset: r.ticks_until_reset,
            reset_threshold: r.reset_threshold,
            consider_inventory: false,
            movement_correction: r.movement_correction,
        };
        let processor = PROCESSOR_BASE + r.active.processor_id();
        m.rotations
            .lock()
            .expect("RotationManager")
            .request_processed(ROTATION_PROVIDER, 40, target, processor, true);
    }
    pub(crate) fn no_web_tick(&self, host: &mut dyn Host) {
        let request = {
            let state = self.state();
            let m = state.behaviors.borrow();
            if !m.query_running(&state.registry, "NoWeb") {
                return;
            };
            let Some(action) = m.no_web.action.as_ref() else {
                return;
            };
            let current = m.rotations.lock().expect("RotationManager").current;
            let rotation =
                current.map_or_else(|| host.client_player_rotation(), |r| [r.yaw, r.pitch]);
            json!({"state":wire::state(&m.no_web.behavior),"action":action,"rotation":rotation})
                .to_string()
        };
        let Some(result) = host
            .client_no_web_execute(&request)
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            return;
        };
        let state = self.state();
        let mut m = state.behaviors.borrow_mut();
        if result["attempted"] == true {
            m.no_web.action = None;
            if !result["state"].is_null() {
                wire::read_state(&result["state"], &mut m.no_web.behavior);
            }
        }
    }
    pub(crate) fn no_web_callback(&self, host: &mut dyn Host, in_game: bool, id: u64) -> bool {
        let conflict = {
            let state = self.state();
            let mut m = state.behaviors.borrow_mut();
            if id != m.no_web.callback {
                return false;
            }
            if !in_game || !m.query_running(&state.registry, "NoWeb") {
                m.no_web.waiting = false;
                m.no_web.wait = 0;
                return false;
            }
            if m.no_web.wait > 0 {
                m.no_web.wait -= 1;
                if m.no_web.wait > 0 {
                    return true;
                }
            }
            let id = m.query_ids.get("AvoidHazards").copied();
            let conflict = id.filter(|id| {
                state.registry.enabled_without_world(*id) && m.avoid_hazards.avoid & (1 << 3) != 0
            });
            if conflict.is_some() {
                m.no_web.wait = 40;
            }
            conflict
        };
        if let Some(id) = conflict {
            let options = crate::runtime::raw_options(host);
            let effects = {
                let mut state = self.state();
                if let Err(e) =
                    state.with_registry(options, |r, c| r.set_enabled_sync(id, false, c))
                {
                    eprintln!("[LiquidBounce] NoWeb compatibility: {e:?}");
                }
                state.take_effects()
            };
            crate::runtime::apply_effects(self, host, effects);
            #[cfg(feature = "webview")]
            {
                let language = horizon_client::lang::code();
                let title = liquidbounce_interop::localization::translate(
                    &language,
                    "liquidbounce.module.noWeb.messages.compatibilityTitle",
                )
                .unwrap_or("Compatibility error");
                let message = liquidbounce_interop::localization::translate(
                    &language,
                    "liquidbounce.module.noWeb.messages.compatibilityError",
                )
                .unwrap_or("NoWeb is incompatible with AvoidHazards");
                self.state().events.borrow_mut().push((
                    "notification".into(),
                    json!({"title":title,"message":message,"severity":"ERROR"}),
                ));
            }
        }
        true
    }
}
